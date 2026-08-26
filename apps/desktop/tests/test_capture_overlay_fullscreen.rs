use std::fs;
use std::path::Path;

fn main_rs() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

fn overlay_slint_block() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/appwindow.slint");
    let source =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"));
    let start = source
        .find("component CaptureOverlayWindow")
        .expect("CaptureOverlayWindow component not found in appwindow.slint");
    let end = source[start..]
        .find("\nexport component AppWindow")
        .map(|rel| start + rel)
        .unwrap_or(source.len());
    source[start..end].to_string()
}

/// BUG-24: the capture overlay opened at Slint's default window size instead of covering the
/// captured monitor, so the selection area appeared tiny and off-screen coordinates were fed
/// into the crop math.
///
/// BUG-26: the first fix (one window sized to the whole stitched virtual desktop) is wrong on a
/// mixed-DPI rig. A Win32 window has exactly one DPI, so one window spanning a 150% and a 175%
/// display renders 1:1 on neither, and crossing the boundary fires `WM_DPICHANGED`, which
/// preserves the logical size and re-derives a *larger* physical one - the window inflates past
/// the desktop and the capture reads as one giant zoomed image. The fix is one overlay window
/// per monitor. Flameshot reached the same conclusion for the same reason (flameshot-org
/// /flameshot#4495, PR #4498).
#[test]
fn one_capture_overlay_is_opened_per_monitor() {
    let source = main_rs();
    let start = source
        .find("on_capture_clicked")
        .expect("on_capture_clicked not found in main.rs");
    let capture_block = &source[start..];

    assert!(
        capture_block.contains("capture_each_monitor"),
        "capture must grab each monitor separately (RegionCapturer::capture_each_monitor) rather \
         than stitching one virtual-desktop canvas: a single spanning overlay cannot render 1:1 \
         on displays with different scale factors (BUG-26)"
    );
    assert!(
        capture_block.contains("for placement in &wanted"),
        "capture must build one overlay window per captured monitor (BUG-26)"
    );
}

/// BUG-27: the overlays must be created once and reused, not rebuilt per capture. A GPU renderer
/// builds each window's surface and pipeline lazily and presents that first frame with only the
/// clear colour, which is the whole-screen black blink; on Windows `hide()` leaves the native
/// window and its renderer alive, so reusing the windows pays that warm-up once per monitor
/// layout instead of once per capture. Falling back to the software renderer is NOT the fix -
/// that removes the blink but cannot repaint a full-screen overlay per pointer move, making the
/// drag choppy.
#[test]
fn capture_overlays_are_reused_across_captures_not_rebuilt() {
    let source = main_rs();
    let start = source
        .find("on_capture_clicked")
        .expect("on_capture_clicked not found in main.rs");
    let capture_block = &source[start..];

    assert!(
        capture_block.contains("layout_unchanged"),
        "capture must decide whether the existing overlays still match the monitor layout and \
         reuse them when they do, instead of recreating a window per capture (BUG-27)"
    );
    assert!(
        capture_block.contains("LIVE_OVERLAYS.with_borrow_mut(std::mem::take)"),
        "capture must take the existing overlays out of LIVE_OVERLAYS to reconfigure and reuse \
         them; clearing them unconditionally would rebuild every window each capture and bring \
         the renderer warm-up blink back (BUG-27)"
    );

    // Reused overlays keep the previous capture's handlers, which close over the PREVIOUS
    // screenshot - so a region would be cropped out of a stale image.
    assert!(
        capture_block.contains("on_capture_completed"),
        "handlers must be (re)installed on every capture so a reused overlay never crops from \
         the previous capture's screenshot (BUG-27)"
    );
}

/// The renderer must be left at Slint's GPU default. See the reuse test for why the software
/// renderer is the wrong answer to the blink.
#[test]
fn the_software_renderer_is_not_forced() {
    let source = main_rs();
    assert!(
        !source.contains("with_renderer_name(\"software\")"),
        "the software renderer must not be hard-coded: it cannot repaint a full-screen overlay \
         per pointer move, so region dragging becomes choppy. The blink it was hiding is solved \
         by reusing the overlay windows instead (BUG-27)"
    );
}

/// Each overlay must be placed and sized to its own monitor, in physical pixels, before it is
/// shown - otherwise the user watches it reposition and resize into place.
#[test]
fn each_overlay_is_placed_and_sized_to_its_monitor_before_being_shown() {
    let source = main_rs();
    let start = source
        .find("CaptureOverlayWindow::new()")
        .expect("CaptureOverlayWindow::new() call not found in main.rs");
    let after_create = &source[start..];

    let set_position = after_create
        .find("set_position")
        .expect("the overlay must be moved to its monitor's origin via Window::set_position");
    let set_size = after_create
        .find("set_size")
        .expect("the overlay must be sized to its monitor via Window::set_size");
    let show = after_create
        .find(".show()")
        .expect("the overlay must be shown");

    assert!(
        after_create.contains("capture.origin_x") && after_create.contains("capture.origin_y"),
        "the overlay must be positioned at its own monitor's virtual-desktop origin, so a \
         monitor left of or above the primary one is covered too (BUG-26)"
    );
    assert!(
        set_position < show && set_size < show,
        "position and size must both be applied BEFORE show(), so the window is born on its \
         target monitor at that monitor's DPI - applying them afterwards is what produced the \
         visible zoom/resize transition (BUG-26)"
    );
}

/// `full-screen` fullscreens on exactly one monitor, so it would fight the per-monitor placement.
#[test]
fn the_overlay_does_not_declare_slint_full_screen() {
    assert!(
        !overlay_slint_block().contains("full-screen: true"),
        "CaptureOverlayWindow must not declare `full-screen: true` - it covers a single monitor \
         and fights the explicit per-monitor placement done in main.rs (BUG-26)"
    );
}

/// A region is confined to the overlay it was drawn in, and each overlay is one monitor - so a
/// selection combining two monitors cannot be produced. Windows keeps delivering move events to
/// the window that took the button-press even once the pointer leaves it, so the clamp is what
/// actually enforces this.
#[test]
fn a_selection_is_clamped_to_the_monitor_it_was_started_on() {
    let overlay = overlay_slint_block();
    assert!(
        overlay.contains("clamp("),
        "the drag must be clamped to this overlay's own bounds, otherwise releasing the pointer \
         over the neighbouring monitor yields a region reaching onto a screen this overlay does \
         not own - a cross-monitor capture, which is explicitly disallowed (BUG-26)"
    );
}
