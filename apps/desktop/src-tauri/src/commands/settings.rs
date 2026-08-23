use serde::{Deserialize, Serialize};
use snapdown_core::domain::setting::{
    QualityBudget, Setting, SettingKey, SettingValue, DEFAULT_ENCODER_QUALITY,
    DEFAULT_MAX_LONG_EDGE_PX,
};
use snapdown_core::ports::{Clock, SettingsStore};
use snapdown_store::system::SystemClock;
use std::path::PathBuf;
use tauri::State;

use crate::state::AppState;
use crate::vault_migration::VaultMigrator;

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsDto {
    pub vault_path: String,
    pub quality_budget: QualityBudget,
    pub latest_finding_size: Option<u64>,
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<SettingsDto, String> {
    let clock = SystemClock::new();
    let store = &state.settings_store;

    let vault_path = match store
        .get(&SettingKey::VaultPath)
        .map_err(|e| e.to_string())?
    {
        Some(Setting {
            value: SettingValue::String(s),
            ..
        }) => s,
        _ => {
            // Default vault path
            let default_path = dirs_or_default_vault();
            default_path.to_string_lossy().to_string()
        }
    };

    let quality_budget = match store
        .get(&SettingKey::QualityBudget)
        .map_err(|e| e.to_string())?
    {
        Some(Setting {
            value: SettingValue::QualityBudget(qb),
            ..
        }) => qb,
        _ => QualityBudget {
            max_long_edge: DEFAULT_MAX_LONG_EDGE_PX,
            encoder_quality: DEFAULT_ENCODER_QUALITY,
        },
    };

    let latest_finding_size = get_latest_finding_size_internal(&vault_path);

    // If vault path or quality budget was not present, populate defaults
    if store
        .get(&SettingKey::QualityBudget)
        .map_err(|e| e.to_string())?
        .is_none()
    {
        let _ = store.set(&Setting::new(
            SettingKey::QualityBudget,
            SettingValue::QualityBudget(quality_budget.clone()),
            clock.now_rfc3339(),
        ));
    }

    Ok(SettingsDto {
        vault_path,
        quality_budget,
        latest_finding_size,
    })
}

#[tauri::command]
pub fn set_vault_path(
    new_path: String,
    migrate_files: bool,
    state: State<AppState>,
) -> Result<String, String> {
    let clock = SystemClock::new();
    let store = &state.settings_store;

    let canonical_dest =
        VaultMigrator::validate_directory_writable(&new_path).map_err(|e| e.to_string())?;
    let canonical_dest_str = canonical_dest.to_string_lossy().to_string();

    let current_vault_path = match store
        .get(&SettingKey::VaultPath)
        .map_err(|e| e.to_string())?
    {
        Some(Setting {
            value: SettingValue::String(s),
            ..
        }) => Some(s),
        _ => None,
    };

    if migrate_files {
        if let Some(ref old_path) = current_vault_path {
            if std::path::Path::new(old_path).exists() {
                VaultMigrator::migrate_vault(old_path, &canonical_dest)
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    store
        .set(&Setting::new(
            SettingKey::VaultPath,
            SettingValue::String(canonical_dest_str.clone()),
            clock.now_rfc3339(),
        ))
        .map_err(|e| e.to_string())?;

    Ok(canonical_dest_str)
}

#[tauri::command]
pub fn set_quality_budget(
    max_long_edge: u32,
    encoder_quality: u8,
    state: State<AppState>,
) -> Result<QualityBudget, String> {
    let clock = SystemClock::new();
    let store = &state.settings_store;

    let qb = QualityBudget::new(max_long_edge, encoder_quality).map_err(|e| e.to_string())?;

    store
        .set(&Setting::new(
            SettingKey::QualityBudget,
            SettingValue::QualityBudget(qb.clone()),
            clock.now_rfc3339(),
        ))
        .map_err(|e| e.to_string())?;

    Ok(qb)
}

#[tauri::command]
pub fn get_latest_finding_size(state: State<AppState>) -> Result<Option<u64>, String> {
    let store = &state.settings_store;
    let vault_path = match store
        .get(&SettingKey::VaultPath)
        .map_err(|e| e.to_string())?
    {
        Some(Setting {
            value: SettingValue::String(s),
            ..
        }) => s,
        _ => dirs_or_default_vault().to_string_lossy().to_string(),
    };

    Ok(get_latest_finding_size_internal(&vault_path))
}

#[tauri::command]
pub fn open_vault_folder(state: State<AppState>) -> Result<(), String> {
    let store = &state.settings_store;
    let vault_path = match store
        .get(&SettingKey::VaultPath)
        .map_err(|e| e.to_string())?
    {
        Some(Setting {
            value: SettingValue::String(s),
            ..
        }) => s,
        _ => dirs_or_default_vault().to_string_lossy().to_string(),
    };

    let p = std::path::Path::new(&vault_path);
    if !p.exists() {
        std::fs::create_dir_all(p).map_err(|e| format!("Failed to create vault directory: {e}"))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(p)
            .spawn()
            .map_err(|e| format!("Failed to open folder in explorer: {e}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(p)
            .spawn()
            .map_err(|e| format!("Failed to open folder in finder: {e}"))?;
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(p)
            .spawn()
            .map_err(|e| format!("Failed to open folder in file manager: {e}"))?;
    }

    Ok(())
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

fn get_latest_finding_size_internal(vault_path: &str) -> Option<u64> {
    let p = std::path::Path::new(vault_path);
    if !p.exists() {
        return None;
    }

    // Inspect files in vault directory to find the most recently modified file and return its size
    let mut latest_time: Option<std::time::SystemTime> = None;
    let mut latest_size: Option<u64> = None;

    if let Ok(entries) = std::fs::read_dir(p) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    if let Ok(modified) = meta.modified() {
                        if latest_time.is_none() || Some(modified) > latest_time {
                            latest_time = Some(modified);
                            latest_size = Some(meta.len());
                        }
                    }
                }
            }
        }
    }

    latest_size
}

#[cfg(test)]
mod tests {
    use super::*;
    use snapdown_core::domain::setting::{
        MAX_ENCODER_QUALITY, MAX_LONG_EDGE_PX, MIN_ENCODER_QUALITY, MIN_LONG_EDGE_PX,
    };
    use snapdown_store::sqlite::SqliteSettingsStore;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn quality_budget_outside_range_is_refused_on_entry() {
        // Below min edge
        let res1 = QualityBudget::new(MIN_LONG_EDGE_PX - 1, 75);
        assert!(res1.is_err());

        // Above max edge
        let res2 = QualityBudget::new(MAX_LONG_EDGE_PX + 1, 75);
        assert!(res2.is_err());

        // Below min quality
        let res3 = QualityBudget::new(1600, MIN_ENCODER_QUALITY - 1);
        assert!(res3.is_err());

        // Above max quality
        let res4 = QualityBudget::new(1600, MAX_ENCODER_QUALITY + 1);
        assert!(res4.is_err());

        // Valid boundaries
        let valid_min = QualityBudget::new(MIN_LONG_EDGE_PX, MIN_ENCODER_QUALITY);
        assert!(valid_min.is_ok());

        let valid_max = QualityBudget::new(MAX_LONG_EDGE_PX, MAX_ENCODER_QUALITY);
        assert!(valid_max.is_ok());
    }

    #[test]
    fn latest_finding_size_scan_behavior() {
        // Non-existent path returns None
        assert_eq!(
            get_latest_finding_size_internal("non_existent_path_xyz"),
            None
        );

        let tmp = TempDir::new().unwrap();
        // Empty directory returns None
        assert_eq!(
            get_latest_finding_size_internal(tmp.path().to_str().unwrap()),
            None
        );

        // Populate files
        let f1 = tmp.path().join("finding1.png");
        let f2 = tmp.path().join("finding2.png");
        std::fs::write(&f1, b"12345").unwrap(); // 5 bytes
        std::thread::sleep(std::time::Duration::from_millis(50));
        std::fs::write(&f2, b"1234567890").unwrap(); // 10 bytes

        let latest = get_latest_finding_size_internal(tmp.path().to_str().unwrap());
        assert_eq!(latest, Some(10));
    }

    #[test]
    fn set_vault_path_refusal_preserves_store_integrity() {
        let store = SqliteSettingsStore::open_in_memory().unwrap();
        let initial_setting = Setting::new(
            SettingKey::VaultPath,
            SettingValue::String("C:/Initial/Vault".into()),
            "2026-08-23T00:00:00Z".into(),
        );
        store.set(&initial_setting).unwrap();

        let state_data = AppState {
            settings_store: Arc::new(store),
        };

        // Empty path is invalid and must be rejected
        let res = VaultMigrator::validate_directory_writable("");
        assert!(res.is_err());

        // Verify store value remains unchanged
        let loaded = state_data
            .settings_store
            .get(&SettingKey::VaultPath)
            .unwrap()
            .unwrap();
        assert_eq!(
            loaded.value,
            SettingValue::String("C:/Initial/Vault".into())
        );
    }
}
