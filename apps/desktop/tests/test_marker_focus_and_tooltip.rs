//! `02-marker-note-focus-and-tooltip.md` (`FR-8`, `UC-5`).
//!
//! Two things this file asserts, and both follow `test_annotation_wiring.rs`'s own convention of
//! reading the real source rather than compiling a Slint test harness (none exists in this repo):
//!
//! 1. the Note field's own text-input claims REAL keyboard focus in both the place-Marker path
//!    (Rust, `on_marker_placed`) and the click-an-existing-Marker path (`appwindow.slint`'s pointer
//!    handler) - and that a drag never does, because the focus request sits behind the same
//!    "a click is not a drag" guard `drag-is-real`/`crop-drag-is-real` already use elsewhere in this
//!    file (`marker-drag-is-real`);
//! 2. the hover tooltip's text is bound to the hovered Marker's OWN `comment` field from the `for m
//!    in root.markers` loop it is declared inside - never to a window-wide property that could still
//!    hold a previous Marker's text after the pointer moves - and never shown at all for an empty
//!    note.

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// The same source with every run of whitespace collapsed to one space - `test_annotation_wiring.rs`
/// and `test_zoom_wiring.rs` both carry this helper for the same reason: a guard `rustfmt` can turn
/// red on a reformat is a guard nobody keeps.
fn flat(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The source with `//` comment lines removed, same as `test_annotation_wiring.rs`'s helper.
///
/// This file's own comments quote short illustrative fragments of the very code they explain (e.g.
/// `library.slint`'s `init => { self.focus(); }`, echoed here to justify copying its shape) - so a
/// raw, un-stripped `find("init => {")` can match the QUOTE inside a comment instead of the real
/// handler several lines below it. Stripping comment lines first is what keeps every search below
/// looking at code, never at a comment's example.
fn code_only(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// From `start` (an index already found in `source`, e.g. via `str::find`), return the source from
/// there through the matching close of the FIRST `{` at or after `start`.
///
/// A fixed byte window (`+2200`, as `test_annotation_wiring.rs` uses) is fragile against this file's
/// own deeply nested `if ... : Rectangle { ... }` blocks - a comment added above a block would push
/// real content past a fixed window's edge without any of the asserted behaviour changing. Brace
/// counting has no such edge. Callers pass comment-stripped source (see `code_only` above), so a
/// brace quoted inside a comment cannot desynchronise the count either.
fn block_from(source: &str, start: usize) -> &str {
    let after = &source[start..];
    let open_rel = after
        .find('{')
        .expect("block_from: no '{' found after the given start");
    let mut depth = 0i32;
    for (i, ch) in after[open_rel..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &after[..open_rel + i + 1];
                }
            }
            _ => {}
        }
    }
    panic!("block_from: unbalanced braces from index {start}");
}

// --- claiming focus -------------------------------------------------------

/// The property both focus-claiming paths share, and the SdTextField whose `focus-input()` they both
/// end up calling - the actual text-input focus claim `text-field.slint` defines as
/// `public function focus-input() { input.focus(); }`.
#[test]
fn a_shared_focus_target_property_exists_and_the_note_field_can_claim_real_focus() {
    let window = code_only(&read("ui/appwindow.slint"));
    assert!(
        window.contains("in-out property <string> marker-focus-target: \"\";"),
        "AppWindow must expose one property naming the Marker whose Note field should claim focus"
    );
    assert!(
        window.contains("marker-note-field := SdTextField {"),
        "the card's Note field must be reachable by id so a sibling handler can call \
         `focus-input()` on it"
    );

    let text_field = code_only(&read("ui/components/text-field.slint"));
    assert!(
        text_field.contains("public function focus-input() {")
            && text_field.contains("input.focus();"),
        "`focus-input()` must claim focus on the real `TextInput`, not merely toggle a property"
    );
}

/// Placing a new Marker: Rust must set the focus target (and switch on the tab the Note field lives
/// in) BEFORE reloading the Marker list, not after - `load_active_detail` replaces the whole
/// `markers` model, and the new row's own `init` only ever sees a value already in place by the time
/// it is created.
#[test]
fn placing_a_marker_sets_the_focus_target_and_tab_before_reloading() {
    let main = code_only(&read("src/main.rs"));
    let at = main
        .find("main_window.on_marker_placed(")
        .expect("on_marker_placed must be wired in main.rs");
    let handler = flat(block_from(&main, at));

    let tab_at = handler
        .find("win.set_active_tab_index(0)")
        .expect("the Marker Notes tab (index 0) must be selected so the Note field's card exists");
    let target_at = handler
        .find("win.set_marker_focus_target(marker_id")
        .expect("the newly placed Marker's own id must be handed to the focus-target property");
    let reload_at = handler
        .find("load_active_detail(&win, &ctx_mkp, &finding_id)")
        .expect("the reload that rebuilds the Marker Notes list must still happen");

    assert!(
        tab_at < reload_at && target_at < reload_at,
        "both the tab switch and the focus-target write must happen BEFORE the reload rebuilds the \
         list - after it is too late for the new row's `init` to see them"
    );
}

/// Clicking an EXISTING Marker on the canvas: the focus request must sit behind the same "a click is
/// not a drag" guard `drag-is-real`/`crop-drag-is-real` already use, and `marker-moved` - the drag
/// commit - must stay unconditional so an actual drag still repositions the Marker exactly as before.
#[test]
fn clicking_an_existing_marker_requests_focus_only_when_the_drag_was_not_real() {
    let window = code_only(&read("ui/appwindow.slint"));

    assert!(
        window.contains("private property <bool> marker-drag-is-real:"),
        "a per-Marker drag-reality guard must exist, mirroring `drag-is-real`/`crop-drag-is-real`"
    );

    let at = window
        .find("marker-touch := TouchArea {")
        .expect("the canvas Marker's own TouchArea must be reachable by id");
    let touch_area = flat(block_from(&window, at));

    // `marker-moved` must fire unconditionally on pointer-up while dragging - only ONE guard
    // (`parent.dragging`) wraps it, not `marker-drag-is-real` too. Changing that would be a
    // regression in the drag itself, which this ticket must not touch.
    let moved_idx = touch_area
        .find("root.marker-moved(m.id, parent.dx, parent.dy);")
        .expect("marker-moved must still fire on release");
    let guard_idx = touch_area
        .find("if (!parent.marker-drag-is-real) {")
        .expect("the focus request must be wrapped in the click-not-drag guard");
    assert!(
        moved_idx < guard_idx,
        "marker-moved must fire BEFORE and OUTSIDE the click-not-drag guard, so a real drag still \
         moves the Marker"
    );

    // Both the tab switch and the focus-target write must be INSIDE that guard.
    let guard_body = flat(block_from(&touch_area, guard_idx));
    assert!(
        guard_body.contains("root.active-tab-index = 0;"),
        "opening an existing Marker must switch to the Marker Notes tab - the tab the Note field's \
         card lives in"
    );
    assert!(
        guard_body.contains("root.marker-focus-target = m.id;"),
        "and it must name THIS Marker as the one whose Note field should claim focus"
    );

    // The threshold is measured the same way the canvas-wide guards already are - image pixels, not
    // canvas pixels, so it does not drift with `canvas-zoom` (`FR-34`).
    let guard_decl_at = window
        .find("private property <bool> marker-drag-is-real:")
        .unwrap();
    let decl = flat(&window[guard_decl_at..(guard_decl_at + 300).min(window.len())]);
    assert!(
        decl.contains("root.active-image.width") && decl.contains("root.active-image.height"),
        "the drag-reality threshold must be measured in image pixels, exactly like \
         `drag-is-real`/`crop-drag-is-real` above it"
    );
}

/// The Marker Notes card claims focus from BOTH the moment it is created with the request already
/// pending (a freshly placed Marker - `init`) and while it already exists and the request arrives
/// later (an existing Marker clicked open - `changed`), and consumes the request either way so an
/// unrelated later reload cannot steal focus back to a stale target.
#[test]
fn the_note_field_claims_focus_on_both_creation_and_a_later_request_and_consumes_it() {
    let window = code_only(&read("ui/appwindow.slint"));

    let at = window
        .find("for marker in root.markers : Rectangle {")
        .expect("the Marker Notes card loop must exist");
    let card = flat(block_from(&window, at));

    assert!(
        card.contains("property <string> focus-target-mirror <=> root.marker-focus-target;"),
        "watching an ancestor's property from inside a `for` item needs a local two-way alias - \
         `changed` only accepts an unqualified identifier owned by the element it is declared on, \
         the same constraint `settings.slint`'s `*-mirror` properties document"
    );

    let init_at = card.find("init => {").expect("an init handler must exist");
    let init_body = flat(block_from(&card, init_at));
    assert!(
        init_body.contains("if (root.marker-focus-target == marker.id) {")
            && init_body.contains("marker-note-field.focus-input();")
            && init_body.contains("root.marker-focus-target = \"\";"),
        "a freshly created card (the just-placed Marker's own row) must claim focus for ITSELF \
         only, then clear the request"
    );

    let changed_at = card
        .find("changed focus-target-mirror => {")
        .expect("a changed handler on the mirror must exist");
    let changed_body = flat(block_from(&card, changed_at));
    assert!(
        changed_body.contains("if (root.marker-focus-target == marker.id) {")
            && changed_body.contains("marker-note-field.focus-input();")
            && changed_body.contains("root.marker-focus-target = \"\";"),
        "an already-existing card (an existing Marker clicked open) must claim focus the same way \
         when the request later names it, and clear the request the same way"
    );
}

// --- hover tooltip ---------------------------------------------------------

/// The tooltip's text must be the hovered Marker's OWN `comment`, read from the same `for m in
/// root.markers` loop the tooltip is declared inside - decoding the actual binding, not merely
/// finding the substring "m.comment" somewhere unrelated in the file.
#[test]
fn the_hover_tooltip_reads_the_hovered_markers_own_bound_note_text() {
    let window = code_only(&read("ui/appwindow.slint"));

    let for_at = window
        .find("for m in root.markers : Rectangle {")
        .expect("the canvas Marker overlay loop must exist");
    let for_body = block_from(&window, for_at);

    // The tooltip must live INSIDE this exact loop, so `m` and `marker-touch` resolve to THIS
    // iteration's own Marker - never to a window-wide "currently hovered" property that could still
    // hold a previous Marker's text after the pointer moves off it.
    let tooltip_at = for_body
        .find("if marker-touch.has-hover && m.comment != \"\" : Rectangle {")
        .expect(
            "the hover tooltip must be gated on THIS Marker's own hover state and its own note",
        );
    let tooltip = flat(block_from(for_body, tooltip_at));

    assert!(
        tooltip.contains("text: m.comment;"),
        "the tooltip's Text must bind to `m.comment` - the hovered Marker's own model field, decoded \
         from the loop variable, not a copy of any test input"
    );

    assert!(
        !window.contains("property <string> hovered-marker") && !window.contains("hovered_marker"),
        "no window-wide \"currently hovered Marker\" property may exist - that shape is exactly what \
         would go stale between one Marker's hover and the next"
    );
}

/// An empty note shows no tooltip at all - the builder's chosen half of the two options the ticket
/// allows - so nothing can ever render as a blank plate over the screenshot.
#[test]
fn an_empty_note_shows_no_tooltip() {
    let window = code_only(&read("ui/appwindow.slint"));
    assert!(
        window.contains("if marker-touch.has-hover && m.comment != \"\" : Rectangle {"),
        "the tooltip's own visibility condition must exclude an empty note"
    );
}
