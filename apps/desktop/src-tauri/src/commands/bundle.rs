use serde::{Deserialize, Serialize};
use snapdown_core::domain::bundle::{Bundle, BundleDetail, BundleItem};
use snapdown_core::domain::markdown::MarkdownSerializer;
use snapdown_core::ports::{BlobStore, BundleStore, FindingStore, PublicationStore, SettingsStore};
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

    // Gather findings in requested order
    let mut finding_details = Vec::new();
    let mut bundle_items = Vec::new();

    for (idx, fid) in input.finding_ids.iter().enumerate() {
        if let Some(detail) = state
            .finding_store
            .get_finding(fid)
            .map_err(|e| e.to_string())?
        {
            let pos = (idx + 1) as u32;
            let burned_path = format!("bundles/{bundle_id}/finding_{pos}_burned.webp");

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

    // Generate markdown content
    let markdown_content = MarkdownSerializer::serialize_bundle(&input.name, &finding_details);

    // Save markdown file in vault
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

    vault_store
        .write_blob(&md_filename, markdown_content.as_bytes())
        .map_err(|e| format!("Failed to write bundle markdown file: {e}"))?;

    let bundle = Bundle {
        id: bundle_id,
        name: input.name,
        markdown: markdown_content,
        markdown_path: md_filename.clone(),
        composed_at,
    };

    if let Err(e) = state.bundle_store.create_bundle(&bundle, &bundle_items) {
        let _ = vault_store.delete_blob(&md_filename);
        return Err(e.to_string());
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
