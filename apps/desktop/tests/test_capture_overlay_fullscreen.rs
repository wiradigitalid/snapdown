use std::fs;
use std::path::Path;

fn main_rs() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// The whole of `main.rs` with `//` and `///` comment lines stripped.
///
/// Same reason as `capture_block_code`, one scope wider: a guard that matches a token to find out
/// WHERE something happens must not be satisfied by prose that merely names it. This is not
/// hypothetical either - a doc comment on `LiveOverlay::snapshot` that referred to
/// `on_capture_clicked` made the start-up guard below find the handler hundreds of lines before
/// the real one, and fail on correct code.
fn main_rs_code() -> String {
    main_rs()
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join(
            "
",
        )
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
        !main_rs_code().contains("with_renderer_name(\"software\")"),
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

/// A region MAY span monitors, as Snagit's does - the owner asked for that explicitly after
/// finding Snagit allows it, with black filling any gap. What it must not do is escape the canvas,
/// because the crop indexes the snapshot directly and an out-of-range region would either panic or
/// be silently clamped into the wrong pixels.
#[test]
fn a_drag_is_bounded_by_the_canvas_not_by_a_monitor() {
    let overlay = overlay_slint_block();

    assert!(
        overlay.contains("clamp(self.mouse-x, 0px, root.width)")
            && overlay.contains("clamp(self.mouse-y, 0px, root.height)"),
        "the drag must be clamped to the canvas - the whole desktop - so a region can span \
         monitors but never index outside the snapshot (BUG-26)"
    );
    assert!(
        !overlay.contains("drag-monitor"),
        "the drag must no longer be pinned to the monitor it started on: a region spanning two \
         monitors is now allowed, matching Snagit (BUG-26)"
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
        overlay.contains("root.crosshair-h") && overlay.contains("root.crosshair-w"),
        "the crosshair lines must span the active monitor's bounds, not the whole desktop (BUG-26)"
    );
}

/// BUG-27: the overlay must be created at start-up, not on first Capture.
///
/// A window's renderer surface and its geometry correction both happen only at creation, and
/// `show()` does not create the native window - the event loop does, on its next turn. Doing that
/// on first Capture is what the user saw as the overlay growing into place on the non-primary
/// monitor, and it is the remaining share of the blink.
#[test]
fn the_overlay_is_created_at_startup_not_on_first_capture() {
    let source = main_rs_code();

    assert!(
        source.contains("fn prewarm_capture_overlay"),
        "the overlay must be built at start-up so first Capture has nothing left to create \
         (BUG-27)"
    );
    assert!(
        source.contains("virtual_desktop_bounds"),
        "pre-warming must size the overlay from the desktop bounds, which needs no pixel grab - \
         capturing the screen at start-up would be both slow and wrong (BUG-27)"
    );

    let prewarm_call = source
        .find("prewarm_capture_overlay();")
        .expect("prewarm_capture_overlay must be called");
    let capture_cb = source
        .find("on_capture_clicked")
        .expect("on_capture_clicked must exist");
    assert!(
        prewarm_call < capture_cb,
        "pre-warming must happen during start-up, before the Capture handler is even installed \
         (BUG-27)"
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

/// BUG-28: the desktop's pixels must go straight into the buffer that is presented.
///
/// The overlay used to appear ~225ms after Capture on a 6000x3840 two-monitor desktop, and only
/// about half of that was the grab. The rest was the app going over 92 MB three times: allocate a
/// canvas, `image::imageops::overlay` each monitor into it - a per-pixel `blend`, not a blit, whose
/// answer for an opaque source over an untouched destination is the source pixel - and then
/// `clone_from_slice` the whole thing into the toolkit's own buffer. Measured in release: 83-91ms
/// for that shape, against 36-38ms once the monitors were blitted straight into the presented
/// buffer, and 4.3-4.6ms once that buffer stopped being reallocated (see the next test).
#[test]
fn the_snapshot_is_written_into_the_buffer_that_is_shown() {
    let block = capture_block_code();

    assert!(
        block.contains("blit_into"),
        "the canvas must be written straight into the buffer that will be shown          (VirtualDesktopCapture::blit_into), not stitched into an image of its own first (BUG-28)"
    );
    assert!(
        !block.contains("clone_from_slice"),
        "the canvas must not be copied into the presentation buffer - that copy is 92 MB on a          6000x3840 desktop (BUG-28)"
    );
    assert!(
        !block.contains("imageops::overlay"),
        "the monitors must not be alpha-blended into a canvas: every source pixel is opaque and          the destination is untouched, so the blend's result is a copy that costs 75ms (BUG-28)"
    );
}

/// BUG-28: the canvas buffers are allocated once per desktop layout and written alternately, never
/// reallocated per capture.
///
/// Allocating is the expensive part, and not for the reason anyone expects:
/// `SharedPixelBuffer::new` fills through `SharedVector`'s `FromIterator`, a per-element push loop
/// rather than a `calloc`, so a 6000x3840 buffer costs 33-37ms while a plain 92 MB write pass costs
/// 3.6ms. Moving that to start-up is the same move that put the renderer warm-up where nobody is
/// looking.
///
/// **Two canvases, not one, and that is the whole point of this test.** `make_mut_bytes` is
/// copy-on-write, so the canvas being written must be held by nothing. The first attempt cleared
/// the overlay's `snapshot-image` property immediately before the blit and assumed that released
/// it. It does not: Slint bindings are lazy, the backdrop and the loupe latch their `source` when
/// they render, and a hidden window never renders - so between captures they still hold the last
/// image shown. That shipped past a unit test and was caught only by the canary, which reported a
/// copy on all 28 captures of a session.
///
/// Alternating needs no release step and no ordering: whatever those elements still hold is the
/// previous capture's canvas, so the other one is free by construction.
#[test]
fn the_capture_canvas_is_reused_and_alternated_not_reallocated() {
    let source = main_rs_code();
    let block = capture_block_code();

    let allocation = source
        .find("SharedPixelBuffer::<slint::Rgba8Pixel>::new")
        .expect(
        "the canvas buffers must be allocated somewhere - reusing them is what keeps a capture \
         off the 33-37ms `SharedPixelBuffer::new` fill (BUG-28)",
    );
    let capture_handler = source
        .find("on_capture_clicked")
        .expect("on_capture_clicked must exist");
    assert!(
        allocation < capture_handler,
        "the canvas buffers must be allocated at start-up, before the Capture handler is even \
         installed - `SharedPixelBuffer::new` is a 33-37ms per-element fill, not a calloc, and \
         paying it on Capture is exactly the latency BUG-28 is about"
    );

    assert!(
        block.contains("entry.snapshots[target].make_mut_bytes()"),
        "the capture must write into one of the overlay's own retained canvases, not a fresh one \
         (BUG-28)"
    );

    // The alternation itself. Without it, the canvas being written is the one the backdrop and the
    // loupe are still holding, and every write is a silent 92 MB copy-on-write.
    assert!(
        block.contains("let target = 1 - entry.current;"),
        "the capture must write the canvas that is NOT on screen. Writing the one just shown means \
         writing what the overlay's own bindings still hold, which copies 92 MB instead (BUG-28)"
    );
    assert!(
        block.contains("entry.current = target;"),
        "the capture must record which canvas is now on screen, or the next capture alternates \
         back onto the one it just wrote and a Finding crops from the wrong one (BUG-28)"
    );

    // A copy-on-write moves the allocation, so the pointer is the evidence. This is the check that
    // caught the single-buffer design after a unit test had passed it.
    assert!(
        block.contains("std::ptr::eq"),
        "the capture must check that the canvas was written in place rather than copied - the \
         failure produces perfectly correct pixels and shows up only as latency (BUG-28)"
    );

    // The completed handler must READ the canvas from the live overlay. A clone captured in that
    // closure would outlive the capture and still be held two captures later.
    assert!(
        block.contains("LIVE_OVERLAYS.with_borrow(")
            && block.contains("entry.snapshots[entry.current].as_bytes()"),
        "the completed handler must read the on-screen canvas out of LIVE_OVERLAYS rather than \
         capturing a clone of it, and it must read the one that is actually on screen (BUG-28)"
    );
}
