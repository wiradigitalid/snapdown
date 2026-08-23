use snapdown_core::domain::finding::FindingDetail;
use snapdown_core::ports::FindingStore;
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

#[tauri::command]
pub fn delete_finding(id: String, state: State<AppState>) -> Result<(), String> {
    state
        .finding_store
        .delete_finding(&id)
        .map_err(|e| e.to_string())
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
    }
}
