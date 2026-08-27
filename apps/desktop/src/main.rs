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
use slint::{ComponentHandle, Model, ModelRc, SharedPixelBuffer, VecModel};
use snapdown_capture::{CaptureTarget, RegionCapturer};
use snapdown_core::domain::finding::{Finding, Note, Region};
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
    /// The capture overlay, kept alive for the life of the process so its renderer warm-up is
    /// paid once rather than on every capture. See [`LiveOverlay`].
    ///
    /// A `Vec` for one element, because the desktop layout is the reuse key and a changed layout
    /// replaces the entry wholesale. It needs an owner that outlives the callback that created
    /// it, and it is only ever touched from the UI thread, so it lives here rather than in an
    /// `Rc` that would have to cross the capture thread (it cannot: `invoke_from_event_loop`
    /// requires `Send`). Its callbacks hold only a `Weak` handle, so nothing here is kept alive
    /// by a reference cycle.
    static LIVE_OVERLAYS: RefCell<Vec<LiveOverlay>> = const { RefCell::new(Vec::new()) };

    /// Where the overlay window should be *born*, as `(x, y, width, height)` in physical
    /// virtual-desktop pixels.
    ///
    /// Setting geometry after the window exists is not enough. Windows creates a window at the
    /// primary monitor's DPI; moving it so it spans a display with a different scale factor then
    /// fires `WM_DPICHANGED`, which rebuilds the surface and re-derives the size - seen as the
    /// overlay growing into place. Feeding position and size into `winit`'s `WindowAttributes`
    /// instead means the window is created covering the desktop already.
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

/// The largest a filmstrip thumbnail is ever drawn: a 120x86 card at 2x for a HiDPI display.
///
/// Every card used to hold the Finding's image at FULL resolution. Measured on the owner's library of
/// 61 Findings: 230.8 MB of decoded RGBA held resident, to fill cards of 120x86 - and it grew with
/// every capture, on top of the 175.8 MB the overlay retains by design. That is most of where 650 MB
/// of private bytes was going.
///
/// `DynamicImage::thumbnail` is the cheap box-filter path, which is the right trade for a 120px card.
const THUMB_MAX_W: u32 = 240;
const THUMB_MAX_H: u32 = 172;

// Checked at COMPILE time rather than in a test. A library of a few hundred Findings holds one
// decoded thumbnail each for the life of the window, so the bound is an invariant of the build, not
// something to discover in CI - and clippy is right that asserting it at runtime asserts a constant.
const _: () = assert!(
    (THUMB_MAX_W as u64) * (THUMB_MAX_H as u64) * 4 < 200 * 1024,
    "one thumbnail must cost under 200 KB decoded: 230.8 MB of full-size images for 61 Findings is      what this bound exists to prevent"
);
const _: () = assert!(
    THUMB_MAX_W >= 240 && THUMB_MAX_H >= 172,
    "and it must still be at least the 120x86 card at 2x, or the filmstrip is visibly soft on a      HiDPI display"
);

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
                rgba_to_slint_image(&dyn_img.thumbnail(THUMB_MAX_W, THUMB_MAX_H).to_rgba8())
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

    if let Some(active_id) = target_active_id {
        load_active_detail(window, ctx, &active_id);
    }
}

/// The selected width and height out of a Finding's `region` column, which holds `"x,y,w,h"`.
///
/// `persist_finding` writes it as `format!("{crop_x},{crop_y},{crop_w},{crop_h}")` - a bare
/// comma-separated string, NOT the JSON form of `Region`. The first attempt at reading it here used
/// `serde_json::from_str::<Region>`, which compiles, raises nothing visible, and always returns
/// `None`: the readout would simply never have shown the reduction. Worse, the guard written
/// alongside it asserted the SHAPE OF THE CODE rather than the shape of the data, so it passed.
///
/// Caught by reading the live database. `AGENTS.md` records four earlier cases of code that compiled,
/// tested green and did nothing; this is the same shape, found before shipping only because the
/// database was opened to check something else.
fn parse_region_field(region: &str) -> Option<(u32, u32)> {
    let mut parts = region.split(',');
    let _x = parts.next()?;
    let _y = parts.next()?;
    let w = parts.next()?.trim().parse::<u32>().ok()?;
    let h = parts.next()?.trim().parse::<u32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((w, h))
}

/// Loads ONE Finding into the Editor: its image, its dimensions, its note, its markers.
///
/// Split out of `load_findings_into_window` because selecting a Finding used to run the whole of it,
/// and the whole of it decodes every thumbnail in the library. Measured on this machine with 58
/// Findings: 320.9 ms per click, of which 310.0 ms was thumbnail decoding and 1.3 ms was the database
/// query. None of that work changes when the selection changes, and the cost grows with every
/// capture the owner takes.
fn load_active_detail(window: &AppWindow, ctx: &AppContext, active_id: &str) {
    let Ok(Some(detail)) = ctx.finding_store.get_finding(active_id) else {
        return;
    };
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
    window.set_active_finding_id(f.id.clone().into());

    // Says what was saved, and - when the Quality Budget shrank it - what it was saved FROM.
    //
    // The overlay reports the region you are selecting, in source pixels; this reports the file on
    // disk, after `ImageReducer` has applied the budget. Those legitimately differ, and with only
    // one number visible the difference read as one of them being wrong. The region is already
    // recorded on the Finding, so both can be shown.
    let selected = parse_region_field(&f.region);
    let resolution = match selected {
        Some((w, h)) if w != f.image_width || h != f.image_height => {
            format!(
                "{} × {} px  ·  from {} × {}",
                f.image_width, f.image_height, w, h
            )
        }
        _ => format!("{} × {} px", f.image_width, f.image_height),
    };
    window.set_resolution_text(resolution.into());

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

    window.set_observation_summary(detail.note.body.clone().into());

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

/// Selects a Finding already present in the filmstrip.
///
/// Loads its detail and moves the `is_active` flag by writing the two affected rows in place. What it
/// deliberately does NOT do is rebuild the filmstrip model, because building that model re-decodes
/// every image in the library - 310 ms of the 321 ms a click used to cost here, and rising with each
/// capture. The set of Findings has not changed, so neither has the strip.
fn select_active_finding(window: &AppWindow, ctx: &AppContext, id: &str) {
    let items = window.get_filmstrip_items();
    if let Some(vec_model) = items.as_any().downcast_ref::<VecModel<FindingThumb>>() {
        for row in 0..vec_model.row_count() {
            let Some(thumb) = vec_model.row_data(row) else {
                continue;
            };
            let should_be_active = thumb.id == id;
            if thumb.is_active != should_be_active {
                vec_model.set_row_data(
                    row,
                    FindingThumb {
                        is_active: should_be_active,
                        ..thumb
                    },
                );
            }
        }
    } else {
        // The model is not the one this function knows how to update in place. Falling back to the
        // full rebuild is slow rather than wrong, and silently doing nothing would be wrong.
        load_findings_into_window(window, ctx, Some(id));
        return;
    }

    load_active_detail(window, ctx, id);
}

thread_local! {
    /// The containers a single click may take, in CANVAS pixels, smallest first.
    ///
    /// Filled by a detection thread that runs alongside the grab, and read by the overlay's
    /// `target-at` hit test. It is deliberately NOT waited for: precomputing these costs 180-345ms
    /// on a busy desktop, more than the grab itself, and the overlay needs them only by the time
    /// the Reviewer starts moving the pointer - not to appear. Until they land, dragging works and
    /// click-to-select simply finds nothing.
    ///
    /// Precomputed rather than queried live because the overlay is the topmost window once it is
    /// up, so a point-based query would return our own window. Enumerating by HWND instead makes
    /// z-order irrelevant, which is what lets this run late.
    static CAPTURE_TARGETS: RefCell<Vec<CaptureTarget>> = const { RefCell::new(Vec::new()) };
}

/// Hides a window from screen-capture APIs, or stops hiding it, without moving it on screen.
///
/// Returns nothing on purpose: there is no useful recovery. `WDA_EXCLUDEFROMCAPTURE` needs Windows
/// 10 2004 or newer, and where it is unavailable the consequence is that Snapdown appears in its
/// own screenshot - the behaviour before this existed. That is worth a line on stderr, not a
/// failed capture.
#[cfg(windows)]
fn set_capture_exclusion(window: &slint::Weak<AppWindow>, exclude: bool) {
    use i_slint_backend_winit::winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use i_slint_backend_winit::WinitWindowAccessor;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
    };

    let Some(main) = window.upgrade() else {
        return;
    };
    let affinity = if exclude {
        WDA_EXCLUDEFROMCAPTURE
    } else {
        WDA_NONE
    };

    let applied = main.window().with_winit_window(|winit_win| {
        let Ok(handle) = winit_win.window_handle() else {
            return false;
        };
        let RawWindowHandle::Win32(win32) = handle.as_raw() else {
            return false;
        };
        let hwnd = HWND(win32.hwnd.get() as *mut std::ffi::c_void);
        unsafe { SetWindowDisplayAffinity(hwnd, affinity) }.is_ok()
    });

    if applied != Some(true) {
        eprintln!(
            "Could not {} the Editor from screen capture; it may appear in its own screenshot.",
            if exclude { "exclude" } else { "restore" }
        );
    }
}

#[cfg(not(windows))]
fn set_capture_exclusion(_window: &slint::Weak<AppWindow>, _exclude: bool) {}

/// The capture overlay, kept alive between captures.
///
/// It is deliberately NOT recreated per capture. A GPU renderer builds a window's surface and
/// shader pipeline lazily, and that first frame gets presented with only the clear colour and no
/// content - the whole-screen black blink. On Windows, Slint's `hide()` maps to winit's
/// `set_visible(false)` and leaves the native window and its renderer intact (it only destroys
/// the window on Wayland, or when `SLINT_DESTROY_WINDOW_ON_HIDE` is set), so holding on to it and
/// re-showing pays that warm-up once per desktop layout instead of once per capture.
struct LiveOverlay {
    window: CaptureOverlayWindow,
    /// `(x, y, width, height)` in physical virtual-desktop pixels. Doubles as the reuse key: if
    /// the desktop layout still matches, the existing window is reused untouched.
    placement: (i32, i32, u32, u32),
    /// The canvas on screen: the desktop as it was when this capture started.
    ///
    /// ONE buffer, allocated fresh for every capture off the event loop, replacing a pair of
    /// permanently retained ones. Both of the earlier designs were about the same hazard and this
    /// one removes it rather than working around it.
    ///
    /// `make_mut_bytes` is copy-on-write, so writing the canvas is only cheap while nothing else
    /// holds it - and something always does. Slint's bindings are lazy: the backdrop and the lens
    /// keep the `source` they latched at their last render, and a hidden window never renders, so
    /// clearing the overlay's `snapshot-image` property releases nothing. The first version reused a
    /// single buffer and silently paid a 92 MB memcpy per capture (the canary in the blit caught it,
    /// 28 times). The second alternated two buffers so the one being written was always a generation
    /// older than anything still held - correct, and 175.8 MB resident for ever.
    ///
    /// A buffer that has just been allocated has a refcount of one by construction, so there is
    /// nothing to alternate away from. The cost moved instead of growing: 33-37 ms of allocation per
    /// capture, paid on its own thread alongside the 132-167 ms grab, where it does not show up in
    /// wall clock. Idle drops by 87.9 MB on this desktop, and the peak during a capture - the old
    /// canvas plus the new one - is the profile Snagit has: low at rest, a spike while in use.
    canvas: Option<SharedPixelBuffer<slint::Rgba8Pixel>>,
}

impl Drop for LiveOverlay {
    /// Hides the native window before the handle goes away.
    ///
    /// A desktop layout change replaces this entry, and dropping the handle is what should close the
    /// window - but Slint routes window destruction through the event loop, and this drop happens
    /// INSIDE an event-loop callback, so the close is scheduled rather than done. Hiding first takes
    /// the window off screen immediately, whatever the destruction is waiting for.
    ///
    /// The canvas needs no such care: it is a plain `SharedPixelBuffer` and drops with this struct.
    fn drop(&mut self) {
        if let Err(e) = self.window.hide() {
            eprintln!("Could not hide the outgoing capture overlay: {e}");
        }
    }
}

/// Crops the capture canvas to the selected region, shrinks it to the active QualityBudget,
/// writes it to the Vault, and records the Finding plus its note.
///
/// `source` is the canvas as raw RGBA8 bytes, `source_size` pixels, and `region` is in that
/// canvas's own pixel space - the same space the overlay reports its selection in, so no scale
/// conversion is involved. It is borrowed rather than owned as an `RgbaImage` because the canvas
/// now lives in the buffer the overlay presents, and owning a second copy of it is exactly the
/// cost `BUG-28` was about.
///
/// Returns the new Finding's id, or `None` if the region could not be persisted.
fn persist_finding(
    ctx: &AppContext,
    source: &[u8],
    source_size: (u32, u32),
    region: (u32, u32, u32, u32),
    monitor_name: &str,
    note_body: &str,
) -> Option<String> {
    let (src_w, src_h) = source_size;
    let (sel_x, sel_y, sel_w, sel_h) = region;

    // Clamp into the source rather than trusting the caller: a drag released off-screen can
    // report a region reaching past the canvas's own bounds.
    let crop_x = sel_x.min(src_w.saturating_sub(1));
    let crop_y = sel_y.min(src_h.saturating_sub(1));
    let crop_w = sel_w.min(src_w - crop_x).max(1);
    let crop_h = sel_h.min(src_h - crop_y).max(1);

    let cropped = match RegionCapturer::crop_rgba_from_slice(
        source,
        src_w,
        src_h,
        &Region::new(crop_x as i32, crop_y as i32, crop_w, crop_h),
    ) {
        Ok(cropped) => cropped,
        Err(e) => {
            eprintln!("Failed to crop the selected region out of the capture: {e}");
            return None;
        }
    };

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

/// Creates, places and then hides the capture overlay, so the first Capture has nothing left to
/// build. See the call site for why this cannot wait until the overlay is actually wanted.
///
/// Failing here is not fatal: the capture path creates the window itself if it finds none, which
/// is the behaviour this is optimising away rather than depending on.
#[cfg(windows)]
fn prewarm_capture_overlay() {
    let (origin_x, origin_y, width, height) = match RegionCapturer::virtual_desktop_bounds() {
        Ok(bounds) => bounds,
        Err(e) => {
            eprintln!("Could not read the desktop bounds to pre-warm the overlay: {e}");
            return;
        }
    };
    let placement = (origin_x, origin_y, width, height);

    let overlay = match CaptureOverlayWindow::new() {
        Ok(overlay) => overlay,
        Err(e) => {
            eprintln!("Could not pre-warm the capture overlay: {e}");
            return;
        }
    };

    NEXT_OVERLAY_PLACEMENT.with_borrow_mut(|slot| *slot = Some(placement));
    if let Err(e) = overlay.show() {
        eprintln!("Could not show the capture overlay to pre-warm it: {e}");
        return;
    }

    // Deliberately NOT a zero-length delay.
    //
    // This runs before `run()` starts the event loop, so a 0ms timer fires before the loop has
    // created the native window - measured: the overlay stayed at Slint's default 800x600 and
    // with_winit_window() had nothing to act on. The capture path can use 0ms because by then the
    // loop is already running. A short wait here is free: nothing is visible either way, and if
    // it still loses the race the capture path re-asserts the same geometry anyway.
    let overlay_weak = overlay.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(400), move || {
        NEXT_OVERLAY_PLACEMENT.with_borrow_mut(|slot| *slot = None);
        let Some(overlay) = overlay_weak.upgrade() else {
            return;
        };
        {
            use i_slint_backend_winit::winit::dpi::{PhysicalPosition, PhysicalSize};
            use i_slint_backend_winit::WinitWindowAccessor;
            let applied = overlay.window().with_winit_window(|winit_win| {
                winit_win.set_outer_position(PhysicalPosition::new(origin_x, origin_y));
                let _ = winit_win.request_inner_size(PhysicalSize::new(width, height));
            });
            if applied.is_none() {
                // Expected, not a fault: `show()` does not create the native window - the event
                // loop does, on its next turn - so the first pre-warm attempt runs before there is
                // a window to place. The geometry timer places it a turn later. Third time this
                // string has been mangled by a line-length edit; it is one literal on one line now.
                eprintln!(
                    "Pre-warm: no winit window yet; the geometry timer will place the overlay."
                );
            }
        }
        if let Err(e) = overlay.hide() {
            eprintln!("Could not hide the pre-warmed capture overlay: {e}");
        }
    });

    LIVE_OVERLAYS.with_borrow_mut(|live| {
        *live = vec![LiveOverlay {
            window: overlay,
            placement,
            // Nothing allocated here any more. The window's renderer warm-up and its geometry
            // correction are what start-up is paying for; the canvas is allocated per capture, off
            // the event loop, and holding one here would only be 87.9 MB waiting for a capture that
            // may never come. See `LiveOverlay::canvas`.
            canvas: None,
        }]
    });
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

    // Bring the capture overlay into existence NOW, at start-up, rather than on first Capture.
    //
    // Two things only happen when a window is first created, and both were happening in front of
    // the user: the renderer builds its surface and pipeline (a blink), and the geometry is
    // corrected one event-loop turn after creation, because `show()` does not create the native
    // window - the loop does, on its next turn. That correction is what appeared as the overlay
    // growing into place on the non-primary monitor.
    //
    // Doing it here pays both costs while the application is still starting, and the capture path
    // then finds a ready window in LIVE_OVERLAYS and merely shows it.
    #[cfg(windows)]
    prewarm_capture_overlay();

    // Populate initial Filmstrip from Vault
    load_findings_into_window(&main_window, &ctx, None);

    // Finding selection from Filmstrip
    let win_weak_sel = main_window.as_weak();
    let ctx_sel = ctx.clone();
    main_window.on_finding_selected(move |id| {
        if let Some(win) = win_weak_sel.upgrade() {
            select_active_finding(&win, &ctx_sel, &id);
        }
    });

    // An edited Observation Summary is written through to the store.
    //
    // It used to live only in the Slint property, so selecting another Finding overwrote it and the
    // edit was gone. Silent data loss, and the kind a Reviewer discovers long afterwards - which is
    // why this writes on every edit rather than on some blur or close event that may never fire. A
    // note is a few hundred bytes and the write is sub-millisecond; a debounce here would be a
    // second place for the text to be lost.
    let win_weak_note = main_window.as_weak();
    let ctx_note = ctx.clone();
    main_window.on_observation_edited(move |body| {
        let Some(win) = win_weak_note.upgrade() else {
            return;
        };
        let id = win.get_active_finding_id().to_string();
        if id.is_empty() {
            return;
        }
        let now = SystemClock::new().now_rfc3339();
        if let Err(e) = ctx_note.finding_store.update_note(&id, body.as_str(), &now) {
            eprintln!("Could not save the note for Finding {id}: {e}");
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
    // ONE overlay window covering the whole virtual desktop, reused for every capture.
    //
    // It was one window per monitor, to give each its own DPI. That fixed the mixed-DPI
    // rendering but bought a class of multi-window defects with it: each window had its own
    // renderer to warm up, which showed as the overlay blinking on the first pointer move of
    // every capture, and its own event-loop turn, so hiding them was never simultaneous and
    // cancelling appeared to clear only one screen. Both are structural to having N windows, and
    // several attempts to paper over them failed - see BUG-27.
    //
    // A single window costs nothing in quality: for a per-monitor-DPI-aware process a window's
    // surface maps 1:1 onto desktop pixels, so a canvas at native physical resolution drawn
    // full-bleed into a window sized in physical pixels is pixel-exact on every monitor whatever
    // their scale factors. Confining the crosshair to one screen, and refusing a region that
    // spans two, are done in the overlay from the monitor rectangles instead of by geometry.
    let window_weak = main_window.as_weak();
    let ctx_capture = ctx.clone();
    main_window.on_capture_clicked(move || {
        // Get Snapdown out of its own screenshot - by making it invisible to the CAPTURE rather
        // than invisible to the Reviewer.
        //
        // Hiding the window was tried first and is the obvious move, but it cannot be timed
        // honestly: `hide()` reaches `ShowWindow` at once, while the desktop only stops containing
        // those pixels after the compositor has presented a frame without them AND whatever was
        // underneath has repainted. A 60ms wait left the owner reporting leftover window shadow in
        // the overlay, and the only way to make a wait reliable is to make it long - which hands
        // back the latency `BUG-28` spent so much measurement removing.
        //
        // `WDA_EXCLUDEFROMCAPTURE` sidesteps the timing entirely: DWM composes the frame that
        // capture APIs see without this window in it, shadow included, from the moment the call
        // returns. Nothing moves on screen, so there is no repaint to wait for and no flicker when
        // the Editor comes back. The overlay covers the desktop anyway, so the Reviewer cannot tell
        // the difference.
        //
        // It also removes a hazard rather than adding one: with nothing hidden, no failure path can
        // leave the Reviewer looking at a product with no window.
        let main_weak = window_weak.clone();
        let ctx_inner = ctx_capture.clone();

        set_capture_exclusion(&main_weak, true);

        // Work out the click-to-select containers on their own thread, concurrently with the grab
        // and WITHOUT the overlay waiting for them. On a busy desktop this takes 180-345ms - longer
        // than the grab - so making the overlay wait would hand back everything BUG-28 bought. The
        // overlay does not need them to appear, only by the time the pointer starts moving.
        std::thread::spawn(|| {
            let targets = RegionCapturer::detect_capture_targets();
            if let Err(e) = slint::invoke_from_event_loop(move || {
                // Reported in virtual-desktop coordinates; the overlay works in canvas pixels,
                // whose origin is the desktop's top-left. That origin is the placement the overlay
                // was built with, so it is read from there rather than recomputed.
                let origin = LIVE_OVERLAYS
                    .with_borrow(|live| live.first().map(|entry| (entry.placement.0, entry.placement.1)));
                let Some((origin_x, origin_y)) = origin else {
                    return;
                };
                let local: Vec<CaptureTarget> = targets
                    .into_iter()
                    .map(|t| CaptureTarget {
                        region: Region::new(
                            t.region.x - origin_x,
                            t.region.y - origin_y,
                            t.region.width,
                            t.region.height,
                        ),
                        depth: t.depth,
                    })
                    .collect();
                CAPTURE_TARGETS.with_borrow_mut(|slot| *slot = local);
            }) {
                eprintln!("Could not hand the capture targets to the overlay: {e}");
            }
        });

        std::thread::spawn(move || {
            // The grab is the blocking syscall and it stays here, off the event loop. What is
            // deliberately NOT done here any more is building the canvas.
            //
            // The canvas used to be blended into an `RgbaImage` and then copied into the toolkit's
            // buffer - 83-91ms on a 6000x3840 two-monitor desktop, measured in release. Writing the
            // monitors straight into a freshly allocated `SharedPixelBuffer` took that to 36-38ms.
            // But 33-37ms of what was left is the ALLOCATION, not the writing: `SharedVector`'s
            // `FromIterator` is a per-element push loop, not a `calloc`. So the buffer is now
            // allocated once per desktop layout and reused, which measures 4.3-4.6ms per capture -
            // and reusing it means writing into state that lives on the UI thread, so the blit
            // moved there with it. 4.5ms on the event loop is a third of a frame; the 92 MB of
            // memcpy it replaced was not. See `BUG-28`.
            //
            // The allocation now happens HERE rather than being avoided, and on its own thread so it
            // overlaps the grab instead of adding to it. `virtual_desktop_bounds` is a cheap
            // enumeration - it is what the start-up pre-warm uses - so the size is known before the
            // grab begins. If the desktop changes shape between the two, the size will not match and
            // the UI thread allocates instead; that is a display being plugged in mid-capture, and
            // paying 37 ms for it is right.
            let planned = RegionCapturer::virtual_desktop_bounds().ok();
            let allocating = planned.map(|(_, _, width, height)| {
                std::thread::spawn(move || {
                    (
                        width,
                        height,
                        SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height),
                    )
                })
            });

            let capture_result = RegionCapturer::capture_virtual_desktop();

            let prepared = allocating.and_then(|handle| match handle.join() {
                Ok(prepared) => Some(prepared),
                Err(_) => {
                    eprintln!("The canvas allocation thread panicked; allocating on the event loop.");
                    None
                }
            });

            if let Err(e) = slint::invoke_from_event_loop(move || {
                // The Editor is still on screen, merely excluded from capture, so the only thing
                // an early return owes it is that exclusion being lifted again.
                let restore_editor = {
                    let main_weak = main_weak.clone();
                    move || set_capture_exclusion(&main_weak, false)
                };

                let captured = match capture_result {
                    Ok(captured) => captured,
                    Err(e) => {
                        eprintln!("Capture failed: {e}");
                        restore_editor();
                        return;
                    }
                };

                let placement = (
                    captured.origin_x,
                    captured.origin_y,
                    captured.width,
                    captured.height,
                );

                let mut live: Vec<LiveOverlay> = LIVE_OVERLAYS.with_borrow_mut(std::mem::take);

                // The desktop's shape decides whether the window has to be RESIZED, never whether
                // it has to be rebuilt.
                //
                // It used to be rebuilt: `live.clear()` then a fresh `CaptureOverlayWindow`. That
                // leaked, and the instrumentation is what settled it rather than argument. Over five
                // captures with four resolution changes the log read four creations and four drops -
                // so `LiveOverlay` and its canvas WERE released every time, and the process still
                // climbed 173 -> 370 -> 384 MB at the same resolution. Two candidate mechanisms were
                // refuted outright by the same log: no strong handle was outstanding, and the
                // pre-allocated canvas never once mismatched.
                //
                // What is left is that dropping a Slint `ComponentHandle` does not free the native
                // window's renderer resources. The jumps were +181 MB and +174 MB per cycle, which is
                // two 87.9 MB allocations - a 6000x3840 framebuffer and its snapshot texture - so the
                // surface of every window ever created is still alive.
                //
                // Rather than fight Slint's destruction semantics, this stops creating a second
                // window at all: one overlay window per process, moved and resized as the desktop
                // changes. That also removes the renderer warm-up on a layout change, which was a
                // cost this code accepted in writing.
                //
                // Resizing is safe HERE and was not safe at creation, which is the distinction
                // `BUG-26` records: `scale_factor()` returns 1.0 until a window has been realised, so
                // sizing a brand-new window from it asked for 6000x3840 LOGICAL pixels and got a
                // 9000x5760 window. This window has been shown, so its scale factor is real.
                let layout_unchanged = live.len() == 1 && live[0].placement == placement;

                if !layout_unchanged {
                    if let Some(entry) = live.first_mut() {
                        // Nothing to do but record the new shape.
                        //
                        // No resize call belongs here, and reaching for one was the wrong instinct:
                        // the geometry timer further down already re-asserts this capture's placement
                        // through winit in DEVICE pixels, on every capture, precisely because that
                        // path needs no scale factor and so cannot be wrong. A Slint-level resize
                        // would have needed `scale_factor()`, which is the call `BUG-26` is about -
                        // and the guard for it caught the attempt within the minute.
                        entry.placement = placement;
                        // The canvas is the old desktop's size and is about to be replaced anyway;
                        // releasing it here keeps the peak to one buffer rather than two.
                        entry.canvas = None;
                    } else {
                        // No window at all - the start-up pre-warm did not manage to build one.
                        //
                        // The placement is deliberately NOT published here. `new()` does not create
                        // the native window; `show()` does, one event-loop turn later, and the
                        // attributes hook reads the placement then. Publishing it around `new()` is
                        // exactly the mistake `BUG-26` records - consumed and gone before the hook
                        // runs, the window created at a default size, the overlay zoomed. It is
                        // published further down, immediately before `show()`.
                        match CaptureOverlayWindow::new() {
                            Ok(window) => live.push(LiveOverlay {
                                window,
                                placement,
                                canvas: None,
                            }),
                            Err(e) => eprintln!("Failed to create CaptureOverlayWindow: {e}"),
                        }
                    }
                }

                let Some(entry) = live.first_mut() else {
                    eprintln!("Capture aborted: the overlay window could not be created.");
                    restore_editor();
                    return;
                };

                // Take the buffer the grab thread allocated, if it is the right size for the desktop
                // that was actually captured.
                //
                // Dropping the previous canvas here rather than at close is deliberate: it is what
                // `persist_finding` reads, and a Finding is written before the overlay closes.
                let mut canvas = match prepared {
                    Some((w, h, buffer)) if w == captured.width && h == captured.height => buffer,
                    _ => {
                        // The desktop changed shape while the grab was running, or the allocating
                        // thread died. Rare, and 37 ms on the event loop beats abandoning the
                        // capture.
                        SharedPixelBuffer::<slint::Rgba8Pixel>::new(captured.width, captured.height)
                    }
                };
                entry.canvas = None;

                let buffer_before = canvas.as_bytes().as_ptr();
                if let Err(e) = captured.blit_into(canvas.make_mut_bytes()) {
                    eprintln!("Capture aborted: the desktop did not fit the overlay canvas: {e}");
                    LIVE_OVERLAYS.with_borrow_mut(|slot| *slot = live);
                    restore_editor();
                    return;
                }
                // Checked rather than trusted, because the failure is invisible: a copy-on-write
                // yields perfectly correct pixels and costs ~40 ms. This line is what caught the
                // single-reused-buffer version after a unit test had said it was fine, and it still
                // earns its place - a fresh buffer has a refcount of one, so a copy here would mean
                // something clones the canvas between allocation and blit.
                if !std::ptr::eq(buffer_before, canvas.as_bytes().as_ptr()) {
                    eprintln!(
                        "The capture canvas was copied instead of written in place - something already holds a clone of it. Correct, but ~40ms per capture slower (BUG-28)."
                    );
                }
                entry
                    .window
                    .set_snapshot_image(slint::Image::from_rgba8(canvas.clone()));
                entry.canvas = Some(canvas);

                let overlay = &entry.window;

                // A reused overlay still carries the previous capture's state.
                overlay.set_has_selection(false);
                overlay.set_is_narrating(false);
                overlay.set_is_dragging(false);
                overlay.set_note_text(slint::SharedString::new());

                let monitor_rects: Vec<MonitorRectData> = captured
                    .monitors
                    .iter()
                    .map(|m| MonitorRectData {
                        x: m.x as f32,
                        y: m.y as f32,
                        width: m.width as f32,
                        height: m.height as f32,
                    })
                    .collect();
                overlay.set_monitors(ModelRc::from(Rc::new(VecModel::from(monitor_rects))));

                // Geometry is deliberately NOT set here.
                //
                // A newly created window was already born with the right physical position and
                // size by the attributes hook, and a reused one still has them - `placement` is
                // the reuse key, so a match means nothing moved. Setting it again can only do
                // harm, and did: this used to derive a logical size from
                // `window().scale_factor()`, which returns 1.0 before the window has ever been
                // realised. On the FIRST capture that produced a 6000x3840 *logical* request,
                // which the real 1.5 scale factor then turned into a 9000x5760 window - the
                // overlay came up zoomed on the first capture of every session and was correct
                // from the second onwards, because by then the window existed and reported its
                // true scale factor.
                //
                // There is no safe way to size this before the window exists other than the hook:
                // set_size(Physical) also divides by that same not-yet-known scale factor.
                #[cfg(not(windows))]
                {
                    // No attributes hook off Windows, so fall back - correct only because the
                    // window has been realised by an earlier capture, or is about to be sized by
                    // the platform anyway.
                    let scale = overlay.window().scale_factor().max(0.01);
                    overlay
                        .window()
                        .set_position(slint::WindowPosition::Physical(
                            slint::PhysicalPosition::new(captured.origin_x, captured.origin_y),
                        ));
                    overlay
                        .window()
                        .set_size(slint::WindowSize::Logical(slint::LogicalSize::new(
                            captured.width as f32 / scale,
                            captured.height as f32 / scale,
                        )));
                }

                // Handlers are reinstalled every capture: a reused overlay's old handlers close
                // over the PREVIOUS canvas, so a region would be cropped out of a stale
                // screenshot. Setting a handler replaces it.
                let monitors = Rc::new(captured.monitors);
                let overlay_weak = overlay.as_weak();

                let close_overlay = {
                    let overlay_weak = overlay_weak.clone();
                    let main_weak = main_weak.clone();
                    move || {
                        if let Some(overlay) = overlay_weak.upgrade() {
                            if let Err(e) = overlay.hide() {
                                eprintln!("Failed to hide the capture overlay: {e}");
                            }
                        }
                        // Snapdown becomes screenshot-able by other tools again the moment its
                        // own capture is over.
                        set_capture_exclusion(&main_weak, false);
                        if let Some(main) = main_weak.upgrade() {
                            if let Err(e) = main.show() {
                                eprintln!("Failed to reshow the main window: {e}");
                            }
                        }
                    }
                };

                overlay.on_capture_completed({
                    let ctx_inner = ctx_inner.clone();
                    let main_weak = main_weak.clone();
                    let monitors = monitors.clone();
                    let close_overlay = close_overlay.clone();
                    // x/y/w/h arrive in CANVAS pixels, not logical ones, so they index the
                    // snapshot directly.
                    move |x, y, sel_w, sel_h, note| {
                        let region = (
                            x.max(0) as u32,
                            y.max(0) as u32,
                            sel_w.max(0) as u32,
                            sel_h.max(0) as u32,
                        );
                        // Attribute the Finding to whichever monitor contains the region's
                        // top-left, so a capture can still be traced to its screen.
                        let monitor_name = monitors
                            .iter()
                            .find(|m| {
                                let (mx, my) = (m.x as i64, m.y as i64);
                                let (rx, ry) = (region.0 as i64, region.1 as i64);
                                rx >= mx
                                    && rx < mx + m.width as i64
                                    && ry >= my
                                    && ry < my + m.height as i64
                            })
                            .map(|m| m.name.clone())
                            .unwrap_or_else(|| "UNKNOWN".to_string());

                        // The canvas is READ from the live overlay, never captured into this
                        // closure. A clone captured here would outlive this capture and still be
                        // alive at the next one's `make_mut_bytes`, turning it into a 92 MB
                        // copy-on-write - the precise cost reusing the buffer exists to avoid.
                        // Nothing on this path takes `LIVE_OVERLAYS` again, so the borrow is safe
                        // to hold across the crop and the write.
                        let finding_id = LIVE_OVERLAYS.with_borrow(|live| {
                            live.first().and_then(|entry| {
                                let canvas = entry.canvas.as_ref()?;
                                persist_finding(
                                    &ctx_inner,
                                    canvas.as_bytes(),
                                    (entry.placement.2, entry.placement.3),
                                    region,
                                    &monitor_name,
                                    note.as_str(),
                                )
                            })
                        });
                        close_overlay();
                        if let Some(main) = main_weak.upgrade() {
                            load_findings_into_window(&main, &ctx_inner, finding_id.as_deref());
                        }
                    }
                });

                // `detect_capture_targets` orders by z-order first and then by area, so the FIRST
                // rectangle containing the pointer is the tightest container of the frontmost
                // window. Taking the first match is therefore what makes occlusion work: a window
                // buried behind another is never offered where the front one covers it.
                //
                // A zero-width answer means "nothing here", which the overlay reads as no
                // highlight and no click target.
                // `level` walks OUTWARD through the containers under the pointer.
                //
                // `detect_capture_targets` orders by z-order and then by area, so level 0 is the
                // tightest container of the frontmost window - which is what makes occlusion work,
                // and also what made a whole window unreachable. The owner could select a panel
                // inside an application and never the application: every deeper candidate always
                // won. Level 1 is the next container out, and so on.
                overlay.on_target_at(|x, y, level| {
                    CAPTURE_TARGETS.with_borrow(|targets| {
                        targets
                            .iter()
                            .filter(|t| {
                                x >= t.region.x
                                    && x < t.region.x + t.region.width as i32
                                    && y >= t.region.y
                                    && y < t.region.y + t.region.height as i32
                            })
                            .nth(level.max(0) as usize)
                            .map(|t| MonitorRectData {
                                x: t.region.x as f32,
                                y: t.region.y as f32,
                                width: t.region.width as f32,
                                height: t.region.height as f32,
                            })
                            .unwrap_or_default()
                    })
                });

                // How many containers sit under the pointer, so the overlay knows whether there is
                // anywhere to walk to and can say so. Without it the affordance is invisible and the
                // Reviewer has no reason to try the wheel.
                overlay.on_target_count_at(|x, y| {
                    CAPTURE_TARGETS.with_borrow(|targets| {
                        targets
                            .iter()
                            .filter(|t| {
                                x >= t.region.x
                                    && x < t.region.x + t.region.width as i32
                                    && y >= t.region.y
                                    && y < t.region.y + t.region.height as i32
                            })
                            .count() as i32
                    })
                });

                overlay.on_overlay_cancelled({
                    let close_overlay = close_overlay.clone();
                    move || close_overlay()
                });

                // Publish the placement immediately before show(), because show() is when the
                // native window is actually created and therefore when the attributes hook runs.
                //
                // This used to be set around CaptureOverlayWindow::new() and cleared again right
                // after - so by the time the hook ran there was nothing left to read, the window
                // was created at a default size, and the overlay came up badly zoomed on every
                // capture. A `set_size` derived from window().scale_factor() had been masking
                // that, which is why removing the mask made it worse rather than better.
                //
                // On a reused window the hook does not run at all (there is nothing to create),
                // and nothing needs it to: the geometry already matches, since `placement` is the
                // reuse key.
                #[cfg(windows)]
                NEXT_OVERLAY_PLACEMENT.with_borrow_mut(|slot| *slot = Some(placement));
                if let Err(e) = overlay.show() {
                    eprintln!("Failed to show the capture overlay: {e}");
                }
                // Native window creation is DEFERRED to the event loop: when show() returns,
                // the winit window does not exist yet. Measured - with_winit_window() returns
                // None here, and window().scale_factor() still reports 1.0.
                //
                // That single fact is behind every geometry bug this overlay has had. It is why
                // the attributes hook never took effect (the placement had already been cleared
                // by the time the loop got round to creating the window), why sizing from
                // scale_factor() inflated the first capture by exactly the scale factor, and why
                // the second capture onwards looked fine - by then the window existed.
                //
                // So the placement is left in place for the hook to find when the loop does
                // create the window, and is only cleared on the next turn. That same turn
                // re-asserts the geometry through winit in device pixels, which needs no scale
                // factor and so cannot be wrong: belt and braces for the case where the hook did
                // not fire. Without a correct size the window would otherwise be sized from the
                // snapshot's intrinsic pixel size treated as LOGICAL - a 6000x3840 canvas asking
                // for a 9000x5760 window at 150%.
                #[cfg(windows)]
                {
                    let overlay_weak = overlay.as_weak();
                    let (x, y, w, h) = placement;
                    slint::Timer::single_shot(std::time::Duration::from_millis(0), move || {
                        NEXT_OVERLAY_PLACEMENT.with_borrow_mut(|slot| *slot = None);
                        let Some(overlay) = overlay_weak.upgrade() else {
                            return;
                        };
                        use i_slint_backend_winit::winit::dpi::{PhysicalPosition, PhysicalSize};
                        use i_slint_backend_winit::WinitWindowAccessor;
                        let applied = overlay.window().with_winit_window(|winit_win| {
                            winit_win.set_outer_position(PhysicalPosition::new(x, y));
                            let _ = winit_win.request_inner_size(PhysicalSize::new(w, h));
                            // Keyboard focus too, so Escape works without a click first.
                            winit_win.focus_window();
                        });
                        // And hand focus to the overlay's own key handler. `init` claims it once,
                        // at creation - but the overlay is created once and reused forever, so
                        // anything that took focus in an earlier capture (the note field, the Save
                        // button) still had it, and Enter went nowhere. Escape survived because the
                        // note field rejects it and it bubbles; Enter does not bubble, which is
                        // exactly why Enter was the one that intermittently did nothing.
                        overlay.invoke_claim_focus();
                        if applied.is_none() {
                            eprintln!("Capture overlay still has no winit window; geometry unset.");
                        }
                    });
                }

                LIVE_OVERLAYS.with_borrow_mut(|slot| *slot = live);
            }) {
                eprintln!("Failed to dispatch the capture overlay to the UI thread: {e}");
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact string shape found in the live `finding.region` column, read out of
    /// `%APPDATA%/id.wiradigital.snapdown/library.db`.
    ///
    /// This test exists because the first version of the caller used
    /// `serde_json::from_str::<Region>` on this column. That compiles, raises nothing, and returns
    /// `None` for every row - so the Editor's "reduced from" readout would never have appeared, and
    /// the guard written beside it asserted the shape of the CODE and passed. A parser has to be
    /// tested against the data it will actually be given.
    #[test]
    fn a_region_field_is_four_comma_separated_numbers() {
        assert_eq!(parse_region_field("419,344,1020,885"), Some((1020, 885)));
        assert_eq!(parse_region_field("0,0,1,1"), Some((1, 1)));
        // Negative origins are legal - a monitor left of the primary one - and only w/h are read.
        assert_eq!(
            parse_region_field("-1440,-559,2160,3840"),
            Some((2160, 3840))
        );
    }

    #[test]
    fn a_region_field_that_is_not_that_shape_yields_nothing() {
        // The JSON form the first attempt assumed. If this ever parses, the caller is guessing.
        assert_eq!(
            parse_region_field(r#"{"x":419,"y":344,"width":1020,"height":885}"#),
            None
        );
        assert_eq!(parse_region_field(""), None);
        assert_eq!(parse_region_field("419,344,1020"), None);
        assert_eq!(parse_region_field("419,344,1020,885,7"), None);
        assert_eq!(parse_region_field("419,344,wide,tall"), None);
    }
}
