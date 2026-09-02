#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod hotkey;
mod startup;
mod tray;

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use slint::{ComponentHandle, Model, ModelRc, SharedPixelBuffer, SharedString, VecModel};
use snapdown_capture::{CaptureTarget, RegionCapturer};
use snapdown_core::domain::bundle::{Bundle, BundleDetail, BundleItem};
use snapdown_core::domain::finding::{
    AnnotationShape, Finding, FindingDetail, Note, Region, VisualAnnotation,
};
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::domain::markdown::MarkdownSerializer;
use snapdown_core::domain::setting::{
    HotkeyAction, NamedBudget, QualityBudget, ResolvedPair, Setting, SettingKey, SettingValue,
};
use snapdown_core::error::CoreError;
use snapdown_core::ports::{
    BlobStore, BundleStore, Clock, EntropySource, FindingStore, HotkeyRegistrar, SettingsStore,
    StartupRegistrar,
};
use snapdown_core::util::id::id_from_parts;
use snapdown_store::image::{ImageReducer, MarkerBurner};
use snapdown_store::sqlite::{SqliteBundleStore, SqliteFindingStore, SqliteSettingsStore};
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
    use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS};
    use windows::Win32::System::Threading::CreateMutexW;

    let wide_name: Vec<u16> = SINGLE_INSTANCE_MUTEX_NAME
        .encode_utf16()
        .chain(Some(0))
        .collect();

    unsafe {
        match CreateMutexW(None, false, PCWSTR(wide_name.as_ptr())) {
            Ok(handle) => {
                if GetLastError() == ERROR_ALREADY_EXISTS {
                    // CLOSED, not leaked. `CreateMutexW` against an existing name still hands
                    // back a valid handle to that same kernel object - an open handle IS a
                    // reference keeping the object alive, from ANY process, including this one.
                    // `main`'s retry loop calls this in a loop specifically to wait out a
                    // relaunch racing its own predecessor's exit; leaving this handle open on
                    // every failed attempt would mean the loop's OWN earlier attempts keep the
                    // mutex alive by themselves, so `ERROR_ALREADY_EXISTS` never clears even
                    // after the old process is long gone - the retry becomes permanent and
                    // indistinguishable from no retry at all, which is exactly what shipped and
                    // was reported back as "still no restart."
                    let _ = CloseHandle(handle);
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
    /// `None` when the Bundle tables could not be opened.
    ///
    /// Deliberately not an in-memory fallback, unlike the two stores above: an in-memory Bundle
    /// library would accept an Assemble, report success, and lose the row on exit. A `None` that
    /// refuses out loud is the honest state, and it is the shape `BUG-12` argues for - the Reviewer
    /// sees a sentence rather than a process that has already died.
    bundle_store: Option<Arc<SqliteBundleStore>>,
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

        let bundle_store = match SqliteBundleStore::open(&db_path) {
            Ok(store) => Some(Arc::new(store)),
            Err(e) => {
                eprintln!(
                    "Failed to open the Bundle library at {db_path:?}: {e}. Assembling a Bundle \
                     will refuse rather than pretend."
                );
                None
            }
        };

        Self {
            vault_store,
            vault_path,
            finding_store,
            settings_store,
            bundle_store,
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

/// The largest a Library row's thumbnail is ever decoded: the artboard's 44x30 row art at 2x for a
/// HiDPI display - the same reasoning `THUMB_MAX_W`/`THUMB_MAX_H` give for the filmstrip, at the
/// Library row's own much smaller size.
const LIBRARY_THUMB_MAX_W: u32 = 88;
const LIBRARY_THUMB_MAX_H: u32 = 60;

/// "just now" / "N minutes ago" / "yesterday" / "last week" / ... from an RFC3339 instant to `now`.
///
/// No crate in `Cargo.toml` offers this - checked before writing it, because a hand-rolled ladder is
/// exactly the kind of thing that already exists somewhere and should not be reinvented twice.
/// `chrono` itself only formats an ABSOLUTE instant; the relative wording is ours to write. `now` is
/// a parameter rather than `Utc::now()` read inside, so the ladder itself can be tested against fixed
/// instants instead of a clock that moves while the test runs.
fn relative_time(rfc3339: &str, now: chrono::DateTime<chrono::Utc>) -> String {
    let Ok(then) = chrono::DateTime::parse_from_rfc3339(rfc3339) else {
        return "an unknown time ago".to_string();
    };
    let then = then.with_timezone(&chrono::Utc);
    let seconds = (now - then).num_seconds().max(0);

    match seconds {
        0..=44 => "just now".to_string(),
        45..=89 => "a minute ago".to_string(),
        90..=2699 => format!("{} minutes ago", (seconds + 30) / 60),
        2700..=5399 => "an hour ago".to_string(),
        5400..=86399 => format!("{} hours ago", (seconds + 1800) / 3600),
        86400..=151199 => "yesterday".to_string(),
        151200..=603799 => format!("{} days ago", (seconds + 43200) / 86400),
        603800..=1209599 => "last week".to_string(),
        _ => {
            let weeks = (seconds + 302400) / 604800;
            if weeks < 5 {
                format!("{weeks} weeks ago")
            } else {
                let months = ((seconds as f64) / (86400.0 * 30.44)).round() as i64;
                if months < 12 {
                    format!("{months} month{} ago", if months == 1 { "" } else { "s" })
                } else {
                    let years = ((seconds as f64) / (86400.0 * 365.25)).round() as i64;
                    format!("{years} year{} ago", if years == 1 { "" } else { "s" })
                }
            }
        }
    }
}

/// Decodes and thumbnails the Bundle's own copy of an image - never a Finding's. Assembling copies
/// (`Further Notes` in the spec: "Assembling copies, it never moves"), so a Bundle's `BundleItem`
/// image survives Discard originals and even a fully sealed Bundle, which is exactly why the Library
/// resolves this path rather than a Finding's `image_path`.
fn load_bundle_thumbnail(ctx: &AppContext, image_path: &str) -> slint::Image {
    let path = if PathBuf::from(image_path).is_absolute() {
        PathBuf::from(image_path)
    } else {
        ctx.vault_path.join(image_path)
    };
    if !path.exists() {
        return slint::Image::default();
    }
    match image::open(&path) {
        Ok(dyn_img) => rgba_to_slint_image(
            &dyn_img
                .thumbnail(LIBRARY_THUMB_MAX_W, LIBRARY_THUMB_MAX_H)
                .to_rgba8(),
        ),
        Err(_) => slint::Image::default(),
    }
}

/// One `BundleDetail` from the store, as one Library row. The meta line is composed here - "N
/// Findings · composed <relative time>", the exact wording `spec.md`'s Implementation Decisions
/// section gives three times over - rather than in Slint, so the pluralisation and the relative-time
/// ladder live in exactly one place.
fn library_row_from_detail(
    ctx: &AppContext,
    detail: &BundleDetail,
    now: chrono::DateTime<chrono::Utc>,
) -> LibraryBundleRow {
    let thumbnail = detail
        .items
        .first()
        .map(|item| load_bundle_thumbnail(ctx, &item.image_path))
        .unwrap_or_default();
    let count = detail.items.len();
    LibraryBundleRow {
        id: detail.bundle.id.clone().into(),
        name: detail.bundle.name.clone().into(),
        thumbnail,
        meta_line: format!(
            "{count} Finding{} · composed {}",
            if count == 1 { "" } else { "s" },
            relative_time(&detail.bundle.composed_at, now)
        )
        .into(),
    }
}

/// Every Bundle, as the Library will show it. `list_bundles` already orders newest-composed first
/// (`bundle_store.rs`'s own `ORDER BY composed_at DESC`); nothing here re-sorts it, per the ticket's
/// own instruction not to.
///
/// Two distinct refusals collapse into the same `Err`, because both read as "the Library could not
/// be read" to the Reviewer and both get the same Try again: `bundle_store` being `None` (the tables
/// never opened, `AppContext::init`'s own comment explains why that is not an in-memory fallback),
/// and `list_bundles` itself failing (a locked or corrupt `library.db`).
fn build_library_rows(
    ctx: &AppContext,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<LibraryBundleRow>, String> {
    let store = ctx
        .bundle_store
        .as_ref()
        .ok_or_else(|| "The Bundle library could not be opened.".to_string())?;
    let details = store
        .list_bundles()
        .map_err(|e| format!("Could not read the Library: {e}"))?;
    Ok(details
        .iter()
        .map(|detail| library_row_from_detail(ctx, detail, now))
        .collect())
}

/// Opens the Library and (re-)reads the store into it. The same function serves the initial open and
/// Try again, which is what makes "Try again re-reads the store" true by construction rather than by
/// two call sites agreeing to do the same thing.
fn open_library(window: &AppWindow, ctx: &AppContext) {
    window.set_library_state("loading".into());
    window.set_library_open(true);

    match build_library_rows(ctx, chrono::Utc::now()) {
        Ok(rows) => {
            window.set_library_rows(ModelRc::from(Rc::new(VecModel::from(rows))));
            window.set_library_error_message(SharedString::new());
            window.set_library_state("ready".into());
        }
        Err(message) => {
            window.set_library_rows(ModelRc::from(Rc::new(VecModel::from(Vec::<
                LibraryBundleRow,
            >::new()))));
            window.set_library_error_message(message.into());
            window.set_library_state("error".into());
        }
    }
}

/// Whether a Bundle's originals are gone - read LIVE from whether every one of its `BundleItem`s'
/// Findings still exists, never a stored flag (`BR-122`). No `sealed` column exists in the schema on
/// purpose (confirmed against `migrations.rs`): a Finding deleted after the Library last opened this
/// Bundle's menu must change the answer the very next time it opens, and a cached bit could not do
/// that. Migration v6 is what legalises the sealed case at all - it dropped `bundle_item`'s FK on
/// `finding_id` precisely so a Bundle can outlive its Findings.
///
/// An empty Bundle (no items) reads as unsealed - vacuously true by the same rule the spec states
/// ("every one of its BundleItems' Findings still exists"), and harmless: Disassemble on one just
/// removes an empty Bundle, the same as Delete would.
fn bundle_is_sealed(ctx: &AppContext, detail: &BundleDetail) -> bool {
    detail
        .items
        .iter()
        .any(|item| !matches!(ctx.finding_store.get_finding(&item.finding_id), Ok(Some(_))))
}

/// Removes a Bundle's row (its `BundleItem`s cascade via the schema's own `ON DELETE CASCADE`) and
/// only then its Vault folder - `AD-2`'s order, record first, then files. Disassemble and Delete both
/// call this: `spec.md`'s Implementation Decisions say the two acts do exactly the same thing to
/// persisted state, and only what happens to the Findings differs (nothing, either way - Disassemble
/// writes no Finding, it only stops holding them; `bundle_is_sealed`'s own doc comment says why that
/// is enough to bring them back to the filmstrip).
///
/// `Ok(true)` means the row went but the folder could not be removed - an orphan the Vault sweeper
/// already owns, and never a listed Bundle whose files are gone, which is the state `AD-2`'s own
/// "Prevents" names. `Ok(false)` means both went cleanly. `Err` means the ROW delete itself failed:
/// nothing was touched, and the message names what refused.
fn remove_bundle_row_and_folder(ctx: &AppContext, bundle_id: &str) -> Result<bool, String> {
    let store = ctx
        .bundle_store
        .as_ref()
        .ok_or_else(|| "The Bundle library could not be opened.".to_string())?;

    store
        .delete_bundle(bundle_id)
        .map_err(|e| format!("Could not delete the Bundle: {e}"))?;

    // The row is already gone at this point. A folder-removal failure below is reported to the
    // caller as an orphan, never as an overall error - the Reviewer asked for the Bundle gone from
    // the Library, and it is; retrying would not un-delete a row that already went.
    let folder = format!("bundles/{bundle_id}");
    match ctx.vault_store.delete_folder(&folder) {
        Ok(()) => Ok(false),
        Err(e) => {
            eprintln!("Deleted Bundle {bundle_id} but left its folder behind: {e}");
            Ok(true)
        }
    }
}

fn load_findings_into_window(
    window: &AppWindow,
    ctx: &AppContext,
    active_finding_id: Option<&str>,
) {
    let all_findings = ctx.finding_store.list_findings().unwrap_or_default();

    // The strip is the queue of Findings not yet handed over, so anything a Bundle already holds
    // leaves it. Read from the Bundle rows rather than tracked on the Finding: the Bundle items are
    // the fact, and a duplicate flag on the Finding would be a second copy of it to go wrong.
    let bundled: std::collections::HashSet<String> = ctx
        .bundle_store
        .as_ref()
        .and_then(|store| store.list_bundles().ok())
        .map(|bundles| {
            bundles
                .iter()
                .flat_map(|detail| detail.items.iter().map(|item| item.finding_id.clone()))
                .collect()
        })
        .unwrap_or_default();

    let findings: Vec<FindingDetail> = all_findings
        .into_iter()
        .filter(|detail| !bundled.contains(&detail.finding.id))
        .collect();

    let mut filmstrip: Vec<FindingThumb> = Vec::new();

    // A capture becomes THE selection: the new Finding alone, ticked and on the canvas.
    //
    // Ticks used to survive a capture, so the Reviewer would not lose a half-built Bundle. That
    // created a trap the owner hit: a capture makes the new Finding ACTIVE - it is what the canvas
    // shows and what Markers get added to - while an older Finding stays TICKED from before. Assemble
    // follows the tick, so the Bundle silently took the wrong Finding, with the right one's Markers
    // still sitting in the inspector.
    //
    // Verified against the owner's own library: the Finding being annotated had 19 Markers, and the
    // Bundle written from that session held a different one with 3 empty ones.
    //
    // Pressing the capture hotkey says "this is what I am working on now", so it selects like a plain
    // click does. Any other rule lets the tick and the canvas point at different Findings, and
    // nothing on screen has to be wrong for the Bundle to be.
    let already_ticked: Vec<String> = match active_finding_id {
        Some(fresh) => {
            // The anchor moves with it. Ticking the new Finding alone while leaving the anchor on the
            // last one clicked would make the next Shift-click range from a card the Reviewer is no
            // longer working on - which is what they reported: "index kedua yang jadi anchor".
            SELECTION_ANCHOR.with(|held| *held.borrow_mut() = fresh.to_string());
            vec![fresh.to_string()]
        }
        None => window
            .get_filmstrip_items()
            .iter()
            .filter(|t| t.is_selected)
            .map(|t| t.id.to_string())
            .collect(),
    };

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
            is_selected: already_ticked.iter().any(|t| t == &f.id),
            is_active,
            image: loaded_img,
        });
    }

    let model = Rc::new(VecModel::from(filmstrip));
    window.set_filmstrip_items(ModelRc::from(model));
    refresh_selection_count(window);

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

    // Kept beyond the `set_active_image` call: a Blur annotation's preview is cut from these same
    // pixels, so decoding once serves both.
    let source_rgba = image::open(&img_path)
        .ok()
        .map(|decoded| decoded.to_rgba8());
    if let Some(rgba) = source_rgba.as_ref() {
        window.set_active_image(rgba_to_slint_image(rgba));
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

    window.set_finding_note(detail.note.body.clone().into());

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

    let slint_annotations: Vec<AnnotationData> =
        detail.visual_annotations.iter().map(shape_to_ui).collect();

    // Computed whenever the Finding has an image, not only when it already carries a redaction: the
    // Reviewer may be about to draw one, and the drag has to show the real blur from its first pixel.
    if let Some(rgba) = source_rgba.as_ref() {
        window.set_blurred_capture(blurred_capture(&f.id, rgba));
    }
    window.set_annotations(ModelRc::from(Rc::new(VecModel::from(slint_annotations))));

    // A selection that survived a reload is re-checked against what actually came back, and the
    // Properties panel is rewritten from it. Deleting the selected annotation, or opening another
    // Finding, would otherwise leave `FR-33`'s handles and the panel's font controls pointing at an
    // id nothing in the model has - the same stale-id class as `BUG-67`.
    let surviving = window.get_selected_annotation_id().to_string();
    apply_selection_mirror(window, &detail.visual_annotations, &surviving);

    // The cost card's two inputs Slint cannot derive: a sum over the Marker model (Slint has no
    // fold) and an image estimate from the STORED dimensions rather than the readout string.
    //
    // `w * h / 750` is the ratio the card's own hard-coded "~1204 tk" was computed with for a
    // 1020x885 image, so making it real does not move the number - it just stops it being a
    // number about one particular capture from months ago.
    window.set_marker_char_count(
        detail
            .markers
            .iter()
            .map(|m| m.comment.chars().count())
            .sum::<usize>() as i32,
    );
    window.set_image_token_estimate(
        (u64::from(f.image_width) * u64::from(f.image_height) / 750) as i32,
    );
}

// ANNOTATIONS - the translation between `AnnotationShape` and what the canvas can hold.
//
// The canvas speaks in two corners and a kind; the domain speaks in five variants with different
// fields. Everything that converts between them lives here, in one place, because the alternative is
// a `match` on `kind` inside each of six callbacks and six chances to forget a variant.

/// The domain shape as the canvas draws it.
///
/// A box's two corners are ordered here, once, so `AnnotationItem`'s eight handles can trust that
/// `x1 < x2`. A drag that ran right-to-left produces the same annotation as one that ran
/// left-to-right, and the store never sees the difference.
fn shape_to_ui(annotation: &VisualAnnotation) -> AnnotationData {
    let boxed = |kind: &str,
                 x: f64,
                 y: f64,
                 w: f64,
                 h: f64,
                 text: &str,
                 size: f64,
                 family: Option<&str>,
                 align: Option<&str>| AnnotationData {
        id: annotation.id.clone().into(),
        kind: kind.into(),
        x1: x as f32,
        y1: y as f32,
        x2: (x + w) as f32,
        y2: (y + h) as f32,
        tail_x: 0.0,
        tail_y: 0.0,
        text: text.into(),
        font_size: size as f32,
        font_family: family.unwrap_or(ANNOTATION_FONT).into(),
        text_align: align.unwrap_or("start").into(),
    };

    match &annotation.data {
        AnnotationShape::Rect {
            x,
            y,
            width,
            height,
            ..
        } => boxed("rect", *x, *y, *width, *height, "", 0.0, None, None),
        AnnotationShape::Blur {
            x,
            y,
            width,
            height,
            ..
        } => boxed("blur", *x, *y, *width, *height, "", 0.0, None, None),
        AnnotationShape::Text {
            x,
            y,
            width,
            height,
            text,
            font_size,
            font_family,
            text_align,
            ..
        } => boxed(
            "text",
            *x,
            *y,
            *width,
            *height,
            text,
            font_size.unwrap_or(DEFAULT_ANNOTATION_FONT_SIZE),
            font_family.as_deref(),
            text_align.as_deref(),
        ),
        AnnotationShape::Callout {
            x,
            y,
            width,
            height,
            tail_x,
            tail_y,
            text,
            font_size,
            font_family,
            text_align,
            ..
        } => {
            let mut data = boxed(
                "callout",
                *x,
                *y,
                *width,
                *height,
                text,
                font_size.unwrap_or(DEFAULT_ANNOTATION_FONT_SIZE),
                font_family.as_deref(),
                text_align.as_deref(),
            );
            data.tail_x = *tail_x as f32;
            data.tail_y = *tail_y as f32;
            data
        }
        AnnotationShape::Arrow {
            start_x,
            start_y,
            end_x,
            end_y,
            ..
        } => AnnotationData {
            id: annotation.id.clone().into(),
            kind: "arrow".into(),
            // An Arrow's two points are NOT ordered. Its direction is the whole content of the
            // annotation - which end has the head is what the Reviewer is saying.
            x1: *start_x as f32,
            y1: *start_y as f32,
            x2: *end_x as f32,
            y2: *end_y as f32,
            tail_x: 0.0,
            tail_y: 0.0,
            text: SharedString::new(),
            font_size: 0.0,
            font_family: ANNOTATION_FONT.into(),
            text_align: "start".into(),
        },
    }
}

/// The one family the burn has, so the canvas cannot preview a face the PNG will not get.
const ANNOTATION_FONT: &str = "IBM Plex Sans";

/// The size a new Text or Callout starts at when nothing has been chosen yet.
///
/// ONE number for both. They were 18 and 14, which meant a Callout and a Text placed side by side on
/// the same capture did not match and the Reviewer had to correct one of them every time.
const DEFAULT_ANNOTATION_FONT_SIZE: f64 = 18.0;

fn annotation_font_size_key() -> SettingKey {
    SettingKey::Custom("annotation_font_size".to_string())
}

/// The size the next annotation will be created at.
///
/// Stored, so choosing 24 once means the next Callout is 24 too - the Reviewer's own default rather
/// than ours. **Forward only.** Existing annotations keep the size they were given; a setting that
/// reached backwards would silently rewrite work that had already been reviewed and, worse, would
/// change images that had already been handed to an agent.
///
/// Clamped to the slider's range on the way out as well as in, so a hand-edited settings row cannot
/// produce an annotation the panel has no way to show.
fn default_annotation_font_size(ctx: &AppContext) -> f64 {
    ctx.settings_store
        .get(&annotation_font_size_key())
        .ok()
        .flatten()
        .and_then(|setting| match setting.value {
            SettingValue::Integer(size) => Some(size as f64),
            _ => None,
        })
        .map(|size| size.clamp(8.0, 72.0))
        .unwrap_or(DEFAULT_ANNOTATION_FONT_SIZE)
}

/// Remembers the size the Reviewer just chose, for the NEXT annotation.
///
/// Best-effort: a failure here loses a preference, not work, and the annotation itself has already
/// been written. Reported to the console rather than to the Reviewer, who did not ask for a setting
/// to be saved and would be told about a failure in something they never did.
fn remember_annotation_font_size(ctx: &AppContext, size: f64) {
    let clock = SystemClock::new();
    let setting = Setting {
        key: annotation_font_size_key(),
        value: SettingValue::Integer(size.round() as i64),
        updated_at: clock.now_rfc3339(),
    };
    if let Err(e) = ctx.settings_store.set(&setting) {
        eprintln!("Could not remember the annotation font size: {e}");
    }
}

/// The WHOLE capture, blurred once, for every Blur annotation to be a window onto.
///
/// Slint 1.17 has no content blur - `blur` exists only on `BoxShadow` - so this is Rust's answer, and
/// it uses `MarkerBurner::blur_rect`: the SAME function the burn calls, not a second approximation of
/// it. That is the point. A redaction preview that differs from the output is the one kind of
/// preview that can get somebody hurt, because the Reviewer approves what they see.
///
/// Whole-image rather than a crop per annotation, for two reasons that both turned out to matter:
///
///   - a DRAG can show the real blur from its first pixel. A per-annotation crop can only ever show
///     the last committed one, stretched, because the region being dragged does not exist yet.
///   - N redactions cost one blur instead of N.
///
/// Cached per Finding, because the reload after every annotation edit would otherwise re-blur a
/// megapixel image for a change that touched none of it.
///
/// The radius is the DEFAULT one, so an annotation carrying its own `blur_radius` previews slightly
/// off. Nothing in the UI sets one; when something does, this becomes a cache keyed by radius too.
fn blurred_capture(finding_id: &str, source: &image::RgbaImage) -> slint::Image {
    BLURRED_CAPTURE.with(|cell| {
        let mut cache = cell.borrow_mut();
        if let Some((cached_id, cached)) = cache.as_ref() {
            if cached_id == finding_id {
                return cached.clone();
            }
        }
        let mut blurred = source.clone();
        let (w, h) = (blurred.width() as i32, blurred.height() as i32);
        MarkerBurner::blur_rect(&mut blurred, 0, 0, w, h, default_canvas_blur_radius(w, h));
        let image = rgba_to_slint_image(&blurred);
        *cache = Some((finding_id.to_string(), image.clone()));
        image
    })
}

/// The same default the burn uses, kept in step with `default_blur_radius` in `burner.rs`.
///
/// Duplicated rather than exported, because it is the store's own default and exporting it would
/// make the canvas a caller of a decision it does not own. If they drift, the preview lies - which
/// is what `the_canvas_blur_matches_the_burn_default` exists to catch.
fn default_canvas_blur_radius(width: i32, height: i32) -> i32 {
    let short = f64::from(width.min(height).max(1));
    (short / 14.0).clamp(3.0, 16.0) as i32
}

/// A fresh shape from the canvas: a kind and the drag's two corners.
fn shape_from_drag(
    kind: &str,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    font_size: f64,
) -> Option<AnnotationShape> {
    let x = x1.min(x2);
    let y = y1.min(y2);
    let width = (x2 - x1).abs();
    let height = (y2 - y1).abs();

    Some(match kind {
        "rect" => AnnotationShape::Rect {
            x,
            y,
            width,
            height,
            stroke_color: None,
            stroke_width: None,
        },
        "blur" => AnnotationShape::Blur {
            x,
            y,
            width,
            height,
            blur_radius: None,
        },
        "text" => AnnotationShape::Text {
            x,
            y,
            width,
            height,
            text: String::new(),
            font_size: Some(font_size),
            font_family: Some(ANNOTATION_FONT.to_string()),
            text_color: None,
            text_align: None,
        },
        // A Callout's drag is the other way round from a box's: `x1,y1` is where the pointer went
        // DOWN, which is the thing being pointed AT, and the bubble is carried away from it. The
        // canvas used to do the inset itself and hand the INSET corner over as `x1,y1`, so the tail
        // anchored to the bubble's own top-left and appeared to snap there the moment the drag
        // ended - which is exactly what the owner saw: *"anchor ekor malah tereset ke node kiri
        // bawah dari shape callout"*. The raw gesture arrives here now and the inset happens below,
        // where both halves can be derived from the same two points.
        "callout" => AnnotationShape::Callout {
            // Inset FROM THE PRESS POINT, not from the top-left of the drag.
            //
            // This was `min + 0.3 * (max - min)`, which is the same distance but always measured
            // from the smaller coordinate - so it only pointed the right way when the Reviewer
            // happened to drag down and to the right. Drag UP and the bubble's far edge landed
            // exactly on `y1`, which is where the tail is, and the arrow came out flush with the
            // bubble's own bottom line: *"dia malah sejajar sama garis bawah shape"*.
            //
            // `x1 + (x2 - x1) * 0.3` carries the sign, so the bubble always sits at the far end of
            // the gesture whichever way it was drawn.
            x: (x1 + (x2 - x1) * 0.3).min(x2),
            y: (y1 + (y2 - y1) * 0.3).min(y2),
            width: width * 0.7,
            height: height * 0.7,
            // The press point itself, untouched by the inset above.
            tail_x: x1.clamp(0.0, 1.0),
            tail_y: y1.clamp(0.0, 1.0),
            text: String::new(),
            font_size: Some(font_size),
            font_family: Some(ANNOTATION_FONT.to_string()),
            bg_color: None,
            text_color: None,
            text_align: None,
        },
        "arrow" => AnnotationShape::Arrow {
            start_x: x1,
            start_y: y1,
            end_x: x2,
            end_y: y2,
            color: None,
            stroke_width: None,
        },
        _ => return None,
    })
}

/// Moves or resizes a shape without touching anything else it carries.
///
/// A Callout that is dragged keeps its words, its size and its tail; the tail moves WITH the bubble,
/// because a tail that stayed behind would be pointing at whatever happens to be under it now.
fn with_geometry(shape: &AnnotationShape, x1: f64, y1: f64, x2: f64, y2: f64) -> AnnotationShape {
    let nx = x1.min(x2);
    let ny = y1.min(y2);
    let nw = (x2 - x1).abs();
    let nh = (y2 - y1).abs();

    match shape {
        AnnotationShape::Rect {
            stroke_color,
            stroke_width,
            ..
        } => AnnotationShape::Rect {
            x: nx,
            y: ny,
            width: nw,
            height: nh,
            stroke_color: stroke_color.clone(),
            stroke_width: *stroke_width,
        },
        AnnotationShape::Blur { blur_radius, .. } => AnnotationShape::Blur {
            x: nx,
            y: ny,
            width: nw,
            height: nh,
            blur_radius: *blur_radius,
        },
        AnnotationShape::Text {
            text,
            font_size,
            font_family,
            text_color,
            text_align,
            ..
        } => AnnotationShape::Text {
            x: nx,
            y: ny,
            width: nw,
            height: nh,
            text: text.clone(),
            font_size: *font_size,
            font_family: font_family.clone(),
            text_color: text_color.clone(),
            text_align: text_align.clone(),
        },
        AnnotationShape::Callout {
            tail_x,
            tail_y,
            text,
            font_size,
            font_family,
            bg_color,
            text_color,
            text_align,
            ..
        } => AnnotationShape::Callout {
            x: nx,
            y: ny,
            width: nw,
            height: nh,
            // THE TAIL DOES NOT FOLLOW THE BUBBLE. It used to, and the owner corrected it:
            // *"Dragging shape dari callout jangan mengubah anchor point dari ekor - ekor harus
            // dipindahkan terpisah."*
            //
            // They are right, and the reason is what a Callout is for. The tail names a place on the
            // screenshot; the bubble is where the words happen to fit. Moving the bubble out of the
            // way of something is the commonest reason to drag one at all, and carrying the tail
            // along would re-point it at whatever now sits under it.
            tail_x: *tail_x,
            tail_y: *tail_y,
            text: text.clone(),
            font_size: *font_size,
            font_family: font_family.clone(),
            bg_color: bg_color.clone(),
            text_color: text_color.clone(),
            text_align: text_align.clone(),
        },
        AnnotationShape::Arrow {
            color,
            stroke_width,
            ..
        } => AnnotationShape::Arrow {
            // Unordered, for the reason `shape_to_ui` gives: the head is the message.
            start_x: x1,
            start_y: y1,
            end_x: x2,
            end_y: y2,
            color: color.clone(),
            stroke_width: *stroke_width,
        },
    }
}

/// The Callout tail's own target, moved on its own.
fn with_tail(shape: &AnnotationShape, tx: f64, ty: f64) -> AnnotationShape {
    match shape {
        AnnotationShape::Callout {
            x,
            y,
            width,
            height,
            text,
            font_size,
            font_family,
            bg_color,
            text_color,
            text_align,
            ..
        } => AnnotationShape::Callout {
            x: *x,
            y: *y,
            width: *width,
            height: *height,
            tail_x: tx,
            tail_y: ty,
            text: text.clone(),
            font_size: *font_size,
            font_family: font_family.clone(),
            bg_color: bg_color.clone(),
            text_color: text_color.clone(),
            text_align: text_align.clone(),
        },
        // Only a Callout has one. Returning the shape unchanged rather than erroring: the UI only
        // offers the handle on a Callout, so reaching here means a stale id, not a bad request.
        other => other.clone(),
    }
}

/// The Properties panel writing back: the words, the size, the family, the alignment.
///
/// `None` for a field the caller is not changing. All four arrive together because the panel is one
/// form - three round trips for one edit would each reload the model and take the caret with it.
fn with_content(
    shape: &AnnotationShape,
    text: Option<&str>,
    font_size: Option<f64>,
    font_family: Option<&str>,
    text_align: Option<&str>,
) -> AnnotationShape {
    match shape {
        AnnotationShape::Text {
            x,
            y,
            width,
            height,
            text: old_text,
            font_size: old_size,
            font_family: old_family,
            text_color,
            text_align: old_align,
        } => AnnotationShape::Text {
            x: *x,
            y: *y,
            width: *width,
            height: *height,
            text: text.map(str::to_string).unwrap_or_else(|| old_text.clone()),
            font_size: font_size.or(*old_size),
            font_family: font_family
                .map(str::to_string)
                .or_else(|| old_family.clone()),
            text_color: text_color.clone(),
            text_align: text_align.map(str::to_string).or_else(|| old_align.clone()),
        },
        AnnotationShape::Callout {
            x,
            y,
            width,
            height,
            tail_x,
            tail_y,
            text: old_text,
            font_size: old_size,
            font_family: old_family,
            bg_color,
            text_color,
            text_align: old_align,
        } => AnnotationShape::Callout {
            x: *x,
            y: *y,
            width: *width,
            height: *height,
            tail_x: *tail_x,
            tail_y: *tail_y,
            text: text.map(str::to_string).unwrap_or_else(|| old_text.clone()),
            font_size: font_size.or(*old_size),
            font_family: font_family
                .map(str::to_string)
                .or_else(|| old_family.clone()),
            bg_color: bg_color.clone(),
            text_color: text_color.clone(),
            text_align: text_align.map(str::to_string).or_else(|| old_align.clone()),
        },
        // A Shape, an Arrow and a Blur carry no words. The Properties panel is not shown for them -
        // the owner scoped it to "hanya muncul di element text dan callout" - so this is unreachable
        // through the UI and is a no-op rather than an error for the same reason `with_tail` is.
        other => other.clone(),
    }
}

/// Reads one annotation back out of the store.
///
/// From the store rather than from the Slint model, because the model carries the CANVAS projection
/// (two corners and a kind), and every edit here has to preserve the fields that projection drops:
/// a colour, a stroke width, a blur radius. Editing the projection would quietly reset them.
fn load_annotation(
    ctx: &AppContext,
    finding_id: &str,
    annotation_id: &str,
) -> Option<VisualAnnotation> {
    ctx.finding_store
        .get_finding(finding_id)
        .ok()
        .flatten()?
        .visual_annotations
        .into_iter()
        .find(|a| a.id == annotation_id)
}

/// Writes the five fields the Properties panel reads.
///
/// The panel lives in the inspector beside the Marker Notes, and a Slint repeater cannot pick one
/// row out of a model - so "which annotation is selected, and what does it say" is mirrored onto the
/// window as scalars. One writer, here, so the mirror cannot disagree with the id: the canvas asks
/// for a selection through `annotation-selected`, and every reload comes back through this too.
///
/// An id that matches nothing clears the panel rather than leaving the previous annotation's font on
/// screen next to a different shape.
fn apply_selection_mirror(window: &AppWindow, annotations: &[VisualAnnotation], id: &str) {
    let selected = annotations.iter().find(|a| a.id == id);

    let Some(selected) = selected else {
        window.set_selected_annotation_id(SharedString::new());
        window.set_selected_annotation_kind(SharedString::new());
        window.set_selected_annotation_text(SharedString::new());
        return;
    };

    window.set_selected_annotation_id(id.into());

    // The canvas projection already answers all five, and re-deriving them here from the enum would
    // be a second `match` that could disagree with `shape_to_ui`. No source image: the Properties
    // panel has no use for a Blur preview, and cutting one would be work for nothing.
    let projected = shape_to_ui(selected);
    window.set_selected_annotation_kind(projected.kind);
    window.set_selected_annotation_text(projected.text);
    window.set_selected_annotation_font_size(projected.font_size);
    window.set_selected_annotation_font_family(projected.font_family);
    window.set_selected_annotation_align(projected.text_align);
}

/// The z-order one annotation should have after a "bring forward" or a "send to back".
///
/// Returns the WHOLE new order, because that is what the port takes and because the four movements
/// are one idea - move this element to a new index - not four operations.
///
/// The SCREENSHOT is always underneath everything and is not in this list. It is not an annotation;
/// it is the thing being annotated, and there is no index at which it could be moved.
///
/// `None` when the movement changes nothing - the first element sent backward, the last brought
/// forward. The caller writes nothing rather than paying a transaction and a reload to store the
/// order it already had.
fn reordered(ids: &[String], target: &str, movement: &str) -> Option<Vec<String>> {
    let from = ids.iter().position(|id| id == target)?;
    let last = ids.len().checked_sub(1)?;

    // Later in the list is drawn later, so it is nearer the front.
    let to = match movement {
        "front" => last,
        "back" => 0,
        "forward" => (from + 1).min(last),
        "backward" => from.saturating_sub(1),
        _ => return None,
    };
    if to == from {
        return None;
    }

    let mut reordered = ids.to_vec();
    let moved = reordered.remove(from);
    reordered.insert(to, moved);
    Some(reordered)
}

/// One reversible annotation edit. `FR-33`: "Redo/Undo history is supported for canvas additions,
/// moves, edits, and deletions."
#[derive(Clone)]
enum AnnEdit {
    Added {
        id: String,
    },
    Removed {
        annotation: VisualAnnotation,
    },
    Changed {
        id: String,
        before: AnnotationShape,
        after: AnnotationShape,
    },
}

thread_local! {
    /// The undo and redo stacks, and the Finding they belong to.
    ///
    /// Scoped to ONE Finding and cleared when the Reviewer opens another. An undo stack that spanned
    /// Findings would let Ctrl+Z remove a redaction from an image that is no longer on screen, and
    /// the Reviewer would have no way to see that it had happened.
    ///
    /// In memory only. A history that survived a restart would be a fourth thing to keep consistent
    /// with the store, and nothing in `CAP-11` asks for one.
    static ANNOTATION_HISTORY: RefCell<(String, Vec<AnnEdit>, Vec<AnnEdit>)> =
        const { RefCell::new((String::new(), Vec::new(), Vec::new())) };
}

/// Records an edit, and drops any redo future - the standard rule: a new edit after an undo
/// replaces what was undone rather than branching.
fn record_edit(finding_id: &str, edit: AnnEdit) {
    ANNOTATION_HISTORY.with(|cell| {
        let mut history = cell.borrow_mut();
        if history.0 != finding_id {
            *history = (finding_id.to_string(), Vec::new(), Vec::new());
        }
        history.1.push(edit);
        history.2.clear();
    });
}

/// One step back, or one step forward. `FR-33`.
///
/// Undo and redo are the same walk in opposite directions, so they are one function rather than two
/// that have to be kept agreeing about what the inverse of each edit is.
fn step_annotation_history(window: &AppWindow, ctx: &AppContext, undo: bool) {
    let finding_id = window.get_active_finding_id().to_string();
    if finding_id.is_empty() {
        return;
    }

    let edit = ANNOTATION_HISTORY.with(|cell| {
        let mut history = cell.borrow_mut();
        if history.0 != finding_id {
            // The history belongs to another Finding. Nothing to undo here, and applying it would
            // edit an image the Reviewer is not looking at.
            return None;
        }
        if undo {
            history.1.pop()
        } else {
            history.2.pop()
        }
    });

    let Some(edit) = edit else {
        toast(
            window,
            if undo {
                "Nothing to undo."
            } else {
                "Nothing to redo."
            },
            false,
        );
        return;
    };

    // What actually gets done, and what it costs to reverse it again. An `Added` undone becomes a
    // `Removed` on the other stack carrying the whole annotation - which is what makes redo able to
    // put back something that no longer exists in the store.
    let outcome: Result<AnnEdit, CoreError> = match &edit {
        AnnEdit::Added { id } => match load_annotation(ctx, &finding_id, id) {
            Some(annotation) => ctx
                .finding_store
                .delete_annotation(&finding_id, id)
                .map(|()| AnnEdit::Removed { annotation }),
            None => Err(CoreError::NotFound(format!("Annotation {id} is gone"))),
        },
        AnnEdit::Removed { annotation } => ctx
            .finding_store
            .add_annotation(
                &finding_id,
                &annotation.id,
                &annotation.data,
                &annotation.created_at,
            )
            // Re-added, which puts it back on TOP rather than at the position it held. Stated
            // plainly because it is a real difference: undoing the deletion of an annotation that
            // something else covered will leave it in front. Restoring the position would mean a
            // port method that takes one, and nothing else in the product needs it.
            .map(|_| AnnEdit::Added {
                id: annotation.id.clone(),
            }),
        AnnEdit::Changed { id, before, after } => {
            let target = if undo { before } else { after };
            ctx.finding_store
                .update_annotation(&finding_id, id, target)
                .map(|_| AnnEdit::Changed {
                    id: id.clone(),
                    before: before.clone(),
                    after: after.clone(),
                })
        }
    };

    match outcome {
        Ok(inverse) => {
            ANNOTATION_HISTORY.with(|cell| {
                let mut history = cell.borrow_mut();
                // A `Changed` is its own inverse either way round, so it moves across unchanged and
                // the direction alone decides which end it is read from next.
                if undo {
                    history.2.push(inverse);
                } else {
                    history.1.push(inverse);
                }
            });
            window.set_selected_annotation_id(SharedString::new());
            load_active_detail(window, ctx, &finding_id);
        }
        Err(e) => {
            // The step is NOT pushed back on the stack it came off. A step that cannot be applied
            // will not become applicable later, and leaving it there makes every subsequent undo
            // fail on the same one.
            toast(
                window,
                format!("Could not {} that: {e}", if undo { "undo" } else { "redo" }),
                true,
            );
        }
    }
}

/// Whether a hotkey capture should bring the Editor forward when it lands (`FR-18`).
///
/// Default OFF, and that default is `BG-6` in one line: the loop has to survive six captures in
/// ninety seconds, and a window taking focus each time turns six captures into six dismissals.
///
/// A read failure is treated as off for the same reason - the quiet behaviour is the safe one to
/// fall back to, because the Reviewer can always open the Editor and cannot un-steal focus.
fn open_editor_after_capture(ctx: &AppContext) -> bool {
    ctx.settings_store
        .get(&SettingKey::OpenEditorAfterCapture)
        .ok()
        .flatten()
        .map(|setting| matches!(setting.value, SettingValue::Boolean(true)))
        .unwrap_or(false)
}

/// The Finding's image, burned, on the clipboard as a bitmap. `FR-36`.
///
/// The BURNED image, not the stored one: this is the one path in the product that hands an image
/// over without a Bundle around it, so a redaction that existed only in the Bundle's copy would make
/// this path leak. It is the same `burn_all` the Bundle uses, over the same Markers and annotations.
///
/// BMP rather than PNG because the Windows clipboard's image format is a DIB, and a BMP file is a
/// DIB with a 14-byte header. `clipboard-win` strips the header; nothing here has to know the
/// layout. The cost is a second encode of an image already encoded once, which is a few milliseconds
/// on a keystroke the Reviewer initiated.
#[cfg(windows)]
fn copy_burned_image(ctx: &AppContext, finding_id: &str) -> Result<String, String> {
    use image::codecs::bmp::BmpEncoder;
    use image::ExtendedColorType;

    let detail = ctx
        .finding_store
        .get_finding(finding_id)
        .map_err(|e| format!("Could not read the Finding: {e}"))?
        .ok_or_else(|| "That Finding is no longer in the Library.".to_string())?;

    let source = ctx
        .vault_store
        .read_blob(&detail.finding.image_path)
        .map_err(|e| format!("Could not read the image: {e}"))?;

    let dims = ImageDimensions {
        width: detail.finding.image_width,
        height: detail.finding.image_height,
    };
    let burned = MarkerBurner::burn_all(
        &source,
        &dims,
        &detail.markers,
        &detail.visual_annotations,
        detail
            .finding
            .resolved_encoder_quality
            .unwrap_or(snapdown_store::image::LOSSLESS),
    )
    .map_err(|e| format!("Could not draw the annotations: {e}"))?;

    let decoded = image::load_from_memory(&burned)
        .map_err(|e| format!("Could not decode the burned image: {e}"))?;
    // RGB, not RGBA. A DIB with an alpha channel is pasted as fully transparent by several
    // applications - including Explorer's preview - and a screenshot has nothing to be transparent
    // about anyway.
    let rgb = decoded.to_rgb8();

    let mut bmp = Vec::new();
    BmpEncoder::new(&mut bmp)
        .encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ExtendedColorType::Rgb8,
        )
        .map_err(|e| format!("Could not encode for the clipboard: {e}"))?;

    // This used to read *"`set_clipboard` opens, empties, writes and closes"* and call it directly.
    // The first half of that sentence is true and the "empties" is NOT, for the Bitmap format - see
    // `put_bitmap_on_clipboard`, which is `BUG-84`. The Editor's Copy button therefore had the same
    // second-copy-pastes-the-first-image defect as the copy-only path, and this comment is why it
    // went unexamined.
    put_bitmap_on_clipboard(&bmp)?;

    Ok(format!(
        "Image copied, with its Markers and annotations ({} x {}).",
        rgb.width(),
        rgb.height()
    ))
}

#[cfg(not(windows))]
fn copy_burned_image(_ctx: &AppContext, _finding_id: &str) -> Result<String, String> {
    Err("Copying an image to the clipboard is implemented on Windows only.".to_string())
}

/// Shows the Finding's file in the operating system's file manager, selected.
///
/// `explorer /select,<path>` rather than opening the folder: the Vault holds every capture in one
/// directory, so opening it unselected leaves the Reviewer looking for a name they do not know.
///
/// `explorer.exe` returns a non-zero exit code on success, which is a documented quirk and not an
/// error - so the status is deliberately not checked. A genuine failure to spawn still surfaces.
#[cfg(windows)]
fn open_file_location(ctx: &AppContext, image_path: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    let absolute = if PathBuf::from(image_path).is_absolute() {
        PathBuf::from(image_path)
    } else {
        ctx.vault_path.join(image_path)
    };
    if !absolute.exists() {
        return Err("That file is no longer in the Vault.".to_string());
    }
    // `raw_arg`, and backslashes, and quotes - all three, and all three are load-bearing.
    //
    // `.arg()` applies Rust's own quoting rules, which wrap an argument containing a comma or a
    // space in quotes as a WHOLE: `"/select,C:\Vault\x.png"`. Explorer cannot parse that, and
    // its documented behaviour when it cannot parse its switch is to open the default folder - so
    // the owner got their Desktop. `raw_arg` passes the string through untouched, and the quotes go
    // around the PATH only, where Explorer expects them.
    //
    // The separators matter too: `image_path` is stored with forward slashes (it is a Vault-relative
    // key, not a Windows path), so joining it produces `C:\Vault/findings/x.png`, which Explorer
    // also rejects.
    let native = absolute.to_string_lossy().replace('/', "\\");
    std::process::Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{native}\""))
        .spawn()
        .map_err(|e| format!("Could not open the file manager: {e}"))?;
    Ok(())
}

#[cfg(not(windows))]
fn open_file_location(_ctx: &AppContext, _image_path: &str) -> Result<(), String> {
    Err("Showing a file in the file manager is implemented on Windows only.".to_string())
}

/// Opens a folder's OWN contents in Explorer - distinct from `open_file_location`, whose
/// `/select,<path>` opens the PARENT of whatever path it is given with that path merely
/// highlighted. Given the Vault root itself, `/select` would open the Vault's parent folder with
/// the Vault highlighted, not what "Show in Explorer" on a Vault means.
#[cfg(windows)]
fn open_folder(path: &Path) -> Result<(), String> {
    if !path.is_dir() {
        return Err("That folder no longer exists.".to_string());
    }
    std::process::Command::new("explorer.exe")
        .arg(path)
        .spawn()
        .map_err(|e| format!("Could not open the file manager: {e}"))?;
    Ok(())
}

#[cfg(not(windows))]
fn open_folder(_path: &Path) -> Result<(), String> {
    Err("Showing a folder in the file manager is implemented on Windows only.".to_string())
}

/// A Bundle's own folder, absolute - its Markdown and its burned image copies together, never a
/// single file inside it. `AD-4`'s layout puts a Bundle's own document at `bundles/{id}/bundle.md`
/// under the Vault root, so the folder is the document's own parent - derived from `markdown_path`
/// rather than assumed as `"bundles".join(id)` a second time, so a Bundle laid out differently one
/// day (nothing in this codebase promises it never will be) is still followed correctly.
fn bundle_folder_path(ctx: &AppContext, bundle: &Bundle) -> PathBuf {
    match Path::new(&bundle.markdown_path).parent() {
        Some(parent) if !parent.as_os_str().is_empty() => ctx.vault_path.join(parent),
        _ => ctx.vault_path.clone(),
    }
}

/// Ticket 12's Copy Markdown: the Bundle's whole stored document, with every image link rewritten
/// to an absolute path a local agent can open - same words, same order, only the link destinations
/// differ (`AD-9` as narrowed by `DEC-012`). The rewriting itself is the composer rebasing its own
/// document (`MarkdownSerializer::rebase_image_links`, ticket 10) - nothing here edits the text.
///
/// Works identically for a sealed Bundle (its original Findings gone): this reads only the Bundle's
/// own stored `markdown`/`markdown_path`, never a Finding.
fn bundle_markdown_for_clipboard(ctx: &AppContext, bundle_id: &str) -> Result<String, String> {
    let store = ctx
        .bundle_store
        .as_ref()
        .ok_or_else(|| "The Bundle library could not be opened.".to_string())?;
    let detail = store
        .get_bundle(bundle_id)
        .map_err(|e| format!("Could not read the Bundle: {e}"))?
        .ok_or_else(|| "That Bundle is no longer in the Library.".to_string())?;

    MarkdownSerializer::rebase_image_links(
        &detail.bundle.markdown,
        &ctx.vault_path.to_string_lossy(),
        &detail.bundle.markdown_path,
    )
    .map_err(|e| format!("Could not prepare the Markdown for copying: {e}"))
}

/// Which Findings a confirmed deletion should take.
///
/// The filmstrip's own rule, the one every file manager uses: a right-click ON the selection acts on
/// the selection; a right-click outside it acts on the one thing under the pointer. Read from the UI
/// model rather than the store, because "selected" is a property of what the Reviewer is looking at
/// and exists nowhere else.
fn findings_to_delete(window: &AppWindow, target: &str) -> Vec<String> {
    let selected = selected_finding_ids(window);
    if selected.iter().any(|id| id == target) {
        selected
    } else {
        vec![target.to_string()]
    }
}

/// Deletes a Finding, its row and its file. `FR-13`, `UC-7`.
///
/// The ROW FIRST, then the file. The other order can leave a Library entry pointing at a file that is
/// gone, which is the orphan class `FR-15` exists to report; this order can only leave a file with no
/// row, which the same sweeper finds and which costs disk rather than a broken Finding.
///
/// The blob failing is reported but does not undo the row: the Reviewer asked for this Finding to be
/// gone, and it is gone from the Library. Saying so and leaving a file behind beats resurrecting
/// something they just confirmed the deletion of.
fn delete_finding_everywhere(ctx: &AppContext, finding_id: &str) -> Result<bool, String> {
    let detail = ctx
        .finding_store
        .get_finding(finding_id)
        .map_err(|e| format!("Could not read the Finding: {e}"))?
        .ok_or_else(|| "That Finding is no longer in the Library.".to_string())?;

    ctx.finding_store
        .delete_finding(finding_id)
        .map_err(|e| format!("Could not delete the Finding: {e}"))?;

    // `true` means the row went and the file did not - an orphan, which `FR-15`'s sweeper will
    // find. Returned rather than reported here so a multiple deletion can count them.
    match ctx.vault_store.delete_blob(&detail.finding.image_path) {
        Ok(()) => Ok(false),
        Err(e) => {
            eprintln!("Deleted Finding {finding_id} but left its image behind: {e}");
            Ok(true)
        }
    }
}

/// How many files the Vault holds, for the confirmation to name.
///
/// Counted rather than estimated: "all 143 files" is a sentence somebody can weigh, and "some files"
/// is not.
fn count_vault_files(root: &Path) -> usize {
    fn walk(dir: &Path) -> usize {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return 0;
        };
        entries
            .flatten()
            .map(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path)
                } else {
                    1
                }
            })
            .sum()
    }
    ["findings", "bundles"]
        .iter()
        .map(|sub| walk(&root.join(sub)))
        .sum()
}

/// Moves every file in the Vault to a new root. `FR-16`.
///
/// **The database is not touched, and that is not an oversight - it is why this is small.**
/// `finding.image_path` is a Vault-RELATIVE key (`findings/xyz.png`), decided when the schema was
/// written and stated in the data model: *"Relative to the Vault root, never absolute. A Vault move
/// must not rewrite rows."* So moving the files and changing one setting is the whole migration.
///
/// Order matters and it is: move everything, THEN write the setting. The other way round leaves the
/// product pointing at a folder the captures have not reached, and every Finding in the Library is
/// broken until they do. This way a failure part-way leaves the old Vault authoritative and the old
/// setting in place - the Reviewer sees an error and nothing is lost.
///
/// `fs::rename` first, because within one volume it is instant and atomic per file. It fails across
/// volumes, so the fallback is copy-then-remove - and the remove is CHECKED. `AGENTS.md` records the
/// archived version of this function swallowing both `fs::remove_file` results with `let _ =`, which
/// left the old copies behind silently.
fn migrate_vault(from: &Path, to: &Path) -> Result<usize, String> {
    if from == to {
        return Ok(0);
    }
    if !to.exists() {
        std::fs::create_dir_all(to)
            .map_err(|e| format!("Could not create {}: {e}", to.display()))?;
    }
    // A Vault inside the old Vault would copy itself into itself for ever.
    if to.starts_with(from) {
        return Err("That folder is inside the current Vault. Pick one beside it.".to_string());
    }

    let mut moved = 0usize;
    // Only the two the product owns. Anything else in the folder is the Reviewer's and is left where
    // it is - the Vault is a directory on their disk, not ours to tidy.
    for sub in ["findings", "bundles"] {
        let source = from.join(sub);
        if !source.is_dir() {
            continue;
        }
        let target = to.join(sub);
        std::fs::create_dir_all(&target)
            .map_err(|e| format!("Could not create {}: {e}", target.display()))?;

        let entries = std::fs::read_dir(&source)
            .map_err(|e| format!("Could not read {}: {e}", source.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Could not read {}: {e}", source.display()))?;
            let name = entry.file_name();
            let (src, dst) = (entry.path(), target.join(&name));

            if src.is_dir() {
                // A Bundle is a directory of burned copies. Recursed rather than flattened, so a
                // Bundle keeps the folder its Markdown's relative links point into.
                moved += migrate_vault_dir(&src, &dst)?;
                continue;
            }
            move_one(&src, &dst)?;
            moved += 1;
        }
        // Best-effort, and deliberately not checked the way `move_one`'s `remove_file` is: nothing
        // downstream depends on this directory being gone, unlike a leftover FILE, which would
        // silently double the disk the Vault uses. An empty `findings`/`bundles` folder left behind
        // is untidy, not incorrect - and it fails harmlessly if the Reviewer's own files (never
        // ours to touch) are still in there.
        let _ = std::fs::remove_dir(&source);
    }
    Ok(moved)
}

fn migrate_vault_dir(from: &Path, to: &Path) -> Result<usize, String> {
    std::fs::create_dir_all(to).map_err(|e| format!("Could not create {}: {e}", to.display()))?;
    let mut moved = 0usize;
    let entries =
        std::fs::read_dir(from).map_err(|e| format!("Could not read {}: {e}", from.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Could not read {}: {e}", from.display()))?;
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if src.is_dir() {
            moved += migrate_vault_dir(&src, &dst)?;
        } else {
            move_one(&src, &dst)?;
            moved += 1;
        }
    }
    // Same best-effort cleanup as `migrate_vault`'s own top-level `findings`/`bundles`: this
    // directory (a Bundle's own folder, recursed into) is left empty once every file inside has
    // moved, and nothing depends on reclaiming it.
    let _ = std::fs::remove_dir(from);
    Ok(moved)
}

/// One file, renamed if the volume allows it and copied if it does not.
fn move_one(src: &Path, dst: &Path) -> Result<(), String> {
    if std::fs::rename(src, dst).is_ok() {
        return Ok(());
    }
    std::fs::copy(src, dst).map_err(|e| format!("Could not copy {}: {e}", src.display()))?;
    // CHECKED. The archived version of this used `let _ =` here and left the old copies behind - a
    // Vault "move" that silently doubled the disk it used.
    std::fs::remove_file(src).map_err(|e| {
        format!(
            "Copied {} but could not remove the original: {e}",
            src.display()
        )
    })?;
    Ok(())
}

/// The domain's `HotkeyAction` for the id the Settings screen carries.
///
/// The ids ARE the domain's own snake_case names, so this is a parse rather than a mapping - and an
/// unknown one returns `None` instead of defaulting, because defaulting would rebind the wrong action.
fn hotkey_action_from_id(id: &str) -> Option<HotkeyAction> {
    match id {
        "capture" => Some(HotkeyAction::Capture),
        "open_editor" => Some(HotkeyAction::OpenEditor),
        _ => None,
    }
}

/// The Vault path as CONFIGURED (what a fresh launch would read at `AppContext::init` time), which
/// is not always `ctx.vault_path` - the one thing that field cannot be, since it is frozen for the
/// life of the process. Falls back the same way `init` does, so a mid-session read and a startup
/// read never disagree about what "nothing chosen yet" means.
fn configured_vault_path(ctx: &AppContext) -> PathBuf {
    match ctx.settings_store.get(&SettingKey::VaultPath) {
        Ok(Some(Setting {
            value: SettingValue::String(s),
            ..
        })) => PathBuf::from(s),
        _ => default_vault_path(),
    }
}

/// The Quality Budget as it stands, or the default if nothing has been chosen.
///
/// A read failure takes the default rather than propagating: this feeds a settings screen, and a
/// screen that refuses to open because one row would not parse is worse than one showing the value
/// the product is actually using.
fn current_budget(ctx: &AppContext) -> QualityBudget {
    ctx.settings_store
        .get(&SettingKey::QualityBudget)
        .ok()
        .flatten()
        .and_then(|setting| match setting.value {
            SettingValue::QualityBudget(budget) => Some(budget),
            _ => None,
        })
        .unwrap_or_else(|| QualityBudget::new(NamedBudget::Auto, None))
}

fn store_budget(ctx: &AppContext, budget: &QualityBudget) -> Result<(), CoreError> {
    ctx.settings_store.set(&Setting {
        key: SettingKey::QualityBudget,
        value: SettingValue::QualityBudget(budget.clone()),
        updated_at: SystemClock::new().now_rfc3339(),
    })
}

/// Why a key combination cannot become a shortcut.
///
/// Borrowed, structure and reasoning, from `wira-desk`'s `ShortcutError` - the owner's own other
/// desktop product, at `D:/Developer/wiradigital.id/wira-desk`. What it gets right and this did not:
/// each failure is a DISTINCT case with its own sentence, and the sentence says what to do next
/// rather than only what went wrong.
///
/// The version this replaces had one path - compose a string, hand it to `validate_and_rebind`, show
/// whatever error came back. So "you held Ctrl and nothing else" and "Windows owns Ctrl+Alt+Del" and
/// "your other Snapdown hotkey already uses this" were the same silence or the same generic refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ShortcutRefusal {
    /// A bare key with no modifier. Not a safe global shortcut: it would fire while typing, in every
    /// application, for ever.
    NoModifier,
    /// A key this product cannot name to the OS.
    UnsupportedKey(String),
    /// Windows or the shell owns this chord. `advice` is empty when nothing can be done about it.
    Reserved { owner: String, advice: String },
}

impl ShortcutRefusal {
    /// One sentence, for the Reviewer, saying what to do next where there is something to do.
    fn message(&self) -> String {
        match self {
            Self::NoModifier => {
                "A shortcut needs at least one of Ctrl, Alt or Win with it. A bare key, or Shift on \
                 its own, would fire while you type, in every application."
                    .to_string()
            }
            Self::UnsupportedKey(key) => {
                format!("Snapdown cannot register {key} as part of a shortcut.")
            }
            Self::Reserved { owner, advice } if advice.is_empty() => {
                format!("Windows uses this combination to {owner}. It cannot be reassigned.")
            }
            Self::Reserved { owner, advice } => {
                format!("Windows uses this combination to {owner}. {advice}")
            }
        }
    }
}

/// Combinations Windows or a well-known tool already owns.
///
/// Covers the Ctrl/Alt/Shift/Win space Snapdown's own hotkeys live in, and only the ones a
/// Reviewer might plausibly reach for; the `Win+` entries mirror the exact set `wira-desk`'s own
/// Key Check panel documents in its tip line. It is deliberately short: a long list of chords
/// nobody would try is a list nobody maintains.
///
/// The point is not to be exhaustive - the OS refuses what it refuses, and that is reported too. The
/// point is that a REFUSAL WE CAN PREDICT should say WHO took it, before the Reviewer has to guess.
fn reserved_chord(shortcut: &str) -> Option<ShortcutRefusal> {
    let owner_and_advice: &[(&str, &str, &str)] = &[
        (
            "CommandOrControl+Alt+Delete",
            "reach the secure sign-in screen",
            "",
        ),
        (
            "CommandOrControl+Shift+Escape",
            "open Task Manager",
            "",
        ),
        ("Alt+Tab", "switch windows", ""),
        ("Alt+F4", "close the active window", ""),
        (
            "CommandOrControl+Shift+S",
            "start a Snipping Tool capture on some Windows builds",
            "Snapdown claims it first when it is running, so this is usually safe - but if a capture \
             ever opens the wrong tool, that is why.",
        ),
        (
            "CommandOrControl+C",
            "copy",
            "Pick something with Shift or Alt in it as well.",
        ),
        (
            "CommandOrControl+V",
            "paste",
            "Pick something with Shift or Alt in it as well.",
        ),
        (
            "CommandOrControl+X",
            "cut",
            "Pick something with Shift or Alt in it as well.",
        ),
        (
            "CommandOrControl+Z",
            "undo",
            "Pick something with Shift or Alt in it as well.",
        ),
        (
            "CommandOrControl+A",
            "select all",
            "Pick something with Shift or Alt in it as well.",
        ),
        (
            "CommandOrControl+S",
            "save, in almost every application",
            "Pick something with Shift or Alt in it as well.",
        ),
        // The `Win+` chords the shell itself intercepts before `RegisterHotKey` ever sees them -
        // not a guess, but the exact set `wira-desk`'s own Key Check panel already documents in its
        // tip line: "Windows reserves Win + 1..9, Win + E, Win + D, and Win + Ctrl + <-/->." Nothing
        // else with `Win` in it is refused here; most other Win combinations register fine.
        ("Super+1", "launch the first pinned taskbar app", ""),
        ("Super+2", "launch the second pinned taskbar app", ""),
        ("Super+3", "launch the third pinned taskbar app", ""),
        ("Super+4", "launch the fourth pinned taskbar app", ""),
        ("Super+5", "launch the fifth pinned taskbar app", ""),
        ("Super+6", "launch the sixth pinned taskbar app", ""),
        ("Super+7", "launch the seventh pinned taskbar app", ""),
        ("Super+8", "launch the eighth pinned taskbar app", ""),
        ("Super+9", "launch the ninth pinned taskbar app", ""),
        ("Super+E", "open File Explorer", ""),
        ("Super+D", "show the desktop", ""),
        (
            "CommandOrControl+Super+Left",
            "move the active window or switch virtual desktops",
            "",
        ),
        (
            "CommandOrControl+Super+Right",
            "move the active window or switch virtual desktops",
            "",
        ),
    ];

    owner_and_advice
        .iter()
        .find(|(chord, _, _)| chord.eq_ignore_ascii_case(shortcut))
        .map(|(_, owner, advice)| ShortcutRefusal::Reserved {
            owner: (*owner).to_string(),
            advice: (*advice).to_string(),
        })
}

/// The physical key `global_hotkey`'s `CommandOrControl` token resolves to on this OS - `Ctrl` on
/// Windows, `Cmd` on macOS. The stored/parsed shortcut string always says `CommandOrControl`
/// (`DEFAULT_HOTKEY_CAPTURE` and every rebind after it), because that is the one spelling
/// `HotKey::from_str` accepts on both platforms; only the DISPLAY needs to pick a side.
#[cfg(target_os = "macos")]
const PRIMARY_MODIFIER_DISPLAY: &str = "Cmd";
#[cfg(not(target_os = "macos"))]
const PRIMARY_MODIFIER_DISPLAY: &str = "Ctrl";

/// The physical key the live-recording panel calls "the OS modifier key" - `Win` on Windows,
/// `Command` on macOS. Distinct from `PRIMARY_MODIFIER_DISPLAY`: this labels the raw `meta` key as
/// it is held during recording, not the composed accelerator token.
#[cfg(target_os = "macos")]
const META_KEY_DISPLAY: &str = "Command";
#[cfg(not(target_os = "macos"))]
const META_KEY_DISPLAY: &str = "Win";

/// A stored shortcut, reworded for the Reviewer rather than for `global_hotkey`.
///
/// `"CommandOrControl+Shift+S"` is what gets parsed and persisted, and showing that literal
/// string was the owner's own complaint: *"apa bisa dia deteksi saja ini macos atau windows,
/// sehingga gak usah ada tulisan CommandOrControl."* The stored spelling cannot change - it is
/// what `HotKey::from_str` and every existing row in the database already agree on - so only the
/// text put in front of the Reviewer changes.
fn display_shortcut(shortcut: &str) -> String {
    if shortcut.is_empty() {
        return String::new();
    }
    shortcut
        .replace("CommandOrControl", PRIMARY_MODIFIER_DISPLAY)
        .replace("Super", META_KEY_DISPLAY)
}

/// A safe, human-facing label for a Slint key event's own `text`, or `None` when there is nothing
/// worth showing.
///
/// Found the hard way: a bare key press with no modifier held (Enter, an arrow key, Ctrl+C's own
/// text arriving as the ASCII control code rather than the letter) was being echoed into the Key
/// Check panel's readout chip as-is, and a control character renders as an unreadable tofu box, not
/// a key a Reviewer can recognise. `char::is_control` catches all of those in one place, rather than
/// the narrow four-item list of bare-modifier codes the first cut of this checked for.
fn displayable_key_text(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }
    if text.chars().count() == 1 {
        let ch = text.chars().next().unwrap_or(' ');
        // Control characters are Slint's bare-modifier signal, already excluded elsewhere. Anything
        // outside letters/digits/ASCII punctuation is a codepoint this product has no name for - a
        // private-use-area key code, say - and showing it raw is how a tofu glyph like `ℐ` ends up
        // in a Reviewer-facing sentence instead of a word they can read.
        return if ch.is_alphanumeric() || ch.is_ascii_punctuation() {
            Some(text.to_uppercase())
        } else {
            None
        };
    }
    // A named key already spelled out by Slint - `F5`, `Home`, `Left` - safe to show as-is.
    Some(text.to_string())
}

/// A live readout of the chord being composed, for the Key Check panel's readout chip.
///
/// Distinct from `shortcut_from_key`: this never refuses anything, it only describes the current
/// key state, including a mid-gesture one with no completing key yet (modifiers alone, trailing
/// nothing). Rust builds it, not Slint, so `displayable_key_text`'s control-character guard is the
/// only place that decision is made.
fn format_chord_preview(ctrl: bool, alt: bool, shift: bool, win: bool, text: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    if ctrl {
        parts.push(PRIMARY_MODIFIER_DISPLAY.to_string());
    }
    if win {
        parts.push(META_KEY_DISPLAY.to_string());
    }
    if alt {
        parts.push("Alt".to_string());
    }
    if shift {
        parts.push("Shift".to_string());
    }
    if let Some(key) = displayable_key_text(text) {
        parts.push(key);
    }
    parts.join(" + ")
}

/// Turns a Slint key event into the shortcut string `validate_and_rebind` expects.
///
/// Composed HERE and not in Slint, because the format is the hotkey registrar's business:
/// `CommandOrControl+Shift+S`, matching `DEFAULT_HOTKEY_CAPTURE` so a rebind and a default read the
/// same way in the database.
///
/// The CANONICAL shortcut string, or why this press is not one.
///
/// Canonical is the point, and it is `wira-desk`'s rule: *"Returns the canonical string to persist,
/// so a caller cannot accidentally store the user's raw input in a non-canonical form."* Modifiers
/// always in the same order, the key always upper case, so two presses of the same combination
/// cannot store two different strings and then fail to compare equal.
///
/// `Ok(None)` is the third state and it matters: the press is INCOMPLETE, not wrong. The Reviewer
/// holding Ctrl on the way to Ctrl+Shift+S is mid-gesture, and telling them off for it would make
/// the field impossible to use.
///
/// `win` qualifies a shortcut the same way `ctrl`/`alt` do - the owner's own question, *"kenapa Win
/// gak bisa jadi shortcut?"* Nothing about `RegisterHotKey` refuses `MOD_WIN`; the physical key was
/// only ever excluded here because the display layer had no way to spell it, which `META_KEY_DISPLAY`
/// now solves. What genuinely cannot be reassigned is the specific set of Win chords the shell
/// itself intercepts before any app sees them, and `reserved_chord` is where those live.
fn shortcut_from_key(
    ctrl: bool,
    alt: bool,
    shift: bool,
    win: bool,
    text: &str,
) -> Result<Option<String>, ShortcutRefusal> {
    // A modifier arriving as its own key. Slint sends these as control characters.
    let bare_modifier = [
        "\u{11}", // Control
        "\u{12}", // Shift
        "\u{13}", // Alt
        "\u{14}", // Meta
    ];
    if text.is_empty() || bare_modifier.contains(&text) {
        // Mid-gesture. Not a refusal, and not a shortcut yet either.
        return Ok(None);
    }

    if !ctrl && !alt && !win && !shift {
        return Err(ShortcutRefusal::NoModifier);
    }
    // Shift alone is not a modifier for this purpose: Shift+A is how you type a capital A.
    if !ctrl && !alt && !win {
        return Err(ShortcutRefusal::NoModifier);
    }

    let key = if text.chars().count() == 1 {
        let ch = text.chars().next().unwrap_or(' ');
        if !ch.is_ascii_alphanumeric() {
            let key_desc = match displayable_key_text(text) {
                Some(label) => format!("the '{label}' key"),
                None => "this key".to_string(),
            };
            return Err(ShortcutRefusal::UnsupportedKey(key_desc));
        }
        text.to_uppercase()
    } else {
        // A named key - `F5`, `Home`. Slint gives these as multi-character strings, and
        // `global_hotkey` parses most of them; the ones it does not are refused by name below.
        text.to_string()
    };

    // Fixed order, `Win` between `Ctrl` and `Alt` - matching the Key Check panel's own chip order,
    // so the row button's canonical text and the panel above it never disagree on how a chord reads.
    let mut parts: Vec<&str> = Vec::new();
    if ctrl {
        parts.push("CommandOrControl");
    }
    if win {
        parts.push("Super");
    }
    if alt {
        parts.push("Alt");
    }
    if shift {
        parts.push("Shift");
    }
    let shortcut = format!("{}+{key}", parts.join("+"));

    if let Some(refusal) = reserved_chord(&shortcut) {
        // A `Reserved` entry with advice is a WARNING, not a wall - Snapdown may well win the race
        // for it. Only the ones with no advice are impossible, and those are the OS's own.
        if matches!(&refusal, ShortcutRefusal::Reserved { advice, .. } if advice.is_empty()) {
            return Err(refusal);
        }
    }

    Ok(Some(shortcut))
}

/// The settings key the theme choice is stored under.
///
/// A function rather than two string literals at the two call sites: a typo in one of them would
/// store the choice and never read it back, and nothing would fail.
fn theme_setting_key() -> SettingKey {
    SettingKey::Custom("theme_dark".to_string())
}

/// Fills the Settings screen from the stores and the registrars.
///
/// Called every time it opens and after every change, so nothing on it is a copy that can drift from
/// what the product is actually using - the mistake `BUG-45` is, one screen over.
fn load_settings_into_window(
    window: &AppWindow,
    ctx: &AppContext,
    startup: &dyn StartupRegistrar,
    hotkeys: &DesktopHotkeyRegistrar,
) {
    match startup.is_enabled() {
        Ok(enabled) => {
            window.set_run_at_startup(enabled);
            window.set_startup_problem(SharedString::new());
        }
        // Reported on the screen rather than swallowed. The registry key may be unreadable, and a
        // toggle that silently shows `false` for that reason is a toggle that lies.
        Err(e) => {
            window.set_run_at_startup(false);
            window.set_startup_problem(
                format!("Windows startup registration could not be read: {e}").into(),
            );
        }
    }

    window.set_open_editor_after_capture(open_editor_after_capture(ctx));
    // The CONFIGURED path, not `ctx.vault_path`: the two deliberately diverge right after a move
    // (`on_vault_migration_confirmed`'s own comment explains why `ctx.vault_path` stays frozen until
    // restart), and the Settings screen must show what was just saved, not what capture is still
    // using in the meantime - the toast already says which one that is and for how long.
    window.set_vault_path(configured_vault_path(ctx).display().to_string().into());

    let budget = current_budget(ctx);
    window.set_budget_name(budget.named.as_str().into());
    let resolved = budget.resolve(u32::MAX);
    window.set_budget_max_long_edge(resolved.max_long_edge as i32);
    window.set_budget_encoder_quality(i32::from(resolved.encoder_quality));
    window.set_budget_resize_percent(i32::from(resolved.resize_percent));
    // Two lines, and they answer different questions. The explainer says what the CHOICE means; the
    // readout says what it currently resolves to. The G3 design has both, and collapsing them into
    // one leaves `Auto` able to say neither - its whole point is that it resolves differently per
    // capture, so one line is either a description with no numbers or numbers that are a lie.
    window.set_budget_explainer(
        match budget.named {
            NamedBudget::Auto => {
                "Adaptive: decided per capture from the area actually selected. A tooltip stays sharp; a 4K dashboard is reduced."
            }
            NamedBudget::Sharp => "Largest and most legible. Costs an agent the most to read.",
            NamedBudget::Balanced => "The default trade between legibility and cost.",
            NamedBudget::Small => "Cheapest to send. Fine text may not survive it.",
            NamedBudget::Custom => {
                "Your own pair, applied to every capture regardless of its size."
            }
        }
        .into(),
    );
    window.set_budget_readout(
        match budget.named {
            NamedBudget::Auto => "Resolved per capture  \u{b7}  no fixed pair".to_string(),
            _ => format!(
                "{} px long edge  \u{b7}  quality {}",
                resolved.max_long_edge, resolved.encoder_quality
            ),
        }
        .into(),
    );

    let bindings = hotkeys.get_bindings();
    let failures = hotkeys.get_startup_failures();
    let rows: Vec<HotkeyRow> = [
        (
            HotkeyAction::Capture,
            "Freezes the desktop and opens the selection overlay. The whole loop starts here.",
        ),
        (
            HotkeyAction::OpenEditor,
            "Brings the Editor forward without taking a capture.",
        ),
    ]
    .into_iter()
    .map(|(action, why)| {
        let shortcut = bindings.get(&action).cloned().unwrap_or_default();
        let problem = failures.get(&action).cloned().unwrap_or_default();
        HotkeyRow {
            // `action.as_str()`/`action.label()`, not a second literal here: the conflict
            // message in `hotkey.rs` names an action with the same `label()`, and two
            // independent copies of that string is exactly the drift that got a defect-register
            // row wrong once already (`AGENTS.md`'s stale-claim pitfall).
            action: action.as_str().into(),
            label: action.label().into(),
            why: why.into(),
            // Displayed, never stored, in the OS's own words - `CommandOrControl` is
            // `global-hotkey`'s cross-platform accelerator token, not something a Reviewer
            // should have to decode.
            shortcut: display_shortcut(&shortcut).into(),
            enabled: hotkeys.is_enabled(action),
            active: hotkeys.is_registered(action.as_str()),
            problem: problem.into(),
        }
    })
    .collect();
    window.set_hotkeys(ModelRc::from(Rc::new(VecModel::from(rows))));
    window.set_hotkey_meta_key_label(META_KEY_DISPLAY.into());

    window.set_app_version(format!("Snapdown {}", env!("CARGO_PKG_VERSION")).into());

    // Stated plainly rather than shown as a working panel. `BUG-59`: the Bridge executable exists
    // and the Local API it talks to does not, so there is nothing here to configure yet.
    window.set_bridge_status(
        "Not available yet. The Bridge lets an agent read your Findings directly, and the part \
         of Snapdown it connects to has not been built - so there is nothing to configure here \
         yet.\n\nUntil it exists, Assemble a Bundle and use Copy Markdown: an agent can read \
         that file and the images beside it."
            .into(),
    );
}

/// Puts one sentence on screen, in the Editor's result line.
///
/// Written before the action's own logging rather than instead of it: `eprintln!` reaches a console
/// a release build on Windows does not have, so it is the developer's copy, not the Reviewer's.
fn toast(window: &AppWindow, message: impl Into<SharedString>, is_error: bool) {
    window.set_toast_is_error(is_error);
    // Cleared first so a second result within the dismissal window restarts the timer instead of
    // inheriting the remainder of the first one's.
    window.set_toast_text(SharedString::new());
    window.set_toast_text(message.into());
}

/// The Hotkeys tab's OWN feedback, not the global toast: a validation error, a conflict, or a
/// bind confirmation, shown in the sticky panel at the bottom of that tab rather than over the
/// Editor window behind it. `BUG` reported by the Reviewer: every one of these used to go through
/// `toast`, which rendered in the wrong window entirely while Settings was open.
fn hotkey_feedback(window: &AppWindow, message: impl Into<SharedString>, is_error: bool) {
    window.set_hotkey_feedback_is_error(is_error);
    window.set_hotkey_feedback(SharedString::new());
    window.set_hotkey_feedback(message.into());
}

/// The Findings the Reviewer has ticked for a Bundle, in the strip's own order.
///
/// The strip's order is the Bundle's order, and that is deliberate: the position a Finding takes in
/// the composed Markdown is the one it was seen in, not one this function invents.
fn selected_finding_ids(window: &AppWindow) -> Vec<String> {
    window
        .get_filmstrip_items()
        .iter()
        .filter(|t| t.is_selected)
        .map(|t| t.id.to_string())
        .collect()
}

/// Re-derives the count the Assemble tile reads, so it is never a second copy that can disagree
/// with the ticks themselves.
fn refresh_selection_count(window: &AppWindow) {
    let count = window
        .get_filmstrip_items()
        .iter()
        .filter(|t| t.is_selected)
        .count();
    window.set_selected_finding_count(count as i32);
}

/// One click on a filmstrip card, read the way a file manager reads one.
///
/// - plain: this Finding alone is selected, and it becomes the anchor;
/// - Ctrl: this Finding's selection flips, the rest keep theirs, and it becomes the anchor;
/// - Shift: everything from the ANCHOR through this Finding is selected, replacing what was there.
///   The anchor does not move;
/// - Ctrl+Shift: the same range, added to what is already selected. The anchor does not move.
///
/// The clicked card is ALWAYS the one the canvas then shows, whatever the modifiers did to the
/// selection.
///
/// **The anchor is the whole point of this function, and the first version got it wrong.** It used
/// the ACTIVE card as the anchor - the one on the canvas - which moves on every click including a
/// Shift-click. So a Shift-click extended from the previous Shift-click rather than from where the
/// Reviewer started, and reversing direction extended away from the anchor instead of shrinking the
/// range back through it. A file manager keeps a separate anchor for exactly this reason: it is what
/// makes Shift reversible.
///
/// Rewritten in place rather than by rebuilding the strip: a rebuild decodes and rescales every
/// thumbnail, which measured 320 ms on the owner's library and is why `BUG-41` exists.
fn click_finding(window: &AppWindow, ctx: &AppContext, id: &str, ctrl: bool, shift: bool) {
    let items = window.get_filmstrip_items();
    let Some(vec_model) = items.as_any().downcast_ref::<VecModel<FindingThumb>>() else {
        // Not the model this function knows how to update in place. A full rebuild is slow rather
        // than wrong; doing nothing would be wrong.
        load_findings_into_window(window, ctx, Some(id));
        return;
    };

    let rows: Vec<FindingThumb> = (0..vec_model.row_count())
        .filter_map(|row| vec_model.row_data(row))
        .collect();
    let Some(clicked) = rows.iter().position(|thumb| thumb.id == id) else {
        return;
    };

    // The anchor is held by id, not by index: a capture rebuilds the strip and every index moves.
    // An anchor that is no longer in the strip - it went into a Bundle, say - falls back to the
    // click, which makes Shift behave like a plain click rather than selecting a wild range.
    let anchor = SELECTION_ANCHOR.with(|held| {
        let held = held.borrow();
        rows.iter()
            .position(|thumb| thumb.id.as_str() == held.as_str())
            .unwrap_or(clicked)
    });
    let (lo, hi) = if anchor <= clicked {
        (anchor, clicked)
    } else {
        (clicked, anchor)
    };

    for (row, thumb) in rows.into_iter().enumerate() {
        let selected = if shift {
            // Ctrl+Shift adds the range to what is there; Shift alone REPLACES. Replacing is what
            // makes reversing direction work: the cards on the far side of the anchor fall out of
            // the range and are therefore deselected, with no separate step to deselect them.
            (lo..=hi).contains(&row) || (ctrl && thumb.is_selected)
        } else if ctrl {
            if row == clicked {
                !thumb.is_selected
            } else {
                thumb.is_selected
            }
        } else {
            row == clicked
        };
        let active = row == clicked;

        if selected != thumb.is_selected || active != thumb.is_active {
            vec_model.set_row_data(
                row,
                FindingThumb {
                    is_selected: selected,
                    is_active: active,
                    ..thumb
                },
            );
        }
    }

    if !shift {
        SELECTION_ANCHOR.with(|held| *held.borrow_mut() = id.to_string());
    }

    refresh_selection_count(window);
    load_active_detail(window, ctx, id);
}

/// Everything one Bundle is about to write, worked out before anything is written.
struct PlannedBundle {
    markdown: String,
    markdown_path: String,
    items: Vec<BundleItem>,
    /// Relative Vault path to burned bytes, in position order.
    blobs: Vec<(String, Vec<u8>)>,
}

/// Hand-written rather than derived: a derived `Debug` would dump every burned pixel of every image
/// into the output of the one test that prints it.
impl std::fmt::Debug for PlannedBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PlannedBundle {{ markdown_path: {:?}, markdown: {} bytes, blobs: [",
            self.markdown_path,
            self.markdown.len()
        )?;
        for (index, (path, bytes)) in self.blobs.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{path} ({} bytes)", bytes.len())?;
        }
        write!(f, "] }}")
    }
}

/// Turns the ticked ids into Findings, refusing the whole Bundle if any of them cannot be resolved.
///
/// `UC-9` is all-or-nothing. `BUG-22` is this loop written as `if let Some(detail)` with no else: a
/// Reviewer who selected five Findings and whose library had lost one got a four-Finding Bundle that
/// reported success, with the positions renumbered silently around the gap.
fn resolve_bundle_findings<F>(ids: &[String], get: F) -> Result<Vec<FindingDetail>, String>
where
    F: Fn(&str) -> Result<Option<FindingDetail>, CoreError>,
{
    let mut details = Vec::with_capacity(ids.len());
    for id in ids {
        match get(id) {
            Ok(Some(detail)) => details.push(detail),
            Ok(None) => {
                return Err(format!(
                    "Finding {id} is no longer in the library. Nothing was written."
                ))
            }
            Err(e) => {
                return Err(format!(
                    "Could not read Finding {id}: {e}. Nothing was written."
                ))
            }
        }
    }
    Ok(details)
}

/// Burns every Finding's Markers into a copy of its image and composes the Markdown that references
/// those copies - all in memory, so a refusal leaves nothing on disk to clean up.
///
/// Two defects in the archived Tauri caller are answered here by construction:
///
/// - `BUG-19`: it recorded `BundleItem.image_path` and never wrote a file there. Every path this
///   returns arrives with the bytes that belong at it, so the caller cannot record one without the
///   other;
/// - `BUG-23`: it read each image inside `if let Ok(bytes)` with no else, dropped the item, and
///   proceeded - putting a Publication live without it. A read failure refuses the Bundle.
fn plan_bundle<R>(
    bundle_id: &str,
    name: &str,
    notes: &str,
    details: &[FindingDetail],
    read_image: R,
) -> Result<PlannedBundle, String>
where
    R: Fn(&str) -> Result<Vec<u8>, CoreError>,
{
    let mut items: Vec<BundleItem> = Vec::with_capacity(details.len());
    let mut blobs: Vec<(String, Vec<u8>)> = Vec::with_capacity(details.len());

    for (index, detail) in details.iter().enumerate() {
        let position = (index + 1) as u32;
        let f = &detail.finding;

        let source_bytes = read_image(&f.image_path).map_err(|e| {
            format!(
                "Could not read the image for Finding {position} ({}): {e}. Nothing was written.",
                f.image_path
            )
        })?;

        let dims = ImageDimensions {
            width: f.image_width,
            height: f.image_height,
        };
        // `burn_all`, not `burn_markers`. The five annotation shapes have been drawable since the
        // burner was written and were never handed any: `plan_bundle` passed the Markers and an
        // empty slice, so a Bundle's copy carried the reticles and dropped every box, arrow, callout
        // and - worst - every blur redaction. That is `BUG-72`, and this line is where it surfaced:
        // a Finding could not HOLD an annotation, so there was never one to pass.
        let burned_bytes = MarkerBurner::burn_all(
            &source_bytes,
            &dims,
            &detail.markers,
            &detail.visual_annotations,
            // The FINDING's own quality. Encoding the handoff lossless while the Finding was
            // quantised would make the copy larger than the original, paying for precision that was
            // already thrown away.
            f.resolved_encoder_quality
                .unwrap_or(snapdown_store::image::LOSSLESS),
        )
        .map_err(|e| {
            format!("Could not draw the Markers for Finding {position}: {e}. Nothing was written.")
        })?;

        let image_path = format!("bundles/{bundle_id}/finding_{position}_burned.png");
        // A deterministic item id. `id_from_parts` would hand every item in this loop the same
        // millisecond, and the ids would collide.
        let item_id = format!("{bundle_id}-item-{position}");
        items.push(
            BundleItem::new(
                item_id,
                bundle_id.to_string(),
                f.id.clone(),
                position,
                image_path.clone(),
            )
            .map_err(|e| format!("Could not record Finding {position} in the Bundle: {e}"))?,
        );
        blobs.push((image_path, burned_bytes));
    }

    let md_items: Vec<(&BundleItem, &FindingDetail)> = items.iter().zip(details.iter()).collect();

    // One binding feeds both the serializer and the record, so the document's location and the base
    // its links are written against cannot drift apart. They did once, and every image link in every
    // Bundle resolved to nothing for five waves (`BUG-86`).
    let markdown_path = format!("bundles/{bundle_id}/bundle.md");

    Ok(PlannedBundle {
        markdown: MarkdownSerializer::serialize_bundle(name, notes, &md_items, &markdown_path),
        markdown_path,
        items,
        blobs,
    })
}

/// A Bundle worked out but not yet written, while the Reviewer looks at it.
///
/// `UC-9`'s own screen has always specified a review step, and until now Assemble wrote the files
/// the instant the tile was clicked - the first sight of the document was a folder in the Vault.
/// Everything here is the output of `plan_bundle`, held rather than written.
struct PendingBundle {
    bundle_id: String,
    name: String,
    /// The Bundle's own note - what this handoff is about. Written in the preview, because it exists
    /// nowhere else in the product: the Findings each describe their own image, and nothing until now
    /// said what the set of them was for.
    notes: String,
    /// Kept beside the plan because the Markdown has to be re-composed when the name changes, and
    /// re-composing needs the Findings that the burned copies came from.
    details: Vec<FindingDetail>,
    planned: PlannedBundle,
}

/// Prepares a Bundle from what is ticked, without writing anything.
fn prepare_bundle(window: &AppWindow, ctx: &AppContext) -> Result<PendingBundle, String> {
    if ctx.bundle_store.is_none() {
        return Err("The Bundle library could not be opened, so nothing can be assembled.".into());
    }

    let ids = selected_finding_ids(window);
    if ids.is_empty() {
        return Err("Tick at least one Finding in the strip first.".into());
    }

    let clock = SystemClock::new();
    let entropy = SystemEntropySource::new();
    let bundle_id = id_from_parts(clock.now_unix_millis(), entropy.random_bytes_10());
    let name = format!("Review {}", chrono::Local::now().format("%Y-%m-%d %H:%M"));

    let details = resolve_bundle_findings(&ids, |id| ctx.finding_store.get_finding(id))?;
    let planned = plan_bundle(&bundle_id, &name, "", &details, |path| {
        ctx.vault_store.read_blob(path)
    })?;

    Ok(PendingBundle {
        bundle_id,
        name,
        notes: String::new(),
        details,
        planned,
    })
}

/// Re-composes only the Markdown, for a name change. The name is the document's H1 and nothing else,
/// so re-burning every image on each keystroke would be absurd.
fn recompose_markdown(pending: &mut PendingBundle) {
    let pairs: Vec<(&BundleItem, &FindingDetail)> = pending
        .planned
        .items
        .iter()
        .zip(pending.details.iter())
        .collect();
    pending.planned.markdown = MarkdownSerializer::serialize_bundle(
        &pending.name,
        &pending.notes,
        &pairs,
        &pending.planned.markdown_path,
    );
}

/// The widest a preview image is ever drawn, so the modal does not hold five full-resolution decodes
/// at once.
///
/// The page is at most ~640px wide and the image takes half of that, so 900 covers it on a HiDPI
/// display. Without a bound, five 1500x1500 Findings would be 45 MB of RGBA held for as long as the
/// modal is open, which is the mistake the filmstrip already made once (`BUG-41`).
const PREVIEW_MAX_EDGE: u32 = 900;

/// The handoff document as a flat sequence of blocks.
///
/// Flat rather than nested because both views walk the same sequence: Preview renders each block,
/// Code prints its markup and then the same editable field. That is what lets a note be edited in
/// the Code view - the two are one document with two skins, not a render beside a source dump.
fn bundle_doc_blocks(pending: &PendingBundle) -> Vec<DocBlock> {
    let mut blocks = Vec::new();

    blocks.push(DocBlock {
        kind: "title".into(),
        text: pending.name.clone().into(),
        ..Default::default()
    });
    blocks.push(DocBlock {
        kind: "bundle-notes".into(),
        text: pending.notes.clone().into(),
        ..Default::default()
    });

    for (index, (item, detail)) in pending
        .planned
        .items
        .iter()
        .zip(pending.details.iter())
        .enumerate()
    {
        let position = item.position as i32;

        blocks.push(DocBlock {
            kind: "finding".into(),
            finding_id: detail.finding.id.clone().into(),
            ordinal: position,
            ..Default::default()
        });

        // The BURNED copy, decoded from the bytes about to be written - not the Finding's clean
        // image. This screen is the last look at what the agent will fetch, and the Markers exist
        // nowhere but the burned copy.
        let image = pending
            .planned
            .blobs
            .get(index)
            .and_then(|(_, bytes)| image::load_from_memory(bytes).ok())
            .map(|decoded| {
                rgba_to_slint_image(
                    &decoded
                        .thumbnail(PREVIEW_MAX_EDGE, PREVIEW_MAX_EDGE)
                        .to_rgba8(),
                )
            })
            .unwrap_or_default();

        blocks.push(DocBlock {
            kind: "image".into(),
            finding_id: detail.finding.id.clone().into(),
            ordinal: position,
            path: format!("./{}", item.image_path.trim_start_matches('/')).into(),
            image,
            ..Default::default()
        });

        // An empty note gets no block, because it gets no section in the document either. There is
        // nothing to review and nothing to print.
        if !detail.note.body.trim().is_empty() {
            blocks.push(DocBlock {
                kind: "note".into(),
                finding_id: detail.finding.id.clone().into(),
                text: detail.note.body.trim().to_string().into(),
                ..Default::default()
            });
        }

        for (marker_index, marker) in detail.markers.iter().enumerate() {
            blocks.push(DocBlock {
                kind: "marker".into(),
                finding_id: detail.finding.id.clone().into(),
                marker_id: marker.id.clone().into(),
                ordinal: marker.ordinal as i32,
                text: marker.comment.trim().to_string().into(),
                // The heading prints once, above the first Marker.
                starts_section: marker_index == 0,
                ..Default::default()
            });
        }
    }

    blocks
}

/// Pushes a pending Bundle into the preview's properties.
fn show_bundle_preview(window: &AppWindow, pending: &PendingBundle) {
    let blocks = bundle_doc_blocks(pending);
    window.set_bundle_preview_blocks(ModelRc::from(Rc::new(VecModel::from(blocks))));
    window.set_bundle_preview_finding_count(pending.planned.items.len() as i32);
    window.set_bundle_preview_markdown(pending.planned.markdown.clone().into());
    window.set_bundle_preview_text_tokens((pending.planned.markdown.len() / 4) as i32);
    // Edit, every time it opens. Preview is for checking what will be handed over, and a checking
    // view that is sticky becomes the default by accident.
    window.set_bundle_preview_shows_source(false);
    window.set_bundle_preview_open(true);
}

fn close_bundle_preview(window: &AppWindow) {
    window.set_bundle_preview_open(false);
    // The blocks hold every decoded preview image. Dropping them on close is what keeps the modal's
    // cost bounded to the time it is open.
    window.set_bundle_preview_blocks(ModelRc::from(Rc::new(VecModel::from(
        Vec::<DocBlock>::new(),
    ))));
}

/// Writes a Bundle the Reviewer has now seen: the burned copies, the Markdown, and the row.
///
/// The sentence it returns is the one the Reviewer reads, on either branch.
fn write_bundle(ctx: &AppContext, pending: &PendingBundle) -> Result<String, String> {
    let Some(bundle_store) = ctx.bundle_store.as_ref() else {
        return Err("The Bundle library could not be opened, so nothing was written.".into());
    };
    if pending.planned.items.is_empty() {
        return Err("Every Finding was removed, so there was nothing to write.".into());
    }

    let planned = &pending.planned;

    // Only from here does anything reach disk. A failure now can leave a burned image without its
    // Markdown; those are orphan blobs, which the Vault sweeper already owns.
    for (path, bytes) in &planned.blobs {
        ctx.vault_store
            .write_blob(path, bytes)
            .map_err(|e| format!("Could not write {path}: {e}"))?;
    }
    ctx.vault_store
        .write_blob(&planned.markdown_path, planned.markdown.as_bytes())
        .map_err(|e| format!("Could not write {}: {e}", planned.markdown_path))?;

    let count = planned.items.len();
    let bundle = Bundle::new(
        pending.bundle_id.clone(),
        pending.name.clone(),
        planned.markdown.clone(),
        planned.markdown_path.clone(),
        SystemClock::new().now_rfc3339(),
    )
    .map_err(|e| format!("Could not record the Bundle: {e}"))?;

    bundle_store
        .create_bundle(&bundle, &planned.items)
        .map_err(|e| format!("The Bundle files were written but the row was not: {e}"))?;

    Ok(format!(
        "{} - {count} Finding{} written to {}",
        pending.name,
        if count == 1 { "" } else { "s" },
        planned.markdown_path
    ))
}

thread_local! {
    /// The active Finding's capture, blurred whole, keyed by its id. See [`blurred_capture`].
    static BLURRED_CAPTURE: RefCell<Option<(String, slint::Image)>> =
        const { RefCell::new(None) };

    /// Whether the Editor should come to the front once THIS capture lands.
    ///
    /// A capture no longer raises the Editor on its way in. It used to: both the tray's Capture item
    /// and the global hotkey called `show()` and then started the capture, so the window appeared,
    /// the overlay covered it a moment later, and the Reviewer saw a flash of an Editor they had not
    /// asked for. The owner reported it from the tray: *"maunya jangan membuka snapdown editor lebih
    /// dulu, langsung capture mode, dan langsung buka snapdown editor"*.
    ///
    /// Raising it AFTERWARDS is also the only version that respects `BG-6`. The hotkey's whole
    /// promise is that the loop survives six captures in ninety seconds, and a window taking focus
    /// each time turns six captures into six dismissals - which is why `OpenEditorAfterCapture`
    /// exists as a setting. That setting had never been read by anything; this is what reads it.
    ///
    /// The tray's Capture item overrides it to true regardless, because reaching for a tray menu is
    /// already leaving the flow: the Reviewer went looking for the application.
    static REVEAL_EDITOR_AFTER_CAPTURE: Cell<bool> = const { Cell::new(false) };

    /// The Finding a Shift-click ranges FROM.
    ///
    /// It lives here rather than in a closure because two different things move the selection - a
    /// click on a card, and a capture - and both have to move the anchor with it. When only the
    /// click did, a capture left the anchor on the previous Finding: the new card was ticked and on
    /// the canvas, and a Shift-click ranged from wherever the Reviewer had been before, which is the
    /// same class of divergence as `BUG-67` one step further in.
    ///
    /// Held by id, not by index: a capture rebuilds the strip and every index moves.
    static SELECTION_ANCHOR: RefCell<String> = const { RefCell::new(String::new()) };

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

    // A hidden or minimized window is already invisible to every capture API, so touching its
    // display affinity protects nothing here - and doing it anyway is exactly what produced the
    // "white window flashes and disappears" the owner reported on the hotkey path, never on the
    // tray path. `SetWindowDisplayAffinity` forces DWM to paint a frame for a window it has not
    // composited yet, which is the flash - and the hotkey path is the one flow this feature
    // exists to let run with the window never shown at all (the tray path always reveals the
    // Editor afterwards, which happened to cover the same flash there). Nothing here needs
    // excluding or restoring while the window is not visible, so skip the call entirely.
    if !main.window().is_visible() {
        return;
    }

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

/// Gives the frameless window the shadow Windows draws for every other window. `BUG-71`, `FR-26`.
///
/// `FR-26` makes the window frameless, and the only thing separating it from the desktop was its own
/// 1px border - so against a window of a similar colour there was no separation at all. Every other
/// floating surface in this product already has a lift: the overlay's panels, the Assemble preview,
/// the Marker badges.
///
/// Two things were tried and abandoned before this one, and both are worth recording so a future
/// reader does not repeat them:
///
/// - `DwmExtendFrameIntoClientArea` + `DWMWA_NCRENDERING_POLICY` (`BUG-71`'s original fix note,
///   correct for a window WITH a frame): a no-op here, because `no-frame: true` (`FR-26`) means
///   this window has no `WS_CAPTION`, and DWM will not compute a shadow region without one.
/// - `CS_DROPSHADOW`, the class-style flag Windows uses for shadowed popups: this DOES apply to
///   caption-less windows, but it is a legacy, thin, one-sided shadow meant for small short-lived
///   windows like menus and tooltips - not the soft four-sided shadow a normal app window gets
///   from DWM. It produced no visible change here even after forcing Windows to re-evaluate the
///   frame (`SWP_FRAMECHANGED`), which is consistent with what it actually is: it was never the
///   right mechanism, not merely misapplied.
///
/// `winit` 0.30 carries a purpose-built answer to this exact situation -
/// `WindowExtWindows::set_undecorated_shadow` - because this is a common enough complaint that it
/// shipped upstream (`rust-windowing/winit` #2419) rather than staying a per-app hack: internally
/// it keeps the window "decorated" for DWM's shadow computation while expanding the client rect
/// to cover the whole window, the same hidden-titlebar technique a hand-rolled
/// `WM_NCCALCSIZE` hook would implement, without this file needing to subclass a window
/// procedure to get it. Its own documented cost is a 1px line at the top of the window - worth
/// the Reviewer's eyes alongside the shadow itself.
///
/// Failure is logged and ignored: a window with no shadow is a cosmetic loss, and there is no
/// version of this worth refusing to start over.
#[cfg(windows)]
fn set_window_shadow(window: &AppWindow) {
    use i_slint_backend_winit::winit::platform::windows::WindowExtWindows;
    use i_slint_backend_winit::WinitWindowAccessor;

    let applied = window.window().with_winit_window(|winit_win| {
        winit_win.set_undecorated_shadow(true);
        true
    });

    if applied != Some(true) {
        eprintln!("Could not give the window a drop shadow; it will have only its 1px border.");
    }
}

#[cfg(not(windows))]
fn set_window_shadow(_window: &AppWindow) {}

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

/// A selected region, cropped out of the canvas and put through the Quality Budget: everything the
/// two things you can do with a capture have in common.
///
/// Extracted from `persist_finding` when the copy-to-clipboard path arrived, rather than copied into
/// it. The clamp arithmetic below is the reason: it is the only thing standing between a drag
/// released off the edge of the desktop and an out-of-bounds crop, and two copies of it would drift.
struct PreparedRegion {
    /// PNG bytes, already reduced.
    bytes: Vec<u8>,
    /// The dimensions of `bytes`, which are the reduced ones - NOT the crop's.
    width: u32,
    height: u32,
    /// The crop actually taken, clamped into the canvas. `x, y, w, h`.
    crop: (u32, u32, u32, u32),
    resolved: ResolvedPair,
    budget_name: String,
}

/// Crops `region` out of `source` and applies the Quality Budget.
///
/// `source` is raw RGBA8, `source_size` its pixel dimensions, and `region` is in that same space -
/// the space the overlay reports its selection in, so no scale conversion is involved.
fn prepare_region(
    ctx: &AppContext,
    source: &[u8],
    source_size: (u32, u32),
    region: (u32, u32, u32, u32),
) -> Option<PreparedRegion> {
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
                eprintln!("Quality-budget reduction failed, using the region unreduced: {e}");
                (png_bytes, crop_w, crop_h)
            }
        };

    Some(PreparedRegion {
        bytes: reduced_bytes,
        width: final_w,
        height: final_h,
        crop: (crop_x, crop_y, crop_w, crop_h),
        resolved,
        budget_name: qb.named.display_name().to_string(),
    })
}

/// What a copy chord in the capture overlay should copy.
///
/// This is the whole text-vs-image rule, and it lives here rather than in `appwindow.slint` on
/// purpose. The overlay's note field takes focus the moment the note panel appears, so both chords
/// arrive at a focused `TextInput` - which means Slint has to be involved in DELIVERING them, and the
/// temptation is to let it decide as well. It must not. The overlay's keys ARE guarded here, by
/// `test_capture_interaction.rs`, but those guards read the `.slint` SOURCE - they can prove a branch
/// exists and never that it decides correctly, so an inverted condition would stay green. Slint asks
/// this function and obeys the answer, and the answer is what the tests below can watch fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CopyChordTarget {
    /// Put the selected region on the clipboard and save nothing.
    Image,
    /// Leave the image alone; the note field copies its own selected text.
    NoteText,
}

/// A BMP onto the Windows clipboard, emptying the clipboard FIRST.
///
/// `clipboard_win::set_clipboard(formats::Bitmap, ..)` does not empty it, and that is the whole of
/// `BUG-84`: the owner reported that the first copy worked and every copy after it pasted the FIRST
/// image again. The library says so in its own source - `raw::set_bitmap` passes
/// `options::NoClear`, commented *"Bitmap format cannot really overlap with much so there is no risk
/// of having non-empty clipboard. Also it is backward compatible behavior. To be changed in 6.x"*.
/// On Windows a `SetClipboardData` with no preceding `EmptyClipboard` leaves the existing handle for
/// that format in place, so the second write is the one that does nothing.
///
/// `copy_burned_image` had the identical defect, and carried a comment asserting the opposite -
/// *"`set_clipboard` opens, empties, writes and closes"* - which is presumably why the Editor's Copy
/// button was never suspected. Both paths now come through here.
///
/// The guard is taken HERE rather than letting `set_clipboard` open internally, because the empty and
/// the write have to happen inside ONE open. That is also why the old comment's warning about taking
/// the lock twice does not apply: nothing below opens the clipboard again.
#[cfg(windows)]
fn put_bitmap_on_clipboard(bmp: &[u8]) -> Result<(), String> {
    let _clip = clipboard_win::Clipboard::new_attempts(10)
        .map_err(|e| format!("Could not open the clipboard: {e}"))?;
    clipboard_win::raw::set_bitmap_with(bmp, clipboard_win::options::DoClear)
        .map_err(|e| format!("Could not write to the clipboard: {e}"))
}

/// Text onto the Windows clipboard, emptying it FIRST - ticket 12's Copy Markdown, and the same
/// `Clipboard::new_attempts(10)` + explicit-clear discipline `put_bitmap_on_clipboard` established
/// above. Unlike `raw::set_bitmap`, `raw::set_string_with` is passed `DoClear` here rather than
/// relying on a default, for the same reason: the crate's own default for the STRING path already
/// clears (`raw::set_string` is `DoClear` by default, unlike the bitmap path's `NoClear`), but the
/// bitmap comment above is exactly why that default is not trusted silently a second time - it is
/// named explicitly instead of assumed.
#[cfg(windows)]
fn put_text_on_clipboard(text: &str) -> Result<(), String> {
    let _clip = clipboard_win::Clipboard::new_attempts(10)
        .map_err(|e| format!("Could not open the clipboard: {e}"))?;
    clipboard_win::raw::set_string_with(text, clipboard_win::options::DoClear)
        .map_err(|e| format!("Could not write to the clipboard: {e}"))
}

#[cfg(not(windows))]
fn put_text_on_clipboard(_text: &str) -> Result<(), String> {
    Err("Writing to the clipboard is implemented on Windows only.".to_string())
}

/// `has_text_selection`: the note field currently holds a text selection.
/// `force_image`: the chord is Ctrl+Enter, which never means text.
fn copy_chord_target(has_text_selection: bool, force_image: bool) -> CopyChordTarget {
    // Ctrl+Enter is unconditional, and that is what makes Ctrl+C safe to make conditional: there is
    // always one chord that means "the image" no matter what the caret is doing.
    if force_image || !has_text_selection {
        CopyChordTarget::Image
    } else {
        CopyChordTarget::NoteText
    }
}

/// The selected region on the clipboard, with nothing written anywhere. `Ctrl+C` / `Ctrl+Enter`.
///
/// The sibling of `copy_burned_image`, and deliberately NOT the same function. That one hands over a
/// stored Finding with its Markers burned in; this one hands over a region that has no Finding, so
/// there is nothing to burn and no redaction available. A shot that needs blurring has to be saved.
///
/// A screenshot that never touches disk cannot be recovered, backed up, or carried through a Vault
/// migration - which is the point, not a side effect.
///
/// The Quality Budget applies, exactly as it does on the way into the Vault. Two reasons, and the
/// second one is not obvious: the same capture copied and saved should not differ, and a DIB is
/// UNCOMPRESSED, so the clipboard cost is `w * h * 3` of the final dimensions - about 25 MB for a 4K
/// screen and 50 MB across two of them. The resolved long edge is what bounds that.
/// Everything the copy path does EXCEPT touch the clipboard: the BMP bytes and their dimensions.
///
/// Split out so it can be tested. The alternative - a test that calls the real thing - would replace
/// whatever the developer had on their clipboard every time `cargo test` ran, and the interesting
/// assertions here are not about the Windows clipboard API anyway: they are that the bytes decode,
/// and that nothing was written to the Vault or the Library on the way.
///
/// `cfg(any(windows, test))` rather than `cfg(windows)`: the clipboard write is Windows-only, but
/// this half is portable and the test needs it on every platform CI runs on.
#[cfg(any(windows, test))]
fn encode_region_for_clipboard(
    ctx: &AppContext,
    source: &[u8],
    source_size: (u32, u32),
    region: (u32, u32, u32, u32),
) -> Result<(Vec<u8>, u32, u32), String> {
    use image::codecs::bmp::BmpEncoder;
    use image::ExtendedColorType;

    let prepared = prepare_region(ctx, source, source_size, region)
        .ok_or_else(|| "Could not read the selected region.".to_string())?;

    let decoded = image::load_from_memory(&prepared.bytes)
        .map_err(|e| format!("Could not decode the captured region: {e}"))?;
    // RGB, not RGBA, for the reason `copy_burned_image` records: a DIB with an alpha channel is
    // pasted as fully transparent by several applications, Explorer's preview included.
    let rgb = decoded.to_rgb8();

    let mut bmp = Vec::new();
    BmpEncoder::new(&mut bmp)
        .encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ExtendedColorType::Rgb8,
        )
        .map_err(|e| format!("Could not encode for the clipboard: {e}"))?;

    Ok((bmp, rgb.width(), rgb.height()))
}

#[cfg(windows)]
fn copy_region_to_clipboard(
    ctx: &AppContext,
    source: &[u8],
    source_size: (u32, u32),
    region: (u32, u32, u32, u32),
) -> Result<String, String> {
    let (bmp, width, height) = encode_region_for_clipboard(ctx, source, source_size, region)?;

    put_bitmap_on_clipboard(&bmp)?;

    Ok(format!(
        "Copied to the clipboard, not saved ({width} x {height})."
    ))
}

#[cfg(not(windows))]
fn copy_region_to_clipboard(
    _ctx: &AppContext,
    _source: &[u8],
    _source_size: (u32, u32),
    _region: (u32, u32, u32, u32),
) -> Result<String, String> {
    Err("Copying an image to the clipboard is implemented on Windows only.".to_string())
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
    let prepared = prepare_region(ctx, source, source_size, region)?;
    let PreparedRegion {
        bytes: reduced_bytes,
        width: final_w,
        height: final_h,
        crop: (crop_x, crop_y, crop_w, crop_h),
        resolved,
        budget_name,
    } = prepared;

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
        budget_name: Some(budget_name),
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
    //
    // RETRIED, not one-shot. A Vault move relaunches Snapdown by spawning the next instance
    // BEFORE exiting this one (`on_vault_migration_confirmed`), so the child's first attempt
    // almost always lands while this process still holds the mutex - a one-shot check read that
    // race as "already running" and quit with no window and no visible error, which is the exact
    // silent-exit shape a Windows release build produces (`AGENTS.md`'s panic pitfall is about a
    // crash, but the symptom on the Reviewer's screen - nothing happens at all - is identical
    // here). The old process's exit releases the mutex within milliseconds, so a sub-second retry
    // window tells the two cases apart: a genuine second launch is still rejected almost
    // immediately; a relaunch racing its own predecessor gets through once that predecessor is
    // actually gone.
    let _single_instance_lock = 'acquire: {
        for _ in 0..20 {
            if let Some(lock) = acquire_single_instance_lock() {
                break 'acquire lock;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        eprintln!("Snapdown is already running.");
        return Ok(());
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

    // The stored theme, before the window is shown, so it does not repaint in front of the
    // Reviewer.
    if let Ok(Some(Setting {
        value: SettingValue::Boolean(dark),
        ..
    })) = ctx.settings_store.get(&theme_setting_key())
    {
        main_window.set_is_dark_theme(dark);
    }

    // Populate initial Filmstrip from Vault
    load_findings_into_window(&main_window, &ctx, None);

    // Filmstrip clicks, with file-manager semantics. See `click_finding`.
    //
    // The Shift anchor lives here rather than in the UI: Slint has the click, but the anchor is a
    // piece of interaction STATE that has to survive a click without being changed by it, and the
    // repeater has nowhere to keep one.
    let win_weak_sel = main_window.as_weak();
    let ctx_sel = ctx.clone();
    main_window.on_finding_clicked(move |id, ctrl, shift| {
        if let Some(win) = win_weak_sel.upgrade() {
            click_finding(&win, &ctx_sel, &id, ctrl, shift);
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
    main_window.on_finding_note_edited(move |body| {
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

    // MARKERS.
    //
    // The reticle overlay, the numbered badge, the Marker Notes list and the seven tool buttons were
    // all built and none of them was connected: `marker-placed` and `delete-marker-clicked` were
    // declared in the UI and handled by nothing, so a click on the canvas placed nothing and the
    // list was permanently empty. Same shape as `BUG-4`, `BUG-5`, `BUG-6` and `BUG-19` - every part
    // present, the join absent - which `AGENTS.md` calls this repository's signature failure.
    let win_weak_mkp = main_window.as_weak();
    let ctx_mkp = ctx.clone();
    main_window.on_marker_placed(move |x, y| {
        let Some(win) = win_weak_mkp.upgrade() else {
            return;
        };
        let finding_id = win.get_active_finding_id().to_string();
        if finding_id.is_empty() {
            toast(
                &win,
                "Open a Finding first - a Marker belongs to an image.",
                true,
            );
            return;
        }
        let clock = SystemClock::new();
        let entropy = SystemEntropySource::new();
        let marker_id = id_from_parts(clock.now_unix_millis(), entropy.random_bytes_10());
        match ctx_mkp.finding_store.add_marker(
            &finding_id,
            &marker_id,
            f64::from(x),
            f64::from(y),
            "",
        ) {
            // Reloaded from the store rather than pushed into the model: the ordinal is the store's
            // to assign (`AD-1` ties it to the Markdown line number), so the UI must be told what it
            // became instead of guessing.
            Ok(_) => load_active_detail(&win, &ctx_mkp, &finding_id),
            Err(e) => toast(&win, format!("Could not place the Marker: {e}"), true),
        }
    });

    let win_weak_mkd = main_window.as_weak();
    let ctx_mkd = ctx.clone();
    main_window.on_delete_marker_clicked(move |marker_id| {
        let Some(win) = win_weak_mkd.upgrade() else {
            return;
        };
        let finding_id = win.get_active_finding_id().to_string();
        if finding_id.is_empty() || marker_id.is_empty() {
            return;
        }
        // The store renumbers the survivors inside the same transaction, so the reload is what keeps
        // the badges and the Markdown line numbers agreeing.
        match ctx_mkd.finding_store.delete_marker(&finding_id, &marker_id) {
            Ok(()) => load_active_detail(&win, &ctx_mkd, &finding_id),
            Err(e) => toast(&win, format!("Could not delete the Marker: {e}"), true),
        }
    });

    // A Marker's own note, written through on every edit for the same reason the Observation Summary
    // is: the alternative is a blur event that may never fire.
    //
    // It deliberately does NOT reload the detail afterwards. A reload rebuilds the Marker Notes list,
    // which would take the field out from under the caret mid-sentence.
    let win_weak_mkc = main_window.as_weak();
    let ctx_mkc = ctx.clone();
    main_window.on_marker_comment_edited(move |marker_id, body| {
        let Some(win) = win_weak_mkc.upgrade() else {
            return;
        };
        let finding_id = win.get_active_finding_id().to_string();
        if finding_id.is_empty() || marker_id.is_empty() {
            return;
        }
        // `update_marker` takes the coordinates too, and they are already on screen. Reading them
        // from the model rather than the database keeps this one write instead of a read and a write.
        let Some(marker) = win.get_markers().iter().find(|m| m.id == marker_id) else {
            return;
        };
        if let Err(e) = ctx_mkc.finding_store.update_marker(
            &finding_id,
            marker_id.as_str(),
            f64::from(marker.x),
            f64::from(marker.y),
            body.as_str(),
        ) {
            eprintln!("Could not save the note for Marker {marker_id}: {e}");
            return;
        }
        // The cost card is on screen while this is being typed, so its Marker total has to follow.
        // Summed from the UI model rather than re-read from the store: the model is what the
        // Reviewer is looking at, and the store has just been told the same thing.
        let total: usize = win
            .get_markers()
            .iter()
            .map(|m| {
                if m.id == marker_id {
                    body.chars().count()
                } else {
                    m.comment.chars().count()
                }
            })
            .sum();
        win.set_marker_char_count(total as i32);
    });

    // ANNOTATIONS - `CAP-11`, `FR-30` to `FR-33`.
    //
    // The capability had a domain type and a burner and nothing in between: no table, no port
    // method, no read, and seven toolbar buttons that set an index and did nothing with it. That is
    // `BUG-72`, and it is `BUG-55` one level up - a whole capability built at both ends and joined
    // in the middle by nothing.
    //
    // Every handler here reloads the Finding afterwards rather than pushing into the Slint model.
    // The store owns z-order, and a model the UI edited directly would drift from it exactly the way
    // the Marker ordinals would if `add_marker`'s result were guessed.
    let win_weak_ad = main_window.as_weak();
    let ctx_ad = ctx.clone();
    main_window.on_annotation_drawn(move |kind, x1, y1, x2, y2| {
        let Some(win) = win_weak_ad.upgrade() else {
            return;
        };
        let finding_id = win.get_active_finding_id().to_string();
        if finding_id.is_empty() {
            toast(
                &win,
                "Open a Finding first - an annotation belongs to an image.",
                true,
            );
            return;
        }
        let Some(shape) = shape_from_drag(
            kind.as_str(),
            f64::from(x1),
            f64::from(y1),
            f64::from(x2),
            f64::from(y2),
            default_annotation_font_size(&ctx_ad),
        ) else {
            return;
        };
        let clock = SystemClock::new();
        let entropy = SystemEntropySource::new();
        let annotation_id = id_from_parts(clock.now_unix_millis(), entropy.random_bytes_10());
        match ctx_ad.finding_store.add_annotation(
            &finding_id,
            &annotation_id,
            &shape,
            &clock.now_rfc3339(),
        ) {
            Ok(_) => {
                record_edit(
                    &finding_id,
                    AnnEdit::Added {
                        id: annotation_id.clone(),
                    },
                );
                // Selected on arrival, so the handles are already under the hand that drew it -
                // and so a Callout or a Text can be typed into without a second click to find it.
                // Set before the reload, which is what rewrites the Properties panel from it.
                win.set_selected_annotation_id(annotation_id.into());
                load_active_detail(&win, &ctx_ad, &finding_id);
            }
            Err(e) => toast(&win, format!("Could not draw that: {e}"), true),
        }
    });

    let win_weak_ag = main_window.as_weak();
    let ctx_ag = ctx.clone();
    main_window.on_annotation_geometry_changed(move |id, x1, y1, x2, y2| {
        let Some(win) = win_weak_ag.upgrade() else {
            return;
        };
        let finding_id = win.get_active_finding_id().to_string();
        if finding_id.is_empty() || id.is_empty() {
            return;
        }
        let Some(existing) = load_annotation(&ctx_ag, &finding_id, id.as_str()) else {
            return;
        };
        let moved = with_geometry(
            &existing.data,
            f64::from(x1),
            f64::from(y1),
            f64::from(x2),
            f64::from(y2),
        );
        match ctx_ag
            .finding_store
            .update_annotation(&finding_id, id.as_str(), &moved)
        {
            Ok(_) => {
                record_edit(
                    &finding_id,
                    AnnEdit::Changed {
                        id: id.to_string(),
                        before: existing.data,
                        after: moved,
                    },
                );
                load_active_detail(&win, &ctx_ag, &finding_id);
            }
            // A drag that ended outside the image is refused by the store, and saying so is better
            // than a shape that springs back with no explanation.
            Err(e) => toast(&win, format!("Could not move that: {e}"), true),
        }
    });

    let win_weak_at = main_window.as_weak();
    let ctx_at = ctx.clone();
    main_window.on_annotation_tail_moved(move |id, tx, ty| {
        let Some(win) = win_weak_at.upgrade() else {
            return;
        };
        let finding_id = win.get_active_finding_id().to_string();
        if finding_id.is_empty() || id.is_empty() {
            return;
        }
        let Some(existing) = load_annotation(&ctx_at, &finding_id, id.as_str()) else {
            return;
        };
        let pointed = with_tail(&existing.data, f64::from(tx), f64::from(ty));
        match ctx_at
            .finding_store
            .update_annotation(&finding_id, id.as_str(), &pointed)
        {
            Ok(_) => {
                record_edit(
                    &finding_id,
                    AnnEdit::Changed {
                        id: id.to_string(),
                        before: existing.data,
                        after: pointed,
                    },
                );
                load_active_detail(&win, &ctx_at, &finding_id);
            }
            Err(e) => toast(&win, format!("Could not move the tail: {e}"), true),
        }
    });

    // The words, written through on every keystroke for the same reason the Observation Summary is:
    // the alternative is a blur event that may never fire.
    //
    // It deliberately does NOT reload afterwards. A reload rebuilds the model, and rebuilding the
    // model under a text field takes the caret out of the sentence being typed - the same trap
    // `on_marker_comment_edited` documents.
    let win_weak_ax = main_window.as_weak();
    let ctx_ax = ctx.clone();
    main_window.on_annotation_text_edited(move |id, text| {
        let Some(win) = win_weak_ax.upgrade() else {
            return;
        };
        let finding_id = win.get_active_finding_id().to_string();
        if finding_id.is_empty() || id.is_empty() {
            return;
        }
        let Some(existing) = load_annotation(&ctx_ax, &finding_id, id.as_str()) else {
            return;
        };
        let typed = with_content(&existing.data, Some(text.as_str()), None, None, None);
        if let Err(e) = ctx_ax
            .finding_store
            .update_annotation(&finding_id, id.as_str(), &typed)
        {
            eprintln!("Could not save the text for annotation {id}: {e}");
            return;
        }
        // Not recorded as one undo step per keystroke - that would make Ctrl+Z delete one letter at
        // a time and bury the move that came before it. The canvas still has to follow the typing,
        // so the model row is updated in place instead of reloading.
        let rows = win.get_annotations();
        for index in 0..rows.row_count() {
            if let Some(mut row) = rows.row_data(index) {
                if row.id == id {
                    row.text = text.clone();
                    rows.set_row_data(index, row);
                    break;
                }
            }
        }
    });

    let win_weak_as = main_window.as_weak();
    let ctx_as = ctx.clone();
    main_window.on_annotation_style_changed(move |id, size, family, align| {
        let Some(win) = win_weak_as.upgrade() else {
            return;
        };
        let finding_id = win.get_active_finding_id().to_string();
        if finding_id.is_empty() || id.is_empty() {
            return;
        }
        let Some(existing) = load_annotation(&ctx_as, &finding_id, id.as_str()) else {
            return;
        };
        let styled = with_content(
            &existing.data,
            None,
            Some(f64::from(size)),
            Some(family.as_str()),
            Some(align.as_str()),
        );
        match ctx_as
            .finding_store
            .update_annotation(&finding_id, id.as_str(), &styled)
        {
            Ok(_) => {
                // The size the Reviewer just chose becomes the default for the next annotation.
                // Forward only - nothing already on the canvas is touched.
                remember_annotation_font_size(&ctx_as, f64::from(size));
                record_edit(
                    &finding_id,
                    AnnEdit::Changed {
                        id: id.to_string(),
                        before: existing.data,
                        after: styled,
                    },
                );
                load_active_detail(&win, &ctx_as, &finding_id);
            }
            Err(e) => toast(&win, format!("Could not restyle that: {e}"), true),
        }
    });

    let win_weak_adel = main_window.as_weak();
    let ctx_adel = ctx.clone();
    main_window.on_annotation_deleted(move |id| {
        let Some(win) = win_weak_adel.upgrade() else {
            return;
        };
        let finding_id = win.get_active_finding_id().to_string();
        if finding_id.is_empty() || id.is_empty() {
            return;
        }
        let Some(existing) = load_annotation(&ctx_adel, &finding_id, id.as_str()) else {
            return;
        };
        match ctx_adel
            .finding_store
            .delete_annotation(&finding_id, id.as_str())
        {
            Ok(()) => {
                record_edit(
                    &finding_id,
                    AnnEdit::Removed {
                        annotation: existing,
                    },
                );
                win.set_selected_annotation_id(SharedString::new());
                load_active_detail(&win, &ctx_adel, &finding_id);
            }
            Err(e) => toast(&win, format!("Could not delete that: {e}"), true),
        }
    });

    // Selection is routed through Rust rather than assigned in Slint, so the id and the five
    // mirrored fields the Properties panel reads are written by one hand. "" deselects.
    let win_weak_asel = main_window.as_weak();
    let ctx_asel = ctx.clone();
    main_window.on_annotation_selected(move |id| {
        let Some(win) = win_weak_asel.upgrade() else {
            return;
        };
        if id.is_empty() {
            apply_selection_mirror(&win, &[], "");
            return;
        }
        let finding_id = win.get_active_finding_id().to_string();
        let annotations = ctx_asel
            .finding_store
            .get_finding(&finding_id)
            .ok()
            .flatten()
            .map(|detail| detail.visual_annotations)
            .unwrap_or_default();
        apply_selection_mirror(&win, &annotations, id.as_str());
    });

    // A Marker can be moved after it is placed.
    //
    // It could not be, and that made a mis-click permanent: the only remedy was delete and re-place,
    // and `delete_marker` renumbers every Marker after it - so correcting Marker 2's POSITION also
    // rewrote Marker 2's line number in the Note. This writes the position and nothing else.
    let win_weak_mmv = main_window.as_weak();
    let ctx_mmv = ctx.clone();
    main_window.on_marker_moved(move |marker_id, x, y| {
        let Some(win) = win_weak_mmv.upgrade() else {
            return;
        };
        let finding_id = win.get_active_finding_id().to_string();
        if finding_id.is_empty() || marker_id.is_empty() {
            return;
        }
        // The comment comes from the model rather than the store: it is what the Reviewer is looking
        // at, and `update_marker` takes the whole row, so reading it back would risk overwriting an
        // edit still in the field with a staler copy.
        let Some(marker) = win.get_markers().iter().find(|m| m.id == marker_id) else {
            return;
        };
        match ctx_mmv.finding_store.update_marker(
            &finding_id,
            marker_id.as_str(),
            f64::from(x),
            f64::from(y),
            marker.comment.as_str(),
        ) {
            Ok(_) => load_active_detail(&win, &ctx_mmv, &finding_id),
            Err(e) => toast(&win, format!("Could not move the Marker: {e}"), true),
        }
    });

    // Z-ORDER. Four movements, one port call, and the arithmetic in `reordered` where it can be
    // read and tested rather than inside a callback.
    //
    // NOT recorded on the undo stack. `AnnEdit` describes a shape, and an order is a property of the
    // collection rather than of any member of it - putting it there would mean either a fifth variant
    // that holds a whole ordering or an undo that silently reorders something else. It is also the
    // cheapest edit to reverse by hand: the opposite movement is right there in the same menu.
    let win_weak_aord = main_window.as_weak();
    let ctx_aord = ctx.clone();
    main_window.on_annotation_reordered(move |id, movement| {
        let Some(win) = win_weak_aord.upgrade() else {
            return;
        };
        let finding_id = win.get_active_finding_id().to_string();
        if finding_id.is_empty() || id.is_empty() {
            return;
        }
        let Ok(Some(detail)) = ctx_aord.finding_store.get_finding(&finding_id) else {
            return;
        };
        let ids: Vec<String> = detail
            .visual_annotations
            .iter()
            .map(|a| a.id.clone())
            .collect();

        let Some(order) = reordered(&ids, id.as_str(), movement.as_str()) else {
            // Already where it was asked to go. Saying so beats a silent no-op on a menu item the
            // Reviewer just clicked.
            toast(&win, "Already there.", false);
            return;
        };
        let borrowed: Vec<&str> = order.iter().map(String::as_str).collect();
        match ctx_aord
            .finding_store
            .reorder_annotations(&finding_id, &borrowed)
        {
            Ok(()) => load_active_detail(&win, &ctx_aord, &finding_id),
            Err(e) => toast(&win, format!("Could not reorder that: {e}"), true),
        }
    });

    // COPY IMAGE, OPEN FILE LOCATION, DELETE FINDING - the three entries the archived Tauri menu
    // had and this build did not. `copy-image-clicked` has been declared and unhandled since the
    // Editor was written; it was on `BUG-61`'s list and in `DELIBERATELY_UNHANDLED`.
    let win_weak_cpy = main_window.as_weak();
    let ctx_cpy = ctx.clone();
    main_window.on_copy_image_clicked(move || {
        let Some(win) = win_weak_cpy.upgrade() else {
            return;
        };
        let finding_id = win.get_active_finding_id().to_string();
        if finding_id.is_empty() {
            toast(&win, "Open a Finding first.", true);
            return;
        }
        match copy_burned_image(&ctx_cpy, &finding_id) {
            Ok(message) => toast(&win, message, false),
            Err(message) => toast(&win, message, true),
        }
    });

    let win_weak_rev = main_window.as_weak();
    let ctx_rev = ctx.clone();
    main_window.on_open_file_location_clicked(move |finding_id| {
        let Some(win) = win_weak_rev.upgrade() else {
            return;
        };
        if finding_id.is_empty() {
            return;
        }
        let path = ctx_rev
            .finding_store
            .get_finding(finding_id.as_str())
            .ok()
            .flatten()
            .map(|detail| detail.finding.image_path);
        let Some(path) = path else {
            toast(&win, "That Finding is no longer in the Library.", true);
            return;
        };
        if let Err(message) = open_file_location(&ctx_rev, &path) {
            toast(&win, message, true);
        }
    });

    // Already confirmed by the time this fires - the dialog is the safety, and it is in the UI so
    // that no code path can reach the delete without passing it.
    //
    // WHAT gets deleted follows the file-manager rule the filmstrip already uses for clicks: if the
    // Finding that was right-clicked is part of the selection, the whole selection goes; if it is
    // not, only that one does. The owner asked for the first half - *"Delete finding harusnya
    // menghapus semua selected finding"* - and the second half is what stops a right-click on an
    // unselected card quietly taking eight others with it.
    let win_weak_dfn = main_window.as_weak();
    let ctx_dfn = ctx.clone();
    main_window.on_delete_finding_confirmed(move |finding_id| {
        let Some(win) = win_weak_dfn.upgrade() else {
            return;
        };
        if finding_id.is_empty() {
            return;
        }

        let targets = findings_to_delete(&win, finding_id.as_str());
        let mut deleted = 0usize;
        let mut orphaned = 0usize;
        let mut first_error: Option<String> = None;

        for target in &targets {
            match delete_finding_everywhere(&ctx_dfn, target) {
                Ok(orphan) => {
                    deleted += 1;
                    if orphan {
                        orphaned += 1;
                    }
                }
                // Kept going. Stopping at the first failure would leave the Reviewer with some of a
                // deletion they confirmed and no way to tell which half.
                Err(message) => {
                    if first_error.is_none() {
                        first_error = Some(message);
                    }
                }
            }
        }

        // Reloaded with no active Finding: the one that was open may be among those that just went,
        // and `load_findings_into_window` picks the first survivor.
        load_findings_into_window(&win, &ctx_dfn, None);

        let noun = if deleted == 1 { "Finding" } else { "Findings" };
        let message = match (first_error, orphaned) {
            (Some(error), _) => format!(
                "{deleted} of {} deleted. The rest failed: {error}",
                targets.len()
            ),
            (None, 0) => format!("{deleted} {noun} deleted, with their images."),
            (None, n) => format!(
                "{deleted} {noun} deleted. {n} image file(s) could not be removed and are now orphans."
            ),
        };
        toast(&win, message, deleted != targets.len());
    });

    let win_weak_au = main_window.as_weak();
    let ctx_au = ctx.clone();
    main_window.on_annotation_undo(move || {
        if let Some(win) = win_weak_au.upgrade() {
            step_annotation_history(&win, &ctx_au, true);
        }
    });

    let win_weak_ar = main_window.as_weak();
    let ctx_ar = ctx.clone();
    main_window.on_annotation_redo(move || {
        if let Some(win) = win_weak_ar.upgrade() {
            step_annotation_history(&win, &ctx_ar, false);
        }
    });

    // BUNDLE ASSEMBLY.
    // The Bundle being reviewed. One at a time, and it lives only as long as the preview is open.
    let pending_bundle: Rc<RefCell<Option<PendingBundle>>> = Rc::new(RefCell::new(None));

    let win_weak_asm = main_window.as_weak();
    let ctx_asm = ctx.clone();
    let pending_asm = pending_bundle.clone();
    main_window.on_assemble_bundle_clicked(move || {
        let Some(win) = win_weak_asm.upgrade() else {
            return;
        };
        match prepare_bundle(&win, &ctx_asm) {
            Ok(pending) => {
                show_bundle_preview(&win, &pending);
                *pending_asm.borrow_mut() = Some(pending);
            }
            Err(message) => {
                eprintln!("Assemble refused: {message}");
                toast(&win, message, true);
            }
        }
    });

    let win_weak_pc = main_window.as_weak();
    let ctx_pc = ctx.clone();
    let pending_pc = pending_bundle.clone();
    main_window.on_bundle_preview_confirmed(move || {
        let Some(win) = win_weak_pc.upgrade() else {
            return;
        };
        let Some(pending) = pending_pc.borrow_mut().take() else {
            return;
        };
        close_bundle_preview(&win);
        match write_bundle(&ctx_pc, &pending) {
            Ok(message) => {
                // The Findings that went in leave the strip - `load_findings_into_window` filters
                // out anything a Bundle already holds. The strip is the queue of what has not been
                // handed over yet, which is what makes "N selected" mean something.
                load_findings_into_window(&win, &ctx_pc, None);
                toast(&win, message, false);
            }
            Err(message) => {
                eprintln!("Assemble failed: {message}");
                toast(&win, message, true);
            }
        }
    });

    let win_weak_px = main_window.as_weak();
    let pending_px = pending_bundle.clone();
    main_window.on_bundle_preview_cancelled(move || {
        // Nothing was written, so nothing has to be undone. That is the whole point of planning in
        // memory first.
        *pending_px.borrow_mut() = None;
        if let Some(win) = win_weak_px.upgrade() {
            close_bundle_preview(&win);
        }
    });

    // EVERY edit in the preview comes through here, and which store it reaches depends on the
    // block's kind. One callback rather than four: the document is one sequence of blocks, and
    // deciding where a block is persisted is Rust's business rather than the layout's.
    //
    // None of these refresh the block model afterwards. Rebuilding the repeater would take the field
    // out from under the caret - and it would also re-decode every preview image on a keystroke.
    // Only the composed document and the token readout move.
    let win_weak_be = main_window.as_weak();
    let ctx_be = ctx.clone();
    let pending_be = pending_bundle.clone();
    main_window.on_bundle_block_edited(move |kind, finding_id, marker_id, text| {
        let Some(win) = win_weak_be.upgrade() else {
            return;
        };
        let mut slot = pending_be.borrow_mut();
        let Some(pending) = slot.as_mut() else {
            return;
        };

        match kind.as_str() {
            // The Bundle's name and its note have no row of their own. They become part of the
            // composed document, which IS stored - `bundle.markdown` is a column, and
            // `SDD-bundle.md` says the document is composed once and stored, not regenerated.
            "title" => pending.name = text.to_string(),
            "bundle-notes" => pending.notes = text.to_string(),

            // A Finding's note and a Marker's are the Finding's own, so they are written through to
            // it. Editing them here is editing the same note the inspector edits.
            "note" => {
                let now = SystemClock::new().now_rfc3339();
                if let Err(e) =
                    ctx_be
                        .finding_store
                        .update_note(finding_id.as_str(), text.as_str(), &now)
                {
                    eprintln!("Could not save the note for Finding {finding_id}: {e}");
                    return;
                }
                for detail in pending.details.iter_mut() {
                    if detail.finding.id == finding_id.as_str() {
                        detail.note.body = text.to_string();
                        detail.note.updated_at = now.clone();
                    }
                }
            }
            "marker" => {
                // `update_marker` takes the coordinates too, and the plan already holds them - so
                // this stays one write rather than a read and a write.
                let coords = pending
                    .details
                    .iter()
                    .filter(|d| d.finding.id == finding_id.as_str())
                    .flat_map(|d| d.markers.iter())
                    .find(|m| m.id == marker_id.as_str())
                    .map(|m| (m.x, m.y));
                let Some((x, y)) = coords else {
                    return;
                };
                if let Err(e) = ctx_be.finding_store.update_marker(
                    finding_id.as_str(),
                    marker_id.as_str(),
                    x,
                    y,
                    text.as_str(),
                ) {
                    eprintln!("Could not save the note for Marker {marker_id}: {e}");
                    return;
                }
                for detail in pending.details.iter_mut() {
                    for marker in detail.markers.iter_mut() {
                        if marker.id == marker_id.as_str() {
                            marker.comment = text.to_string();
                        }
                    }
                }
            }
            other => {
                // `finding` and `image` are generated from the plan and are not editable in either
                // view, so nothing should be able to send them here.
                eprintln!("A preview block of kind `{other}` reported an edit; ignoring it.");
                return;
            }
        }

        recompose_markdown(pending);
        win.set_bundle_preview_markdown(pending.planned.markdown.clone().into());
        win.set_bundle_preview_text_tokens((pending.planned.markdown.len() / 4) as i32);
    });

    // Native Window Dragging on Titlebar
    #[cfg(windows)]
    {
        use i_slint_backend_winit::WinitWindowAccessor;
        // A PRESS on the titlebar. Windows refuses `SC_MOVE` for a maximized window, so there is
        // nothing to do for one here - and doing something anyway is what made a single click
        // restore the window, as if it had been a double click.
        //
        // The swallowed Result this replaced (`let _ = winit_win.drag_window()`) is the class
        // `AGENTS.md` records: what it discarded was the error message explaining why the titlebar
        // was dead.
        let win_drag = main_window.as_weak();
        main_window.on_drag_window_requested(move || {
            let Some(win) = win_drag.upgrade() else {
                return;
            };
            if win.window().is_maximized() {
                return;
            }
            win.window().with_winit_window(|winit_win| {
                if let Err(e) = winit_win.drag_window() {
                    eprintln!("The titlebar drag was refused by the window manager: {e}");
                }
            });
        });

        // MOVEMENT while the titlebar is held. Only here is a maximized window restored, because
        // only movement says a drag was meant rather than a click - and once restored it has to be
        // dragged in the same gesture, or the window snaps back to a corner and stays there.
        let win_drag_moved = main_window.as_weak();
        main_window.on_drag_window_moved(move || {
            let Some(win) = win_drag_moved.upgrade() else {
                return;
            };
            if !win.window().is_maximized() {
                // Already being dragged by the system; nothing to do.
                return;
            }
            win.window().set_maximized(false);
            win.window().with_winit_window(|winit_win| {
                if let Err(e) = winit_win.drag_window() {
                    eprintln!("The titlebar drag was refused after restoring: {e}");
                }
            });
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

    // Theme Toggle.
    //
    // The UI flips `is-dark-theme` itself before firing this, so the repaint is not this handler's
    // job - persisting the choice is. It used to print the value it was about to lose, so the Editor
    // reopened in light mode every time however many times the Reviewer had switched.
    //
    // `SettingKey::Custom` rather than a new enum variant: a new variant reaches the domain, the
    // store and the migration, which is a lot of surface for one boolean.
    let win_theme = main_window.as_weak();
    let ctx_theme = ctx.clone();
    main_window.on_theme_toggle_clicked(move || {
        let Some(win) = win_theme.upgrade() else {
            return;
        };
        let setting = Setting {
            key: theme_setting_key(),
            value: SettingValue::Boolean(win.get_is_dark_theme()),
            updated_at: SystemClock::new().now_rfc3339(),
        };
        if let Err(e) = ctx_theme.settings_store.set(&setting) {
            eprintln!("Could not remember the theme choice: {e}");
        }
    });

    // THE LIBRARY (ticket 11 of the Bundle Library spec). Opens over the Editor, lists every
    // composed Bundle, and closes back to it - the Editor's own canvas/selection/scroll are never
    // touched by any of this, which is what makes the round trip free.
    let win_library = main_window.as_weak();
    let ctx_library = ctx.clone();
    main_window.on_library_clicked(move || {
        let Some(win) = win_library.upgrade() else {
            return;
        };
        open_library(&win, &ctx_library);
    });

    let win_library_closed = main_window.as_weak();
    main_window.on_library_closed(move || {
        if let Some(win) = win_library_closed.upgrade() {
            win.set_library_open(false);
        }
    });

    let win_library_retry = main_window.as_weak();
    let ctx_library_retry = ctx.clone();
    main_window.on_library_try_again_clicked(move || {
        let Some(win) = win_library_retry.upgrade() else {
            return;
        };
        open_library(&win, &ctx_library_retry);
    });

    // The Library's own "cannot be read" state has no Bundle to point at - what refused was
    // `library.db` itself - so this reveals the database file, not a Bundle's folder the way
    // ticket 12's row-level Open file location will.
    let win_library_reveal = main_window.as_weak();
    let ctx_library_reveal = ctx.clone();
    main_window.on_library_open_file_location_clicked(move || {
        let Some(win) = win_library_reveal.upgrade() else {
            return;
        };
        let db_path = app_database_path();
        if let Err(e) = open_file_location(&ctx_library_reveal, &db_path.to_string_lossy()) {
            toast(&win, e, true);
        }
    });

    // COPY MARKDOWN (ticket 12) - the Bundle's whole stored document, image links rebased to an
    // absolute path a local agent can open. The toast follows the house pattern of saying what did
    // and did not travel: the paths carry the operator's user name, so it says so.
    let win_library_copy_md = main_window.as_weak();
    let ctx_library_copy_md = ctx.clone();
    main_window.on_library_copy_markdown_clicked(move |bundle_id| {
        let Some(win) = win_library_copy_md.upgrade() else {
            return;
        };
        let markdown = match bundle_markdown_for_clipboard(&ctx_library_copy_md, bundle_id.as_str())
        {
            Ok(markdown) => markdown,
            Err(message) => {
                toast(&win, message, true);
                return;
            }
        };
        match put_text_on_clipboard(&markdown) {
            Ok(()) => toast(
                &win,
                "Markdown copied. The image links carry their location on this disk.",
                false,
            ),
            Err(message) => toast(&win, message, true),
        }
    });

    // OPEN FILE LOCATION, per row (ticket 12) - the Bundle's OWN folder, not one file inside it
    // (`open_folder`, not `open_file_location` - see that pair's own doc comments for why they are
    // two different functions). Distinct from `library-open-file-location-clicked` above, which has
    // no Bundle to point at.
    let win_library_reveal_bundle = main_window.as_weak();
    let ctx_library_reveal_bundle = ctx.clone();
    main_window.on_library_bundle_open_file_location_clicked(move |bundle_id| {
        let Some(win) = win_library_reveal_bundle.upgrade() else {
            return;
        };
        let Some(store) = ctx_library_reveal_bundle.bundle_store.as_ref() else {
            toast(&win, "The Bundle library could not be opened.", true);
            return;
        };
        let bundle = match store.get_bundle(bundle_id.as_str()) {
            Ok(Some(detail)) => detail.bundle,
            Ok(None) => {
                toast(&win, "That Bundle is no longer in the Library.", true);
                return;
            }
            Err(e) => {
                toast(&win, format!("Could not read the Bundle: {e}"), true);
                return;
            }
        };
        let folder = bundle_folder_path(&ctx_library_reveal_bundle, &bundle);
        if let Err(message) = open_folder(&folder) {
            toast(&win, message, true);
        }
    });

    // THE ROW MENU'S DESTRUCTIVE GROUP (ticket 16 of the Bundle Library spec). Copy Markdown and
    // Open file location are ticket 12's rows in the SAME menu (`library.slint`'s `row-menu`); this
    // handler only ever sees "disassemble-bundle"/"delete-bundle" because the menu resolves those
    // two actions locally and forwards only the destructive group here. Whichever single verb the
    // destructive entry offers is decided HERE, live, every time the menu opens: never cached on the
    // row (`BR-122`), so a Finding deleted between two menu openings changes the verb the very next
    // time.
    let win_menu_req = main_window.as_weak();
    let ctx_menu_req = ctx.clone();
    main_window.on_library_row_menu_requested(move |id, x, y| {
        let Some(win) = win_menu_req.upgrade() else {
            return;
        };
        let Some(store) = ctx_menu_req.bundle_store.as_ref() else {
            toast(&win, "The Bundle library could not be opened.", true);
            return;
        };
        match store.get_bundle(id.as_str()) {
            Ok(Some(detail)) => {
                let sealed = bundle_is_sealed(&ctx_menu_req, &detail);
                win.set_library_menu_target(id);
                win.set_library_menu_sealed(sealed);
                win.set_library_menu_x(x);
                win.set_library_menu_y(y);
            }
            Ok(None) => {
                // Gone since the row was drawn - refresh rather than open a menu for a Bundle that
                // no longer exists.
                open_library(&win, &ctx_menu_req);
            }
            Err(e) => toast(&win, format!("Could not read the Bundle: {e}"), true),
        }
    });

    let win_menu_dismissed = main_window.as_weak();
    main_window.on_library_row_menu_dismissed(move || {
        if let Some(win) = win_menu_dismissed.upgrade() {
            win.set_library_menu_target(SharedString::new());
        }
    });

    // An item was chosen. Neither destructive action fires from the menu itself - both go through
    // their own confirmation, the same one-click-away safety `pending-delete-finding` already gives
    // deleting a Finding.
    let win_menu_action = main_window.as_weak();
    let ctx_menu_action = ctx.clone();
    main_window.on_library_row_menu_action(move |action, id| {
        let Some(win) = win_menu_action.upgrade() else {
            return;
        };
        win.set_library_menu_target(SharedString::new());

        let Some(store) = ctx_menu_action.bundle_store.as_ref() else {
            toast(&win, "The Bundle library could not be opened.", true);
            return;
        };
        let detail = match store.get_bundle(id.as_str()) {
            Ok(Some(detail)) => detail,
            Ok(None) => {
                open_library(&win, &ctx_menu_action);
                return;
            }
            Err(e) => {
                toast(&win, format!("Could not read the Bundle: {e}"), true);
                return;
            }
        };
        win.set_library_pending_bundle_name(detail.bundle.name.clone().into());
        win.set_library_pending_bundle_finding_count(detail.items.len() as i32);
        match action.as_str() {
            "disassemble-bundle" => win.set_library_pending_disassemble(id),
            "delete-bundle" => win.set_library_pending_delete(id),
            _ => {}
        }
    });

    let win_disassemble_cancel = main_window.as_weak();
    main_window.on_library_disassemble_cancelled(move || {
        if let Some(win) = win_disassemble_cancel.upgrade() {
            win.set_library_pending_disassemble(SharedString::new());
        }
    });

    let win_delete_cancel = main_window.as_weak();
    main_window.on_library_delete_cancelled(move || {
        if let Some(win) = win_delete_cancel.upgrade() {
            win.set_library_pending_delete(SharedString::new());
        }
    });

    // DISASSEMBLE - the Bundle's row and folder go, and its Findings come back to the filmstrip
    // untouched: `spec.md` states plainly that Disassemble writes no Finding, and
    // `load_findings_into_window`'s own filter is what makes them reappear (they were only ever
    // hidden because a `bundle_item` row held them).
    let win_disassemble_confirm = main_window.as_weak();
    let ctx_disassemble_confirm = ctx.clone();
    main_window.on_library_disassemble_confirmed(move |id| {
        let Some(win) = win_disassemble_confirm.upgrade() else {
            return;
        };
        win.set_library_pending_disassemble(SharedString::new());

        match remove_bundle_row_and_folder(&ctx_disassemble_confirm, id.as_str()) {
            Ok(orphaned) => {
                open_library(&win, &ctx_disassemble_confirm);
                load_findings_into_window(&win, &ctx_disassemble_confirm, None);
                let message = if orphaned {
                    "Bundle disassembled. Its Findings are back in the strip. The Bundle's folder \
                     could not be removed and is now an orphan."
                        .to_string()
                } else {
                    "Bundle disassembled. Its Findings are back in the strip.".to_string()
                };
                toast(&win, message, false);
            }
            Err(e) => toast(&win, format!("Could not disassemble the Bundle: {e}"), true),
        }
    });

    // DELETE, on an already-sealed Bundle - the row and folder go; nothing returns to the filmstrip
    // because the originals were discarded earlier (ticket 17's Discard originals, out of this
    // ticket's scope).
    let win_delete_confirm = main_window.as_weak();
    let ctx_delete_confirm = ctx.clone();
    main_window.on_library_delete_confirmed(move |id| {
        let Some(win) = win_delete_confirm.upgrade() else {
            return;
        };
        win.set_library_pending_delete(SharedString::new());

        match remove_bundle_row_and_folder(&ctx_delete_confirm, id.as_str()) {
            Ok(orphaned) => {
                open_library(&win, &ctx_delete_confirm);
                let message = if orphaned {
                    "Bundle deleted. Its folder could not be removed and is now an orphan."
                        .to_string()
                } else {
                    "Bundle deleted.".to_string()
                };
                toast(&win, message, false);
            }
            Err(e) => toast(&win, format!("Could not delete the Bundle: {e}"), true),
        }
    });

    // Bundles Drawer Toggle. Deliberately untouched by this ticket - the spec's own "Out of Scope"
    // names it as a separate, undescribed pattern nothing has asked for yet.
    main_window.on_bundles_drawer_clicked(|| {
        println!("Toggle Bundles Drawer clicked");
    });

    // Settings Toggle
    // `on_settings_clicked` is wired further down, after the hotkey and startup registrars exist -
    // the screen reads from both, and they are built late because `init_from_store` needs the
    // settings store that `AppContext` carries.

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
                        // Deliberately no `main.show()` here. `set_capture_exclusion` only ever
                        // sets a display-affinity flag - the main window was never actually
                        // HIDDEN by starting a capture, only excluded from what other apps can
                        // screenshot, so there is nothing to restore. A `main.show()` used to sit
                        // here unconditionally, and its effect was invisible whenever the window
                        // was already visible - which is why it went unnoticed - but it forced a
                        // window the Reviewer had minimised to the tray back onto the screen on
                        // EVERY capture, "Open the Editor after a hotkey capture" OFF included.
                        // `on_capture_completed` already shows the window on purpose, gated on
                        // that setting; this closure has no business doing it unconditionally too.
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
                            // Reset either way: the flag describes ONE capture, and a stale `true`
                            // would raise the Editor on the next hotkey press for no reason.
                            let reveal = REVEAL_EDITOR_AFTER_CAPTURE.replace(false);
                            if reveal && finding_id.is_some() {
                                let _ = main.show();
                                main.window().set_minimized(false);
                            }
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

                // Copy the region and save nothing. The sibling of `on_capture_completed`, and every
                // difference from it is deliberate:
                //
                // - no `persist_finding`, so no blob and no Finding row;
                // - no `load_findings_into_window`, because nothing was added to the Library and the
                //   Reviewer's current selection is none of this path's business;
                // - `REVEAL_EDITOR_AFTER_CAPTURE` is reset but never acted on. There is nothing to
                //   edit, so raising the Editor would be an interruption with no destination - and
                //   leaving the flag set would raise it on the NEXT capture for no reason.
                //
                // Returns whether the press was consumed. `false` sends the note field back to
                // `TextInput`'s own Ctrl+C, which copies the text the Reviewer had selected.
                overlay.on_copy_chord({
                    let ctx_inner = ctx_inner.clone();
                    let main_weak = main_weak.clone();
                    let close_overlay = close_overlay.clone();
                    move |x, y, sel_w, sel_h, has_text_selection, force_image| {
                        if copy_chord_target(has_text_selection, force_image)
                            == CopyChordTarget::NoteText
                        {
                            return false;
                        }

                        let region = (
                            x.max(0) as u32,
                            y.max(0) as u32,
                            sel_w.max(0) as u32,
                            sel_h.max(0) as u32,
                        );

                        // Read the canvas from the live overlay, never clone it into this closure -
                        // the reason is spelled out on `on_capture_completed` above, and it is worth
                        // ~92 MB per capture.
                        let outcome = LIVE_OVERLAYS.with_borrow(|live| {
                            live.first()
                                .and_then(|entry| {
                                    let canvas = entry.canvas.as_ref()?;
                                    Some(copy_region_to_clipboard(
                                        &ctx_inner,
                                        canvas.as_bytes(),
                                        (entry.placement.2, entry.placement.3),
                                        region,
                                    ))
                                })
                                .unwrap_or_else(|| {
                                    Err("The capture is no longer available to copy.".to_string())
                                })
                        });

                        close_overlay();
                        REVEAL_EDITOR_AFTER_CAPTURE.replace(false);

                        if let Some(main) = main_weak.upgrade() {
                            match outcome {
                                Ok(message) => toast(&main, message, false),
                                Err(message) => toast(&main, message, true),
                            }
                        }
                        true
                    }
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

    // Importing an image file as a Finding.
    //
    // The dialog used to open, filter, let the Reviewer choose - and then print the path and discard
    // it. Every visible part worked, which is what made it look implemented.
    //
    // An import goes through the SAME `persist_finding` a capture does: decode to RGBA, apply the
    // active Quality Budget, write to the Vault, record the Finding and its note. An imported image
    // is therefore not a second kind of Finding with its own rules - the region is simply the whole
    // image.
    let win_weak_open = main_window.as_weak();
    let ctx_open = ctx.clone();
    main_window.on_open_file_clicked(move || {
        let Some(win) = win_weak_open.upgrade() else {
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        else {
            // Cancelled. Not a failure, and not worth a line on screen.
            return;
        };

        let label = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "the chosen file".to_string());

        let dyn_img = match image::open(&path) {
            Ok(img) => img,
            Err(e) => {
                toast(&win, format!("Could not read {label}: {e}"), true);
                return;
            }
        };

        let rgba = dyn_img.to_rgba8();
        let (width, height) = (rgba.width(), rgba.height());

        match persist_finding(
            &ctx_open,
            rgba.as_raw(),
            (width, height),
            (0, 0, width, height),
            &format!("Imported: {label}"),
            "",
        ) {
            Some(id) => {
                load_findings_into_window(&win, &ctx_open, Some(&id));
                toast(
                    &win,
                    format!("{label} imported as a Finding ({width} x {height})"),
                    false,
                );
            }
            None => toast(
                &win,
                format!("Could not import {label}. The Vault write or the database insert failed."),
                true,
            ),
        }
    });

    // --- Tray icon, global hotkeys, and startup registration ---
    let tray_icon_bytes = include_bytes!("../assets/app-icon.png");
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

    let mut startup_registrar = DesktopStartupRegistrar::new(autostart_backend.clone());
    let boot_clock = SystemClock::new();
    let is_autostart_launch = std::env::args().any(|arg| arg == "--autostart");
    let _ = reconcile_startup_on_boot(
        ctx.settings_store.as_ref(),
        &mut startup_registrar,
        &boot_clock,
    );

    // ===== SETTINGS =====================================================================
    //
    // Wired here rather than with the rest, because it is the first surface that needs the hotkey and
    // startup registrars, and those are built after `AppContext`. Nothing below invents state: every
    // value comes from a store or a registrar that already existed and had no screen.
    let startup_for_settings: Rc<DesktopStartupRegistrar> =
        Rc::new(DesktopStartupRegistrar::new(autostart_backend.clone()));

    {
        let win = main_window.as_weak();
        let ctx_open = ctx.clone();
        let startup = startup_for_settings.clone();
        let registrar = hotkey_registrar.clone();
        main_window.on_settings_clicked(move || {
            let Some(win) = win.upgrade() else {
                return;
            };
            if let Ok(hotkeys) = registrar.lock() {
                load_settings_into_window(&win, &ctx_open, startup.as_ref(), &hotkeys);
            }
            win.set_settings_open(true);
        });
    }

    {
        let win = main_window.as_weak();
        main_window.on_settings_closed(move || {
            if let Some(win) = win.upgrade() {
                win.set_settings_open(false);
            }
        });
    }

    {
        let win = main_window.as_weak();
        let ctx_st = ctx.clone();
        let startup = startup_for_settings.clone();
        let registrar = hotkey_registrar.clone();
        main_window.on_startup_toggled(move |on| {
            let Some(win) = win.upgrade() else {
                return;
            };
            let result = if on {
                startup.enable()
            } else {
                startup.disable()
            };
            // The PREFERENCE is recorded whether or not the OS accepted it, which is `BR-112`'s
            // reconciliation contract: `reconcile_startup_on_boot` reads this on every launch and
            // re-applies it, so a registration refused today is retried tomorrow.
            let _ = ctx_st.settings_store.set(&Setting {
                key: SettingKey::RunAtStartup,
                value: SettingValue::Boolean(on),
                updated_at: SystemClock::new().now_rfc3339(),
            });
            if let Err(e) = result {
                toast(&win, format!("Windows refused that: {e}"), true);
            }
            if let Ok(hotkeys) = registrar.lock() {
                load_settings_into_window(&win, &ctx_st, startup.as_ref(), &hotkeys);
            }
        });
    }

    {
        let win = main_window.as_weak();
        let ctx_oe = ctx.clone();
        main_window.on_open_editor_after_capture_toggled(move |on| {
            let Some(win) = win.upgrade() else {
                return;
            };
            match ctx_oe.settings_store.set(&Setting {
                key: SettingKey::OpenEditorAfterCapture,
                value: SettingValue::Boolean(on),
                updated_at: SystemClock::new().now_rfc3339(),
            }) {
                Ok(()) => win.set_open_editor_after_capture(on),
                Err(e) => toast(&win, format!("Could not save that: {e}"), true),
            }
        });
    }

    {
        let win = main_window.as_weak();
        let ctx_vr = ctx.clone();
        main_window.on_vault_reveal_clicked(move || {
            let Some(win) = win.upgrade() else {
                return;
            };
            // The ACTIVE Vault, not the configured one: this button reveals where captures are
            // really landing right now, which after a not-yet-restarted move is still the old
            // folder (`configured_vault_path`'s own doc comment explains the split).
            if let Err(message) = open_folder(&ctx_vr.vault_path) {
                toast(&win, message, true);
            }
        });
    }

    {
        let win = main_window.as_weak();
        let ctx_vb = ctx.clone();
        main_window.on_vault_browse_clicked(move || {
            let Some(win) = win.upgrade() else {
                return;
            };
            let Some(chosen) = rfd::FileDialog::new()
                .set_title("Choose a folder for the Snapdown Vault")
                .set_directory(&ctx_vb.vault_path)
                .pick_folder()
            else {
                return;
            };
            if chosen == ctx_vb.vault_path {
                return;
            }
            // ASKS, and moves nothing. The Reviewer gets to see the folder and the file count before
            // a hundred captures start moving - the owner asked for exactly this.
            win.set_pending_vault_file_count(count_vault_files(&ctx_vb.vault_path) as i32);
            win.set_pending_vault_folder(chosen.display().to_string().into());
        });
    }

    {
        let win = main_window.as_weak();
        let ctx_vm = ctx.clone();
        main_window.on_vault_migration_confirmed(move |folder| {
            let Some(win) = win.upgrade() else {
                return;
            };
            let chosen = PathBuf::from(folder.to_string());

            // The files first. A failure here leaves the old Vault authoritative and the setting
            // unchanged, which is the only ordering where a half-finished move loses nothing.
            let moved = match migrate_vault(&ctx_vm.vault_path, &chosen) {
                Ok(count) => count,
                Err(message) => {
                    toast(&win, format!("Nothing was moved: {message}"), true);
                    return;
                }
            };

            if let Err(e) = ctx_vm.settings_store.set(&Setting {
                key: SettingKey::VaultPath,
                value: SettingValue::String(chosen.display().to_string()),
                updated_at: SystemClock::new().now_rfc3339(),
            }) {
                toast(
                    &win,
                    format!(
                        "{moved} file(s) moved to {}, but the setting could not be saved: {e}. Set                          the folder again before restarting, or move them back.",
                        chosen.display()
                    ),
                    true,
                );
                return;
            }

            // AN ACTUAL RESTART, not just a screen saying one is needed.
            //
            // `AppContext.vault_path` is frozen for the life of the process (see
            // `configured_vault_path`'s doc comment) - every capture, and every Finding thumbnail
            // the filmstrip resolves, keeps reading the OLD folder until the process is gone.
            // The files just moved OUT of that folder, so leaving the Reviewer to restart by hand
            // meant every thumbnail broke and every new capture missed the new Vault until they
            // remembered to - which is not what the Settings screen's own caption already promises
            // them ("Snapdown restarts into the new folder"). Relaunching is the same `Quit` path
            // the tray menu already uses (`std::process::exit`), just preceded by spawning the next
            // instance first.
            let relaunched = std::env::current_exe()
                .and_then(|exe| std::process::Command::new(exe).spawn())
                .is_ok();
            if !relaunched {
                toast(
                    &win,
                    format!(
                        "{moved} file(s) moved to {}. Could not restart automatically - please restart Snapdown yourself.",
                        chosen.display()
                    ),
                    true,
                );
                return;
            }
            std::process::exit(0);
        });
    }

    {
        let win = main_window.as_weak();
        let ctx_bp = ctx.clone();
        let startup = startup_for_settings.clone();
        let registrar = hotkey_registrar.clone();
        main_window.on_budget_chosen(move |name| {
            let Some(win) = win.upgrade() else {
                return;
            };
            let named = match name.as_str() {
                "sharp" => NamedBudget::Sharp,
                "balanced" => NamedBudget::Balanced,
                "small" => NamedBudget::Small,
                "custom" => NamedBudget::Custom,
                _ => NamedBudget::Auto,
            };
            // A named preset drops any custom pair, which is what choosing a preset MEANS. Keeping
            // the pair would leave the screen showing "Balanced" over numbers Balanced does not use.
            // `Custom` is the one exception in spirit but not in code: dropping to `None` here just
            // means `resolve()` falls back to its own defaults until a slider is actually moved,
            // which is the right starting point for "pick your own combination from here".
            if let Err(e) = store_budget(&ctx_bp, &QualityBudget::new(named, None)) {
                toast(&win, format!("Could not save the budget: {e}"), true);
                return;
            }
            if let Ok(hotkeys) = registrar.lock() {
                load_settings_into_window(&win, &ctx_bp, startup.as_ref(), &hotkeys);
            }
        });
    }

    {
        let win = main_window.as_weak();
        let ctx_ba = ctx.clone();
        let startup = startup_for_settings.clone();
        let registrar = hotkey_registrar.clone();
        main_window.on_budget_advanced(move |edge, quality, percent| {
            let Some(win) = win.upgrade() else {
                return;
            };
            // Setting either number by hand makes the budget Custom. It is not a decoration: `Auto`
            // resolves a DIFFERENT pair per capture, so a hand-set pair and `Auto` cannot both be
            // true - and `NamedBudget::Custom` is the domain's own word for that.
            let pair = ResolvedPair {
                max_long_edge: edge.clamp(480, 3840) as u32,
                encoder_quality: quality.clamp(10, 100) as u8,
                resize_percent: percent.clamp(25, 100) as u8,
            };
            if let Err(e) = store_budget(
                &ctx_ba,
                &QualityBudget::new(NamedBudget::Custom, Some(pair)),
            ) {
                toast(&win, format!("Could not save the budget: {e}"), true);
                return;
            }
            if let Ok(hotkeys) = registrar.lock() {
                load_settings_into_window(&win, &ctx_ba, startup.as_ref(), &hotkeys);
            }
        });
    }

    {
        let win = main_window.as_weak();
        let ctx_hk = ctx.clone();
        let startup = startup_for_settings.clone();
        let registrar = hotkey_registrar.clone();
        main_window.on_hotkey_key_pressed(move |action, ctrl, alt, shift, meta, text| {
            let Some(win) = win.upgrade() else {
                return;
            };
            // Updated before the match below, and unconditionally: the Key Check panel's readout
            // chip describes the CURRENT key state even on a mid-gesture press or a refusal, not
            // only a successful bind.
            win.set_hotkey_chord_preview(
                format_chord_preview(ctrl, alt, shift, meta, text.as_str()).into(),
            );
            let shortcut = match shortcut_from_key(ctrl, alt, shift, meta, text.as_str()) {
                // Mid-gesture: the Reviewer is holding Ctrl on the way to something. Say nothing.
                Ok(None) => return,
                Ok(Some(shortcut)) => shortcut,
                // A refusal we could predict, named before the OS is even asked.
                Err(refusal) => {
                    hotkey_feedback(&win, refusal.message(), true);
                    return;
                }
            };
            let Some(target) = hotkey_action_from_id(action.as_str()) else {
                return;
            };
            let outcome = registrar
                .lock()
                .map_err(|e| CoreError::Validation(e.to_string()))
                .and_then(|mut hotkeys| hotkeys.validate_and_rebind(target, &shortcut));
            match outcome {
                Ok(()) => {
                    // A chord we know is contested is bound AND warned about, because Snapdown often
                    // wins the race for it - refusing outright would be the product deciding
                    // something it does not know.
                    let warning = reserved_chord(&shortcut).map(|r| r.message());
                    match warning {
                        Some(message) => {
                            hotkey_feedback(&win, format!("{shortcut} bound. {message}"), false)
                        }
                        None => hotkey_feedback(&win, format!("{shortcut} bound."), false),
                    }
                    win.set_hotkey_listening(SharedString::new());
                }
                // `BR-27` refuses a combination another Snapdown action already holds, and the OS
                // refuses one another application holds. Both come back here as text.
                Err(e) => hotkey_feedback(&win, format!("{shortcut} was refused: {e}"), true),
            }
            if let Ok(hotkeys) = registrar.lock() {
                load_settings_into_window(&win, &ctx_hk, startup.as_ref(), &hotkeys);
            }
        });
    }

    {
        let win = main_window.as_weak();
        let ctx_he = ctx.clone();
        let startup = startup_for_settings.clone();
        let registrar = hotkey_registrar.clone();
        main_window.on_hotkey_enabled_toggled(move |action, on| {
            let Some(win) = win.upgrade() else {
                return;
            };
            let Some(target) = hotkey_action_from_id(action.as_str()) else {
                return;
            };
            let outcome = registrar
                .lock()
                .map_err(|e| CoreError::Validation(e.to_string()))
                .and_then(|mut hotkeys| hotkeys.set_enabled(target, on));
            if let Err(e) = outcome {
                hotkey_feedback(
                    &win,
                    format!(
                        "Could not {} that: {e}",
                        if on { "enable" } else { "disable" }
                    ),
                    true,
                );
            }
            if let Ok(hotkeys) = registrar.lock() {
                load_settings_into_window(&win, &ctx_he, startup.as_ref(), &hotkeys);
            }
        });
    }

    // Per FR-18/BR-121: launching via Windows startup opens no window, tray icon only.
    if !is_autostart_launch {
        main_window.show()?;
        // NOT synchronously after `show()`, on a timer instead - the same fix the capture
        // overlay's pre-warm already needed, and the same root cause: `show()` requests the
        // window, it does not create it, the event loop does on its next turn
        // (`with_winit_window`'s own docs: it "will only succeed when the event loop is
        // active"). Calling `set_window_shadow` right here, before `run_event_loop_until_quit()`
        // below has even started, made every attempt at this silently find no winit window and
        // do nothing - three attempts in a row were mistaken for the WRONG shadow technique
        // rather than the right one landing too early. A `Timer` fires from inside the running
        // loop, by which point the window exists.
        let shadow_target = main_window.as_weak();
        slint::Timer::single_shot(std::time::Duration::from_millis(50), move || {
            if let Some(win) = shadow_target.upgrade() {
                set_window_shadow(&win);
            }
        });
    }

    // Poll tray-icon and global-hotkey events on the UI thread; both crates deliver events
    // through crossbeam channels rather than hooking into Slint's winit event loop directly.
    let window_for_events = main_window.as_weak();
    let hotkey_poll_registrar = hotkey_registrar.clone();
    let ctx_hotkey = ctx.clone();
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
                                // Straight into capture. The Editor arrives when the capture does.
                                REVEAL_EDITOR_AFTER_CAPTURE.set(true);
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
                                    .map(|_| (*action, shortcut.clone()))
                            })
                    });

                    if let Some((action, shortcut)) = action {
                        let action_id: SharedString = match action {
                            HotkeyAction::Capture => "capture",
                            HotkeyAction::OpenEditor => "open_editor",
                        }
                        .into();
                        // Reported to the Settings screen before it is acted on, so a Reviewer who
                        // has just bound a chord can press it and see the row confirm. Registration
                        // alone does not prove delivery - another application's low-level hook can
                        // swallow a keystroke Windows happily registered - and this is the only proof
                        // available without leaving the screen.
                        if let Some(win) = window_for_events.upgrade() {
                            if win.get_settings_open() {
                                win.set_hotkey_last_fired(action_id.clone());
                                // Worded for the Key Check panel, which is where this proof of
                                // arrival now lives instead of on the row itself.
                                win.set_hotkey_last_fired_text(
                                    format!(
                                        "{} ({}) reached Snapdown just now.",
                                        display_shortcut(&shortcut),
                                        action.label()
                                    )
                                    .into(),
                                );
                                // A row listening for a NEW chord never sees its own already-bound
                                // one as a normal keypress: Windows delivers an active global
                                // hotkey as WM_HOTKEY, not to the focused FocusScope, so the row's
                                // button would otherwise sit on "Listening…" forever with no way to
                                // tell the Reviewer why. Re-pressing the row's own current
                                // combination changes nothing, so ending the gesture here - the
                                // same way a completed keypress-driven bind does - is the correct
                                // outcome, not a special case of it.
                                if win.get_hotkey_listening() == action_id {
                                    win.set_hotkey_listening(SharedString::new());
                                    // The completing keypress may ALSO have reached the row's own
                                    // FocusScope as a normal key event before this one landed, and
                                    // read as a refusal there - the row's already-bound combination
                                    // is not a shape `shortcut_from_key` was ever asked to parse.
                                    // Nothing was actually refused: the OS just proved the SAME
                                    // shortcut still arrives, which is a success, not an error, and
                                    // must not leave a red banner behind saying otherwise.
                                    hotkey_feedback(&win, "", false);
                                }
                            }
                        }
                        // Settings shows "Pressed just now" above regardless, so the Reviewer
                        // testing a binding from the Hotkeys tab still gets that confirmation -
                        // but the REAL action does not also fire underneath them. Without this, a
                        // Reviewer holding down the Capture shortcut to check it, or to rebind it
                        // to itself, got a real capture opening behind Settings every time,
                        // because the global hotkey fires regardless of window focus.
                        let settings_open = window_for_events
                            .upgrade()
                            .is_some_and(|win| win.get_settings_open());
                        match action {
                            HotkeyAction::Capture => {
                                if let Some(win) = window_for_events.upgrade() {
                                    // The setting decides, and `BG-6` is why there is a setting:
                                    // the hotkey exists to keep the Reviewer in the flow they are
                                    // already in. Default off.
                                    REVEAL_EDITOR_AFTER_CAPTURE
                                        .set(open_editor_after_capture(&ctx_hotkey));
                                    if !settings_open {
                                        win.invoke_capture_clicked();
                                    }
                                }
                            }
                            HotkeyAction::OpenEditor => {
                                if let Some(win) = window_for_events.upgrade() {
                                    if !settings_open {
                                        win.show().unwrap();
                                        win.window().set_minimized(false);
                                    }
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
    /// Every combination of the two inputs to the copy chords, enumerated.
    ///
    /// Four cases, and three of them copy the image. The one that does not is the whole reason this
    /// is a function and not an expression inside `appwindow.slint`: a keypress cannot be reached
    /// from any test seam in this repository (`OQ-23`), so a rule written in Slint is a rule that
    /// can be inverted without anything going red.
    #[test]
    fn ctrl_c_copies_the_image_unless_the_note_field_has_text_selected() {
        // The ordinary case: a region selected, nothing selected in the note field.
        assert_eq!(copy_chord_target(false, false), CopyChordTarget::Image);
        // Text is selected, so Ctrl+C means what it means in every other text field.
        assert_eq!(copy_chord_target(true, false), CopyChordTarget::NoteText);
    }

    /// Ctrl+Enter is the unconditional escape hatch, and that is exactly what makes Ctrl+C safe to
    /// make conditional: there is always one chord that means "the image" whatever the caret is
    /// doing. Lose this and a Reviewer with text selected has no way to reach the image at all.
    #[test]
    fn ctrl_enter_copies_the_image_even_with_text_selected() {
        assert_eq!(copy_chord_target(false, true), CopyChordTarget::Image);
        assert_eq!(copy_chord_target(true, true), CopyChordTarget::Image);
    }

    /// A `Ctrl+C` leaves the Vault and the Library exactly as it found them, and what it hands over
    /// is a real image.
    ///
    /// Both halves matter and they fail differently. The "nothing was written" half is the promise
    /// the feature is FOR - a screenshot that never touches disk - and it is the half that would
    /// pass trivially if the test never exercised the path, which is why the assertions are a
    /// directory listing and a row count rather than a look at the code. The decode is this
    /// repository's own rule, learned expensively: a 17-byte fake `PNG` header with a plausible
    /// width and height passed every image assertion here for five waves. A signature and a
    /// dimension is a test a fabrication passes; a decode is not.
    #[test]
    fn a_copy_writes_nothing_and_hands_over_a_decodable_image() {
        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let ctx = AppContext {
            vault_store: VaultBlobStore::new(vault_dir.path()).expect("a vault at a temp path"),
            vault_path: vault_dir.path().to_path_buf(),
            finding_store: Arc::new(
                SqliteFindingStore::open_in_memory().expect("a findings store"),
            ),
            settings_store: Arc::new(
                SqliteSettingsStore::open_in_memory().expect("a settings store"),
            ),
            bundle_store: None,
        };

        // A 4x2 canvas: the left half opaque red, the right half opaque blue. Cropping the right
        // half is what proves the crop is applied and not merely computed - a copy of the whole
        // canvas would come back red in its first pixel.
        let red = [255u8, 0, 0, 255];
        let blue = [0u8, 0, 255, 255];
        let mut source = Vec::with_capacity(4 * 2 * 4);
        for _ in 0..2 {
            source.extend_from_slice(&red);
            source.extend_from_slice(&red);
            source.extend_from_slice(&blue);
            source.extend_from_slice(&blue);
        }

        let (bmp, width, height) = encode_region_for_clipboard(&ctx, &source, (4, 2), (2, 0, 2, 2))
            .expect("the region to encode");

        let decoded = image::load_from_memory(&bmp).expect("the clipboard bytes to be an image");
        assert_eq!(
            (decoded.width(), decoded.height()),
            (width, height),
            "the reported dimensions have to be the decoded ones"
        );
        // Which half was taken, not an exact byte. The Quality Budget's encoder quantises - this
        // pixel comes back as 254 rather than 255 - and that off-by-one is itself worth noting: it
        // is the proof that the budget is genuinely applied on this path, because a lossless copy
        // would have handed back the 255 that went in.
        let pixel = decoded.to_rgba8().get_pixel(0, 0).0;
        assert!(
            pixel[2] > 200 && pixel[0] < 50,
            "the crop should have taken the blue half, got {pixel:?} at {width}x{height}"
        );

        assert!(
            std::fs::read_dir(vault_dir.path())
                .expect("to read the vault")
                .next()
                .is_none(),
            "a copy must leave no file behind in the Vault"
        );
        assert!(
            ctx.finding_store
                .list_findings()
                .expect("to read the Library")
                .is_empty(),
            "a copy must record no Finding"
        );
    }

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
    fn display_shortcut_never_shows_the_stored_command_or_control_token() {
        let displayed = display_shortcut("CommandOrControl+Shift+S");
        assert!(!displayed.contains("CommandOrControl"));
        assert_eq!(displayed, format!("{PRIMARY_MODIFIER_DISPLAY}+Shift+S"));
        assert_eq!(display_shortcut(""), "");
        // A shortcut with no primary modifier in it (should not occur today, since
        // `shortcut_from_key` always includes one, but the function must not invent one) passes
        // through unchanged.
        assert_eq!(display_shortcut("Alt+Shift+S"), "Alt+Shift+S");
    }

    #[test]
    fn display_shortcut_never_shows_the_stored_super_token() {
        let displayed = display_shortcut("Super+Alt+Q");
        assert!(!displayed.contains("Super"));
        assert_eq!(displayed, format!("{META_KEY_DISPLAY}+Alt+Q"));
    }

    #[test]
    fn a_control_character_is_never_a_displayable_key() {
        // The bug the Key Check panel actually shipped with: a bare Enter/Backspace/arrow press,
        // or a Ctrl+letter combination whose text arrives as its ASCII control code rather than the
        // letter, rendered as an unreadable tofu box in the readout chip.
        for code in [
            '\u{11}', '\u{12}', '\u{13}', '\u{14}', '\r', '\t', '\u{8}', '\u{3}',
        ] {
            assert_eq!(displayable_key_text(&code.to_string()), None);
        }
        assert_eq!(displayable_key_text(""), None);
    }

    #[test]
    fn an_ordinary_or_named_key_is_displayable() {
        assert_eq!(displayable_key_text("q"), Some("Q".to_string()));
        assert_eq!(displayable_key_text("F5"), Some("F5".to_string()));
        assert_eq!(displayable_key_text("Left"), Some("Left".to_string()));
    }

    #[test]
    fn the_chord_preview_never_leaks_a_control_character() {
        assert_eq!(format_chord_preview(false, false, false, false, "\r"), "");
        assert_eq!(
            format_chord_preview(true, false, false, true, "q"),
            format!("{PRIMARY_MODIFIER_DISPLAY} + {META_KEY_DISPLAY} + Q")
        );
        // Mid-gesture - modifiers held, no completing key yet - is a legitimate, if incomplete,
        // preview, not an error.
        assert_eq!(
            format_chord_preview(true, true, false, false, ""),
            format!("{PRIMARY_MODIFIER_DISPLAY} + Alt")
        );
    }

    #[test]
    fn win_alone_is_a_sufficient_modifier_for_a_shortcut() {
        // The owner's own question: "kenapa Win gak bisa jadi shortcut?" - it can, as long as the
        // chord is not one of the specific ones `reserved_chord` documents the shell intercepting.
        assert_eq!(
            shortcut_from_key(false, false, false, true, "Q"),
            Ok(Some("Super+Q".to_string()))
        );
    }

    #[test]
    fn win_shift_still_needs_no_extra_modifier_to_qualify() {
        assert_eq!(
            shortcut_from_key(false, false, true, true, "Q"),
            Ok(Some("Super+Shift+Q".to_string()))
        );
    }

    #[test]
    fn win_and_ctrl_compose_in_fixed_order() {
        assert_eq!(
            shortcut_from_key(true, false, false, true, "Q"),
            Ok(Some("CommandOrControl+Super+Q".to_string()))
        );
    }

    #[test]
    fn an_unnameable_key_is_refused_without_leaking_a_raw_glyph() {
        // A private-use-area key code, or anything else outside letters/digits/ASCII punctuation,
        // must never land in the Reviewer-facing sentence as-is - that is the exact bug reported: a
        // tofu-box glyph inside "Snapdown cannot register the X key...".
        let refusal = shortcut_from_key(true, false, false, false, "\u{f8ff}").unwrap_err();
        assert_eq!(
            refusal,
            ShortcutRefusal::UnsupportedKey("this key".to_string())
        );
        assert_eq!(
            refusal.message(),
            "Snapdown cannot register this key as part of a shortcut."
        );
    }

    #[test]
    fn an_ascii_punctuation_key_is_named_in_its_own_refusal() {
        let refusal = shortcut_from_key(true, false, false, false, ",").unwrap_err();
        assert_eq!(
            refusal.message(),
            "Snapdown cannot register the ',' key as part of a shortcut."
        );
    }

    #[test]
    fn a_bare_win_press_is_still_mid_gesture_not_a_shortcut() {
        // Slint sends a lone Meta key down as its own control character, same as Ctrl/Alt/Shift.
        assert_eq!(
            shortcut_from_key(false, false, false, true, "\u{14}"),
            Ok(None)
        );
    }

    #[test]
    fn windows_reserved_win_chords_are_refused_by_name() {
        for (shortcut, key) in [("Super+5", "5"), ("Super+E", "E"), ("Super+D", "D")] {
            let win_only = shortcut_from_key(false, false, false, true, key);
            assert_eq!(
                win_only,
                Err(reserved_chord(shortcut).expect("documented as reserved"))
            );
        }
        assert_eq!(
            shortcut_from_key(true, false, false, true, "Left"),
            Err(reserved_chord("CommandOrControl+Super+Left").expect("documented as reserved"))
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

    // ---- Bundle assembly -----------------------------------------------------
    //
    // These test the two pure halves against real bytes. `plan_bundle` DECODES nothing itself, so
    // every assertion below decodes the planned output: an image test that checks a signature and a
    // dimension is a test a 17-byte fake header passes, and this repository has already shipped one
    // for five waves.

    use image::ImageEncoder;
    use snapdown_core::domain::finding::Marker;

    /// A real PNG of a recognisable gradient, so a burned copy can be told apart from a blank one.
    fn png_fixture(width: u32, height: u32) -> Vec<u8> {
        let mut img = image::RgbaImage::new(width, height);
        for (x, y, px) in img.enumerate_pixels_mut() {
            *px = image::Rgba([
                (x * 255 / width.max(1)) as u8,
                (y * 255 / height.max(1)) as u8,
                128,
                255,
            ]);
        }
        let mut bytes: Vec<u8> = Vec::new();
        image::codecs::png::PngEncoder::new(&mut bytes)
            .write_image(img.as_raw(), width, height, image::ExtendedColorType::Rgba8)
            .expect("the fixture encoder must succeed");
        bytes
    }

    fn detail(id: &str, width: u32, height: u32, markers: Vec<Marker>) -> FindingDetail {
        FindingDetail {
            finding: Finding {
                id: id.to_string(),
                image_path: format!("findings/{id}.png"),
                image_width: width,
                image_height: height,
                captured_at: "2026-08-27T10:00:00Z".to_string(),
                source_monitor: "\\\\.\\DISPLAY1".to_string(),
                region: format!("0,0,{width},{height}"),
                resolved_long_edge: None,
                resolved_encoder_quality: None,
                budget_name: None,
            },
            note: Note {
                id: format!("note-{id}"),
                finding_id: id.to_string(),
                body: "the header overlaps the logo".to_string(),
                updated_at: "2026-08-27T10:00:00Z".to_string(),
            },
            markers,
            visual_annotations: Vec::new(),
        }
    }

    fn marker(id: &str, finding_id: &str, ordinal: u32, comment: &str) -> Marker {
        Marker {
            id: id.to_string(),
            finding_id: finding_id.to_string(),
            ordinal,
            x: 0.4,
            y: 0.6,
            comment: comment.to_string(),
        }
    }

    #[test]
    fn a_planned_bundle_carries_bytes_for_every_path_it_records() {
        let details = vec![
            detail("f1", 40, 30, vec![marker("m1", "f1", 1, "this button")]),
            detail("f2", 24, 24, vec![]),
        ];
        let planned = plan_bundle("b1", "Bundle One", "", &details, |path| {
            Ok(png_fixture_for(path))
        })
        .expect("planning must succeed");

        assert_eq!(planned.items.len(), 2);
        assert_eq!(planned.blobs.len(), 2);

        // BUG-19 was a recorded path with no file behind it. Every path is paired with its bytes
        // here, and the bytes are a decodable image at the Finding's own dimensions.
        for (index, item) in planned.items.iter().enumerate() {
            let (path, bytes) = &planned.blobs[index];
            assert_eq!(
                &item.image_path, path,
                "item {} records a path the plan has no bytes for",
                item.position
            );
            let decoded = image::load_from_memory(bytes)
                .unwrap_or_else(|e| panic!("burned copy {path} does not decode: {e}"));
            assert_eq!(
                (decoded.width(), decoded.height()),
                (
                    details[index].finding.image_width,
                    details[index].finding.image_height
                ),
                "AD-4: a Bundle's image is the Finding's image at the SAME dimensions"
            );
        }

        // A copy that decodes at the right size is NOT proof the Markers were drawn - that is the
        // signature-and-dimension trap this repository already fell into for five waves, wearing a
        // different hat. f1 carries an active Marker and MUST differ from its source; f2 carries
        // none and MUST be byte-identical, which is AD-9's promise for that case.
        let f1_source = png_fixture_for("findings/f1.png");
        assert_ne!(
            planned.blobs[0].1, f1_source,
            "Finding 1 has an active Marker, so its burned copy cannot be the source unchanged"
        );
        let f2_source = png_fixture_for("findings/f2.png");
        assert_eq!(
            planned.blobs[1].1, f2_source,
            "AD-9: a Finding with nothing to draw is copied byte-for-byte"
        );
    }

    #[test]
    fn a_planned_bundle_points_its_markdown_at_the_burned_copy_not_the_finding() {
        let details = vec![detail("f1", 40, 30, vec![marker("m1", "f1", 1, "here")])];
        let planned = plan_bundle("b7", "Bundle Seven", "", &details, |path| {
            Ok(png_fixture_for(path))
        })
        .expect("planning must succeed");

        // BUG-21: the composed Markdown referenced `finding.image_path`, so a reader following it
        // landed on the clean image and FR-8 stayed unmet even once the burn was written.
        //
        // The link is written relative to `bundle.md`'s own folder rather than to the Vault
        // root - what NFR-8 requires and what `BUG-86` broke - so the burned copy is named
        // without its `bundles/b7/` prefix. The invariant guarded here is unchanged: the
        // burned copy, never the clean Finding image.
        assert!(
            planned.markdown.contains("](./finding_1_burned.png)"),
            "the Markdown must reference the Bundle's own burned copy:\n{}",
            planned.markdown
        );
        assert!(
            !planned.markdown.contains("findings/f1.png"),
            "the Markdown must NOT reference the Finding's clean image:\n{}",
            planned.markdown
        );
    }

    #[test]
    fn a_planned_bundle_refuses_an_image_it_cannot_read() {
        let details = vec![detail("f1", 40, 30, vec![]), detail("f2", 40, 30, vec![])];
        let err = plan_bundle("b2", "Bundle Two", "", &details, |path| {
            if path.contains("f2") {
                Err(CoreError::NotFound(format!("no blob at {path}")))
            } else {
                Ok(png_fixture_for(path))
            }
        })
        .expect_err("an unreadable image must refuse the whole Bundle");

        // BUG-23: the archived caller swallowed this and put the Publication live without the image.
        assert!(
            err.contains("findings/f2.png"),
            "the error must name the file: {err}"
        );
        assert!(
            err.contains("Nothing was written"),
            "the error must say the Bundle was refused, not partially written: {err}"
        );
    }

    #[test]
    fn a_planned_bundle_refuses_an_image_whose_recorded_dimensions_are_wrong() {
        // The stored row says 40x30; the blob is 10x10. AD-4 cannot be satisfied for it, and the
        // burner's dimension check is what says so.
        let details = vec![detail("f1", 40, 30, vec![])];
        let err = plan_bundle("b3", "Bundle Three", "", &details, |_| {
            Ok(png_fixture(10, 10))
        })
        .expect_err("a dimension mismatch must refuse the Bundle");
        assert!(
            err.contains("Nothing was written"),
            "a mismatch must refuse rather than write: {err}"
        );
    }

    #[test]
    fn resolving_findings_refuses_an_id_the_library_has_lost() {
        let ids = vec!["f1".to_string(), "gone".to_string(), "f3".to_string()];
        let err = resolve_bundle_findings(&ids, |id| {
            if id == "gone" {
                Ok(None)
            } else {
                Ok(Some(detail(id, 8, 8, vec![])))
            }
        })
        .expect_err("UC-9 is all-or-nothing: an unresolvable id must refuse the Bundle");

        // BUG-22: the archived caller skipped it and renumbered silently around the gap.
        assert!(err.contains("gone"), "the error must name the id: {err}");
    }

    #[test]
    fn resolving_findings_keeps_the_order_it_was_given() {
        // The position in the Markdown is the position in the strip. If this ever sorts, AD-1's tie
        // between a Marker ordinal and a Markdown line number is being read against the wrong image.
        let ids = vec!["c".to_string(), "a".to_string(), "b".to_string()];
        let details = resolve_bundle_findings(&ids, |id| Ok(Some(detail(id, 8, 8, vec![]))))
            .expect("resolution must succeed");
        let got: Vec<&str> = details.iter().map(|d| d.finding.id.as_str()).collect();
        assert_eq!(got, vec!["c", "a", "b"]);
    }

    /// The fixture reader: a real PNG at whatever size the path's Finding was declared with.
    fn png_fixture_for(path: &str) -> Vec<u8> {
        if path.contains("f2") && path.contains("findings") {
            png_fixture(24, 24)
        } else {
            png_fixture(40, 30)
        }
    }

    // ===== ticket 11: the Library opens and lists every Bundle =============================

    /// The ladder the spec's own wording implies: "just now / N minutes ago / yesterday / last
    /// week". Fixed instants on both sides, not `Utc::now()`, so this cannot flake on a slow CI
    /// runner the way a real-clock comparison would.
    #[test]
    fn relative_time_reads_a_ladder_of_plain_words() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-09-02T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let ago = |seconds: i64| (now - chrono::Duration::seconds(seconds)).to_rfc3339();

        assert_eq!(relative_time(&ago(10), now), "just now");
        assert_eq!(relative_time(&ago(60), now), "a minute ago");
        assert_eq!(relative_time(&ago(60 * 5), now), "5 minutes ago");
        assert_eq!(relative_time(&ago(60 * 60), now), "an hour ago");
        assert_eq!(relative_time(&ago(60 * 60 * 3), now), "3 hours ago");
        assert_eq!(relative_time(&ago(60 * 60 * 30), now), "yesterday");
        assert_eq!(relative_time(&ago(60 * 60 * 24 * 3), now), "3 days ago");
        assert_eq!(relative_time(&ago(60 * 60 * 24 * 8), now), "last week");
        assert_eq!(relative_time(&ago(60 * 60 * 24 * 21), now), "3 weeks ago");
        assert_eq!(relative_time(&ago(60 * 60 * 24 * 90), now), "3 months ago");
        assert_eq!(relative_time(&ago(60 * 60 * 24 * 400), now), "1 year ago");
    }

    /// A composed time this build cannot parse must not panic the Library open - it must read as
    /// "an unknown time ago" rather than crash the whole overlay over one bad row.
    #[test]
    fn relative_time_refuses_gracefully_on_unparseable_input() {
        let now = chrono::Utc::now();
        assert_eq!(relative_time("not a timestamp", now), "an unknown time ago");
    }

    /// `bundle_store: None` is one of the two "cannot be read" triggers `AppContext` documents on
    /// its own field - the tables never opened. `build_library_rows` must refuse out loud with a
    /// message that names what refused, the same shape `prepare_bundle` already uses for the same
    /// condition, rather than silently reporting an empty Library.
    #[test]
    fn build_library_rows_refuses_out_loud_when_the_bundle_store_is_none() {
        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let ctx = AppContext {
            vault_store: VaultBlobStore::new(vault_dir.path()).expect("a vault at a temp path"),
            vault_path: vault_dir.path().to_path_buf(),
            finding_store: Arc::new(
                SqliteFindingStore::open_in_memory().expect("a findings store"),
            ),
            settings_store: Arc::new(
                SqliteSettingsStore::open_in_memory().expect("a settings store"),
            ),
            bundle_store: None,
        };

        let err = build_library_rows(&ctx, chrono::Utc::now())
            .expect_err("a None store must refuse, not read empty");
        assert!(
            err.contains("could not be opened"),
            "the message must name what refused: {err}"
        );
    }

    /// A `list_bundles` failure - a locked or corrupt `library.db` - is the other trigger
    /// `AppContext.bundle_store`'s own doc comment names, and it must surface the underlying
    /// reason rather than a generic sentence, so the Reviewer sees what actually refused.
    ///
    /// Built on a connection that was never migrated, so `list_bundles`'s own query fails for
    /// real - "no such table: bundle" - rather than a `CoreError` faked to look like one.
    #[test]
    fn build_library_rows_refuses_out_loud_when_list_bundles_itself_fails() {
        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let unmigrated = rusqlite::Connection::open_in_memory().expect("an in-memory connection");
        let bundle_store = SqliteBundleStore::new(Arc::new(Mutex::new(unmigrated)));

        let ctx = AppContext {
            vault_store: VaultBlobStore::new(vault_dir.path()).expect("a vault at a temp path"),
            vault_path: vault_dir.path().to_path_buf(),
            finding_store: Arc::new(
                SqliteFindingStore::open_in_memory().expect("a findings store"),
            ),
            settings_store: Arc::new(
                SqliteSettingsStore::open_in_memory().expect("a settings store"),
            ),
            bundle_store: Some(Arc::new(bundle_store)),
        };

        let err = build_library_rows(&ctx, chrono::Utc::now())
            .expect_err("an unmigrated connection must refuse");
        assert!(
            err.contains("Could not read the Library"),
            "the message must name what refused: {err}"
        );
    }

    /// The row-building path end to end, against a real in-memory store and real files on a temp
    /// Vault: every Bundle appears once, newest-composed first (the store's own `ORDER BY
    /// composed_at DESC`, not re-sorted here), the thumbnail is the BundleItem at position 1 and
    /// actually DECODES (not merely a signature-and-dimensions fake - `AGENTS.md`'s own rule), and
    /// the meta line reads "N Findings · composed <relative time>" exactly as `spec.md`'s
    /// Implementation Decisions section states it three times over.
    #[test]
    fn build_library_rows_lists_every_bundle_newest_composed_first_with_a_decoded_thumbnail() {
        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let bundle_store = SqliteBundleStore::open_in_memory().expect("a bundle store");

        let older = Bundle::new(
            "b-older".into(),
            "Older review".into(),
            "# Older review".into(),
            "bundles/b-older/bundle.md".into(),
            "2026-08-28T10:00:00Z".into(),
        )
        .unwrap();
        let older_items = vec![BundleItem::new(
            "bi-older-1".into(),
            "b-older".into(),
            "f-older-1".into(),
            1,
            "bundles/b-older/finding_1_burned.png".into(),
        )
        .unwrap()];

        let newer = Bundle::new(
            "b-newer".into(),
            "Newer review".into(),
            "# Newer review".into(),
            "bundles/b-newer/bundle.md".into(),
            "2026-09-01T10:00:00Z".into(),
        )
        .unwrap();
        let newer_items = vec![
            BundleItem::new(
                "bi-newer-1".into(),
                "b-newer".into(),
                "f-newer-1".into(),
                1,
                "bundles/b-newer/finding_1_burned.png".into(),
            )
            .unwrap(),
            BundleItem::new(
                "bi-newer-2".into(),
                "b-newer".into(),
                "f-newer-2".into(),
                2,
                "bundles/b-newer/finding_2_burned.png".into(),
            )
            .unwrap(),
        ];

        bundle_store
            .create_bundle(&older, &older_items)
            .expect("creating the older Bundle must succeed");
        bundle_store
            .create_bundle(&newer, &newer_items)
            .expect("creating the newer Bundle must succeed");

        for item in older_items.iter().chain(newer_items.iter()) {
            let path = vault_dir.path().join(&item.image_path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, png_fixture(12, 8)).unwrap();
        }

        let ctx = AppContext {
            vault_store: VaultBlobStore::new(vault_dir.path()).expect("a vault at a temp path"),
            vault_path: vault_dir.path().to_path_buf(),
            finding_store: Arc::new(
                SqliteFindingStore::open_in_memory().expect("a findings store"),
            ),
            settings_store: Arc::new(
                SqliteSettingsStore::open_in_memory().expect("a settings store"),
            ),
            bundle_store: Some(Arc::new(bundle_store)),
        };

        // A fixed "now", not `Utc::now()`: `composed_at` above is fixed too, so the relative-time
        // wording below is deterministic whatever day this test actually runs.
        let now = chrono::DateTime::parse_from_rfc3339("2026-09-02T10:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        let rows = build_library_rows(&ctx, now).expect("a healthy store must list its Bundles");
        assert_eq!(rows.len(), 2, "every Bundle must appear once");

        // Newest-composed first - the store's own order, preserved rather than re-sorted.
        assert_eq!(rows[0].id, "b-newer");
        assert_eq!(rows[1].id, "b-older");

        assert_eq!(
            rows[0].meta_line, "2 Findings · composed yesterday",
            "the meta line must read exactly \"N Findings · composed <relative time>\""
        );
        assert_eq!(rows[1].meta_line, "1 Finding · composed 5 days ago");

        // The thumbnail must actually DECODE, not merely carry a plausible signature and
        // dimensions - the fabrication this repository has been burned by before.
        assert!(
            rows[0].thumbnail.size().width > 0,
            "the thumbnail must be a real decoded image, not the empty default"
        );
    }

    // ===== ticket 12: Copy Markdown and Open file location, from a Library row ==============

    /// Builds a `(Bundle, [BundleItem])` pair whose `markdown` is a real composed document (via
    /// `MarkdownSerializer::serialize_bundle`, not a hand-typed fixture), so `rebase_image_links`
    /// has the exact grammar it expects to parse.
    fn bundle_fixture(bundle_id: &str) -> (Bundle, Vec<BundleItem>) {
        let fid = format!("f-{bundle_id}");
        let finding_detail = detail(&fid, 40, 30, vec![]);
        let item = BundleItem::new(
            format!("bi-{bundle_id}"),
            bundle_id.to_string(),
            fid,
            1,
            format!("bundles/{bundle_id}/finding_1_burned.png"),
        )
        .unwrap();
        let markdown_path = format!("bundles/{bundle_id}/bundle.md");
        let markdown = MarkdownSerializer::serialize_bundle(
            "Checkout Flow Review",
            "",
            &[(&item, &finding_detail)],
            &markdown_path,
        );
        let bundle = Bundle::new(
            bundle_id.to_string(),
            "Checkout Flow Review".to_string(),
            markdown,
            markdown_path,
            "2026-09-01T10:00:00Z".to_string(),
        )
        .unwrap();
        (bundle, vec![item])
    }

    fn library_test_ctx(vault_dir: &Path, bundle_store: SqliteBundleStore) -> AppContext {
        AppContext {
            vault_store: VaultBlobStore::new(vault_dir).expect("a vault at a temp path"),
            vault_path: vault_dir.to_path_buf(),
            finding_store: Arc::new(
                SqliteFindingStore::open_in_memory().expect("a findings store"),
            ),
            settings_store: Arc::new(
                SqliteSettingsStore::open_in_memory().expect("a settings store"),
            ),
            bundle_store: Some(Arc::new(bundle_store)),
        }
    }

    /// The acceptance criterion at its own seam: the clipboard text is produced by calling
    /// `MarkdownSerializer::rebase_image_links` - not a second, UI-side reimplementation - so this
    /// asserts equality against calling that function directly, the same way the ticket's checklist
    /// asks for.
    #[test]
    fn bundle_markdown_for_clipboard_is_exactly_what_rebase_image_links_produces() {
        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let bundle_store = SqliteBundleStore::open_in_memory().expect("a bundle store");
        let (bundle, items) = bundle_fixture("b-1");
        bundle_store
            .create_bundle(&bundle, &items)
            .expect("creating the Bundle must succeed");
        let ctx = library_test_ctx(vault_dir.path(), bundle_store);

        let got = bundle_markdown_for_clipboard(&ctx, "b-1").expect("must produce clipboard text");
        let expected = MarkdownSerializer::rebase_image_links(
            &bundle.markdown,
            &vault_dir.path().to_string_lossy(),
            &bundle.markdown_path,
        )
        .expect("rebase must succeed independently too");
        assert_eq!(
            got, expected,
            "Copy Markdown must hand the composer's own rebase output to the clipboard, unchanged"
        );

        // And the diff from the stored document is exactly the image link destinations: absolute,
        // forward-slashed, `<>`-wrapped - never anything else in the document.
        assert_ne!(
            got, bundle.markdown,
            "rebasing must actually change something"
        );
        assert!(
            got.contains('<'),
            "the rebased link must be angle-bracket wrapped"
        );
        let vault_forward_slash = vault_dir.path().to_string_lossy().replace('\\', "/");
        assert!(
            got.contains(&format!(
                "<{vault_forward_slash}/bundles/b-1/finding_1_burned.png>"
            )),
            "the rebased link must be absolute and point at the Bundle's own burned copy: {got}"
        );
    }

    /// The stored file itself is never touched by Copy Markdown - `bundle_markdown_for_clipboard`
    /// only reads the row and transforms a string in memory. Proven against a real file on disk,
    /// not merely by reading the function's own source.
    #[test]
    fn bundle_markdown_for_clipboard_leaves_the_stored_file_byte_identical() {
        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let bundle_store = SqliteBundleStore::open_in_memory().expect("a bundle store");
        let (bundle, items) = bundle_fixture("b-1");
        bundle_store
            .create_bundle(&bundle, &items)
            .expect("creating the Bundle must succeed");

        let folder = vault_dir.path().join("bundles").join("b-1");
        std::fs::create_dir_all(&folder).unwrap();
        let markdown_file = folder.join("bundle.md");
        std::fs::write(&markdown_file, bundle.markdown.as_bytes()).unwrap();
        let before = std::fs::read(&markdown_file).unwrap();

        let ctx = library_test_ctx(vault_dir.path(), bundle_store);
        bundle_markdown_for_clipboard(&ctx, "b-1").expect("must succeed");

        let after = std::fs::read(&markdown_file).unwrap();
        assert_eq!(
            before, after,
            "Copy Markdown must not write to the stored Markdown file at all"
        );
    }

    /// `AD-11`/`BR-11`: a sealed Bundle (its Findings deleted) works exactly the same as an
    /// unsealed one, because nothing here ever reads a Finding - it stands entirely on the Bundle's
    /// own stored `markdown`/`markdown_path`. Proven by never creating the Finding at all; the
    /// `finding_store` in `library_test_ctx` is empty for both this test and the one above.
    #[test]
    fn bundle_markdown_for_clipboard_works_the_same_for_a_sealed_bundle() {
        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let bundle_store = SqliteBundleStore::open_in_memory().expect("a bundle store");
        let (bundle, items) = bundle_fixture("b-sealed");
        bundle_store
            .create_bundle(&bundle, &items)
            .expect("creating the Bundle must succeed");
        // No Finding is ever created for "f-b-sealed" - the Bundle is sealed from the moment it
        // exists, per `BR-122`'s "computed live, no stored flag" rule.
        let ctx = library_test_ctx(vault_dir.path(), bundle_store);

        let got = bundle_markdown_for_clipboard(&ctx, "b-sealed").expect(
            "a sealed Bundle must copy exactly like an unsealed one - nothing here reads a Finding",
        );
        assert!(got.contains("Checkout Flow Review"));
    }

    #[test]
    fn bundle_markdown_for_clipboard_refuses_when_the_bundle_is_gone() {
        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let bundle_store = SqliteBundleStore::open_in_memory().expect("a bundle store");
        let ctx = library_test_ctx(vault_dir.path(), bundle_store);

        let err = bundle_markdown_for_clipboard(&ctx, "does-not-exist")
            .expect_err("a missing Bundle must refuse");
        assert!(
            err.contains("no longer in the Library"),
            "the message must say what refused: {err}"
        );
    }

    #[test]
    fn bundle_markdown_for_clipboard_refuses_when_the_store_is_none() {
        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let ctx = AppContext {
            vault_store: VaultBlobStore::new(vault_dir.path()).expect("a vault at a temp path"),
            vault_path: vault_dir.path().to_path_buf(),
            finding_store: Arc::new(
                SqliteFindingStore::open_in_memory().expect("a findings store"),
            ),
            settings_store: Arc::new(
                SqliteSettingsStore::open_in_memory().expect("a settings store"),
            ),
            bundle_store: None,
        };

        let err = bundle_markdown_for_clipboard(&ctx, "b-1").expect_err("a None store must refuse");
        assert!(
            err.contains("could not be opened"),
            "the message must name what refused: {err}"
        );
    }

    /// The Bundle's OWN folder - `AD-4`'s layout, `bundles/{id}/` under the Vault root - derived
    /// from `markdown_path`'s own parent rather than a second `"bundles".join(id)` construction.
    #[test]
    fn bundle_folder_path_is_the_markdown_files_own_folder() {
        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let (bundle, _items) = bundle_fixture("b-1");
        let ctx = library_test_ctx(
            vault_dir.path(),
            SqliteBundleStore::open_in_memory().expect("a bundle store"),
        );

        let folder = bundle_folder_path(&ctx, &bundle);
        assert_eq!(folder, vault_dir.path().join("bundles").join("b-1"));
    }

    /// The degenerate case: a Bundle whose document sits at the Vault root has no folder of its
    /// own to speak of, so the Vault root itself is what "its folder" means - the same fallback
    /// `MarkdownSerializer::image_reference` uses for a document with no folder prefix.
    #[test]
    fn bundle_folder_path_falls_back_to_the_vault_root_when_markdown_path_has_no_folder() {
        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let mut bundle = bundle_fixture("b-1").0;
        bundle.markdown_path = "bundle.md".to_string();
        let ctx = library_test_ctx(
            vault_dir.path(),
            SqliteBundleStore::open_in_memory().expect("a bundle store"),
        );

        let folder = bundle_folder_path(&ctx, &bundle);
        assert_eq!(folder, vault_dir.path().to_path_buf());
    }

    /// Open file location's other half - a folder that no longer exists must refuse rather than
    /// silently do nothing, which is what lets `on_library_bundle_open_file_location_clicked` toast
    /// instead of leaving the Reviewer looking at nothing happening. Only the "not a directory"
    /// branch is exercised here: `open_folder` would spawn `explorer.exe` for a path that DOES
    /// exist, which a background test run must never do.
    #[cfg(windows)]
    #[test]
    fn open_folder_refuses_when_the_bundles_folder_no_longer_exists() {
        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let (bundle, _items) = bundle_fixture("b-gone");
        let ctx = library_test_ctx(
            vault_dir.path(),
            SqliteBundleStore::open_in_memory().expect("a bundle store"),
        );
        let folder = bundle_folder_path(&ctx, &bundle);
        assert!(!folder.exists(), "the fixture must not create the folder");

        let err = open_folder(&folder).expect_err("a missing folder must refuse, not do nothing");
        assert_eq!(err, "That folder no longer exists.");
    }

    // ===== ticket 16: disassemble a Bundle, or delete a sealed one ==========================

    fn finding_fixture(finding_store: &SqliteFindingStore, id: &str) {
        let finding = Finding {
            id: id.to_string(),
            image_path: format!("findings/{id}.png"),
            image_width: 40,
            image_height: 30,
            captured_at: "2026-09-01T10:00:00Z".to_string(),
            source_monitor: "\\\\.\\DISPLAY1".to_string(),
            region: "0,0,40,30".to_string(),
            resolved_long_edge: None,
            resolved_encoder_quality: None,
            budget_name: None,
        };
        let note = Note {
            id: format!("note-{id}"),
            finding_id: id.to_string(),
            body: String::new(),
            updated_at: "2026-09-01T10:00:00Z".to_string(),
        };
        finding_store
            .create_finding(&finding, &note, &[])
            .expect("creating the fixture Finding must succeed");
    }

    /// Same shape as ticket 12's `bundle_fixture` above, but writes straight into a bundle store
    /// instead of returning an in-memory pair - named `store_bundle_fixture` rather than reusing
    /// `bundle_fixture` because the two signatures collided when the tickets merged and this one is
    /// the store-writing side of that collision.
    fn store_bundle_fixture(
        bundle_store: &SqliteBundleStore,
        bundle_id: &str,
        finding_ids: &[&str],
    ) {
        let bundle = Bundle::new(
            bundle_id.to_string(),
            "Fixture Bundle".to_string(),
            "# Fixture Bundle".to_string(),
            format!("bundles/{bundle_id}/bundle.md"),
            "2026-09-01T12:00:00Z".to_string(),
        )
        .expect("a valid fixture Bundle");
        let items: Vec<BundleItem> = finding_ids
            .iter()
            .enumerate()
            .map(|(i, fid)| {
                BundleItem::new(
                    format!("bi-{bundle_id}-{i}"),
                    bundle_id.to_string(),
                    fid.to_string(),
                    (i + 1) as u32,
                    format!("bundles/{bundle_id}/finding_{}_burned.png", i + 1),
                )
                .expect("a valid fixture BundleItem")
            })
            .collect();
        bundle_store
            .create_bundle(&bundle, &items)
            .expect("creating the fixture Bundle must succeed");
    }

    /// `BR-122`: which verb the menu offers must be read LIVE, never from a stored flag - proven
    /// here by asking the SAME question twice, with a Finding deleted in between, exactly the shape
    /// the ticket's own acceptance criterion describes ("a fixture where a Finding is deleted between
    /// two menu openings"). No `sealed` column exists anywhere in the schema to cache the answer in,
    /// so the only way this can pass is if the check re-reads the Finding store every time.
    #[test]
    fn bundle_is_sealed_reads_live_never_a_cached_answer() {
        let bundle_store = SqliteBundleStore::open_in_memory().expect("a bundle store");
        let finding_store = SqliteFindingStore::open_in_memory().expect("a findings store");
        finding_fixture(&finding_store, "f1");
        finding_fixture(&finding_store, "f2");
        store_bundle_fixture(&bundle_store, "b1", &["f1", "f2"]);

        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let ctx = AppContext {
            vault_store: VaultBlobStore::new(vault_dir.path()).expect("a vault at a temp path"),
            vault_path: vault_dir.path().to_path_buf(),
            finding_store: Arc::new(finding_store),
            settings_store: Arc::new(
                SqliteSettingsStore::open_in_memory().expect("a settings store"),
            ),
            bundle_store: Some(Arc::new(bundle_store)),
        };

        // Menu opening #1: every Finding still exists.
        let detail = ctx
            .bundle_store
            .as_ref()
            .unwrap()
            .get_bundle("b1")
            .unwrap()
            .unwrap();
        assert!(
            !bundle_is_sealed(&ctx, &detail),
            "a Bundle whose Findings all exist must read as unsealed"
        );

        // A Finding goes, exactly as it would through the ordinary Delete Finding flow.
        ctx.finding_store
            .delete_finding("f1")
            .expect("deleting the fixture Finding must succeed");

        // Menu opening #2: the SAME Bundle, re-fetched, exactly as a second `get_bundle` call would
        // be made on a second menu open. If the answer were cached anywhere it would still read
        // unsealed here.
        let detail_again = ctx
            .bundle_store
            .as_ref()
            .unwrap()
            .get_bundle("b1")
            .unwrap()
            .unwrap();
        assert!(
            bundle_is_sealed(&ctx, &detail_again),
            "the very next read must see the Finding is gone and flip to sealed"
        );
    }

    /// The clean path: both the row and the folder go, and the folder is genuinely removed from
    /// disk, not merely reported gone.
    #[test]
    fn remove_bundle_row_and_folder_removes_both_cleanly() {
        let bundle_store = SqliteBundleStore::open_in_memory().expect("a bundle store");
        store_bundle_fixture(&bundle_store, "b-clean", &["f1"]);

        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let vault_store = VaultBlobStore::new(vault_dir.path()).expect("a vault at a temp path");
        vault_store
            .write_blob("bundles/b-clean/bundle.md", b"# Fixture Bundle")
            .expect("writing the fixture Markdown must succeed");
        vault_store
            .write_blob("bundles/b-clean/finding_1_burned.png", &png_fixture(4, 4))
            .expect("writing the fixture image must succeed");

        let ctx = AppContext {
            vault_store,
            vault_path: vault_dir.path().to_path_buf(),
            finding_store: Arc::new(
                SqliteFindingStore::open_in_memory().expect("a findings store"),
            ),
            settings_store: Arc::new(
                SqliteSettingsStore::open_in_memory().expect("a settings store"),
            ),
            bundle_store: Some(Arc::new(bundle_store)),
        };

        let orphaned = remove_bundle_row_and_folder(&ctx, "b-clean")
            .expect("a healthy row and a healthy folder must both go without error");
        assert!(!orphaned, "nothing should be left behind on the clean path");

        assert!(
            ctx.bundle_store
                .as_ref()
                .unwrap()
                .get_bundle("b-clean")
                .unwrap()
                .is_none(),
            "the row must be gone"
        );
        assert!(
            !vault_dir.path().join("bundles/b-clean").exists(),
            "the whole folder must be gone, not merely emptied"
        );
    }

    /// A row-delete failure must leave EVERYTHING intact - the row (trivially, since it never
    /// left) and the files, and the message must name what refused. Built on a connection that
    /// was never migrated, the same trick `build_library_rows_refuses_out_loud_when_list_bundles_itself_fails`
    /// already uses, so `delete_bundle` fails for real ("no such table: bundle"), not on a faked
    /// `CoreError`.
    #[test]
    fn a_row_delete_failure_leaves_the_row_and_the_folder_both_untouched() {
        let unmigrated = rusqlite::Connection::open_in_memory().expect("an in-memory connection");
        let broken_bundle_store = SqliteBundleStore::new(Arc::new(Mutex::new(unmigrated)));

        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let vault_store = VaultBlobStore::new(vault_dir.path()).expect("a vault at a temp path");
        let md_rel = "bundles/b-broken/bundle.md";
        vault_store
            .write_blob(md_rel, b"# Untouched")
            .expect("writing the fixture Markdown must succeed");

        let ctx = AppContext {
            vault_store,
            vault_path: vault_dir.path().to_path_buf(),
            finding_store: Arc::new(
                SqliteFindingStore::open_in_memory().expect("a findings store"),
            ),
            settings_store: Arc::new(
                SqliteSettingsStore::open_in_memory().expect("a settings store"),
            ),
            bundle_store: Some(Arc::new(broken_bundle_store)),
        };

        let err = remove_bundle_row_and_folder(&ctx, "b-broken")
            .expect_err("a database whose `bundle` table does not exist must refuse out loud");
        assert!(
            err.contains("Could not delete the Bundle"),
            "the message must name what refused: {err}"
        );
        assert!(
            vault_dir.path().join(md_rel).exists(),
            "a row-delete failure must leave the files exactly as it found them"
        );
    }

    /// `AD-2`'s ordering, proven under a REAL injected failure rather than a mock: the row goes
    /// first, and when the folder removal that follows fails, the row stays gone and the files stay
    /// put - never the reverse. Windows refuses to delete a file another handle holds open without
    /// `FILE_SHARE_DELETE` - the same mechanism `AGENTS.md`'s own "a leftover Snapdown.exe process
    /// locks its own file" pitfall describes - so holding the Bundle's own Markdown file open with
    /// an EXPLICIT share mode of zero (no read, no write, no delete for anyone else) is a real,
    /// deterministic fault injector, not a fake one.
    ///
    /// This is `share_mode(0)` via `OpenOptionsExt`, not a plain `File::open`, and that distinction
    /// is exactly what the first version of this test got wrong: current Rust's `std::fs::File::open`
    /// on Windows already requests `FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE` by
    /// default (for POSIX rename/delete-of-open-file parity), so a plain `File::open` held here did
    /// NOT block `remove_dir_all` at all - the first run of this test failed outright, the folder
    /// went, and `result.unwrap()` (the orphan flag) came back `false` where the test expected `true`.
    /// The explicit `share_mode(0)` below is what actually withholds `FILE_SHARE_DELETE`.
    ///
    /// **Seen red first, by hand** (not automated - the acceptance criterion asks for the order to
    /// be swapped and watched fail): with the two calls inside `remove_bundle_row_and_folder`
    /// swapped so the folder removal runs BEFORE `store.delete_bundle`, this test fails, because the
    /// same held-open file that blocks the folder removal now runs before the row is ever touched -
    /// the function returns `Err` without deleting the row at all, so
    /// `get_bundle("b-lock").unwrap().is_none()` fails: the row is still present. Restoring the
    /// correct order (row first, as written below) makes it pass again.
    #[test]
    fn folder_removal_failing_after_the_row_is_gone_never_leaves_the_row_present_with_the_files_gone(
    ) {
        let bundle_store = SqliteBundleStore::open_in_memory().expect("a bundle store");
        store_bundle_fixture(&bundle_store, "b-lock", &["f1"]);

        let vault_dir = tempfile::tempdir().expect("a temp dir");
        let vault_store = VaultBlobStore::new(vault_dir.path()).expect("a vault at a temp path");
        let md_rel = "bundles/b-lock/bundle.md";
        vault_store
            .write_blob(md_rel, b"# Locked Bundle")
            .expect("writing the fixture Markdown must succeed");
        vault_store
            .write_blob("bundles/b-lock/finding_1_burned.png", &png_fixture(4, 4))
            .expect("writing the fixture image must succeed");

        let ctx = AppContext {
            vault_store,
            vault_path: vault_dir.path().to_path_buf(),
            finding_store: Arc::new(
                SqliteFindingStore::open_in_memory().expect("a findings store"),
            ),
            settings_store: Arc::new(
                SqliteSettingsStore::open_in_memory().expect("a settings store"),
            ),
            bundle_store: Some(Arc::new(bundle_store)),
        };

        let md_path = vault_dir.path().join(md_rel);
        // `share_mode(0)`: no `FILE_SHARE_*` flag at all, so no other handle - including the one
        // `remove_dir_all` needs to delete this file - can even be opened while this one lives.
        use std::os::windows::fs::OpenOptionsExt;
        let held_open = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&md_path)
            .expect("to open the file exclusively, holding it locked");

        let result = remove_bundle_row_and_folder(&ctx, "b-lock");

        assert!(
            result.is_ok(),
            "a folder-removal failure after a successful row delete must not bubble up as an \
             overall Err - the Reviewer asked for the Bundle gone, and it is: {result:?}"
        );
        assert!(
            result.unwrap(),
            "the folder failed to go, so this must be reported as an orphan"
        );

        assert!(
            ctx.bundle_store
                .as_ref()
                .unwrap()
                .get_bundle("b-lock")
                .unwrap()
                .is_none(),
            "the row must be gone even though the folder removal failed"
        );
        assert!(
            md_path.exists(),
            "the file must still be on disk - AD-2 forbids the reverse state (row gone, files \
             gone too, or worse, row present with files gone), and a file surviving here is what \
             proves the row went FIRST"
        );

        drop(held_open);
    }
}
