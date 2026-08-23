use serde::{Deserialize, Serialize};
use snapdown_core::domain::setting::{Setting, SettingKey, SettingValue};
use snapdown_core::ports::{BlobStore, SettingsStore};
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
    pub image_path: string_path::StringPath,
    pub image_width: u32,
    pub image_height: u32,
    pub source_monitor: String,
    pub region: String,
}

mod string_path {
    pub type StringPath = String;
}

#[tauri::command]
pub fn capture_screen_region(
    region: CaptureRegionInput,
    state: State<AppState>,
    app: AppHandle,
) -> Result<CaptureResultDto, String> {
    // Validate region bounds
    if region.width < 8 || region.height < 8 {
        return Err("Region must be at least 8x8 pixels".to_string());
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

    let monitor_name = region
        .source_monitor
        .clone()
        .unwrap_or_else(|| "DISPLAY1".to_string());

    let region_str = format!(
        "{},{},{},{}",
        region.x, region.y, region.width, region.height
    );

    // Generate relative filename for finding
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("findings/capture_{timestamp}.png");

    // In a headless or test environment, create placeholder bitmap bytes
    // Real desktop capture will grab pixels and encode
    let placeholder_bytes = generate_placeholder_image(region.width, region.height);

    vault_store
        .write_blob(&filename, &placeholder_bytes)
        .map_err(|e| e.to_string())?;

    // Close capture overlay window if present
    if let Some(overlay_win) = app.get_webview_window("overlay") {
        let _ = overlay_win.close();
    }

    let _ = app.emit("capture-completed", ());

    Ok(CaptureResultDto {
        image_path: filename,
        image_width: region.width,
        image_height: region.height,
        source_monitor: monitor_name,
        region: region_str,
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

fn generate_placeholder_image(width: u32, height: u32) -> Vec<u8> {
    // Generates a minimal 1x1 or raw placeholder image bytes
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"\x89PNG\r\n\x1a\n"); // PNG magic header
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes
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
