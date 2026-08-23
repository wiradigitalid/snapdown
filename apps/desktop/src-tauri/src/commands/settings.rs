use serde::{Deserialize, Serialize};
use snapdown_core::domain::setting::{
    NamedBudget, QualityBudget, ResolvedPair, Setting, SettingKey, SettingValue,
};
use snapdown_core::ports::{Clock, FindingStore, SettingsStore};
use snapdown_store::system::SystemClock;
use std::path::PathBuf;
use tauri::State;

use crate::state::AppState;
use crate::vault_migration::VaultMigrator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityBudgetPresetDto {
    pub name: String,
    pub label: String,
    pub prose: String,
    pub fixed_pair: Option<ResolvedPair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestFindingAttributionDto {
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub budget_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityBudgetDto {
    pub named: NamedBudget,
    pub prose: String,
    pub custom_pair: Option<ResolvedPair>,
    pub max_long_edge: u32,
    pub encoder_quality: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsDto {
    pub vault_path: String,
    pub quality_budget: QualityBudgetDto,
    pub latest_finding_size: Option<u64>,
    pub latest_finding: Option<LatestFindingAttributionDto>,
}

fn to_quality_budget_dto(qb: &QualityBudget) -> QualityBudgetDto {
    let resolved = qb.resolve(1920);
    QualityBudgetDto {
        named: qb.named,
        prose: qb.prose().to_string(),
        custom_pair: qb.custom_pair,
        max_long_edge: qb
            .custom_pair
            .map(|p| p.max_long_edge)
            .unwrap_or(resolved.max_long_edge),
        encoder_quality: qb
            .custom_pair
            .map(|p| p.encoder_quality)
            .unwrap_or(resolved.encoder_quality),
    }
}

pub fn get_settings_impl(state: &AppState) -> Result<SettingsDto, String> {
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
        _ => QualityBudget::default(),
    };

    // Ensure default QualityBudget is stored if none exists
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

    let latest_attribution = get_latest_finding_attribution_internal(state, &vault_path);
    let latest_finding_size = latest_attribution
        .as_ref()
        .map(|a| a.size_bytes)
        .or_else(|| get_latest_finding_size_internal(&vault_path));

    Ok(SettingsDto {
        vault_path,
        quality_budget: to_quality_budget_dto(&quality_budget),
        latest_finding_size,
        latest_finding: latest_attribution,
    })
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<SettingsDto, String> {
    get_settings_impl(&state)
}

#[tauri::command]
pub fn get_quality_budget_presets() -> Vec<QualityBudgetPresetDto> {
    vec![
        QualityBudgetPresetDto {
            name: "auto".to_string(),
            label: "Auto".to_string(),
            prose: NamedBudget::Auto.prose().to_string(),
            fixed_pair: None,
        },
        QualityBudgetPresetDto {
            name: "sharp".to_string(),
            label: "Sharp".to_string(),
            prose: NamedBudget::Sharp.prose().to_string(),
            fixed_pair: NamedBudget::Sharp.fixed_pair(),
        },
        QualityBudgetPresetDto {
            name: "balanced".to_string(),
            label: "Balanced".to_string(),
            prose: NamedBudget::Balanced.prose().to_string(),
            fixed_pair: NamedBudget::Balanced.fixed_pair(),
        },
        QualityBudgetPresetDto {
            name: "small".to_string(),
            label: "Small".to_string(),
            prose: NamedBudget::Small.prose().to_string(),
            fixed_pair: NamedBudget::Small.fixed_pair(),
        },
    ]
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
                let _report = VaultMigrator::migrate_vault(old_path, &canonical_dest)
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

pub fn set_quality_budget_impl(
    budget: NamedBudget,
    advanced: Option<ResolvedPair>,
    state: &AppState,
) -> Result<QualityBudgetDto, String> {
    let clock = SystemClock::new();
    let store = &state.settings_store;

    let target_budget = if let Some(adv) = advanced {
        // Validate explicit ranges (BR-117)
        let validated =
            ResolvedPair::new(adv.max_long_edge, adv.encoder_quality).map_err(|e| e.to_string())?;
        QualityBudget::new(NamedBudget::Custom, Some(validated))
    } else {
        QualityBudget::new(budget, None)
    };

    store
        .set(&Setting::new(
            SettingKey::QualityBudget,
            SettingValue::QualityBudget(target_budget.clone()),
            clock.now_rfc3339(),
        ))
        .map_err(|e| e.to_string())?;

    Ok(to_quality_budget_dto(&target_budget))
}

#[tauri::command]
pub fn set_quality_budget(
    budget: NamedBudget,
    advanced: Option<ResolvedPair>,
    state: State<AppState>,
) -> Result<QualityBudgetDto, String> {
    set_quality_budget_impl(budget, advanced, &state)
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

    if let Some(attr) = get_latest_finding_attribution_internal(&state, &vault_path) {
        return Ok(Some(attr.size_bytes));
    }

    Ok(get_latest_finding_size_internal(&vault_path))
}

#[tauri::command]
pub fn pick_vault_folder() -> Result<Option<String>, String> {
    #[cfg(target_os = "windows")]
    {
        let script = r#"
Add-Type -AssemblyName System.Windows.Forms
$dialog = New-Object System.Windows.Forms.FolderBrowserDialog
$dialog.Description = "Select Snapdown Vault Folder"
$dialog.ShowNewFolderButton = $true
if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
    Write-Output $dialog.SelectedPath
}
"#;
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .output()
            .map_err(|e| format!("Failed to launch folder picker: {e}"))?;

        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return Ok(Some(path_str));
            }
        }
        Ok(None)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(None)
    }
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

fn get_latest_finding_attribution_internal(
    state: &AppState,
    vault_path: &str,
) -> Option<LatestFindingAttributionDto> {
    if let Ok(findings) = state.finding_store.list_findings() {
        if let Some(first) = findings.first() {
            let rel = &first.finding.image_path;
            let full_path = std::path::Path::new(vault_path).join(rel);
            let size = if full_path.exists() {
                std::fs::metadata(&full_path).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };
            return Some(LatestFindingAttributionDto {
                size_bytes: size,
                width: first.finding.image_width,
                height: first.finding.image_height,
                budget_name: first
                    .finding
                    .budget_name
                    .clone()
                    .unwrap_or_else(|| "Auto".to_string()),
            });
        }
    }
    None
}

fn get_latest_finding_size_internal(vault_path: &str) -> Option<u64> {
    let p = std::path::Path::new(vault_path);
    if !p.exists() {
        return None;
    }

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

    #[test]
    fn an_advanced_value_outside_its_range_is_refused_and_does_not_enter_custom() {
        // Below min edge
        let res1 = ResolvedPair::new(MIN_LONG_EDGE_PX - 1, 75);
        assert!(res1.is_err());

        // Above max edge
        let res2 = ResolvedPair::new(MAX_LONG_EDGE_PX + 1, 75);
        assert!(res2.is_err());

        // Below min quality
        let res3 = ResolvedPair::new(1600, MIN_ENCODER_QUALITY - 1);
        assert!(res3.is_err());

        // Above max quality
        let res4 = ResolvedPair::new(1600, MAX_ENCODER_QUALITY + 1);
        assert!(res4.is_err());

        // Valid boundaries
        let valid_min = ResolvedPair::new(MIN_LONG_EDGE_PX, MIN_ENCODER_QUALITY);
        assert!(valid_min.is_ok());

        let valid_max = ResolvedPair::new(MAX_LONG_EDGE_PX, MAX_ENCODER_QUALITY);
        assert!(valid_max.is_ok());
    }

    #[test]
    fn the_named_state_and_its_resolved_pair_are_written_together() {
        let store = SqliteSettingsStore::open_in_memory().unwrap();
        let state = AppState {
            settings_store: Arc::new(store),
            finding_store: Arc::new(
                snapdown_store::sqlite::SqliteFindingStore::open_in_memory().unwrap(),
            ),
            bundle_store: Arc::new(
                snapdown_store::sqlite::SqliteBundleStore::open_in_memory().unwrap(),
            ),
            access_key_store: Arc::new(
                snapdown_store::sqlite::SqliteAccessKeyStore::open_in_memory().unwrap(),
            ),
            publication_store: Arc::new(
                snapdown_store::sqlite::SqlitePublicationStore::open_in_memory().unwrap(),
            ),
            hotkey_registrar: Arc::new(std::sync::Mutex::new(
                crate::hotkey::DesktopHotkeyRegistrar::new(
                    Arc::new(SqliteSettingsStore::open_in_memory().unwrap()),
                    None,
                ),
            )),
            startup_registrar: Arc::new(std::sync::Mutex::new(
                crate::startup::DesktopStartupRegistrar::new(Arc::new(
                    crate::startup::tests::MockAutoStartBackend::default(),
                )),
            )),
        };

        // Preset Sharp
        let res_sharp = set_quality_budget_impl(NamedBudget::Sharp, None, &state).unwrap();
        assert_eq!(res_sharp.named, NamedBudget::Sharp);

        // Custom valid
        let custom_pair = ResolvedPair {
            max_long_edge: 2000,
            encoder_quality: 85,
        };
        let res_custom =
            set_quality_budget_impl(NamedBudget::Auto, Some(custom_pair), &state).unwrap();
        assert_eq!(res_custom.named, NamedBudget::Custom);
        assert_eq!(res_custom.custom_pair, Some(custom_pair));

        // Invalid advanced refused and does not change state
        let invalid_pair = ResolvedPair {
            max_long_edge: 100,
            encoder_quality: 85,
        };
        let res_err = set_quality_budget_impl(NamedBudget::Auto, Some(invalid_pair), &state);
        assert!(res_err.is_err());

        // State remains previous Custom
        let current = get_settings_impl(&state).unwrap();
        assert_eq!(current.quality_budget.named, NamedBudget::Custom);
    }
}
