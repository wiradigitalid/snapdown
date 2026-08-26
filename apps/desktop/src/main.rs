#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod hotkey;
mod startup;
mod tray;

use std::cell::RefCell;
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

thread_local! {
    /// The capture overlays - one per monitor - kept alive for the life of the process so their
    /// renderer warm-up is paid once rather than on every capture. See [`LiveOverlay`].
    ///
    /// They need an owner that outlives the callback that created them, and they are only ever
    /// touched from the UI thread, so they live here rather than in an `Rc` that would have to
    /// cross the capture thread (it cannot: `invoke_from_event_loop` requires `Send`). Each
    /// overlay's own callbacks hold only `Weak` handles, so nothing here is kept alive by a
    /// reference cycle.
    static LIVE_OVERLAYS: RefCell<Vec<LiveOverlay>> = const { RefCell::new(Vec::new()) };

    /// Where the next overlay window should be *born*, as `(x, y, width, height)` in physical
    /// virtual-desktop pixels.
    ///
    /// Setting geometry after the window exists is not enough. Windows creates a window at the
    /// primary monitor's DPI; moving it onto a display with a different scale factor then fires
    /// `WM_DPICHANGED`, which rebuilds the surface and re-derives the size - visible as a brief
    /// "growing into place" transition on the non-primary monitor only. Feeding the position and
    /// size into `winit`'s `WindowAttributes` instead means the window is created on its target
    /// monitor already, so no DPI change ever happens.
    ///
    /// Read and cleared by the window-attributes hook installed on the backend. The hook runs on
    /// the event-loop thread, which is the same thread that sets this, so a thread-local is the
    /// right channel.
    #[cfg(windows)]
    static NEXT_OVERLAY_PLACEMENT: RefCell<Option<(i32, i32, u32, u32)>> =
        const { RefCell::new(None) };
}

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

/// A capture overlay, kept alive between captures.
///
/// Overlays are deliberately NOT recreated per capture. A GPU renderer builds each window's
/// surface and shader pipeline lazily, and that first frame gets presented with only the clear
/// colour and no content - which is the whole-screen black blink. On Windows, Slint's `hide()`
/// maps to winit's `set_visible(false)` and leaves the native window and its renderer intact (it
/// only destroys the window on Wayland, or when `SLINT_DESTROY_WINDOW_ON_HIDE` is set), so
/// holding on to these and re-showing them pays that warm-up once per monitor layout instead of
/// once per capture.
struct LiveOverlay {
    window: CaptureOverlayWindow,
    /// `(x, y, width, height)` in physical virtual-desktop pixels. Doubles as the reuse key: if
    /// the monitor layout still matches, the existing windows are reused untouched.
    placement: (i32, i32, u32, u32),
}

/// Crops one monitor's capture to the selected region, shrinks it to the active QualityBudget,
/// writes it to the Vault, and records the Finding plus its note.
///
/// `region` is in the source image's own pixel space, which for a per-monitor overlay is the
/// same space its pointer coordinates arrive in - one overlay covers exactly one monitor at 1:1,
/// so no virtual-desktop translation is involved.
///
/// Returns the new Finding's id, or `None` if the region could not be persisted.
fn persist_finding(
    ctx: &AppContext,
    source: &image::RgbaImage,
    region: (u32, u32, u32, u32),
    monitor_name: &str,
    note_body: &str,
) -> Option<String> {
    let (src_w, src_h) = (source.width(), source.height());
    let (sel_x, sel_y, sel_w, sel_h) = region;

    // Clamp into the source rather than trusting the caller: a drag released off-screen can
    // report a region reaching past the monitor's own bounds.
    let crop_x = sel_x.min(src_w.saturating_sub(1));
    let crop_y = sel_y.min(src_h.saturating_sub(1));
    let crop_w = sel_w.min(src_w - crop_x).max(1);
    let crop_h = sel_h.min(src_h - crop_y).max(1);

    let cropped = crop_imm(source, crop_x, crop_y, crop_w, crop_h).to_image();

    let mut png_bytes = Vec::new();
    if let Err(e) = image::ImageEncoder::write_image(
        image::codecs::png::PngEncoder::new(&mut png_bytes),
        &cropped,
        crop_w,
        crop_h,
        image::ExtendedColorType::Rgba8,
    ) {
        eprintln!("Failed to encode captured region as PNG: {e}");
        return None;
    }

    let qb = match ctx.settings_store.get(&SettingKey::QualityBudget) {
        Ok(Some(Setting {
            value: SettingValue::QualityBudget(budget),
            ..
        })) => budget,
        _ => QualityBudget::default(),
    };
    let resolved = qb.resolve(crop_w.max(crop_h));
    let orig_dims = ImageDimensions::new(crop_w, crop_h).unwrap_or(ImageDimensions {
        width: crop_w,
        height: crop_h,
    });

    let (reduced_bytes, final_w, final_h) =
        match ImageReducer::reduce_image(&png_bytes, orig_dims, &resolved, false) {
            Ok(red) => (red.bytes, red.dimensions.width, red.dimensions.height),
            Err(e) => {
                eprintln!("Quality-budget reduction failed, storing the region unreduced: {e}");
                (png_bytes, crop_w, crop_h)
            }
        };

    let clock = SystemClock::new();
    let entropy = SystemEntropySource::new();
    let finding_id =
        snapdown_core::util::id::id_from_parts(clock.now_unix_millis(), entropy.random_bytes_10());
    let captured_at = clock.now_rfc3339();
    let rel_filename = format!(
        "findings/capture_{}.png",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );

    if let Err(e) = ctx.vault_store.write_blob(&rel_filename, &reduced_bytes) {
        eprintln!("Failed to write the captured region into the Vault: {e}");
        return None;
    }

    let finding = Finding {
        id: finding_id.clone(),
        image_path: rel_filename,
        image_width: final_w,
        image_height: final_h,
        captured_at: captured_at.clone(),
        source_monitor: monitor_name.to_string(),
        region: format!("{crop_x},{crop_y},{crop_w},{crop_h}"),
        resolved_long_edge: Some(resolved.max_long_edge),
        resolved_encoder_quality: Some(resolved.encoder_quality),
        budget_name: Some(qb.named.display_name().to_string()),
    };

    let note_record = Note {
        id: format!("note-{finding_id}"),
        finding_id: finding_id.clone(),
        body: note_body.to_string(),
        updated_at: captured_at,
    };

    if let Err(e) = ctx
        .finding_store
        .create_finding(&finding, &note_record, &[])
    {
        eprintln!("Failed to record the Finding: {e}");
        return None;
    }

    Some(finding_id)
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

    // A custom backend is installed for one reason: the window-attributes hook, which lets each
    // capture overlay be *created* already on its target monitor. See NEXT_OVERLAY_PLACEMENT for
    // why that has to happen at creation rather than afterwards.
    //
    // The renderer is left at Slint's default, which is GPU-accelerated. A GPU renderer builds
    // each window's surface and shader pipeline lazily, so the first frame of a newly created
    // window is presented with only the clear colour - the whole-screen black blink. The answer
    // is NOT to fall back to the software renderer: that removes the blink but cannot repaint a
    // full-screen 8-megapixel overlay per pointer move, so dragging a region out becomes choppy,
    // and every attempt to make that cheap cost visual correctness. Instead the overlays are
    // created once and reused, so the warm-up is paid once - see LiveOverlay.
    //
    // SLINT_BACKEND is still honoured when set, so the renderers stay A/B comparable without a
    // rebuild (`SLINT_BACKEND=software` for the CPU path).
    #[cfg(windows)]
    {
        let mut builder =
            i_slint_backend_winit::Backend::builder().with_window_attributes_hook(|attributes| {
                match NEXT_OVERLAY_PLACEMENT.with_borrow_mut(|placement| placement.take()) {
                    Some((x, y, width, height)) => attributes
                        .with_position(i_slint_backend_winit::winit::dpi::PhysicalPosition::new(
                            x, y,
                        ))
                        .with_inner_size(i_slint_backend_winit::winit::dpi::PhysicalSize::new(
                            width, height,
                        )),
                    None => attributes,
                }
            });
        // Accepted names are matched in i-slint-backend-winit's lib.rs - "software"/"sw",
        // "skia", "femtovg" and friends. NOT "renderer-software", which `with_renderer_name`'s
        // own doc comment shows: that matches nothing, and the backend then logs "unrecognized
        // renderer ... falling back to <default>" and silently hands back the default. That log
        // goes to Slint's debug output, which a release windows-subsystem build has nowhere to
        // print, so a typo here fails completely silently.
        if let Some(name) = std::env::var_os("SLINT_BACKEND").and_then(|v| v.into_string().ok()) {
            builder = builder.with_renderer_name(name);
        }
        match builder.build() {
            Ok(backend) => {
                if let Err(e) = slint::platform::set_platform(Box::new(backend)) {
                    eprintln!("Could not install the winit backend, using the default: {e}");
                }
            }
            Err(e) => eprintln!("Could not build the winit backend, using the default: {e}"),
        }
    }

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

    // Setup Capture callback.
    //
    // One overlay window per attached monitor, each covering exactly that monitor and showing
    // only that monitor's own pixels. That is what confines the crosshair to a single screen and
    // makes a region spanning two monitors impossible to draw, and it is also the only correct
    // option on Windows: a window has exactly one DPI, so one window spanning displays at 150%
    // and 175% renders 1:1 on neither, and crossing the boundary fires WM_DPICHANGED which
    // re-derives its physical size and inflates it.
    //
    // The windows are created once and then reused for every later capture, which is what keeps
    // the GPU renderer's per-window warm-up from being visible - see LiveOverlay.
    let window_weak = main_window.as_weak();
    let ctx_capture = ctx.clone();
    main_window.on_capture_clicked(move || {
        // The monitor grab is a blocking syscall; run it off the UI thread so the window does
        // not freeze while it runs, then hop back onto the event loop to build the overlays.
        let main_weak = window_weak.clone();
        let ctx_inner = ctx_capture.clone();
        std::thread::spawn(move || {
            let capture_result = RegionCapturer::capture_each_monitor();
            if let Err(e) = slint::invoke_from_event_loop(move || {
                let captures = match capture_result {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Capture failed: {e}");
                        return;
                    }
                };

                // Take the overlays out while they are being reconfigured, so nothing else can
                // borrow the thread-local mid-flight. They go back at the end.
                let mut live: Vec<LiveOverlay> = LIVE_OVERLAYS.with_borrow_mut(std::mem::take);

                let wanted: Vec<(i32, i32, u32, u32)> = captures
                    .iter()
                    .map(|c| (c.origin_x, c.origin_y, c.width, c.height))
                    .collect();

                // Reuse only while the monitor layout is unchanged. If a display was added,
                // removed, moved or re-scaled, the old windows are the wrong shape and the set is
                // rebuilt - the warm-up cost is paid again, which is correct: it is a new layout.
                let layout_unchanged = live.len() == wanted.len()
                    && live.iter().zip(&wanted).all(|(l, w)| l.placement == *w);

                if !layout_unchanged {
                    // Drop the stale windows before creating replacements, so a monitor's overlay
                    // is never briefly duplicated.
                    live.clear();
                    for placement in &wanted {
                        // Publish the placement so the attributes hook can create this window
                        // directly on its target monitor, at that monitor's DPI.
                        NEXT_OVERLAY_PLACEMENT.with_borrow_mut(|slot| *slot = Some(*placement));
                        let overlay = match CaptureOverlayWindow::new() {
                            Ok(ov) => ov,
                            Err(e) => {
                                eprintln!("Failed to create CaptureOverlayWindow: {e}");
                                continue;
                            }
                        };
                        live.push(LiveOverlay {
                            window: overlay,
                            placement: *placement,
                        });
                    }
                    NEXT_OVERLAY_PLACEMENT.with_borrow_mut(|slot| *slot = None);
                }

                if live.is_empty() {
                    eprintln!("Capture aborted: no overlay window could be created.");
                    return;
                }

                // Weak handles only, so the callbacks that close every overlay do not keep any
                // overlay alive.
                let siblings: Rc<Vec<slint::Weak<CaptureOverlayWindow>>> =
                    Rc::new(live.iter().map(|l| l.window.as_weak()).collect());

                for (self_index, (entry, capture)) in live.iter().zip(captures).enumerate() {
                    let overlay = &entry.window;
                    let monitor_name = capture.name;
                    let monitor_image = Rc::new(capture.image);

                    // A reused overlay still carries the previous capture's state, so reset it
                    // before it is shown again.
                    overlay.set_interactive(true);
                    overlay.set_has_selection(false);
                    overlay.set_is_narrating(false);
                    overlay.set_is_dragging(false);
                    overlay.set_note_text(slint::SharedString::new());
                    overlay.set_snapshot_image(slint::Image::from_rgba8(SharedPixelBuffer::<
                        slint::Rgba8Pixel,
                    >::clone_from_slice(
                        monitor_image.as_raw(),
                        capture.width,
                        capture.height,
                    )));

                    // Belt-and-braces geometry for the case where the attributes hook did not
                    // apply. The size MUST be given in LOGICAL pixels derived from this monitor's
                    // own scale factor: set_size(Physical) divides by the window's *current*
                    // scale factor, which is 1.0 before the window exists, so physical numbers
                    // would be stored as logical ones and then multiplied by the real scale
                    // factor - the original "everything is zoomed and grows into place" bug.
                    overlay
                        .window()
                        .set_position(slint::WindowPosition::Physical(
                            slint::PhysicalPosition::new(capture.origin_x, capture.origin_y),
                        ));
                    overlay
                        .window()
                        .set_size(slint::WindowSize::Logical(slint::LogicalSize::new(
                            capture.width as f32 / capture.scale_factor,
                            capture.height as f32 / capture.scale_factor,
                        )));

                    // Callbacks are reinstalled every capture: a reused overlay's old handlers
                    // still close over the PREVIOUS capture's image, which would crop this
                    // capture's region out of a stale screenshot. Setting a handler replaces it.
                    let close_all = {
                        let siblings = siblings.clone();
                        move || {
                            // Take every overlay off the screen in this one pass, before any
                            // Slint-level bookkeeping.
                            //
                            // `hide()` alone is not enough: Slint applies it per window on that
                            // window's own event-loop turn, so the overlay that did not have
                            // focus stayed on screen for a visible beat after the focused one
                            // vanished - cancelling looked like it only closed one monitor. Going
                            // straight to winit's set_visible(false) hides them all now, in one
                            // turn, and the hide() below then keeps Slint's own state in step.
                            #[cfg(windows)]
                            {
                                use i_slint_backend_winit::WinitWindowAccessor;
                                for sibling in siblings.iter() {
                                    if let Some(sibling) = sibling.upgrade() {
                                        sibling.window().with_winit_window(|winit_win| {
                                            winit_win.set_visible(false)
                                        });
                                    }
                                }
                            }
                            for sibling in siblings.iter() {
                                if let Some(sibling) = sibling.upgrade() {
                                    if let Err(e) = sibling.hide() {
                                        eprintln!("Failed to hide a capture overlay: {e}");
                                    }
                                }
                            }
                        }
                    };

                    // Selecting on one monitor stands the others down, so only one screen can be
                    // mid-selection while its note popup is open.
                    overlay.on_narration_started({
                        let siblings = siblings.clone();
                        move || {
                            for (index, sibling) in siblings.iter().enumerate() {
                                if index == self_index {
                                    continue;
                                }
                                if let Some(sibling) = sibling.upgrade() {
                                    sibling.set_interactive(false);
                                }
                            }
                        }
                    });

                    overlay.on_capture_completed({
                        let ctx_inner = ctx_inner.clone();
                        let main_weak = main_weak.clone();
                        let monitor_image = monitor_image.clone();
                        let close_all = close_all.clone();
                        move |x, y, sel_w, sel_h, note| {
                            let region = (
                                x.max(0) as u32,
                                y.max(0) as u32,
                                sel_w.max(0) as u32,
                                sel_h.max(0) as u32,
                            );
                            let finding_id = persist_finding(
                                &ctx_inner,
                                &monitor_image,
                                region,
                                &monitor_name,
                                note.as_str(),
                            );
                            close_all();
                            if let Some(main) = main_weak.upgrade() {
                                load_findings_into_window(&main, &ctx_inner, finding_id.as_deref());
                                if let Err(e) = main.show() {
                                    eprintln!("Failed to reshow the main window: {e}");
                                }
                            }
                        }
                    });

                    overlay.on_overlay_cancelled({
                        let main_weak = main_weak.clone();
                        move || {
                            close_all();
                            if let Some(main) = main_weak.upgrade() {
                                if let Err(e) = main.show() {
                                    eprintln!("Failed to reshow the main window: {e}");
                                }
                            }
                        }
                    });
                }

                // Show them in one tight pass, after every one is fully configured.
                for entry in &live {
                    if let Err(e) = entry.window.show() {
                        eprintln!("Failed to show a capture overlay: {e}");
                    }
                }

                // The first overlay takes keyboard focus, so Escape works without a click first.
                #[cfg(windows)]
                {
                    use i_slint_backend_winit::WinitWindowAccessor;
                    if let Some(entry) = live.first() {
                        entry
                            .window
                            .window()
                            .with_winit_window(|winit_win| winit_win.focus_window());
                    }
                }

                LIVE_OVERLAYS.with_borrow_mut(|slot| *slot = live);
            }) {
                eprintln!("Failed to dispatch the capture overlays to the UI thread: {e}");
            }
        });
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
