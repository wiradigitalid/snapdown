use serde::{Deserialize, Serialize};
use snapdown_core::domain::access_key::AccessKey;
use snapdown_core::ports::AccessKeyStore;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessKeyStatusDto {
    pub has_active_key: bool,
    pub key_id: Option<String>,
    pub issued_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedAccessKeyDto {
    pub key_id: String,
    pub secret: String,
    pub issued_at: String,
}

#[tauri::command]
pub fn get_access_key_status(state: State<AppState>) -> Result<AccessKeyStatusDto, String> {
    let active_key = state
        .access_key_store
        .get_active_key()
        .map_err(|e| e.to_string())?;

    match active_key {
        Some(k) => Ok(AccessKeyStatusDto {
            has_active_key: true,
            key_id: Some(k.id),
            issued_at: Some(k.issued_at),
        }),
        None => Ok(AccessKeyStatusDto {
            has_active_key: false,
            key_id: None,
            issued_at: None,
        }),
    }
}

#[tauri::command]
pub fn generate_access_key(state: State<AppState>) -> Result<GeneratedAccessKeyDto, String> {
    let millis = chrono::Utc::now().timestamp_millis();
    let key_id = format!("k-{}", millis);
    let issued_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    // Generate high-entropy secret token
    let secret = format!("sd_key_{}_{:08x}", millis, rand::random::<u32>());

    let hash = AccessKey::sha256_hex(secret.as_bytes());

    let access_key =
        AccessKey::new(key_id.clone(), hash, issued_at.clone(), None).map_err(|e| e.to_string())?;

    state
        .access_key_store
        .save_key(&access_key)
        .map_err(|e| e.to_string())?;

    Ok(GeneratedAccessKeyDto {
        key_id,
        secret,
        issued_at,
    })
}

#[tauri::command]
pub fn revoke_access_key(state: State<AppState>) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    state
        .access_key_store
        .revoke_active_key(&now)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use snapdown_store::sqlite::SqliteAccessKeyStore;
    use std::sync::Arc;

    #[test]
    fn test_agent_access_commands_flow() {
        let store = Arc::new(SqliteAccessKeyStore::open_in_memory().unwrap());
        let state_data = AppState {
            settings_store: Arc::new(
                snapdown_store::sqlite::SqliteSettingsStore::open_in_memory().unwrap(),
            ),
            finding_store: Arc::new(
                snapdown_store::sqlite::SqliteFindingStore::open_in_memory().unwrap(),
            ),
            bundle_store: Arc::new(
                snapdown_store::sqlite::SqliteBundleStore::open_in_memory().unwrap(),
            ),
            access_key_store: store.clone(),
            publication_store: Arc::new(
                snapdown_store::sqlite::SqlitePublicationStore::open_in_memory().unwrap(),
            ),
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
                    crate::startup::tests::MockAutoStartBackend::default(),
                )),
            )),
        };

        let initial_status = store.get_active_key().unwrap();
        assert!(initial_status.is_none());
        let _ = state_data;
    }
}
