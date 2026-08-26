use std::fs;
use std::path::Path;

fn main_rs() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

fn capture_block() -> String {
    let source = main_rs();
    let start = source
        .find("on_capture_clicked")
        .expect("on_capture_clicked not found in main.rs");
    source[start..].to_string()
}

/// The capture block with `//` line comments stripped.
///
/// Needed because these are text-matching guards: without this, a comment that *names* the
/// mistake it is warning about counts as the mistake. That is not hypothetical - the guard below
/// fired on its own explanation of the scale_factor bug.
fn capture_block_code() -> String {
    capture_block()
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
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

/// BUG-26: the overlay must cover the whole virtual desktop from a canvas captured at native
/// physical resolution.
///
/// The first attempt at this was a single window sized to a stitched canvas, and it rendered as
/// one giant zoomed image - but the cause was the sizing (physical numbers stored as logical and
/// then multiplied by the scale factor), not a limit on what one window can represent. A window's
/// surface maps 1:1 onto desktop pixels for a per-monitor-DPI-aware process.
///
/// The intervening design - one window per monitor - is what BUG-27 is about: it fixed the
/// rendering but bought per-window renderer warm-up (a blink on every capture) and per-window
/// event-loop turns (a cancel that cleared one screen at a time).
#[test]
fn the_overlay_is_a_single_window_over_the_whole_virtual_desktop() {
    let block = capture_block();

    assert!(
        block.contains("capture_virtual_desktop"),
        "capture must grab the whole virtual desktop as one canvas \
         (RegionCapturer::capture_virtual_desktop) so a single overlay can present it (BUG-26)"
    );
    assert!(
        block.contains("captured.origin_x") && block.contains("captured.origin_y"),
        "the overlay must be placed at the virtual desktop's own origin, so monitors left of or \
         above the primary one are covered too (BUG-26)"
    );
    assert!(
        !block.contains("for placement in &wanted"),
        "the overlay must no longer be built one window per monitor - that is the multi-window \
         design BUG-27 records the cost of (BUG-27)"
    );
}

/// BUG-27: the overlay is created once and reused. A GPU renderer builds a window's surface and
/// pipeline lazily and presents that first frame with only the clear colour, which is the
/// whole-screen black blink; on Windows `hide()` leaves the native window and its renderer alive,
/// so reuse pays that warm-up once per desktop layout rather than once per capture.
///
/// Falling back to the software renderer is NOT the fix - that removes the blink but cannot
/// repaint a full-screen overlay per pointer move, making region dragging choppy.
#[test]
fn the_capture_overlay_is_reused_across_captures_not_rebuilt() {
    let block = capture_block();

    assert!(
        block.contains("layout_unchanged"),
        "capture must decide whether the existing overlay still matches the desktop layout and \
         reuse it when it does, instead of recreating the window per capture (BUG-27)"
    );
    assert!(
        block.contains("LIVE_OVERLAYS.with_borrow_mut(std::mem::take)"),
        "capture must take the existing overlay out of LIVE_OVERLAYS to reconfigure and reuse it; \
         clearing unconditionally would rebuild the window each capture and bring the renderer \
         warm-up blink back (BUG-27)"
    );
    assert!(
        block.contains("on_capture_completed"),
        "handlers must be (re)installed on every capture so a reused overlay never crops from the \
         previous capture's canvas (BUG-27)"
    );
}

/// The renderer must be left at Slint's GPU default. See the reuse test for why software is the
/// wrong answer to the blink.
#[test]
fn the_software_renderer_is_not_forced() {
    assert!(
        !main_rs().contains("with_renderer_name(\"software\")"),
        "the software renderer must not be hard-coded: it cannot repaint a full-screen overlay \
         per pointer move, so region dragging becomes choppy (BUG-27)"
    );
}

/// BUG-26: the overlay's geometry must come from window CREATION, and must never be re-derived
/// from `window().scale_factor()`.
///
/// `scale_factor()` returns 1.0 until the window has actually been realised. Sizing from it meant
/// the first capture of every session asked for a 6000x3840 *logical* window, which the real 1.5
/// scale factor turned into 9000x5760 - the overlay came up zoomed on the first capture and was
/// correct from the second onwards, once the window existed and reported its true scale. There is
/// no safe pre-creation alternative: `set_size(Physical)` divides by that same unknown factor.
#[test]
fn the_overlay_geometry_comes_from_creation_not_a_scale_factor_guess() {
    let block = capture_block_code();

    // Specifically the OVERLAY's show, not the main window's - `main.show()` appears earlier in
    // this block, inside the close handler, and matching it made this guard fail on correct code.
    let show = block
        .find("overlay.show()")
        .expect("the overlay must be shown via overlay.show()");
    let placement = block.find("NEXT_OVERLAY_PLACEMENT").expect(
        "the placement must be handed to window creation via NEXT_OVERLAY_PLACEMENT, so the \
         window is born covering the desktop instead of being moved there afterwards (BUG-26)",
    );
    let new_call = block
        .find("CaptureOverlayWindow::new()")
        .expect("the overlay must be constructed");

    assert!(
        placement < show,
        "the placement must be published BEFORE show(), which is when the native window is \
         actually created (BUG-26)"
    );
    // The native window is created by show(), NOT by new(). Publishing the placement around
    // new() and clearing it again leaves nothing for the hook to read. This exact mistake shipped
    // once and mis-sized the overlay on every capture.
    assert!(
        placement > new_call,
        "the placement must be published AFTER CaptureOverlayWindow::new() and immediately before \
         show() - new() does not create the native window, so a placement set around it is \
         already gone by the time the attributes hook runs (BUG-26)"
    );

    // The authoritative sizing is winit's, in device pixels. It takes no scale factor, so it
    // cannot be wrong before the window has been realised.
    assert!(
        block.contains("request_inner_size") && block.contains("set_outer_position"),
        "overlay geometry must be applied through winit in physical pixels \
         (request_inner_size / set_outer_position); every Slint-level alternative needs a scale \
         factor that is not yet known when the window is first shown (BUG-26)"
    );

    // Anything derived from window().scale_factor() must stay behind cfg(not(windows)). That call
    // reports 1.0 until the window is realised, so on Windows it sized the first overlay of every
    // session at scale 1.0 and the real 1.5 factor then inflated it.
    let non_windows = block.find("#[cfg(not(windows))]");
    if let Some(at) = block.find("scale_factor()") {
        assert!(
            non_windows.is_some_and(|fallback| fallback < at),
            "`scale_factor()` appears in the Windows path. It reports 1.0 before the window is \
             realised, which is what made the first capture of every session come up zoomed \
             (BUG-26)"
        );
    }
}

/// `full-screen` fullscreens on exactly one monitor, so it cannot cover a multi-monitor desktop.
#[test]
fn the_overlay_does_not_declare_slint_full_screen() {
    assert!(
        !overlay_slint_block().contains("full-screen: true"),
        "CaptureOverlayWindow must not declare `full-screen: true` - it covers a single monitor \
         and fights the explicit placement done in main.rs (BUG-26)"
    );
}

/// One window now spans every monitor, so nothing about the geometry stops a drag from running
/// across a screen boundary. The monitor the drag STARTED on is pinned and every clamp uses it,
/// which is what keeps a capture from combining two monitors.
#[test]
fn a_selection_is_clamped_to_the_monitor_the_drag_started_on() {
    let overlay = overlay_slint_block();

    assert!(
        overlay.contains("drag-monitor"),
        "the overlay must pin the monitor a drag starts on; with one window spanning every \
         display, geometry alone no longer prevents a region spanning two monitors (BUG-26)"
    );
    assert!(
        overlay.contains("root.drag-monitor = root.active-monitor()"),
        "the pinned monitor must be captured on pointer-down, not recomputed as the pointer moves \
         - otherwise dragging onto the next screen would extend the region there (BUG-26)"
    );
    assert!(
        overlay.contains("clamp("),
        "the drag must be clamped to the pinned monitor's bounds (BUG-26)"
    );
}

/// The crosshair must follow only the monitor under the pointer, which is what the owner asked
/// for. With one window it takes real arithmetic against the monitor rectangles.
#[test]
fn the_crosshair_is_confined_to_the_monitor_under_the_pointer() {
    let overlay = overlay_slint_block();

    assert!(
        overlay.contains("active-monitor()") && overlay.contains("monitor-holds-pointer"),
        "the overlay must resolve which monitor the pointer is over from the monitor rectangles \
         (BUG-26)"
    );
    assert!(
        overlay.contains("root.clamp-h") && overlay.contains("root.clamp-w"),
        "the crosshair lines must span the active monitor's bounds, not the whole desktop (BUG-26)"
    );
}

/// The selection is reported in canvas pixels so the caller can crop the snapshot directly. If it
/// were reported in logical pixels the crop would be wrong by the scale factor on a HiDPI display.
#[test]
fn the_selection_is_reported_in_canvas_pixels() {
    let overlay = overlay_slint_block();
    assert!(
        overlay.contains("root.source-scale"),
        "the overlay must convert the selection to snapshot pixels before emitting it (BUG-26)"
    );
    assert!(
        !overlay.contains("capture-completed(\n                                sel-x / 1px"),
        "the selection must not be emitted in raw logical pixels - on a 150%/175% display that \
         crops the wrong region (BUG-26)"
    );
}
