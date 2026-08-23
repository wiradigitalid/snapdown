use snapdown_core::domain::finding::{FindingDetail, Marker};
use snapdown_core::domain::setting::{Setting, SettingKey, SettingValue};
use snapdown_core::ports::{BlobStore, FindingStore, SettingsStore};
use snapdown_store::vault::{OrphanScanReport, OrphanSweeper, VaultBlobStore};
use std::path::PathBuf;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn list_findings(state: State<AppState>) -> Result<Vec<FindingDetail>, String> {
    state
        .finding_store
        .list_findings()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_finding_detail(
    id: String,
    state: State<AppState>,
) -> Result<Option<FindingDetail>, String> {
    state
        .finding_store
        .get_finding(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_note(finding_id: String, body: String, state: State<AppState>) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    state
        .finding_store
        .update_note(&finding_id, &body, &now)
        .map_err(|e| e.to_string())
}

pub fn delete_finding_impl(id: &str, state: &AppState) -> Result<(), String> {
    // 1. Get finding image path to delete file first (AD-2, INV-DELETE-001)
    if let Ok(Some(detail)) = state.finding_store.get_finding(id) {
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

        if let Ok(vault_store) = VaultBlobStore::new(&vault_path) {
            let _ = vault_store.delete_blob(&detail.finding.image_path);
        }
    }

    // 2. Delete database row
    state
        .finding_store
        .delete_finding(id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_finding(id: String, state: State<AppState>) -> Result<(), String> {
    delete_finding_impl(&id, &state)
}

#[tauri::command]
pub fn scan_orphans(state: State<AppState>) -> Result<OrphanScanReport, String> {
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
    OrphanSweeper::scan_orphans(state.finding_store.as_ref(), &vault_store)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clean_orphans(orphan_files: Vec<String>, state: State<AppState>) -> Result<usize, String> {
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
    OrphanSweeper::clean_orphans(&vault_store, &orphan_files).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_marker(
    finding_id: String,
    marker_id: String,
    x: f64,
    y: f64,
    comment: String,
    state: State<AppState>,
) -> Result<Marker, String> {
    state
        .finding_store
        .add_marker(&finding_id, &marker_id, x, y, &comment)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_marker(
    finding_id: String,
    marker_id: String,
    x: f64,
    y: f64,
    comment: String,
    state: State<AppState>,
) -> Result<Marker, String> {
    state
        .finding_store
        .update_marker(&finding_id, &marker_id, x, y, &comment)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_marker(
    finding_id: String,
    marker_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    state
        .finding_store
        .delete_marker(&finding_id, &marker_id)
        .map_err(|e| e.to_string())
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
    use snapdown_core::domain::finding::{Finding, Note};
    use snapdown_store::sqlite::SqliteFindingStore;

    #[test]
    fn finding_commands_execution() {
        let store = SqliteFindingStore::open_in_memory().unwrap();
        let fid = "018f2345-6789-7abc-8def-0123456789aa";
        let finding = Finding {
            id: fid.to_string(),
            image_path: "findings/test.png".to_string(),
            image_width: 800,
            image_height: 600,
            captured_at: "2026-08-23T10:00:00Z".to_string(),
            source_monitor: "DISPLAY1".to_string(),
            region: "0,0,800,600".to_string(),
            resolved_long_edge: Some(1280),
            resolved_encoder_quality: Some(92),
            budget_name: Some("Auto".to_string()),
        };
        let note = Note {
            id: "note-1".to_string(),
            finding_id: fid.to_string(),
            body: "Original note".to_string(),
            updated_at: "2026-08-23T10:00:00Z".to_string(),
        };
        store.create_finding(&finding, &note, &[]).unwrap();

        let detail = store.get_finding(fid).unwrap().unwrap();
        assert_eq!(detail.finding.id, fid);
        assert_eq!(detail.note.body, "Original note");

        store
            .update_note(fid, "Modified note", "2026-08-23T11:00:00Z")
            .unwrap();
        let detail2 = store.get_finding(fid).unwrap().unwrap();
        assert_eq!(detail2.note.body, "Modified note");

        // Test marker CRUD & renumbering
        let m1 = store
            .add_marker(fid, "m1", 0.2, 0.3, "First marker")
            .unwrap();
        assert_eq!(m1.ordinal, 1);

        let m2 = store
            .add_marker(fid, "m2", 0.4, 0.5, "Second marker")
            .unwrap();
        assert_eq!(m2.ordinal, 2);

        store.delete_marker(fid, "m1").unwrap();
        let detail3 = store.get_finding(fid).unwrap().unwrap();
        assert_eq!(detail3.markers.len(), 1);
        assert_eq!(detail3.markers[0].id, "m2");
        assert_eq!(detail3.markers[0].ordinal, 1);
    }
}
