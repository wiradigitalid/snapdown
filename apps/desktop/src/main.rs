#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod hotkey;
mod startup;
mod tray;

use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use image::imageops::crop_imm;
use slint::{ComponentHandle, ModelRc, SharedPixelBuffer, VecModel};
use snapdown_capture::RegionCapturer;
use snapdown_core::domain::finding::{Finding, Note};
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::domain::setting::{
    HotkeyAction, QualityBudget, Setting, SettingKey, SettingValue,
};
use snapdown_core::ports::{BlobStore, Clock, EntropySource, FindingStore, SettingsStore};
use snapdown_store::image::ImageReducer;
use snapdown_store::sqlite::{SqliteFindingStore, SqliteSettingsStore};
use snapdown_store::system::{SystemClock, SystemEntropySource};
use snapdown_store::vault::VaultBlobStore;

use hotkey::{DesktopGlobalHotkeyBackend, DesktopHotkeyRegistrar, GlobalShortcutBackend};
#[cfg(not(windows))]
use startup::NoopAutoStartBackend;
#[cfg(windows)]
use startup::WindowsRegistryAutoStartBackend;
use startup::{reconcile_startup_on_boot, AutoStartBackend, DesktopStartupRegistrar};
use tray::{AppTray, TrayAction};

const SINGLE_INSTANCE_MUTEX_NAME: &str = "Snapdown-SingleInstance-id.wiradigital.snapdown";

slint::include_modules!();

/// Acquires a named OS mutex for the lifetime of the process. Returns `None` when another
/// instance already holds it, so the caller can exit instead of opening a second tray icon.
/// The returned handle must be kept alive (bound to a variable) until the process exits.
#[cfg(windows)]
fn acquire_single_instance_lock() -> Option<windows::Win32::Foundation::HANDLE> {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows::Win32::System::Threading::CreateMutexW;

    let wide_name: Vec<u16> = SINGLE_INSTANCE_MUTEX_NAME
        .encode_utf16()
        .chain(Some(0))
        .collect();

    unsafe {
        match CreateMutexW(None, false, PCWSTR(wide_name.as_ptr())) {
            Ok(handle) => {
                if GetLastError() == ERROR_ALREADY_EXISTS {
                    None
                } else {
                    Some(handle)
                }
            }
            Err(_) => Some(windows::Win32::Foundation::HANDLE::default()),
        }
    }
}

#[cfg(not(windows))]
fn acquire_single_instance_lock() -> Option<()> {
    Some(())
}

fn default_vault_path() -> PathBuf {
    if let Some(user_dirs) = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
    {
        user_dirs.join("SnapdownVault")
    } else {
        PathBuf::from("./SnapdownVault")
    }
}

fn app_database_path() -> PathBuf {
    if let Some(app_data) = std::env::var_os("APPDATA").map(PathBuf::from) {
        let dir1 = app_data.join("id.wiradigital.snapdown");
        if dir1.join("library.db").exists() {
            return dir1.join("library.db");
        }
        let dir2 = app_data.join("Snapdown");
        if dir2.join("library.db").exists() {
            return dir2.join("library.db");
        }
        if !dir1.exists() {
            let _ = std::fs::create_dir_all(&dir1);
        }
        dir1.join("library.db")
    } else {
        PathBuf::from("./library.db")
    }
}

struct AppContext {
    vault_store: VaultBlobStore,
    vault_path: PathBuf,
    finding_store: Arc<SqliteFindingStore>,
    settings_store: Arc<SqliteSettingsStore>,
}

impl AppContext {
    fn init() -> Self {
        let db_path = app_database_path();
        let finding_store = match SqliteFindingStore::open(&db_path) {
            Ok(store) => Arc::new(store),
            Err(e) => {
                eprintln!(
                    "Warning: Failed to open DB at {:?}: {e}, falling back to in-memory",
                    db_path
                );
                Arc::new(SqliteFindingStore::open_in_memory().unwrap())
            }
        };

        let settings_store = match SqliteSettingsStore::open(&db_path) {
            Ok(store) => Arc::new(store),
            Err(e) => {
                eprintln!(
                    "Warning: Failed to open settings DB at {:?}: {e}, falling back to in-memory",
                    db_path
                );
                Arc::new(SqliteSettingsStore::open_in_memory().unwrap())
            }
        };

        let vault_path = match settings_store.get(&SettingKey::VaultPath) {
            Ok(Some(Setting {
                value: SettingValue::String(s),
                ..
            })) => PathBuf::from(s),
            _ => default_vault_path(),
        };

        let vault_store = match VaultBlobStore::new(&vault_path) {
            Ok(store) => store,
            Err(e) => {
                eprintln!("Failed to init vault store at {:?}: {e}", vault_path);
                let fallback = default_vault_path();
                let _ = std::fs::create_dir_all(&fallback);
                VaultBlobStore::new(&fallback).unwrap()
            }
        };

        Self {
            vault_store,
            vault_path,
            finding_store,
            settings_store,
        }
    }
}

fn rgba_to_slint_image(img: &image::RgbaImage) -> slint::Image {
    let (w, h) = (img.width(), img.height());
    let pixel_buffer = SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(img.as_raw(), w, h);
    slint::Image::from_rgba8(pixel_buffer)
}

fn load_findings_into_window(
    window: &AppWindow,
    ctx: &AppContext,
    active_finding_id: Option<&str>,
) {
    let findings = ctx.finding_store.list_findings().unwrap_or_default();
    let mut filmstrip: Vec<FindingThumb> = Vec::new();

    let target_active_id = active_finding_id
        .map(|s| s.to_string())
        .or_else(|| findings.first().map(|f| f.finding.id.clone()));

    for finding_detail in &findings {
        let f = &finding_detail.finding;
        let is_active = target_active_id.as_deref() == Some(&f.id);

        let img_path = if PathBuf::from(&f.image_path).is_absolute() {
            PathBuf::from(&f.image_path)
        } else {
            ctx.vault_path.join(&f.image_path)
        };

        let loaded_img = if img_path.exists() {
            if let Ok(dyn_img) = image::open(&img_path) {
                rgba_to_slint_image(&dyn_img.to_rgba8())
            } else {
                slint::Image::default()
            }
        } else {
            slint::Image::default()
        };

        let time_str = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&f.captured_at) {
            dt.format("%H:%M").to_string()
        } else if f.captured_at.len() >= 16 {
            f.captured_at[11..16].to_string()
        } else {
            "00:00".to_string()
        };

        let dim_str = format!("{}×{}", f.image_width, f.image_height);

        filmstrip.push(FindingThumb {
            id: f.id.clone().into(),
            time_str: time_str.into(),
            dimensions_str: dim_str.into(),
            is_selected: false,
            is_active,
            image: loaded_img,
        });
    }

    let model = Rc::new(VecModel::from(filmstrip));
    window.set_filmstrip_items(ModelRc::from(model));

    // Update active finding details if available
    if let Some(active_id) = target_active_id {
        if let Ok(Some(detail)) = ctx.finding_store.get_finding(&active_id) {
            let f = &detail.finding;
            let img_path = if PathBuf::from(&f.image_path).is_absolute() {
                PathBuf::from(&f.image_path)
            } else {
                ctx.vault_path.join(&f.image_path)
            };

            if let Ok(dyn_img) = image::open(&img_path) {
                let rgba = dyn_img.to_rgba8();
                window.set_active_image(rgba_to_slint_image(&rgba));
            }

            let filename = PathBuf::from(&f.image_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| f.image_path.clone());

            window.set_current_filename(filename.into());
            window.set_resolution_text(format!("{} × {} px", f.image_width, f.image_height).into());

            if let Ok(metadata) = std::fs::metadata(&img_path) {
                let kb = metadata.len() as f64 / 1024.0;
                window.set_size_text(format!("{:.1} KB", kb).into());
            } else {
                window.set_size_text(
                    format!(
                        "{:.1} KB",
                        (f.image_width * f.image_height * 4) as f64 / 1024.0 / 6.0
                    )
                    .into(),
                );
            }

            window.set_observation_summary(detail.note.body.into());

            // Convert markers
            let slint_markers: Vec<MarkerData> = detail
                .markers
                .iter()
                .map(|m| MarkerData {
                    id: m.id.clone().into(),
                    ordinal: m.ordinal as i32,
                    x: m.x as f32,
                    y: m.y as f32,
                    comment: m.comment.clone().into(),
                })
                .collect();
            window.set_markers(ModelRc::from(Rc::new(VecModel::from(slint_markers))));
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Keep the mutex handle alive for the whole process; a second launch finds it already
    // held and exits instead of opening a duplicate tray icon and window.
    let _single_instance_lock = match acquire_single_instance_lock() {
        Some(lock) => lock,
        None => {
            eprintln!("Snapdown is already running.");
            return Ok(());
        }
    };

    let main_window = AppWindow::new()?;
    let ctx = Arc::new(AppContext::init());

    // Populate initial Filmstrip from Vault
    load_findings_into_window(&main_window, &ctx, None);

    // Finding selection from Filmstrip
    let win_weak_sel = main_window.as_weak();
    let ctx_sel = ctx.clone();
    main_window.on_finding_selected(move |id| {
        if let Some(win) = win_weak_sel.upgrade() {
            load_findings_into_window(&win, &ctx_sel, Some(&id));
        }
    });

    // Native Window Dragging on Titlebar
    #[cfg(windows)]
    {
        use i_slint_backend_winit::WinitWindowAccessor;
        let win_drag = main_window.as_weak();
        main_window.on_drag_window_requested(move || {
            if let Some(win) = win_drag.upgrade() {
                win.window().with_winit_window(|winit_win| {
                    let _ = winit_win.drag_window();
                });
            }
        });
    }

    // Window controls
    let win_min = main_window.as_weak();
    main_window.on_minimize_clicked(move || {
        if let Some(win) = win_min.upgrade() {
            win.window().set_minimized(true);
        }
    });

    let win_max = main_window.as_weak();
    main_window.on_maximize_clicked(move || {
        if let Some(win) = win_max.upgrade() {
            let is_max = win.window().is_maximized();
            win.window().set_maximized(!is_max);
        }
    });

    // Close hides to the tray instead of exiting; Exit is reached only via the tray menu.
    let win_close = main_window.as_weak();
    main_window.on_close_clicked(move || {
        if let Some(win) = win_close.upgrade() {
            win.hide().unwrap();
        }
    });

    // Theme Toggle
    let win_theme = main_window.as_weak();
    main_window.on_theme_toggle_clicked(move || {
        if let Some(win) = win_theme.upgrade() {
            let cur = win.get_is_dark_theme();
            println!("Theme toggled. Dark mode is now: {}", !cur);
        }
    });

    // Bundles Drawer / Library Toggle
    main_window.on_library_clicked(|| {
        println!("Open Snapdown Library / Bundle History clicked");
    });
    main_window.on_bundles_drawer_clicked(|| {
        println!("Toggle Bundles Drawer clicked");
    });

    // Settings Toggle
    main_window.on_settings_clicked(|| {
        println!("Open Settings clicked");
    });

    // Setup Capture callback
    let window_weak = main_window.as_weak();
    let ctx_capture = ctx.clone();
    main_window.on_capture_clicked(move || {
        let (raw_img, w, h) = match RegionCapturer::capture_monitor_image(None) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Capture failed: {e}");
                return;
            }
        };

        // Convert raw RgbaImage to Slint RgbaImage
        let pixel_buffer =
            SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(raw_img.as_raw(), w, h);
        let slint_img = slint::Image::from_rgba8(pixel_buffer);

        let overlay = match CaptureOverlayWindow::new() {
            Ok(ov) => ov,
            Err(e) => {
                eprintln!("Failed to create CaptureOverlayWindow: {e}");
                return;
            }
        };
        overlay.set_snapshot_image(slint_img);

        let overlay_weak = overlay.as_weak();
        let main_weak = window_weak.clone();
        let ctx_inner = ctx_capture.clone();
        let img_clone = raw_img.clone();

        overlay.on_capture_completed(move |x, y, sel_w, sel_h, note| {
            if let Some(main) = main_weak.upgrade() {
                // Bounds safety check for cropping
                let crop_x = (x as u32).min(w.saturating_sub(1));
                let crop_y = (y as u32).min(h.saturating_sub(1));
                let crop_w = (sel_w as u32).min(w - crop_x).max(1);
                let crop_h = (sel_h as u32).min(h - crop_y).max(1);

                let cropped = crop_imm(&img_clone, crop_x, crop_y, crop_w, crop_h).to_image();

                // Encode cropped image to PNG
                let mut png_bytes = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
                let _ = image::ImageEncoder::write_image(
                    encoder,
                    &cropped,
                    crop_w,
                    crop_h,
                    image::ExtendedColorType::Rgba8,
                );

                // Reduce image using QualityBudget
                let qb = match ctx_inner.settings_store.get(&SettingKey::QualityBudget) {
                    Ok(Some(Setting {
                        value: SettingValue::QualityBudget(budget),
                        ..
                    })) => budget,
                    _ => QualityBudget::default(),
                };
                let region_long_edge = crop_w.max(crop_h);
                let resolved = qb.resolve(region_long_edge);
                let orig_dims = ImageDimensions::new(crop_w, crop_h).unwrap_or(ImageDimensions {
                    width: crop_w,
                    height: crop_h,
                });

                let (reduced_bytes, final_w, final_h) = if let Ok(red) =
                    ImageReducer::reduce_image(&png_bytes, orig_dims, &resolved, false)
                {
                    (red.bytes, red.dimensions.width, red.dimensions.height)
                } else {
                    (png_bytes, crop_w, crop_h)
                };

                // Generate timestamp and file in Vault
                let timestamp_str = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
                let rel_filename = format!("findings/capture_{timestamp_str}.png");

                let _ = ctx_inner
                    .vault_store
                    .write_blob(&rel_filename, &reduced_bytes);

                // Save to FindingStore
                let clock = SystemClock::new();
                let entropy = SystemEntropySource::new();
                let finding_id = snapdown_core::util::id::id_from_parts(
                    clock.now_unix_millis(),
                    entropy.random_bytes_10(),
                );
                let captured_at = clock.now_rfc3339();

                let finding = Finding {
                    id: finding_id.clone(),
                    image_path: rel_filename.clone(),
                    image_width: final_w,
                    image_height: final_h,
                    captured_at: captured_at.clone(),
                    source_monitor: "DISPLAY1".to_string(),
                    region: format!("{crop_x},{crop_y},{crop_w},{crop_h}"),
                    resolved_long_edge: Some(resolved.max_long_edge),
                    resolved_encoder_quality: Some(resolved.encoder_quality),
                    budget_name: Some(qb.named.display_name().to_string()),
                };

                let note_record = Note {
                    id: format!("note-{finding_id}"),
                    finding_id: finding_id.clone(),
                    body: note.to_string(),
                    updated_at: captured_at,
                };

                let _ = ctx_inner
                    .finding_store
                    .create_finding(&finding, &note_record, &[]);

                // Reload filmstrip with the new finding active
                load_findings_into_window(&main, &ctx_inner, Some(&finding_id));

                main.show().unwrap();
            }
            if let Some(ov) = overlay_weak.upgrade() {
                ov.hide().unwrap();
            }
        });

        let overlay_cancel = overlay.as_weak();
        let main_restore = window_weak.clone();
        overlay.on_overlay_cancelled(move || {
            if let Some(ov) = overlay_cancel.upgrade() {
                ov.hide().unwrap();
            }
            if let Some(main) = main_restore.upgrade() {
                main.show().unwrap();
            }
        });

        overlay.show().unwrap();
    });

    main_window.on_open_file_clicked(|| {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        {
            println!("Selected file: {:?}", path);
        }
    });

    // --- Tray icon, global hotkeys, and startup registration ---
    let tray_icon_bytes = include_bytes!("../assets/icon.png");
    let tray_icon_rgba = image::load_from_memory(tray_icon_bytes)
        .expect("embedded tray icon must decode")
        .to_rgba8();
    let (tray_icon_w, tray_icon_h) = tray_icon_rgba.dimensions();
    let app_tray = match AppTray::new(tray_icon_rgba.into_raw(), tray_icon_w, tray_icon_h) {
        Ok(tray) => Some(tray),
        Err(e) => {
            eprintln!("Failed to create tray icon: {e}");
            None
        }
    };

    let hotkey_backend = match DesktopGlobalHotkeyBackend::new() {
        Ok(backend) => Some(Arc::new(backend) as Arc<dyn GlobalShortcutBackend>),
        Err(e) => {
            eprintln!("Failed to init global hotkey manager: {e}");
            None
        }
    };
    let mut hotkey_registrar =
        DesktopHotkeyRegistrar::new(ctx.settings_store.clone(), hotkey_backend);
    if let Err(e) = hotkey_registrar.init_from_store() {
        eprintln!("Failed to init hotkeys from store: {e}");
    }
    let hotkey_registrar = Arc::new(Mutex::new(hotkey_registrar));

    #[cfg(windows)]
    let autostart_backend: Arc<dyn AutoStartBackend> =
        match WindowsRegistryAutoStartBackend::current_executable(
            "Snapdown",
            vec!["--autostart".to_string()],
        ) {
            Ok(backend) => Arc::new(backend),
            Err(e) => {
                eprintln!("Failed to resolve current executable for autostart: {e}");
                Arc::new(WindowsRegistryAutoStartBackend::new(
                    "Snapdown",
                    PathBuf::from("Snapdown.exe"),
                    vec!["--autostart".to_string()],
                ))
            }
        };
    #[cfg(not(windows))]
    let autostart_backend: Arc<dyn AutoStartBackend> = Arc::new(NoopAutoStartBackend);

    let mut startup_registrar = DesktopStartupRegistrar::new(autostart_backend);
    let boot_clock = SystemClock::new();
    let is_autostart_launch = std::env::args().any(|arg| arg == "--autostart");
    let _ = reconcile_startup_on_boot(
        ctx.settings_store.as_ref(),
        &mut startup_registrar,
        &boot_clock,
    );

    // Per FR-18/BR-121: launching via Windows startup opens no window, tray icon only.
    if !is_autostart_launch {
        main_window.show()?;
    }

    // Poll tray-icon and global-hotkey events on the UI thread; both crates deliver events
    // through crossbeam channels rather than hooking into Slint's winit event loop directly.
    let window_for_events = main_window.as_weak();
    let hotkey_poll_registrar = hotkey_registrar.clone();
    let poll_timer = slint::Timer::default();
    poll_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(150),
        move || {
            if let Some(tray) = &app_tray {
                if let Some(action) = tray.poll_action() {
                    match action {
                        TrayAction::Capture => {
                            if let Some(win) = window_for_events.upgrade() {
                                win.show().unwrap();
                                win.invoke_capture_clicked();
                            }
                        }
                        TrayAction::OpenEditor => {
                            if let Some(win) = window_for_events.upgrade() {
                                win.show().unwrap();
                                win.window().set_minimized(false);
                            }
                        }
                        TrayAction::Settings => {
                            if let Some(win) = window_for_events.upgrade() {
                                win.show().unwrap();
                                win.window().set_minimized(false);
                                win.invoke_settings_clicked();
                            }
                        }
                        TrayAction::Quit => {
                            std::process::exit(0);
                        }
                    }
                }
            }

            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                if event.state == HotKeyState::Pressed {
                    let action = hotkey_poll_registrar.lock().ok().and_then(|registrar| {
                        registrar
                            .get_bindings()
                            .iter()
                            .find_map(|(action, shortcut)| {
                                if shortcut.is_empty() {
                                    return None;
                                }
                                HotKey::from_str(shortcut)
                                    .ok()
                                    .filter(|hk| hk.id() == event.id)
                                    .map(|_| *action)
                            })
                    });

                    if let Some(action) = action {
                        match action {
                            HotkeyAction::Capture => {
                                if let Some(win) = window_for_events.upgrade() {
                                    win.show().unwrap();
                                    win.invoke_capture_clicked();
                                }
                            }
                            HotkeyAction::OpenEditor => {
                                if let Some(win) = window_for_events.upgrade() {
                                    win.show().unwrap();
                                    win.window().set_minimized(false);
                                }
                            }
                        }
                    }
                }
            }
        },
    );

    // `run_event_loop()` quits as soon as the last window is hidden, which would tear down
    // the tray icon along with the window on close. Only the tray's Exit action (which calls
    // `std::process::exit`) or `slint::quit_event_loop()` should end the process.
    slint::run_event_loop_until_quit()?;
    Ok(())
}
