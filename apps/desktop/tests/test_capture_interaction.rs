//! Guards for what the capture overlay must let the Reviewer DO, as opposed to how it is drawn or
//! sized (`test_capture_overlay_fullscreen.rs` owns that).
//!
//! All of these were reported by the owner against a build whose tests were green, which is the
//! recurring shape of defect in this repository: a thing exists, compiles, and is reachable by
//! nobody. `AGENTS.md` records four earlier instances.

use std::fs;
use std::path::Path;

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// Source with `//` comment lines stripped, so a guard that matches a token to locate BEHAVIOUR is
/// never satisfied by prose that merely names it. This file's siblings learned that the hard way.
fn code(rel: &str) -> String {
    read(rel)
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether a `.slint` source declares an element of this exact name.
///
/// Matched at the start of a trimmed line rather than as a bare substring, because `SdActionButton {`
/// and `IconButton {` both CONTAIN "Button {" - a substring check failed on this product's own
/// components the moment the shared action button was introduced.
fn declares_element(source: &str, element: &str) -> bool {
    let opener = format!("{element} {{");
    source
        .lines()
        .any(|line| line.trim_start().starts_with(&opener))
}

fn overlay_block() -> String {
    let source = code("ui/appwindow.slint");
    let start = source
        .find("component CaptureOverlayWindow")
        .expect("CaptureOverlayWindow not found");
    let end = source[start..]
        .find("\nexport component AppWindow")
        .map(|rel| start + rel)
        .unwrap_or(source.len());
    source[start..end].to_string()
}

/// The overlay's own hint text promises "Enter to save · Esc to cancel". Enter used to be handled
/// only by the note field's `accepted`, so it worked while that field had focus and did nothing
/// otherwise - which is most of the time, because dragging a region moves focus away. Escape was
/// already on the window's FocusScope; Enter has to be too.
#[test]
fn enter_saves_from_the_window_not_only_from_the_note_field() {
    let overlay = overlay_block();

    let scope = overlay
        .find("key-pressed(event)")
        .expect("the overlay must have a window-level key handler");
    let tail = &overlay[scope..];

    assert!(
        tail.contains("Key.Return"),
        "Enter must be handled by the window's FocusScope, not only by the note field - the hint \
         text promises it and the field rarely has focus when the Reviewer presses it"
    );
    assert!(
        tail.contains("Key.Escape"),
        "Escape must stay handled at the window level (BUG-25)"
    );
    // Saving on Enter with no region would emit a zero-sized crop.
    let ret = tail.find("Key.Return").expect("checked above");
    assert!(
        tail[ret..ret + 200].contains("root.has-selection"),
        "Enter must only save when there is a selection to save"
    );
}

/// One text field for the whole product, and it is ours.
///
/// `std-widgets`' `LineEdit` and `TextEdit` paint themselves from Slint's own style palette, not from
/// `theme.slint`. On a white panel the Observation Summary therefore rendered white-on-white, became
/// legible only while focused, and turned grey while being edited - three appearances for one idea,
/// none of them the product's. The capture overlay's note field had already been hand-drawn from the
/// tokens after the same discovery, so the fix was to extract that one and use it everywhere.
///
/// A field also has to LOOK like somewhere you can type before it is focused, which is what the
/// styled widget got wrong first: the owner reported not knowing it was an input until it changed
/// colour, which happens only once you have already clicked it.
#[test]
fn the_product_has_exactly_one_text_entry_component() {
    let field = code("ui/components/text-field.slint");

    for token in [
        "Theme.bg-app",
        "Theme.text-primary",
        "Theme.border-strong",
        "Theme.border-focus",
        "Theme.text-dim",
    ] {
        assert!(
            field.contains(token),
            "the shared field must take its colours from the product's tokens - `{token}` missing. \
             Painting from Slint's own palette is what made the Observation Summary invisible"
        );
    }
    assert!(
        field.contains("border-width: input.has-focus ? 2px : 1px"),
        "the fill, the border and the placeholder must all be visible BEFORE focus; focus may only \
         change the border"
    );
    assert!(
        field.contains("if root.text == \"\" : Text"),
        "an empty field must still say what it is for"
    );

    // And nowhere else in the product may use the std-widget text entries.
    let app = read("ui/appwindow.slint");
    for widget in ["LineEdit", "TextEdit"] {
        assert!(
            !declares_element(&app, widget),
            "`{widget}` must not be used anywhere: it paints from Slint's palette rather than \
             `theme.slint`, and as an imported component in a plain Rectangle it also takes no \
             geometry. Use SdTextField"
        );
    }
    assert!(
        app.contains("import { ScrollView } from \"std-widgets.slint\";"),
        "and they must not even be imported, so reaching for one is a compile error rather than a \
         review catch"
    );
}

/// The note field must take focus when the popup opens, or the note is never captured at all.
///
/// This is a regression that arrived WITH the fix above. Nothing had ever focused the field, which
/// was survivable while nothing else held the keyboard - the Reviewer clicked it and typed. Once the
/// window's FocusScope began re-claiming focus for every capture, its key handler swallowed every
/// keystroke (it rejects everything but Escape and Enter), so the field stayed empty and Enter saved
/// an empty note. FR-2 promises the Editor shows "exactly the text that was typed at capture time".
#[test]
fn the_note_field_takes_focus_when_the_popup_opens() {
    let overlay = overlay_block();

    let field = overlay
        .find("note-input := SdTextField")
        .expect("the capture note field must exist");
    let body = &overlay[field..(field + 900).min(overlay.len())];

    assert!(
        body.contains("self.focus-input()"),
        "the note field must claim focus as soon as it exists. Without it the window's FocusScope \
         keeps the keyboard, rejects every printable key, and the Finding is saved with an empty \
         note (FR-2)"
    );
    assert!(
        body.contains("placeholder: \"Describe what is wrong here"),
        "and it must say what it is for before anything is typed"
    );
}

/// FR-1 promises "1-click container selection". The detection that makes it possible has existed in
/// `snapdown-capture` since before this test and was called by NOTHING - the fourth instance in
/// this repository of a finished component nobody could reach.
#[test]
fn window_and_panel_detection_is_actually_reachable() {
    let main = code("src/main.rs");
    let overlay = overlay_block();

    assert!(
        main.contains("RegionCapturer::detect_capture_targets()"),
        "the container detection must be CALLED. `detect_element_at_point` sat in \
         snapdown-capture, complete and unit-tested, reachable from nowhere - FR-1 promises \
         1-click container selection and nothing delivered it"
    );
    assert!(
        main.contains("on_target_at"),
        "the overlay's hit test must be implemented in Rust - Slint's expression language has no \
         loop, so a list of candidates cannot be searched in the .slint"
    );
    assert!(
        overlay.contains("pure callback target-at"),
        "the overlay must declare the hit test as a pure callback, so the hovered container can be \
         used in a binding"
    );
    assert!(
        overlay.contains("root.has-hover-target"),
        "the hovered container must be shown before it is clicked, or 1-click selection is \
         invisible until it has already happened"
    );
    assert!(
        overlay.contains("root.hover-target.width > 0"),
        "a plain click must take the hovered container when there is one (FR-1)"
    );
}

/// Detection must not be waited for. It costs 180-345ms on a busy desktop - longer than the grab -
/// and BUG-28 spent a lot of measurement getting the overlay down to ~150ms.
#[test]
fn detection_does_not_block_the_overlay() {
    let main = code("src/main.rs");

    let detect = main
        .find("RegionCapturer::detect_capture_targets()")
        .expect("checked by the test above");
    let before = &main[..detect];
    let spawn = before
        .rfind("std::thread::spawn")
        .expect("detection must run on a thread of its own");
    assert!(
        !main[spawn..detect].contains("join()"),
        "the capture must not join the detection thread: it takes longer than the grab, and the \
         overlay needs the targets only by the time the pointer moves (BUG-28)"
    );
}

/// Snapdown appeared in its own screenshots, and the fix is exclusion rather than hiding.
///
/// Hiding the window was tried first. It cannot be timed honestly: `hide()` reaches `ShowWindow`
/// at once, but the desktop only stops containing those pixels once the compositor has presented a
/// frame without them AND whatever was underneath has repainted. A 60ms wait left the owner
/// reporting leftover window shadow in the overlay, and the only reliable wait is a long one -
/// which hands back the latency BUG-28 spent so much measurement removing.
///
/// `WDA_EXCLUDEFROMCAPTURE` has no timing at all: DWM composes the frame capture APIs see without
/// the window in it, shadow included, from the moment the call returns. Nothing moves on screen, so
/// there is nothing to wait for and nothing to flicker.
#[test]
fn the_editor_is_excluded_from_the_capture_rather_than_hidden() {
    let main = code("src/main.rs");

    assert!(
        main.contains("WDA_EXCLUDEFROMCAPTURE"),
        "the Editor must be excluded from screen capture, so it is not in its own screenshot"
    );

    let handler = main
        .find("on_capture_clicked")
        .expect("on_capture_clicked must exist");
    let block = &main[handler..];

    let exclude = block
        .find("set_capture_exclusion(&main_weak, true)")
        .expect("the exclusion must be applied when a capture starts");
    let grab = block
        .find("capture_virtual_desktop")
        .expect("the capture must grab the desktop");
    assert!(
        exclude < grab,
        "the exclusion must be applied BEFORE the grab, or the Editor's pixels are already in the \
         snapshot"
    );

    assert!(
        block.contains("set_capture_exclusion(&main_weak, false)"),
        "the exclusion must be lifted when the capture ends, or Snapdown stays invisible to every \
         other screen-capture tool for the rest of the session"
    );

    // Hiding is what created the hazard this design removes: a bare `return` on a failure path
    // would have left the Reviewer with no window and no way to get one.
    assert!(
        !block.contains("main.hide()"),
        "the Editor must not be hidden for a capture. Hiding cannot be timed reliably, and it makes \
         every early return capable of stranding the product with no visible window"
    );
}

/// A press must clear the previous selection's SIZE, not just its flag.
///
/// The release handler decides "drag or click" by looking at sel-w/sel-h. Leaving the previous
/// drag's size in them made a plain click look like a drag, so it re-armed the OLD region while the
/// highlight had been showing the container under the pointer - the owner saw one rectangle
/// outlined and a different one selected.
#[test]
fn a_press_clears_the_previous_selection_size() {
    let overlay = overlay_block();

    let down = overlay
        .find("PointerEventKind.down")
        .expect("the overlay must handle a press");
    let up = overlay
        .find("PointerEventKind.up")
        .expect("the overlay must handle a release");
    let press_block = &overlay[down..up];

    assert!(
        press_block.contains("root.sel-w = 0px") && press_block.contains("root.sel-h = 0px"),
        "a press must reset sel-w and sel-h. The release handler reads them to tell a click from a \
         drag, so stale values make a click select whatever was dragged last time"
    );
}

/// Focus has to be re-claimed per capture, not once at construction.
///
/// The overlay is created at start-up and reused forever, so `init` fires exactly once. Anything
/// that took focus during an earlier capture still had it on the next one, and Enter - which,
/// unlike Escape, is not rejected-and-bubbled by the note field - went nowhere. That is why Enter
/// failed intermittently rather than never.
#[test]
fn keyboard_focus_is_reclaimed_on_every_capture() {
    let overlay = overlay_block();
    let main = code("src/main.rs");

    assert!(
        overlay.contains("public function claim-focus()"),
        "the overlay must expose a way to hand focus back to its key handler"
    );
    assert!(
        main.contains("overlay.invoke_claim_focus()"),
        "the capture path must re-claim keyboard focus on every capture - `init` runs once, and \
         this overlay outlives every capture that borrows focus from it"
    );
}

/// Detection must not offer rectangles the Reviewer cannot see.
///
/// Without these filters the owner was offered "kotak-kotak yang tidak jelas": panes belonging to
/// windows buried behind others, and to Store apps that DWM was not drawing at all.
#[test]
fn detection_ignores_what_is_not_actually_on_screen() {
    let capturer = {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/snapdown-capture/src/capturer.rs");
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
    };
    let capturer = capturer
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        capturer.contains("DWMWA_CLOAKED"),
        "cloaked windows must be skipped: `IsWindowVisible` is true for a suspended Store app that \
         DWM draws nothing for, so its panes would be offered as targets pointing at nothing"
    );
    assert!(
        capturer.contains("CurrentIsOffscreen"),
        "off-screen UIA elements must be skipped: a scrolled-out or collapsed pane still reports a \
         perfectly good bounding rectangle"
    );
    assert!(
        capturer.contains("pub depth: u32"),
        "targets must carry their window's z-order, or a window buried behind another is offered \
         where the front one covers it"
    );
    // Matched on the ordering KEY, not on how the call is formatted: the first version of this
    // guard pinned the literal `sort_by_key(|t| {` and went red the moment rustfmt put the closure
    // on one line - which says nothing about whether the ordering is right.
    let sort = capturer
        .rfind("targets.sort_by_key")
        .expect("targets must be ordered before they are returned");
    let key = &capturer[sort..(sort + 200).min(capturer.len())];
    let depth_at = key.find("t.depth").expect(
        "z-order must be part of the ordering key, or a window buried behind another is offered \n         where the front one covers it",
    );
    let area_at = key
        .find("width")
        .expect("area must be part of the ordering key, so the tightest container wins");
    assert!(
        depth_at < area_at,
        "z-order must be the PRIMARY sort key: ordering by area first would offer a small pane from          a buried window in front of the window actually on top"
    );
}

/// The filmstrip must scroll rather than widen the window's demands.
///
/// Its cards are a fixed width, so a bare layout reports a minimum width that grows by one card per
/// Finding. That minimum propagates up through the workspace and squeezed the 440px
/// Notes/Properties panel to nothing after a dozen captures. The owner read it as the canvas
/// growing; what grew was this strip's appetite.
#[test]
fn the_filmstrip_scrolls_instead_of_widening_the_workspace() {
    let source = code("ui/appwindow.slint");
    let start = source
        .find("export component AppWindow")
        .expect("AppWindow not found");
    let app = &source[start..];

    let strip = app
        .find("filmstrip-items")
        .expect("the filmstrip must be built from filmstrip-items");
    let region = &app[strip.saturating_sub(2000)..];

    assert!(
        region.contains("filmstrip-scroll := ScrollView"),
        "the filmstrip must live inside a ScrollView. Without one its fixed-width cards impose a \
         minimum width on the whole workspace, which pushes the Notes/Properties panel off screen \
         as Findings accumulate"
    );
    assert!(
        region.contains("viewport-width: 24px + 110px + root.filmstrip-items.length * 132px"),
        "the filmstrip's viewport must be sized from the number of Findings, or the strip cannot \
         scroll to reach the later ones"
    );
}

/// A note that runs off to the right hides what you just typed, so the field wraps - while keeping
/// single-line SEMANTICS, because `accepted` (and therefore Enter-to-save) only fires when it is.
///
/// Those two are compatible, and it was checked rather than assumed: `TextInput::layout_info` in
/// i-slint-core consults `wrap()` without regard for `single_line()`.
#[test]
fn the_note_field_wraps_while_staying_single_line() {
    let field = code("ui/components/text-field.slint");

    // Scoped to the INPUT. Matching `wrap: word-wrap` anywhere in the file passed while the input
    // itself was set to `no-wrap`, because the placeholder Text carries the same property - a
    // mutation caught that, which is the only thing that would have.
    let input = field
        .find("input := TextInput")
        .expect("the shared field must be built on a TextInput");
    // Bounded at the input's own closing brace. A fixed 900-character window overshot into the
    // placeholder Text, which carries the same `wrap` property - and the mutant survived twice
    // before that was noticed.
    let input_end = field[input..]
        .find(
            "
    }",
        )
        .map(|rel| input + rel)
        .unwrap_or(field.len());
    let input_body = &field[input..input_end];
    assert!(
        input_body.contains("wrap: word-wrap"),
        "the shared field's INPUT must wrap, or a longer note scrolls out of sight as it is typed"
    );
    assert!(
        input_body.contains("single-line: root.commit-on-enter"),
        "single-line must follow `commit-on-enter`: `accepted`, which is what makes Enter save, \
         does not fire on a multi-line TextInput"
    );

    let overlay = overlay_block();
    let capture = overlay
        .find("note-input := SdTextField")
        .expect("the capture note field must exist");
    let body = &overlay[capture..(capture + 900).min(overlay.len())];
    assert!(
        body.contains("commit-on-enter: true"),
        "the CAPTURE note must commit on Enter - the hint text under it promises exactly that"
    );
    assert!(
        body.contains("height: max(38px, note-input.content-height + 20px)"),
        "and its box must grow with the wrapped text rather than clipping it"
    );
}

/// The note panel must stay on the visible SCREEN, not merely inside the canvas.
///
/// It used to be unconditionally below the selection, so a region dragged near the bottom of a
/// screen put the panel off the canvas entirely - and with it the only way to write the Note, which
/// makes FR-2 unreachable for that region rather than merely awkward.
///
/// The first attempt at this added a flip-above test against `root.height` and did NOT fix it: the
/// canvas is the bounding box of every monitor, so on a two-monitor desktop its bottom edge is below
/// the shorter screen's and a panel that "fitted the canvas" was still off the visible screen. The
/// bound has to be the monitor holding the SELECTION - and derived from the selection, not the
/// pointer, because the pointer moves onto the panel and may cross to another screen.
#[test]
fn the_note_panel_is_kept_inside_the_monitor_holding_the_selection() {
    let overlay = overlay_block();

    let panel = overlay
        .find("if is-narrating : Rectangle")
        .expect("the note panel must exist");
    let body = &overlay[panel..(panel + 1600).min(overlay.len())];

    assert!(
        body.contains("root.panel-mon-y + root.panel-mon-h"),
        "the panel must be bounded by the MONITOR holding the selection. Bounding it by \
         `root.height` - the canvas - is what left it off the bottom of the shorter screen on a \
         two-monitor desktop"
    );
    assert!(
        !body.contains("> root.height)"),
        "the fit test must not compare against the canvas height at all: on a multi-monitor \
         desktop that answer is wrong in the direction that hides the panel"
    );
    assert!(
        body.matches("clamp(").count() >= 2,
        "both axes must be CLAMPED into the monitor, not merely flipped. A flip alone has nowhere \
         to put the panel when the selection is taller than the screen, and it leaves the screen \
         again - the clamp is the guarantee and the flip is only the preference"
    );

    // And the bound itself must be measured from the selection, not from the pointer.
    assert!(
        overlay.contains("property <MonitorRectData> panel-monitor: root.monitor-at(")
            && overlay.contains("root.sel-x / 1px * root.source-scale"),
        "the panel's monitor must be found from the SELECTION's position. Using the pointer's \
         would move the panel to another screen the moment the Reviewer's mouse crossed over"
    );
}

/// Whatever a click would take must be UNDIMMED, the way a dragged selection is.
///
/// A hovered container used to be an outline plus a translucent wash on top of the full scrim, so
/// hovering and selecting spoke two different visual languages for one idea and the owner could not
/// tell what a click was about to take.
///
/// One rectangle now drives both the cut-out and the outline, for the hovered container AND for the
/// whole screen while the full-screen control is hovered - so the two can never disagree about what
/// is on offer.
#[test]
fn whatever_a_click_would_take_cuts_the_scrim_rather_than_being_washed_over() {
    let overlay = overlay_block();

    assert!(
        overlay.contains("visible: !root.has-selection && !root.has-preview"),
        "the full scrim must stand down whenever something is being previewed, or the preview \
         stays dimmed and looks nothing like the selection it becomes"
    );

    let group = overlay
        .find("visible: root.has-preview;")
        .expect("there must be a scrim group for the previewed rectangle");
    let body = &overlay[group..(group + 1400).min(overlay.len())];
    let panels = body.matches("background: Theme.overlay-scrim").count();
    assert!(
        panels >= 4,
        "the previewed rectangle must be cut out of the scrim with four rectangles AROUND it, the \
         same construction the selection uses - found {panels}. A translucent overlay instead of a \
         cut-out is what this replaced"
    );

    // The outline and the cut-out must read the SAME rectangle, or they can disagree.
    assert!(
        overlay.contains("visible: root.has-preview;\n            x: root.preview-x;"),
        "the outline must be placed from the same preview-* properties the cut-out uses"
    );
    assert!(
        overlay.contains(
            "property <bool> has-preview: root.fullscreen-preview || root.has-hover-target;"
        ),
        "the preview must cover BOTH the hovered container and the hovered full-screen control - \
         the owner asked for the screen to be suggested on hover, without a click"
    );
}

/// FR-1's "1-click container selection" includes the commonest container of all: the screen.
///
/// It must be on the monitor under the pointer, and it must be declared AFTER the catch-all
/// TouchArea, since Slint puts later siblings on top and the catch-all covers the whole desktop.
///
/// That ordering is enforced by the compiler as well as by this test: the control reads ids declared
/// above it, so moving it earlier fails the BUILD.
#[test]
fn a_full_screen_target_exists_on_the_monitor_under_the_pointer() {
    let overlay = overlay_block();

    let control = overlay.find("fullscreen-touch := TouchArea").expect(
        "there must be a one-click full-screen target - FR-1 promises 1-click container \
             selection and a whole screen is the container people take most",
    );
    let catch_all = overlay
        .find("hover-tracker := TouchArea")
        .expect("the catch-all TouchArea must exist");
    assert!(
        catch_all < control,
        "the full-screen target must be declared AFTER the catch-all TouchArea, or the catch-all \
         sits on top of it and swallows every click"
    );

    assert!(
        overlay.contains("root.crosshair-x + (root.crosshair-w"),
        "it must be placed from the same active-monitor geometry the crosshair is confined by - a \
         control on the other screen is a control you have to go and find. WHERE on that monitor \
         is the_full_screen_control_is_centred_on_the_top_edge_and_reads_against_the_scrim's \
         business"
    );
    let body = &overlay[control..(control + 600).min(overlay.len())];
    assert!(
        body.contains("root.sel-w = root.crosshair-w")
            && body.contains("root.sel-h = root.crosshair-h"),
        "clicking it must select the ACTIVE monitor's bounds, not the whole canvas"
    );
}

/// The full-screen control must not flicker when the pointer reaches it.
///
/// Its visibility was gated on `hover-tracker.has-hover`, and it sits ON TOP of that TouchArea - so
/// moving the pointer onto the control took the hover away from the tracker, which hid the control,
/// which handed the hover straight back. The oscillation was in the visibility condition; nothing
/// about the rendering was involved.
#[test]
fn the_full_screen_control_visibility_does_not_depend_on_the_tracker_it_covers() {
    let overlay = overlay_block();

    let control = overlay
        .find("visible: !root.has-selection && !root.is-narrating;\n            x: root.crosshair-x + (root.crosshair-w")
        .expect(
            "the full-screen control must be visible on the selection state alone. Gating it on \
             `hover-tracker.has-hover` makes it hide itself the moment the pointer arrives, \
             because it covers that tracker",
        );
    let body = &overlay[control..(control + 400).min(overlay.len())];
    assert!(
        !body.contains("hover-tracker.has-hover"),
        "the control must not read the hover state of the TouchArea it occludes"
    );

    // Its own hover state has to travel upward, because the properties that consume it are declared
    // above it and a Slint id resolves backwards only.
    assert!(
        overlay.contains("changed has-hover => {")
            && overlay.contains("root.fullscreen-hover = self.has-hover;"),
        "the control's hover must be pushed to a root property, which is what lets the preview \
         above it react without a forward id reference"
    );
}

/// Changing your mind must not cost an Escape and a fresh capture.
///
/// Once a region was selected the catch-all TouchArea was DISABLED, so a click anywhere else did
/// nothing at all. That was not a choice about interaction: the note panel was nested inside the
/// selection frame, which put it under that TouchArea, and leaving it enabled meant every click
/// meant for the note field was swallowed. Lifting the panel to the top of the z-order removes the
/// conflict, and the catch-all can stay live.
#[test]
fn clicking_elsewhere_starts_a_new_selection_without_escape() {
    let overlay = overlay_block();

    let tracker = overlay
        .find("hover-tracker := TouchArea")
        .expect("the catch-all TouchArea must exist");
    let body = &overlay[tracker..(tracker + 400).min(overlay.len())];
    assert!(
        !body.contains("enabled: !root.is-narrating"),
        "the catch-all must stay enabled while a note is being written, or the Reviewer cannot \
         re-aim without pressing Escape and starting the capture again"
    );

    let panel = overlay
        .find("if is-narrating : Rectangle")
        .expect("the note panel must exist");
    assert!(
        tracker < panel,
        "the note panel must be declared AFTER the catch-all, or that still-enabled TouchArea \
         sits on top of it and swallows every click meant for the note field"
    );
    let panel_body = &overlay[panel..(panel + 2600).min(overlay.len())];
    assert!(
        panel_body.contains("TouchArea {"),
        "the panel must absorb clicks on its own padding, or they fall through to the catch-all \
         and clear the very selection the panel is about"
    );
}

/// FR-1 calls the magnifier a "pixel loupe". For its cells to BE pixels, the picture has to have
/// pixels in it - which took four attempts to arrive at:
///   1. no grid at all;
///   2. a grid whose cell count did not match the source count, because `loupe-source-size` was
///      scaled by `source-scale` - 24 pixels read out under 16 drawn cells on a 150% display;
///   3. the lens moved onto the pointer and the image slid by the sub-pixel remainder. Worse:
///      offsetting a magnified image by a fraction of a pixel resamples it every frame, so the view
///      shimmered. Reverted, not built on;
///   4. nearest-neighbour sampling. With smooth filtering there are no pixels in the picture for a
///      grid to belong to - the blocks are a blur and the only crisp thing is the drawn grid, which
///      is exactly why it kept reading as part of the glass.
#[test]
fn the_loupe_shows_source_pixels_as_pixels() {
    let overlay = overlay_block();

    assert!(
        overlay.contains("image-rendering: pixelated;"),
        "the lens must sample nearest-neighbour, or the source pixels are a smear. Excluded on a \
         BUG-27 measurement taken while the Image laid out the WHOLE screenshot at 3x; it now clips \
         to a few hundred pixels, the premise expired, and OQ-28 re-opened it deliberately"
    );
    assert!(
        overlay.contains("property <int> loupe-source-size: root.loupe-pixels;"),
        "the lens must read out exactly as many source pixels as the grid draws cells. Scaling that \
         count by `source-scale` made a cell correspond to no pixel at all on a HiDPI display"
    );
    // The cell is chosen first, as a whole number of PHYSICAL pixels, and the face follows from it.
    //
    // It used to be the other way round - a 96px face over 20 pixels, so 4.8px logical, which at
    // this display's 1.5 scale puts grid lines at 7.2, 14.4, 21.6 physical pixels. The renderer
    // antialiased each one across two, so some rows read sharp and others read blurred. The 1px
    // lines had the same fault by construction: one logical pixel is 1.5 physical.
    assert!(
        overlay.contains(
            "round(root.loupe-cell-target / 1px * root.source-scale) / root.source-scale * 1px"
        ),
        "the cell must be snapped to a whole number of PHYSICAL pixels, or grid lines land \
         mid-pixel and the renderer blurs them - which is what the owner saw as some rows being \
         out of focus"
    );
    assert!(
        overlay.contains("property <length> loupe-face: root.loupe-cell * root.loupe-pixels;"),
        "and the face must follow FROM the cell, not the other way round: a round face divided by a \
         pixel count gives a fractional cell"
    );
    assert!(
        overlay.contains("property <length> loupe-line: 1px / root.source-scale;"),
        "a grid line must be exactly ONE physical pixel. `1px` is a LOGICAL pixel and is a 50% \
         smear at scale 1.5"
    );
    assert!(
        overlay.contains("function snap(v: length) -> length {"),
        "and the lens's own origin must be snapped too: the pointer arrives at fractional logical \
         coordinates, so an unsnapped lens shifts its whole content by a fraction of a physical \
         pixel and blurs every line inside it at once"
    );
    assert!(
        overlay.contains("source-clip-x: root.loupe-origin-x;")
            && overlay.contains("property <int> loupe-origin-x: clamp(")
            && overlay.contains("floor(root.pointer-src-x)"),
        "the view must start on a WHOLE source pixel, so blocks land on cell boundaries"
    );
    assert!(
        !overlay.contains("loupe-slide-x"),
        "the sub-pixel slide must be gone, not merely unused: it resampled the magnified image every \
         frame and the lens shimmered"
    );
}

/// The lens is a magnifying glass, and the owner specified it precisely: round, small, BESIDE the
/// pointer with a crosshair of its own at the middle, and only on screen once the pointer settles.
///
/// Every one of those was wrong at some point. It was a 144px square centred on the pointer - which
/// hides the very thing being aimed at - permanently visible, chasing a fast mouse across the screen.
#[test]
fn the_lens_sits_beside_the_pointer_and_waits_for_it_to_settle() {
    let overlay = overlay_block();

    assert!(
        overlay.contains("x: root.snap((root.mouse-x + 18px + root.loupe-face")
            && overlay.contains("? root.mouse-x - 18px - root.loupe-face"),
        "the lens must sit BESIDE the pointer, flipping side near a monitor edge, and its origin \
         must be snapped to a physical pixel. Centred on the pointer it covers the pixel it is \
         magnifying; unsnapped it blurs its own grid"
    );
    assert!(
        !overlay.contains("x: root.mouse-x - root.loupe-face / 2;"),
        "the centred placement is what this replaced"
    );
    assert!(
        overlay.contains("border-radius: root.loupe-face / 2;"),
        "the lens must be round, to specification - the cost is the outermost cells, and the pixel \
         that matters is the one at the centre"
    );

    // Its own crosshair, placed on the pixel under the pointer rather than at the frame's middle.
    assert!(
        overlay.contains(
            "x: root.loupe-cursor-col * root.loupe-cell\n                    + (root.loupe-cell - root.loupe-line) / 2;"
        ),
        "the lens must carry a crosshair crossing on the pixel under the pointer, derived from the \
         CLAMPED origin: at a screen edge the view cannot be centred and the middle of the frame is \
         not that pixel"
    );

    // Settling, sampled on a tick rather than per event.
    //
    // Anchored on the LENS's own block, found from its round radius and walked back to the enclosing
    // Rectangle. Searching the whole overlay for the gate passed while the lens was ungated, because
    // the coordinate readout beside it carries the identical line - a mutation caught that, and
    // nothing else would have.
    let radius = overlay
        .find("border-radius: root.loupe-face / 2;")
        .expect("the lens must be round");
    let lens_open = overlay[..radius]
        .rfind("        Rectangle {")
        .expect("the lens must be a Rectangle");
    assert!(
        overlay[lens_open..radius].contains("root.pointer-settled"),
        "the lens itself must wait for the pointer to settle - a magnifier chasing a fast pointer is \
         noise, and Snagit does not do it either"
    );
    assert!(
        overlay.matches("root.pointer-settled;").count() >= 2,
        "and so must its coordinate readout, or the pill hangs there on its own while the lens is \
         gone"
    );
    assert!(
        overlay.contains("interval: 100ms;") && overlay.contains("running: hover-tracker.has-hover;"),
        "settling must be sampled on a fixed tick, which is a real velocity - a per-`moved`-event \
         distance threshold is a different speed on every machine and every frame rate. And the tick \
         must stop while the overlay is hidden between captures"
    );
    assert!(
        overlay.contains("abs(root.mouse-x - root.tick-x) + abs(root.mouse-y - root.tick-y)"),
        "and it must compare against the position one tick ago, not against the last event"
    );

    // TWO thresholds, with a band between them where nothing changes.
    //
    // One threshold and a binary flip is a comparator without hysteresis: hold the pointer near the
    // boundary and consecutive ticks land on opposite sides, so the lens blinks. That shipped, and
    // the owner reported it as "saya pelan, tapi kadang hilang, kadang muncul".
    assert!(
        overlay.contains("property <length> settle-slow:")
            && overlay.contains("property <length> settle-fast:"),
        "settling must have a SHOW threshold and a separate HIDE threshold. A single one oscillates \
         for any pointer speed near it, which is the flicker this replaced"
    );
    let slow = overlay
        .find("property <length> settle-slow: ")
        .map(|i| &overlay[i + 31..i + 36])
        .unwrap_or("");
    let fast = overlay
        .find("property <length> settle-fast: ")
        .map(|i| &overlay[i + 31..i + 36])
        .unwrap_or("");
    let num = |t: &str| {
        t.chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>()
            .parse::<u32>()
            .unwrap_or(0)
    };
    assert!(
        num(slow) > 0 && num(fast) > num(slow),
        "the hide threshold must be strictly LOOSER than the show threshold, or the band is empty \
         and there is no hysteresis at all - got show={slow:?} hide={fast:?}"
    );
    assert!(
        overlay.contains("root.fast-ticks >= 2"),
        "and hiding must need two consecutive fast ticks, so one jerk of the wrist does not dismiss \
         a lens the Reviewer is reading"
    );

    // The crosshair replaces the cursor, so the cursor has to go - but ONLY while the crosshair is
    // actually drawn.
    //
    // Hiding it unconditionally shipped, and left the Reviewer with no pointer at all once a region
    // was selected: the crosshair is gated on `!is-narrating`, so between selecting and saving there
    // was nothing on screen except over the note panel, whose own TouchArea restores the default.
    // The Reviewer was aiming at a Save button they could not see.
    assert!(
        overlay
            .contains("mouse-cursor: root.is-narrating ? MouseCursor.default : MouseCursor.none;"),
        "the OS cursor must be hidden while the crosshair stands in for it, and RESTORED once it \
         does not"
    );
    assert!(
        !overlay.contains("mouse-cursor: none;"),
        "and it must not be hidden unconditionally - the condition has to be the crosshair's own \
         visibility condition, or the two drift apart again"
    );
}

/// The Observation Summary must reflect the Note it was handed, be visible without hunting, and
/// survive being edited.
///
/// Three attempts, three different causes, and only the third was the one that showed:
///   1. a `<=>` alias inside `if active-tab-index == 0` - a subtree CREATED and DESTROYED rather
///      than shown and hidden - was re-linked against the widget's empty string on rebuild;
///   2. replacing it with a one-way `text:` binding avoided that and broke something worse, since
///      Slint drops a binding as soon as anything writes the property and the widget writes to
///      itself on every keystroke;
///   3. the widget had NO GEOMETRY. The accessibility tree showed one Edit control, holding the
///      right note, with an infinite bounding rectangle. The value had been arriving all along.
#[test]
fn the_observation_summary_is_visible_persistent_and_outside_the_tab_body() {
    let source = code("ui/appwindow.slint");
    let start = source
        .find("export component AppWindow")
        .expect("AppWindow must exist");
    let app = &source[start..];

    let field = app.find("text <=> root.observation-summary;").expect(
        "the Observation Summary must be linked two-way. A one-way `text:` binding is severed \
             the first time the widget writes to itself, after which a newly selected Finding's \
             note never appears (BUG-45)",
    );
    let tab_body = app
        .find("if root.active-tab-index == 0")
        .expect("the Notes tab body must exist");
    assert!(
        field < tab_body,
        "it must be declared BEFORE - and therefore outside - the conditional tab body, whose \
         subtree is destroyed and rebuilt"
    );

    // It must be the shared field, which is what gives it both a visible palette and a size.
    let decl = app[..field].rfind("SdTextField {").expect(
        "the Observation Summary must be an SdTextField. A std-widget TextEdit paints from Slint's \
         palette (white-on-white here) and takes no geometry in a plain Rectangle (BUG-45)",
    );
    let body = &app[decl..(decl + 700).min(app.len())];
    assert!(
        body.contains("height: 120px"),
        "and it must be given an explicit height"
    );
    assert!(
        body.contains("root.observation-edited(root.observation-summary)"),
        "an edit must be pushed out to be SAVED. Held only in this property, it was overwritten the \
         moment another Finding was selected - silent data loss"
    );

    assert!(
        app.contains("placeholder: \"No note on this Finding.\""),
        "an empty box must say which of two things it means. \"No note\" and \"the note did not \
         arrive\" looked identical, and that is what made BUG-45 hard to read"
    );
    assert!(
        !source.contains(r#"observation-summary: "test"#),
        "the property must not default to placeholder content: a reader cannot tell fake content \
         from a real note, so a binding that never delivered looked like a note saved wrong (BUG-45)"
    );
}

/// BUG-44: the point-based detection is gone, not merely unused.
///
/// It could never work under the capture overlay - the overlay is topmost, so ElementFromPoint
/// returns our own window - and AGENTS.md records four earlier cases in this repository of code that
/// existed, compiled, and was reachable by nobody.
#[test]
fn the_point_based_detection_is_deleted() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/snapdown-capture/src/capturer.rs");
    let capturer =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"));

    assert!(
        !capturer.contains("fn detect_element_at_point"),
        "detect_element_at_point must be deleted, not kept as dead code - it cannot work under the \
         overlay, and leaving it invites the next reader to believe it is the detection in use"
    );
}

/// NFR-17 puts colour in exactly one file, and its stated enforcement - a lint refusing a colour
/// literal outside the token file - covers `web/ui/src` only. The Slint surfaces had no equivalent,
/// which is how the overlay accumulated `#ffffff`, `#475569`, `#94a3b8`, `#38bdf8` and five
/// different alphas of black while `theme.slint` sat beside it defining the product's palette.
///
/// This is that lint for the overlay. It is deliberately narrow - `CaptureOverlayWindow` only - so
/// it states a fact that is true today rather than a goal for the whole file.
#[test]
fn the_capture_overlay_takes_every_colour_from_the_token_set() {
    let overlay = overlay_block();

    let literals: Vec<&str> = overlay
        .lines()
        .map(str::trim)
        .filter(|line| {
            // A hex colour: `#` followed by at least three hex digits.
            line.split('#').skip(1).any(|rest| {
                rest.chars()
                    .take(3)
                    .filter(|c| c.is_ascii_hexdigit())
                    .count()
                    == 3
            })
        })
        .collect();

    assert!(
        literals.is_empty(),
        "every colour in the capture overlay must come from `theme.slint` (NFR-17). Found {}: {:?}\n\
         The overlay's own tokens are deliberately theme-INVARIANT - a light scrim over a dark \
         screenshot is invisible - so the answer is a token in group 6, not a literal here",
        literals.len(),
        literals
    );

    // And the tokens it uses must actually exist, or this guard passes on a typo.
    let theme = read("ui/theme.slint");
    for token in [
        "overlay-scrim",
        "overlay-ring",
        "overlay-grid",
        "overlay-canvas",
        "overlay-chrome",
        "overlay-chrome-border",
        "overlay-text",
        "overlay-text-muted",
        "overlay-shadow",
    ] {
        assert!(
            theme.contains(&format!("out property <color> {token}:")),
            "theme.slint must define `{token}` - the overlay reads it"
        );
    }
}

/// BUG-45, third and actual cause: a std-widget inside a plain `Rectangle` has no size.
///
/// A `Rectangle` child with no geometry fills its parent. An imported COMPONENT does not - it takes
/// its own preferred size, and a `TextEdit`'s resolves to nothing. The Observation Summary held the
/// right note for two rounds of "fixes" aimed at its binding, with an infinite bounding rectangle in
/// the accessibility tree: the value was arriving and had nowhere to be drawn.
///
/// The same construction sits under the marker list, unseen only because that list has always been
/// empty - which is exactly how the first one survived. `LineEdit`s inside a `*Layout` are fine; the
/// layout gives them geometry.
#[test]
fn a_std_widget_in_a_plain_rectangle_is_given_explicit_geometry() {
    let source = code("ui/appwindow.slint");
    let lines: Vec<&str> = source.lines().collect();

    let mut offenders: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !(trimmed.starts_with("TextEdit {") || trimmed.starts_with("LineEdit {")) {
            continue;
        }

        // The nearest preceding line that opens a block is the parent element's declaration.
        let parent = lines[..i]
            .iter()
            .rev()
            .find(|l| l.trim_end().ends_with('{'))
            .map(|l| l.trim())
            .unwrap_or("");
        if parent.contains("Layout {") {
            continue; // a layout hands out geometry
        }

        // Scan this widget's own block for an explicit size.
        let block: String = lines[i..(i + 14).min(lines.len())].join("\n");
        let block = block
            .split_once("\n                                }")
            .map(|(head, _)| head.to_string())
            .unwrap_or(block);
        if !(block.contains("width:") && block.contains("height:")) {
            offenders.push(format!("line {} in `{}`", i + 1, parent));
        }
    }

    assert!(
        offenders.is_empty(),
        "a TextEdit/LineEdit whose parent is not a layout must set its own width AND height, or it \
         renders with no geometry and its content is invisible while every binding around it looks \
         correct (BUG-45). Offenders: {offenders:?}"
    );
}

/// BUG-41: selecting a Finding must not rebuild the filmstrip.
///
/// Measured on the owner's library of 58 Findings: a click cost 320.9 ms, of which 310.0 ms was
/// decoding every thumbnail in the library and 1.3 ms was the database query. None of that work
/// changes when the selection changes, and it grows with every capture taken. After the split, the
/// same click costs 10.5 ms - one full-size decode - and stops scaling with the library.
#[test]
fn selecting_a_finding_does_not_rebuild_the_filmstrip() {
    let main = code("src/main.rs");

    let handler = main
        .find("on_finding_selected")
        .expect("the filmstrip's selection handler must exist");
    let body = &main[handler..(handler + 400).min(main.len())];
    assert!(
        body.contains("select_active_finding"),
        "selection must go through the path that updates rows in place. Calling \
         `load_findings_into_window` here re-decodes every image in the library on every click"
    );
    assert!(
        !body.contains("load_findings_into_window"),
        "the selection handler must not rebuild the whole strip - that is the 310 ms"
    );

    let fast = main
        .find("fn select_active_finding")
        .expect("select_active_finding must exist");
    let fast_body = &main[fast..(fast + 1400).min(main.len())];
    assert!(
        fast_body.contains("set_row_data"),
        "the active flag must be moved by writing the affected rows, not by rebuilding the model"
    );
    assert!(
        fast_body.contains("load_active_detail"),
        "and the selected Finding's own detail must still be loaded"
    );

    // The capture path DOES have to rebuild, because a new Finding changes the set.
    assert!(
        main.contains("load_findings_into_window(&main, &ctx_inner, finding_id.as_deref())"),
        "a completed capture must still rebuild the strip: the set of Findings has changed"
    );
}

/// The full-screen control has to be SEEN, and the owner has to be able to find it where they look.
///
/// It was `overlay-chrome` - near-black - on a 65% black scrim, and the owner reported not being able
/// to tell it was there. A control nobody notices is the same failure as FR-1's affordance being
/// absent; contrast is the fix, not a bigger hit area.
#[test]
fn the_full_screen_control_is_centred_on_the_top_edge_and_reads_against_the_scrim() {
    let overlay = overlay_block();

    assert!(
        overlay.contains("x: root.crosshair-x + (root.crosshair-w - self.width) / 2"),
        "the control must sit at the TOP CENTRE of the monitor under the pointer - asked for twice, \
         after a corner placement was tried"
    );
    assert!(
        overlay.contains(
            "background: root.fullscreen-hover ? Theme.accent-hover : Theme.accent-primary;"
        ),
        "its resting state must be the accent, not overlay chrome: near-black furniture on a 65% \
         black scrim is invisible"
    );
    assert!(
        !overlay.contains("Theme.overlay-chrome : Theme.accent-primary"),
        "the accent must not be reserved for the hover state - the control has to be findable before \
         the pointer is on it"
    );
}

/// The right panel's first tab holds per-marker commentary and nothing else, since the Observation
/// Summary moved above the tab bar. Its label and its icon both have to say so.
///
/// `file-text.svg` described the summary and, left behind, named the wrong tab. `marker.svg` is a
/// FILLED RED pin - right for a marker on the canvas, wrong for a tab, whose partner `palette.svg`
/// on Properties is a blue outline. The pair have to speak one language, so a new outline file was
/// added rather than the canvas marker icons being touched.
#[test]
fn the_marker_tab_label_and_icon_agree() {
    let source = code("ui/appwindow.slint");
    let start = source
        .find("export component AppWindow")
        .expect("AppWindow must exist");
    let app = &source[start..];

    let tab = app
        .find("Rectangle {\n                                    width: 140px;")
        .expect("the first tab must exist");
    let body = &app[tab..(tab + 2400).min(app.len())];
    assert!(
        body.contains(r#"text: "Marker Notes";"#),
        "the first tab must name what is left in it"
    );
    assert!(
        body.contains("icons/marker-outline.svg"),
        "and carry a blue OUTLINE marker, matching `palette.svg` on the Properties tab beside it"
    );

    let icon = read("../../apps/desktop/assets/icons/marker-outline.svg");
    assert!(
        icon.contains("stroke=\"#2563eb\"") && !icon.contains("fill=\"#e11d48\""),
        "the tab icon must be a blue stroke with no red fill"
    );
    // The canvas marker icons are a different job and were explicitly to be left alone.
    let canvas_marker = read("../../apps/desktop/assets/icons/marker-inactive.svg");
    assert!(
        canvas_marker.contains("#475569"),
        "the toolbar's outline marker must be untouched - the owner asked for it to be left as it is"
    );
}

/// A filmstrip card is 120x86, and must not hold the Finding's full-resolution image.
///
/// Measured on the owner's library of 61 Findings: 230.8 MB of decoded RGBA held resident to fill
/// those cards, on top of the 175.8 MB the overlay retains by design - and growing with every
/// capture. Private bytes were 650 MB at start-up.
#[test]
fn filmstrip_thumbnails_are_downscaled_before_they_are_retained() {
    let main = code("src/main.rs");

    assert!(
        main.contains("const THUMB_MAX_W: u32 = 240;")
            && main.contains("const THUMB_MAX_H: u32 = 172;"),
        "there must be an explicit thumbnail bound - a 120x86 card at 2x for a HiDPI display"
    );
    let build = main
        .find("let loaded_img = if img_path.exists()")
        .expect("the thumbnail load must exist");
    let body = &main[build..(build + 400).min(main.len())];
    assert!(
        body.contains("thumbnail(THUMB_MAX_W, THUMB_MAX_H)"),
        "each thumbnail must be downscaled BEFORE it is put in the model, not after. The model \
         holds every one of them for the life of the window"
    );

    // The full-size decode belongs to the canvas, and only there.
    let detail = main
        .find("fn load_active_detail")
        .expect("load_active_detail must exist");
    let detail_body = &main[detail..(detail + 900).min(main.len())];
    assert!(
        detail_body.contains("set_active_image"),
        "the selected Finding's own image is the one that must stay full-size: it is the canvas"
    );
}

/// An edited note must reach the store.
///
/// It lived only in the Slint property, so selecting another Finding overwrote it and the edit was
/// gone - silent data loss, of the kind a Reviewer finds out about long afterwards.
#[test]
fn an_edited_observation_summary_is_written_to_the_store() {
    let main = code("src/main.rs");

    let handler = main
        .find("on_observation_edited")
        .expect("an edit handler must exist, or the note is only ever held in a UI property");
    let body = &main[handler..(handler + 700).min(main.len())];
    assert!(
        body.contains("update_note"),
        "the edit must be written through to the FindingStore"
    );
    assert!(
        body.contains("get_active_finding_id"),
        "and written to the Finding the panel is actually showing - the id has to come from the \
         window, not be assumed"
    );
    assert!(
        body.contains("if id.is_empty()"),
        "with no Finding selected there is nothing to write to, and writing anyway would attach the \
         note to whatever row happened to be first"
    );
}

/// Every dimension the overlay shows must be a SOURCE pixel count.
///
/// The overlay measured itself in logical pixels while everything downstream - the crop, the stored
/// file, the Editor - is physical. On the owner's 175% display that is a 1.75x lie about the size of
/// what is being taken, and it read as two parts of the product disagreeing.
#[test]
fn the_overlay_reports_dimensions_in_source_pixels() {
    let overlay = overlay_block();

    assert!(
        overlay.contains("round(sel-w / 1px * root.source-scale)"),
        "the selection badge must report source pixels - what actually gets saved - not the logical \
         pixels the window happens to be measured in"
    );
    assert!(
        overlay.contains("root.active-monitor().width + \" × \" + root.active-monitor().height"),
        "and the full-screen readout must come straight from the monitor rectangle, which is already \
         in snapshot pixels, so nothing is converted and nothing can drift"
    );
    assert!(
        !overlay.contains("round(root.crosshair-w / 1px) + \" × \""),
        "the logical-pixel readout is what this replaced"
    );
}

/// The Editor must say what the Quality Budget did, rather than leaving the difference to look like
/// a defect.
///
/// The overlay reports the region being selected; the Editor reports the file on disk, after
/// `ImageReducer` has applied the budget. Those legitimately differ - the owner saw 2194x1234 in one
/// and 1600x900 in the other - and with only one number visible there was no way to tell a reduction
/// from a bug.
#[test]
fn the_editor_shows_what_the_quality_budget_reduced() {
    let main = code("src/main.rs");

    // The parser itself is unit-tested in main.rs against the real column shape - see
    // a_region_field_is_four_comma_separated_numbers. That test exists because the first version of
    // this readout used serde_json on a column holding "x,y,w,h", which compiles, returns None for
    // every row, and passed a guard shaped exactly like this one.
    assert!(
        main.contains("parse_region_field(&f.region)")
            && !main.contains("serde_json::from_str::<Region>"),
        "the region column must be parsed as the comma-separated string it actually is"
    );

    let res = main
        .find("let resolution = match selected {")
        .expect("the resolution readout must consider the originally selected region");
    let body = &main[res..(res + 500).min(main.len())];
    assert!(
        body.contains("w != f.image_width || h != f.image_height"),
        "it must compare the saved size against the selected size, and only then say anything - a \
         capture the budget did not touch must not carry a confusing second number"
    );
    assert!(
        body.contains("from {} × {}"),
        "and when they differ it must show what the image was reduced FROM"
    );
}

/// One design system, which means the capture panel and the Editor's inspector read the same values
/// rather than agreeing by eye.
///
/// The owner raised this four times, and each time the answer had the same shape: a `std-widgets`
/// component painting itself from Slint's palette instead of `theme.slint`. `TextEdit` gave the
/// Observation Summary white-on-white; `Button` gave the capture panel a Save action at a different
/// size, colour and type scale from every button in the Editor. Neither could be fixed by adjusting
/// a number - the numbers were not ours to adjust.
#[test]
fn the_capture_panel_and_the_inspector_share_one_design_system() {
    let app = read("ui/appwindow.slint");

    // No std-widget can style itself in this product.
    for widget in ["LineEdit", "TextEdit", "Button"] {
        assert!(
            !declares_element(&app, widget),
            "`{widget}` must not be used: it paints from Slint's own palette rather than \
             `theme.slint`. Use SdTextField / SdActionButton"
        );
    }
    assert!(
        app.contains("import { ScrollView } from \"std-widgets.slint\";"),
        "and only ScrollView may be imported, so reaching for one of the others is a compile error \
         rather than something a review has to catch"
    );

    // The shared components carry the Editor's values, so both panels move together.
    let btn = code("ui/components/action-button.slint");
    for token in [
        "Theme.accent-pressed",
        "Theme.accent-hover",
        "Theme.accent-primary",
        "Theme.text-on-accent",
        "Theme.radius-sharp",
    ] {
        assert!(
            btn.contains(token),
            "the shared action must use the Editor's own accent triple and radius - `{token}` missing"
        );
    }
    assert!(
        btn.contains("font-weight: 700;") && btn.contains("in property <length> label-size: 11px;"),
        "and the Editor's type scale for an action label: 700 weight, 10-11px"
    );

    // Both section headings are the same component, not two Texts that happen to match.
    let overlay = overlay_block();
    let inspector_start = code("ui/appwindow.slint")
        .find("export component AppWindow")
        .expect("AppWindow must exist");
    let inspector = code("ui/appwindow.slint")[inspector_start..].to_string();
    assert!(
        overlay.contains("SdSectionLabel {") && inspector.contains("SdSectionLabel {"),
        "the capture panel's heading and the inspector's must be the SAME component. Two Texts with \
         matching numbers drift the first time one of them is touched"
    );
    assert!(
        !overlay.contains("font-weight: 700;\n                    color: Theme.text-secondary;"),
        "the capture panel must not carry its own hand-set heading style any more"
    );

    // And the same spacing, which was 14px/10px here against the inspector's 16px/6px.
    let panel = overlay
        .find("note-body := VerticalLayout {")
        .expect("the capture panel's layout must exist");
    let body = &overlay[panel..(panel + 200).min(overlay.len())];
    assert!(
        body.contains("padding: 16px;") && body.contains("spacing: 6px;"),
        "the capture panel must use the inspector's padding and spacing, not numbers chosen for it \
         alone"
    );
}

/// A whole window must be selectable, not only the panel inside it.
///
/// `detect_capture_targets` returns candidates smallest-first, which is what makes occlusion work -
/// the tightest container of the FRONTMOST window is what a click should take. It is also what made a
/// window unreachable: a panel inside an application always won, and there was no way to ask for the
/// application. Owner: "karena dia mendeteksi panel per panel, saya jadi kesulitan untuk mendeteksi 1
/// window penuh."
#[test]
fn the_reviewer_can_walk_outward_to_a_whole_window() {
    let overlay = overlay_block();
    let main = code("src/main.rs");

    assert!(
        overlay.contains("pure callback target-at(int, int, int) -> MonitorRectData;"),
        "the hit test must take a LEVEL: without one, only the tightest container is ever reachable"
    );
    assert!(
        overlay.contains("property <int> target-level: 0;"),
        "and the overlay must hold which level is being shown"
    );
    assert!(
        main.contains(".nth(level.max(0) as usize)"),
        "Rust must index the candidates under the pointer by that level, rather than taking the \
         first. `.find()` is what made every deeper candidate win"
    );

    // The wheel is the control, and it must be clamped rather than wrapped.
    let scroll = overlay
        .find("scroll-event(event) =>")
        .expect("the wheel must walk the levels - it is the only control that does");
    let body = &overlay[scroll..(scroll + 700).min(overlay.len())];
    assert!(
        body.contains("min(root.target-level + 1, max(0, root.target-count - 1))"),
        "walking outward must be clamped to the number of candidates. Wrapping means one notch too \
         many silently returns to the panel being escaped, which looks like the wheel not working"
    );
    assert!(
        body.contains("max(root.target-level - 1, 0)"),
        "and it must walk back in"
    );

    // Moving the pointer invalidates the level, because the candidate list is different there.
    let moved = overlay
        .find("moved => {")
        .expect("the tracker must handle movement");
    let moved_body = &overlay[moved..(moved + 300).min(overlay.len())];
    assert!(
        moved_body.contains("root.target-level = 0;"),
        "the level must reset when the pointer moves: the containers under a new point are a \
         different list, so level 2 there means something unrelated"
    );

    // An invisible affordance is not an affordance.
    assert!(
        overlay.contains(r#"text: "Scroll to widen  ·  " + (root.target-level + 1) + " / " + root.target-count;"#),
        "the overlay must SAY that the wheel widens, and show where in the stack it is - the \
         detection walks panels before windows, so a Reviewer wanting the window has no way to know \
         one is available"
    );
    assert!(
        overlay.contains(
            "visible: root.has-preview && !root.fullscreen-preview && root.target-count > 1;"
        ),
        "and only while there is somewhere to widen TO, or the hint is noise at every other point"
    );
    assert!(
        main.contains("overlay.on_target_count_at"),
        "which needs the count from Rust; the .slint has no loop to compute it"
    );
}
