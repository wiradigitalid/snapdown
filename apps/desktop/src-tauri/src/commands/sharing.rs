use serde::{Deserialize, Serialize};
use snapdown_core::domain::publication::Publication;
use snapdown_core::domain::setting::{Setting, SettingKey, SettingValue};
use snapdown_core::ports::{BlobStore, BundleStore, PublicationStore, SettingsStore};
use snapdown_store::vault::VaultBlobStore;
use std::path::PathBuf;
use tauri::State;

use crate::publish::PublishClient;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationStatusDto {
    pub is_published: bool,
    pub slug: Option<String>,
    pub url: Option<String>,
    pub published_at: Option<String>,
    pub last_error: Option<String>,
}

#[tauri::command]
pub fn get_publication_status(
    bundle_id: String,
    state: State<AppState>,
) -> Result<PublicationStatusDto, String> {
    let pub_opt = state
        .publication_store
        .get_by_bundle_id(&bundle_id)
        .map_err(|e| e.to_string())?;

    match pub_opt {
        Some(p) => {
            let is_live = p.is_live();
            let url = if is_live {
                Some(format!("{}/b/{}", p.base_url.trim_end_matches('/'), p.slug))
            } else {
                None
            };
            Ok(PublicationStatusDto {
                is_published: is_live,
                slug: Some(p.slug),
                url,
                published_at: Some(p.published_at),
                last_error: p.last_error,
            })
        }
        None => Ok(PublicationStatusDto {
            is_published: false,
            slug: None,
            url: None,
            published_at: None,
            last_error: None,
        }),
    }
}

#[tauri::command]
pub fn publish_bundle(
    bundle_id: String,
    state: State<AppState>,
) -> Result<PublicationStatusDto, String> {
    // 1. Load Bundle
    let bundle_detail = state
        .bundle_store
        .get_bundle(&bundle_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Bundle not found: {bundle_id}"))?;

    // 2. Get existing slug or generate new 160-bit slug (AD-8)
    let existing_pub = state
        .publication_store
        .get_by_bundle_id(&bundle_id)
        .map_err(|e| e.to_string())?;

    let slug = match existing_pub {
        Some(ref p) => p.slug.clone(),
        None => {
            let mut rand_bytes = [0u8; 20];
            for b in &mut rand_bytes {
                *b = rand::random();
            }
            Publication::generate_slug_from_bytes(&rand_bytes)
        }
    };

    // 3. Resolve Web Service base URL
    let base_url = match state
        .settings_store
        .get(&SettingKey::WebServiceAddress)
        .map_err(|e| e.to_string())?
    {
        Some(Setting {
            value: SettingValue::String(s),
            ..
        }) if !s.trim().is_empty() => s,
        _ => "http://127.0.0.1:8080".to_string(),
    };

    // 4. Load files from vault
    let vault_path = match state
        .settings_store
        .get(&SettingKey::VaultPath)
        .map_err(|e| e.to_string())?
    {
        Some(Setting {
            value: SettingValue::String(s),
            ..
        }) => s,
        _ => dirs_or_default_vault().to_string_lossy().to_string(),
    };

    let vault_store = VaultBlobStore::new(&vault_path).map_err(|e| e.to_string())?;

    let mut file_payloads: Vec<(String, Vec<u8>)> = Vec::new();
    for item in &bundle_detail.items {
        if let Ok(bytes) = vault_store.read_blob(&item.image_path) {
            let fname = item
                .image_path
                .split('/')
                .next_back()
                .unwrap_or(&item.image_path)
                .to_string();
            file_payloads.push((fname, bytes));
        }
    }

    let files_ref: Vec<(&str, &[u8])> = file_payloads
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
        .collect();

    // 5. Execute publish via PublishClient
    let client = PublishClient::new(base_url.clone(), None);
    let pub_res = client.publish(&slug, &bundle_detail.bundle.markdown, &files_ref);

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    match pub_res {
        Ok(_) => {
            let pub_record = Publication {
                id: format!("pub-{bundle_id}"),
                bundle_id: bundle_id.clone(),
                slug: slug.clone(),
                base_url: base_url.clone(),
                published_at: now.clone(),
                unpublished_at: None,
                last_error: None,
            };
            state
                .publication_store
                .save(&pub_record)
                .map_err(|e| e.to_string())?;

            let url = format!("{}/b/{slug}", base_url.trim_end_matches('/'));
            Ok(PublicationStatusDto {
                is_published: true,
                slug: Some(slug),
                url: Some(url),
                published_at: Some(now),
                last_error: None,
            })
        }
        Err(err) => {
            let _ = state
                .publication_store
                .set_last_error(&bundle_id, Some(&err));
            Err(format!("Failed to publish bundle: {err}"))
        }
    }
}

#[tauri::command]
pub fn unpublish_bundle(bundle_id: String, state: State<AppState>) -> Result<(), String> {
    let pub_opt = state
        .publication_store
        .get_by_bundle_id(&bundle_id)
        .map_err(|e| e.to_string())?;

    let publication = match pub_opt {
        Some(p) => p,
        None => return Ok(()),
    };

    let client = PublishClient::new(publication.base_url.clone(), None);

    // Call unpublish
    let unpub_res = client.unpublish(&publication.slug);

    if let Err(ref err) = unpub_res {
        // Sticky error tracking: unpublish failure keeps bundle marked as published (BR-20, BR-96, BR-97)
        let _ = state
            .publication_store
            .set_last_error(&bundle_id, Some(err));
        return Err(format!("Unpublish failed: {err}"));
    }

    // Reconcile check: verify slug is indeed gone from remote
    if let Ok(is_still_served) = client.reconcile(&publication.slug) {
        if is_still_served {
            let _ = state.publication_store.set_last_error(
                &bundle_id,
                Some("Remote service still serving slug after delete"),
            );
            return Err("Remote service did not remove slug".into());
        }
    }

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    state
        .publication_store
        .mark_unpublished(&bundle_id, &now)
        .map_err(|e| e.to_string())?;

    let _ = state.publication_store.set_last_error(&bundle_id, None);

    Ok(())
}

#[tauri::command]
pub fn reconcile_publication(bundle_id: String, state: State<AppState>) -> Result<bool, String> {
    let pub_opt = state
        .publication_store
        .get_by_bundle_id(&bundle_id)
        .map_err(|e| e.to_string())?;

    let publication = match pub_opt {
        Some(p) => p,
        None => return Ok(false),
    };

    let client = PublishClient::new(publication.base_url, None);
    client.reconcile(&publication.slug)
}

fn dirs_or_default_vault() -> PathBuf {
    if let Some(user_dirs) = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
    {
        user_dirs.join("SnapdownVault")
    } else {
        PathBuf::from("./SnapdownVault")
    }
}
