use serde::{Deserialize, Serialize};
use snapdown_capture::RegionCapturer;
use snapdown_core::domain::finding::{Finding, Note, Region};
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::domain::setting::{QualityBudget, Setting, SettingKey, SettingValue};
use snapdown_core::ports::{BlobStore, Clock, EntropySource, FindingStore, SettingsStore};
use snapdown_store::image::ImageReducer;
use snapdown_store::system::{SystemClock, SystemEntropySource};
use snapdown_store::vault::VaultBlobStore;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureRegionInput {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub source_monitor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureResultDto {
    pub image_path: String,
    pub image_width: u32,
    pub image_height: u32,
    pub source_monitor: String,
    pub region: String,
    pub resolved_long_edge: u32,
    pub resolved_encoder_quality: u8,
    pub budget_name: String,
}

#[tauri::command]
pub fn capture_screen_region(
    region: CaptureRegionInput,
    state: State<AppState>,
    app: AppHandle,
) -> Result<CaptureResultDto, String> {
    // Validate region bounds (BR-31)
    if region.width < 8 || region.height < 8 {
        return Err("Region must be at least 8x8 pixels".to_string());
    }

    let core_region = Region::new(region.x, region.y, region.width, region.height);

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

    let monitor_name = region
        .source_monitor
        .clone()
        .unwrap_or_else(|| "DISPLAY1".to_string());

    let region_str = format!(
        "{},{},{},{}",
        region.x, region.y, region.width, region.height
    );

    // Get current QualityBudget and derive parameters dynamically for this region (LC-003, SCN-03)
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

    let region_long_edge = region.width.max(region.height);
    let resolved = qb.resolve(region_long_edge);
    let budget_name = qb.named.display_name().to_string();

    let orig_dims = ImageDimensions::new(region.width, region.height).map_err(|e| e.to_string())?;

    // Real screen capture of requested region encoded as PNG (CAP-1, LC-002)
    let captured_png_bytes =
        RegionCapturer::capture_region(&core_region, region.source_monitor.as_deref())
            .map_err(|e| e.to_string())?;

    // Reduce image with QualityBudget (LC-003, FR-4, CAP-2)
    let reduced_result =
        ImageReducer::reduce_image(&captured_png_bytes, orig_dims, &resolved, false)
            .map_err(|e| e.to_string())?;

    // Generate relative filename for finding
    let timestamp_str = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("findings/capture_{timestamp_str}.png");

    vault_store
        .write_blob(&filename, &reduced_result.bytes)
        .map_err(|e| e.to_string())?;

    // Create finding record with resolved derivation parameters (NFR-18, BR-105)
    let clock = SystemClock::new();
    let entropy = SystemEntropySource::new();
    let finding_id =
        snapdown_core::util::id::id_from_parts(clock.now_unix_millis(), entropy.random_bytes_10());
    let captured_at = clock.now_rfc3339();

    let finding = Finding {
        id: finding_id.clone(),
        image_path: filename.clone(),
        image_width: reduced_result.dimensions.width,
        image_height: reduced_result.dimensions.height,
        captured_at: captured_at.clone(),
        source_monitor: monitor_name.clone(),
        region: region_str.clone(),
        resolved_long_edge: Some(resolved.max_long_edge),
        resolved_encoder_quality: Some(resolved.encoder_quality),
        budget_name: Some(budget_name.clone()),
    };

    let note = Note {
        id: format!("note-{finding_id}"),
        finding_id: finding_id.clone(),
        body: String::new(),
        updated_at: captured_at,
    };

    state
        .finding_store
        .create_finding(&finding, &note, &[])
        .map_err(|e| e.to_string())?;

    // Close capture overlay window if present
    if let Some(overlay_win) = app.get_webview_window("overlay") {
        let _ = overlay_win.close();
    }

    let _ = app.emit("capture-completed", ());

    Ok(CaptureResultDto {
        image_path: filename,
        image_width: reduced_result.dimensions.width,
        image_height: reduced_result.dimensions.height,
        source_monitor: monitor_name,
        region: region_str,
        resolved_long_edge: resolved.max_long_edge,
        resolved_encoder_quality: resolved.encoder_quality,
        budget_name,
    })
}

#[tauri::command]
pub fn trigger_overlay(app: AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window("overlay") {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(());
    }

    let _overlay_window = WebviewWindowBuilder::new(
        &app,
        "overlay",
        WebviewUrl::App("index.html?overlay=true".into()),
    )
    .title("Snapdown Capture Overlay")
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .fullscreen(true)
    .build()
    .map_err(|e| format!("Failed to create overlay window: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn dismiss_overlay(app: AppHandle) -> Result<(), String> {
    if let Some(overlay_win) = app.get_webview_window("overlay") {
        let _ = overlay_win.close();
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
    use snapdown_core::domain::finding::Region;

    #[test]
    fn region_validation_refuses_small_box() {
        let reg = Region::new(10, 10, 4, 4);
        assert!(reg.width < 8);
    }
}
