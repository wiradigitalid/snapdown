#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod hotkey;
mod startup;
mod tray;

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use slint::{ComponentHandle, Model, ModelRc, SharedPixelBuffer, SharedString, VecModel};
use snapdown_capture::{CaptureTarget, RegionCapturer};
use snapdown_core::domain::bundle::{Bundle, BundleItem};
use snapdown_core::domain::finding::{
    AnnotationShape, Finding, FindingDetail, Note, Region, VisualAnnotation,
};
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::domain::markdown::MarkdownSerializer;
use snapdown_core::domain::setting::{
    HotkeyAction, QualityBudget, Setting, SettingKey, SettingValue,
};
use snapdown_core::error::CoreError;
use snapdown_core::ports::{
    BlobStore, BundleStore, Clock, EntropySource, FindingStore, SettingsStore,
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
    let burned =
        MarkerBurner::burn_all(&source, &dims, &detail.markers, &detail.visual_annotations)
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

    // `set_clipboard` opens, empties, writes and closes. Holding a `Clipboard` guard around it
    // would take the lock twice and deadlock.
    clipboard_win::set_clipboard(clipboard_win::formats::Bitmap, &bmp)
        .map_err(|e| format!("Could not write to the clipboard: {e}"))?;

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

/// The settings key the theme choice is stored under.
///
/// A function rather than two string literals at the two call sites: a typo in one of them would
/// store the choice and never read it back, and nothing would fail.
fn theme_setting_key() -> SettingKey {
    SettingKey::Custom("theme_dark".to_string())
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

    Ok(PlannedBundle {
        markdown: MarkdownSerializer::serialize_bundle(name, notes, &md_items),
        markdown_path: format!("bundles/{bundle_id}/bundle.md"),
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
    pending.planned.markdown =
        MarkdownSerializer::serialize_bundle(&pending.name, &pending.notes, &pairs);
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
                                    .map(|_| *action)
                            })
                    });

                    if let Some(action) = action {
                        match action {
                            HotkeyAction::Capture => {
                                if let Some(win) = window_for_events.upgrade() {
                                    // The setting decides, and `BG-6` is why there is a setting:
                                    // the hotkey exists to keep the Reviewer in the flow they are
                                    // already in. Default off.
                                    REVEAL_EDITOR_AFTER_CAPTURE
                                        .set(open_editor_after_capture(&ctx_hotkey));
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
        assert!(
            planned.markdown.contains("bundles/b7/finding_1_burned.png"),
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
}
