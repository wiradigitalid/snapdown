use snapdown_core::domain::finding::{Finding, FindingDetail, Marker, Note};
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::domain::setting::{QualityBudget, Setting, SettingKey, SettingValue};
use snapdown_core::ports::{BlobStore, Clock, EntropySource, FindingStore, SettingsStore};
use snapdown_store::image::ImageReducer;
use snapdown_store::system::{SystemClock, SystemEntropySource};
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

#[tauri::command]
pub fn import_image_data(
    image_bytes_base64: String,
    note: Option<String>,
    source_name: Option<String>,
    state: State<AppState>,
) -> Result<FindingDetail, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(image_bytes_base64)
        .map_err(|e| format!("Invalid base64 image data: {e}"))?;

    // Decode image to check dimensions and validity
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode imported image: {e}"))?;
    let (width, height) = (img.width(), img.height());

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

    let qb = match state
        .settings_store
        .get(&SettingKey::QualityBudget)
        .map_err(|e| e.to_string())?
    {
        Some(Setting {
            value: SettingValue::QualityBudget(budget),
            ..
        }) => budget,
        _ => QualityBudget::default(),
    };

    let long_edge = width.max(height);
    let resolved = qb.resolve(long_edge);
    let budget_name = qb.named.display_name().to_string();

    let orig_dims = ImageDimensions::new(width, height).map_err(|e| e.to_string())?;
    let reduced_result = ImageReducer::reduce_image(&bytes, orig_dims, &resolved, false)
        .map_err(|e| e.to_string())?;

    let timestamp_str = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("findings/import_{timestamp_str}.png");

    vault_store
        .write_blob(&filename, &reduced_result.bytes)
        .map_err(|e| e.to_string())?;

    let clock = SystemClock::new();
    let entropy = SystemEntropySource::new();
    let finding_id =
        snapdown_core::util::id::id_from_parts(clock.now_unix_millis(), entropy.random_bytes_10());
    let captured_at = clock.now_rfc3339();

    let source = source_name.unwrap_or_else(|| "Imported Image".to_string());

    let finding = Finding {
        id: finding_id.clone(),
        image_path: filename,
        image_width: reduced_result.dimensions.width,
        image_height: reduced_result.dimensions.height,
        captured_at: captured_at.clone(),
        source_monitor: source,
        region: format!("0,0,{width},{height}"),
        resolved_long_edge: Some(resolved.max_long_edge),
        resolved_encoder_quality: Some(resolved.encoder_quality),
        budget_name: Some(budget_name),
    };

    let note = Note {
        id: format!("note-{finding_id}"),
        finding_id: finding_id.clone(),
        body: note.unwrap_or_default(),
        updated_at: captured_at,
    };

    state
        .finding_store
        .create_finding(&finding, &note, &[])
        .map_err(|e| e.to_string())?;

    Ok(FindingDetail {
        finding,
        note,
        markers: vec![],
    })
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
