use serde::{Deserialize, Serialize};
use snapdown_core::domain::bundle::{Bundle, BundleDetail, BundleItem};
use snapdown_core::domain::finding::FindingDetail;
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::domain::markdown::MarkdownSerializer;
use snapdown_core::ports::{BlobStore, BundleStore, FindingStore, PublicationStore, SettingsStore};
use snapdown_store::image::MarkerBurner;
use snapdown_store::vault::VaultBlobStore;
use std::path::PathBuf;
use tauri::State;

use crate::commands::sharing::unpublish_bundle_impl;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBundleInput {
    pub name: String,
    pub finding_ids: Vec<String>,
}

fn rollback_written_blobs(
    vault_store: &VaultBlobStore,
    written: &[String],
    original_err: &str,
) -> String {
    let mut cleanup_failures = Vec::new();
    for path in written {
        if let Err(e) = vault_store.delete_blob(path) {
            cleanup_failures.push(format!("{path} ({e})"));
        }
    }
    if cleanup_failures.is_empty() {
        original_err.to_string()
    } else {
        format!(
            "{original_err}; additionally failed to clean up files during rollback: {}",
            cleanup_failures.join(", ")
        )
    }
}

pub fn create_bundle_impl(
    input: CreateBundleInput,
    state: &AppState,
) -> Result<BundleDetail, String> {
    if input.name.trim().is_empty() {
        return Err("Bundle name cannot be empty".into());
    }

    let bundle_id = format!("b-{}", chrono::Utc::now().timestamp_millis());
    let composed_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let md_filename = format!("bundles/{bundle_id}.md");

    // Resolve the vault path and open VaultBlobStore before reading or writing
    let vault_path = match state
        .settings_store
        .get(&snapdown_core::domain::setting::SettingKey::VaultPath)
        .map_err(|e| e.to_string())?
    {
        Some(s) => match s.value {
            snapdown_core::domain::setting::SettingValue::String(p) => p,
            _ => dirs_or_default_vault().to_string_lossy().to_string(),
        },
        _ => dirs_or_default_vault().to_string_lossy().to_string(),
    };

    let vault_store = VaultBlobStore::new(&vault_path)
        .map_err(|e| format!("Failed to open vault at {vault_path}: {e}"))?;

    // Gather findings and prepare burned image copies in memory
    let mut finding_details = Vec::new();
    let mut bundle_items = Vec::new();
    let mut pending_writes: Vec<(String, Vec<u8>)> = Vec::new();

    for (idx, fid) in input.finding_ids.iter().enumerate() {
        if let Some(detail) = state
            .finding_store
            .get_finding(fid)
            .map_err(|e| e.to_string())?
        {
            let pos = (idx + 1) as u32;
            let burned_path = format!("bundles/{bundle_id}/finding_{pos}_burned.png");

            // Refuse if finding image is absent from vault (BR-13, UC-9 failure flow 1)
            if !vault_store
                .blob_exists(&detail.finding.image_path)
                .map_err(|e| e.to_string())?
            {
                return Err(format!(
                    "Finding {} image file is missing from vault: {}",
                    detail.finding.id, detail.finding.image_path
                ));
            }

            let stored_bytes = vault_store
                .read_blob(&detail.finding.image_path)
                .map_err(|e| {
                    format!(
                        "Failed to read image for finding {}: {e}",
                        detail.finding.id
                    )
                })?;

            let dims =
                ImageDimensions::new(detail.finding.image_width, detail.finding.image_height)
                    .map_err(|e| e.to_string())?;

            let burned_bytes = MarkerBurner::burn_markers(&stored_bytes, &dims, &detail.markers)
                .map_err(|e| e.to_string())?;

            pending_writes.push((burned_path.clone(), burned_bytes));

            bundle_items.push(BundleItem {
                id: format!("bi-{bundle_id}-{pos}"),
                bundle_id: bundle_id.clone(),
                finding_id: fid.clone(),
                position: pos,
                image_path: burned_path,
            });

            finding_details.push(detail);
        }
    }

    // Generate markdown content referencing the bundle items' burned copies (BUG-21, FR-8)
    let items_for_ser: Vec<(&BundleItem, &FindingDetail)> =
        bundle_items.iter().zip(finding_details.iter()).collect();
    let markdown_content = MarkdownSerializer::serialize_bundle(&input.name, &items_for_ser);

    // Write all pending image blobs and markdown with transactional cleanup (AD-2, UC-9)
    let mut written: Vec<String> = Vec::new();

    for (path, bytes) in pending_writes {
        if let Err(e) = vault_store.write_blob(&path, &bytes) {
            let err_msg = format!("Failed to write burned image file {path}: {e}");
            return Err(rollback_written_blobs(&vault_store, &written, &err_msg));
        }
        written.push(path);
    }

    if let Err(e) = vault_store.write_blob(&md_filename, markdown_content.as_bytes()) {
        let err_msg = format!("Failed to write bundle markdown file: {e}");
        return Err(rollback_written_blobs(&vault_store, &written, &err_msg));
    }
    written.push(md_filename.clone());

    let bundle = Bundle {
        id: bundle_id,
        name: input.name,
        markdown: markdown_content,
        markdown_path: md_filename,
        composed_at,
    };

    if let Err(e) = state.bundle_store.create_bundle(&bundle, &bundle_items) {
        let err_msg = e.to_string();
        return Err(rollback_written_blobs(&vault_store, &written, &err_msg));
    }

    Ok(BundleDetail {
        bundle,
        items: bundle_items,
    })
}

#[tauri::command]
pub fn create_bundle(
    input: CreateBundleInput,
    state: State<AppState>,
) -> Result<BundleDetail, String> {
    create_bundle_impl(input, &state)
}

#[tauri::command]
pub fn list_bundles(state: State<AppState>) -> Result<Vec<BundleDetail>, String> {
    state.bundle_store.list_bundles().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_bundle_detail(
    id: String,
    state: State<AppState>,
) -> Result<Option<BundleDetail>, String> {
    state
        .bundle_store
        .get_bundle(&id)
        .map_err(|e| e.to_string())
}

pub fn delete_bundle_impl(id: &str, state: &AppState) -> Result<(), String> {
    // 1. If bundle is currently published, unpublish automatically (BR-20, BR-23)
    // If unpublish fails, abort immediately without deleting local DB records or files
    if let Some(pub_record) = state
        .publication_store
        .get_by_bundle_id(id)
        .map_err(|e| e.to_string())?
    {
        if pub_record.is_live() {
            unpublish_bundle_impl(id, state)?;
        }
    }

    // 2. Delete associated vault markdown file and burned images if present (AD-2, INV-BUNDLE-001)
    if let Some(detail) = state
        .bundle_store
        .get_bundle(id)
        .map_err(|e| e.to_string())?
    {
        let vault_path = match state
            .settings_store
            .get(&snapdown_core::domain::setting::SettingKey::VaultPath)
            .map_err(|e| e.to_string())?
        {
            Some(s) => match s.value {
                snapdown_core::domain::setting::SettingValue::String(p) => p,
                _ => dirs_or_default_vault().to_string_lossy().to_string(),
            },
            _ => dirs_or_default_vault().to_string_lossy().to_string(),
        };

        let vault_store = VaultBlobStore::new(&vault_path)
            .map_err(|e| format!("Failed to open vault at {vault_path}: {e}"))?;

        if vault_store
            .blob_exists(&detail.bundle.markdown_path)
            .map_err(|e| e.to_string())?
        {
            vault_store
                .delete_blob(&detail.bundle.markdown_path)
                .map_err(|e| {
                    format!(
                        "Failed to delete bundle markdown file {}: {e}",
                        detail.bundle.markdown_path
                    )
                })?;
        }

        for item in &detail.items {
            if vault_store
                .blob_exists(&item.image_path)
                .map_err(|e| e.to_string())?
            {
                vault_store.delete_blob(&item.image_path).map_err(|e| {
                    format!("Failed to delete bundle image {}: {e}", item.image_path)
                })?;
            }
        }
    }

    // 3. Cascade delete bundle and bundle_item records from database
    state
        .bundle_store
        .delete_bundle(id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_bundle(id: String, state: State<AppState>) -> Result<(), String> {
    delete_bundle_impl(&id, &state)
}

pub fn copy_bundle_to_clipboard_impl(id: &str, state: &AppState) -> Result<String, String> {
    let detail = state
        .bundle_store
        .get_bundle(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Bundle not found: {id}"))?;

    Ok(detail.bundle.markdown)
}

#[tauri::command]
pub fn copy_bundle_to_clipboard(id: String, state: State<AppState>) -> Result<String, String> {
    copy_bundle_to_clipboard_impl(&id, &state)
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

#[cfg(test)]
mod tests {
    use super::*;
    use snapdown_store::sqlite::{
        SqliteAccessKeyStore, SqliteBundleStore, SqliteFindingStore, SqlitePublicationStore,
    };
    use std::sync::Arc;

    #[test]
    fn bundle_commands_execution() {
        let b_store = Arc::new(SqliteBundleStore::open_in_memory().unwrap());
        let f_store = Arc::new(SqliteFindingStore::open_in_memory().unwrap());
        let p_store = Arc::new(SqlitePublicationStore::open_in_memory().unwrap());

        let state_data = AppState {
            settings_store: Arc::new(
                snapdown_store::sqlite::SqliteSettingsStore::open_in_memory().unwrap(),
            ),
            finding_store: f_store,
            bundle_store: b_store.clone(),
            access_key_store: Arc::new(SqliteAccessKeyStore::open_in_memory().unwrap()),
            publication_store: p_store,
            hotkey_registrar: Arc::new(std::sync::Mutex::new(
                crate::hotkey::DesktopHotkeyRegistrar::new(
                    Arc::new(
                        snapdown_store::sqlite::SqliteSettingsStore::open_in_memory().unwrap(),
                    ),
                    None,
                ),
            )),
            startup_registrar: Arc::new(std::sync::Mutex::new(
                crate::startup::DesktopStartupRegistrar::new(Arc::new(
                    crate::startup::NoopAutoStartBackend,
                )),
            )),
        };

        let list = b_store.list_bundles().unwrap();
        assert_eq!(list.len(), 0);
        let _ = state_data;
    }
}
