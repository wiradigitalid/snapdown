use snapdown_core::domain::finding::{Finding, FindingDetail, Marker, Note};
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::domain::setting::{QualityBudget, Setting, SettingKey, SettingValue};
use snapdown_core::ports::{BlobStore, Clock, EntropySource, FindingStore, SettingsStore};
use snapdown_store::image::{ImageReducer, MarkerBurner};
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
pub fn get_burned_image_base64(
    finding_id: String,
    state: State<AppState>,
) -> Result<String, String> {
    use base64::Engine;
    let detail = state
        .finding_store
        .get_finding(&finding_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Finding not found".to_string())?;

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
    let raw_bytes = vault_store
        .read_blob(&detail.finding.image_path)
        .map_err(|e| format!("Failed to read image blob: {e}"))?;

    // Burn markers if any exist
    let dims = ImageDimensions::new(detail.finding.image_width, detail.finding.image_height)
        .map_err(|e| e.to_string())?;

    let final_bytes = if !detail.markers.is_empty() {
        MarkerBurner::burn_markers(&raw_bytes, &dims, &detail.markers)
            .map_err(|e| format!("Failed to burn markers: {e}"))?
    } else {
        raw_bytes
    };

    Ok(base64::engine::general_purpose::STANDARD.encode(final_bytes))
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
        visual_annotations: vec![],
    })
}

#[derive(serde::Deserialize)]
pub struct CropFindingInput {
    pub finding_id: String,
    pub x: f64,      // fractional 0..1
    pub y: f64,      // fractional 0..1
    pub width: f64,  // fractional 0..1
    pub height: f64, // fractional 0..1
}

#[tauri::command]
pub fn crop_finding(
    input: CropFindingInput,
    state: State<AppState>,
) -> Result<FindingDetail, String> {
    let detail = state
        .finding_store
        .get_finding(&input.finding_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Finding not found".to_string())?;

    if input.width <= 0.0 || input.height <= 0.0 || input.x < 0.0 || input.y < 0.0 {
        return Err("Invalid crop coordinates".to_string());
    }

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
    let raw_bytes = vault_store
        .read_blob(&detail.finding.image_path)
        .map_err(|e| format!("Failed to read image blob: {e}"))?;

    let img =
        image::load_from_memory(&raw_bytes).map_err(|e| format!("Failed to decode image: {e}"))?;

    let orig_w = img.width();
    let orig_h = img.height();

    let crop_x = ((input.x * orig_w as f64).round() as u32).min(orig_w.saturating_sub(1));
    let crop_y = ((input.y * orig_h as f64).round() as u32).min(orig_h.saturating_sub(1));
    let crop_w = ((input.width * orig_w as f64).round() as u32)
        .max(1)
        .min(orig_w - crop_x);
    let crop_h = ((input.height * orig_h as f64).round() as u32)
        .max(1)
        .min(orig_h - crop_y);

    let cropped_img = image::imageops::crop_imm(&img, crop_x, crop_y, crop_w, crop_h).to_image();

    // Re-encode cropped image
    let mut out_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut out_bytes);
    image::ImageEncoder::write_image(
        encoder,
        cropped_img.as_raw(),
        crop_w,
        crop_h,
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|e| format!("Failed to encode cropped image: {e}"))?;

    // Overwrite blob
    vault_store
        .write_blob(&detail.finding.image_path, &out_bytes)
        .map_err(|e| e.to_string())?;

    // Update finding dimensions in database
    state
        .finding_store
        .update_finding_image(
            &input.finding_id,
            &detail.finding.image_path,
            crop_w,
            crop_h,
        )
        .map_err(|e| e.to_string())?;

    // Recalibrate markers relative to cropped area
    for marker in &detail.markers {
        // Calculate new normalized position relative to crop box
        let new_x = (marker.x - input.x) / input.width;
        let new_y = (marker.y - input.y) / input.height;

        let clamped_x = new_x.clamp(0.0, 1.0);
        let clamped_y = new_y.clamp(0.0, 1.0);

        let _ = state.finding_store.update_marker(
            &input.finding_id,
            &marker.id,
            clamped_x,
            clamped_y,
            &marker.comment,
        );
    }

    let updated_detail = state
        .finding_store
        .get_finding(&input.finding_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Finding not found after crop".to_string())?;

    Ok(updated_detail)
}

#[tauri::command]
pub fn show_item_in_folder(image_path: String, state: State<AppState>) -> Result<(), String> {
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

    let raw_path = if image_path.starts_with('/')
        || image_path.starts_with('\\')
        || (image_path.len() >= 2 && image_path.chars().nth(1) == Some(':'))
    {
        std::path::PathBuf::from(image_path)
    } else {
        std::path::Path::new(&vault_path).join(image_path)
    };

    // Normalize path with canonical/native separators
    let path = if let Ok(canonical) = std::fs::canonicalize(&raw_path) {
        // Strip extended-length prefix \\?\ on Windows
        let s = canonical.to_string_lossy();
        if let Some(stripped) = s.strip_prefix(r"\\?\") {
            std::path::PathBuf::from(stripped)
        } else {
            canonical
        }
    } else {
        raw_path
    };

    if !path.exists() {
        // Fall back to opening vault folder if specific file does not exist
        let parent = path
            .parent()
            .unwrap_or_else(|| std::path::Path::new(&vault_path));
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer").arg(parent).spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open").arg(parent).spawn();
        }
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        {
            let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
        }
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, explorer.exe requires `/select,"<path>"` passed as a single command-line
        // without wrapping the `/select,` prefix in quotes (which Rust Command does automatically when given a comma).
        // Using PowerShell Start-Process or raw_arg guarantees correct parameter transmission to explorer.exe.
        let path_str = path.to_string_lossy().replace('/', "\\");
        let arg = format!("/select,\"{path_str}\"");
        let _ = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!("Start-Process explorer -ArgumentList '{arg}'"),
            ])
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn();
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let parent = path
            .parent()
            .unwrap_or_else(|| std::path::Path::new(&vault_path));
        let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
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
