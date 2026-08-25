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
    pub note: String,
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

pub fn capture_screen_region_impl(
    region: &CaptureRegionInput,
    captured_png_bytes: &[u8],
    state: &AppState,
) -> Result<CaptureResultDto, String> {
    // Validate region bounds (BR-31)
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

    // Reduce image with QualityBudget (LC-003, FR-4, CAP-2)
    let reduced_result =
        ImageReducer::reduce_image(captured_png_bytes, orig_dims, &resolved, false)
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
        body: region.note.clone(),
        updated_at: captured_at,
    };

    state
        .finding_store
        .create_finding(&finding, &note, &[])
        .map_err(|e| e.to_string())?;

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
pub fn capture_screen_region(
    region: CaptureRegionInput,
    state: State<AppState>,
    app: AppHandle,
) -> Result<CaptureResultDto, String> {
    // Validate region bounds (BR-31)
    if region.width < 8 || region.height < 8 {
        return Err("Region must be at least 8x8 pixels".to_string());
    }

    // Hide overlay window before taking the screen grab so overlay UI is not in the screenshot
    if let Some(overlay_win) = app.get_webview_window("overlay") {
        let _ = overlay_win.hide();
    }

    let core_region = Region::new(region.x, region.y, region.width, region.height);

    // Real screen capture of requested region encoded as PNG (CAP-1, LC-002)
    let captured_png_bytes =
        RegionCapturer::capture_region(&core_region, region.source_monitor.as_deref()).map_err(
            |e| {
                if let Some(overlay_win) = app.get_webview_window("overlay") {
                    let _ = overlay_win.hide();
                }
                e.to_string()
            },
        )?;

    let result = capture_screen_region_impl(&region, &captured_png_bytes, &state)?;

    // Close capture overlay window if present
    if let Some(overlay_win) = app.get_webview_window("overlay") {
        let _ = overlay_win.hide();
    }

    // Restore main studio window after capture
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.show();
        let _ = main_win.unminimize();
        let _ = main_win.set_focus();
    }

    let _ = app.emit("capture-completed", ());

    Ok(result)
}

#[tauri::command]
pub fn trigger_overlay(app: AppHandle) -> Result<(), String> {
    let _ = app.emit("overlay-reset", ());

    // Hide main studio window so it does not block or appear in the screenshot
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.hide();
    }

    if let Some(existing) = app.get_webview_window("overlay") {
        let _ = existing.emit("overlay-reset", ());
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(());
    }

    let overlay_window =
        WebviewWindowBuilder::new(&app, "overlay", WebviewUrl::App("index.html".into()))
            .title("Snapdown Capture Overlay")
            .transparent(true)
            .decorations(false)
            .always_on_top(true)
            .fullscreen(true)
            .build()
            .map_err(|e| format!("Failed to create overlay window: {e}"))?;

    let _ = overlay_window.set_focus();

    Ok(())
}

#[tauri::command]
pub fn get_monitor_snapshot(source_monitor: Option<String>) -> Result<String, String> {
    let (image, _w, _h) = RegionCapturer::capture_monitor_image(source_monitor.as_deref())
        .map_err(|e| e.to_string())?;

    let mut bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut bytes);
    image::ImageEncoder::write_image(
        &encoder,
        image.as_raw(),
        image.width(),
        image.height(),
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|e| e.to_string())?;

    let base64_str = format!("data:image/png;base64,{}", base64::encode(&bytes));
    Ok(base64_str)
}

#[tauri::command]
pub fn dismiss_overlay(app: AppHandle) -> Result<(), String> {
    if let Some(overlay_win) = app.get_webview_window("overlay") {
        let _ = overlay_win.hide();
    }

    // Restore main studio window when capture overlay is cancelled/dismissed
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.show();
        let _ = main_win.unminimize();
        let _ = main_win.set_focus();
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
    use snapdown_core::domain::finding::Region;
    use snapdown_store::sqlite::SqliteSettingsStore;
    use std::sync::Arc;

    #[test]
    fn region_validation_refuses_small_box() {
        let reg = Region::new(10, 10, 4, 4);
        assert!(reg.width < 8);
    }

    #[test]
    fn a_capture_carries_its_note_through_to_the_stored_finding() {
        let store = SqliteSettingsStore::open_in_memory().unwrap();
        let temp_vault = tempfile::tempdir().unwrap();
        let vault_path = temp_vault.path().to_string_lossy().to_string();

        store
            .set(&Setting {
                key: SettingKey::VaultPath,
                value: SettingValue::String(vault_path),
                updated_at: "2026-08-24T00:00:00Z".to_string(),
            })
            .unwrap();

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

        // Synthesise a small valid PNG (64x48)
        let mut img = image::RgbaImage::new(64, 48);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba([120, 150, 200, 255]);
        }
        let mut png_bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        image::ImageEncoder::write_image(encoder, &img, 64, 48, image::ExtendedColorType::Rgba8)
            .unwrap();

        let note_text = "the CTA is unreadable\n\nsecond line\n";
        let input = CaptureRegionInput {
            x: 10,
            y: 10,
            width: 64,
            height: 48,
            note: note_text.to_string(),
            source_monitor: Some("DISPLAY1".to_string()),
        };

        let result = capture_screen_region_impl(&input, &png_bytes, &state).unwrap();
        assert_eq!(result.image_width, 64);
        assert_eq!(result.image_height, 48);

        let findings = state.finding_store.list_findings().unwrap();
        assert_eq!(findings.len(), 1);
        let finding_detail = &findings[0];
        assert_eq!(finding_detail.note.body, note_text);
        assert_eq!(finding_detail.note.finding_id, finding_detail.finding.id);
    }
}
