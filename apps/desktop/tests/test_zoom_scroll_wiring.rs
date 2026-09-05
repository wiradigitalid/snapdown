//! `01-canvas-zoom-ctrl-scroll.md` (`FR-34`'s second input path). Same convention
//! `test_zoom_wiring.rs` and `test_annotation_wiring.rs` establish: a handler that compiles and does
//! nothing real behind it is this repository's signature failure (`BUG-4`, `BUG-5`, `BUG-6`,
//! `BUG-72`).
//!
//! This file does not simulate an actual OS scroll-wheel event — there is no Slint testing backend
//! wired into this crate's `dev-dependencies` (`apps/desktop/Cargo.toml` has none), so, like its two
//! siblings, this is a structural check over the compiled-in source rather than a runtime one. What
//! it decodes is the STRUCTURE of the one `scroll-event` handler declared inside `canvas-surface`'s
//! `TouchArea` (found by brace-matching, not a fixed-offset slice, so a reformat cannot silently
//! point it at the wrong handler): the Ctrl branch must call `zoom-in-clicked`/`zoom-out-clicked`
//! and `accept`, and the non-Ctrl branch must do neither and must `reject` — the same shape an
//! absent handler already has today, which is how a plain Scroll keeps reaching `scroll-view`'s own
//! panning unchanged. A test that only grepped the whole file for "modifiers.control" and
//! "zoom-in-clicked" anywhere would pass even if the two were unrelated, or if the branches were
//! swapped, or if the non-Ctrl branch also zoomed — this one cannot.

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// The `canvas-surface := TouchArea { ... }` block, extracted by counting braces from its own
/// opening one rather than trusting a fixed offset — the block contains several other handlers
/// (`pointer-event`, `moved`, `clicked`) before the one this file cares about.
fn canvas_surface_block(window: &str) -> &str {
    let marker = "canvas-surface := TouchArea {";
    let start = window
        .find(marker)
        .unwrap_or_else(|| panic!("canvas-surface := TouchArea {{ must exist in appwindow.slint"));
    let open_brace = start + marker.len() - 1;
    let body_start = open_brace + 1;
    let mut depth = 1i32;
    for (offset, ch) in window[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &window[body_start..body_start + offset];
                }
            }
            _ => {}
        }
    }
    panic!("canvas-surface's TouchArea block never closes its opening brace");
}

/// The `scroll-event(ev) => { ... }` handler inside `canvas_surface_block`, same brace-matching
/// approach — this handler's own body contains nested `if`/`else` braces, so a fixed-offset slice
/// (the way `test_zoom_wiring.rs`'s `handler_body` works for a single-statement Rust callback)
/// would not land on the right closing brace here.
fn scroll_event_handler_body(surface_block: &str) -> &str {
    let marker = "scroll-event(ev) => {";
    let start = surface_block.find(marker).unwrap_or_else(|| {
        panic!(
            "canvas-surface's TouchArea must declare a scroll-event(ev) handler for Ctrl+Scroll zoom"
        )
    });
    let open_brace = start + marker.len() - 1;
    let body_start = open_brace + 1;
    let mut depth = 1i32;
    for (offset, ch) in surface_block[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &surface_block[body_start..body_start + offset];
                }
            }
            _ => {}
        }
    }
    panic!("canvas-surface's scroll-event handler never closes its opening brace");
}

/// Splits a handler body into (if-control branch, else branch) on the FIRST top-level
/// `} else {` — a plain `.find("} else {")` on the whole file would risk matching some other
/// if/else elsewhere; scoping the search to the already brace-matched handler body avoids that.
fn split_branches(body: &str) -> (&str, &str) {
    let at = body.find("} else {").unwrap_or_else(|| {
        panic!(
            "the scroll-event handler must branch on ev.modifiers.control with an if/else, not a \
             single unconditional path — got: {body}"
        )
    });
    (&body[..at], &body[at + "} else {".len()..])
}

#[test]
fn ctrl_scroll_branch_checks_the_control_modifier() {
    let window = read("ui/appwindow.slint");
    let surface = canvas_surface_block(&window);
    let handler = scroll_event_handler_body(surface);
    assert!(
        handler.contains("ev.modifiers.control"),
        "the scroll-event handler must gate zooming on ev.modifiers.control, not fire on every \
         scroll — got: {handler}"
    );
}

#[test]
fn ctrl_scroll_up_calls_zoom_in_clicked_not_a_reimplementation() {
    let window = read("ui/appwindow.slint");
    let surface = canvas_surface_block(&window);
    let handler = scroll_event_handler_body(surface);
    let (control_branch, _) = split_branches(handler);
    assert!(
        control_branch.contains("ev.delta-y < 0"),
        "scrolling up (delta-y < 0, this file's own convention at the target-level wheel handler \
         above) must be the zoom-in direction — got: {control_branch}"
    );
    assert!(
        control_branch.contains("root.zoom-in-clicked()"),
        "Ctrl+Scroll up must fire the EXISTING zoom-in-clicked callback — the same one the toolbar \
         button fires, which main.rs already wires to set_canvas_zoom(zoomed_in(...)) — not a \
         second implementation of the clamp/step arithmetic — got: {control_branch}"
    );
}

#[test]
fn ctrl_scroll_down_calls_zoom_out_clicked_not_a_reimplementation() {
    let window = read("ui/appwindow.slint");
    let surface = canvas_surface_block(&window);
    let handler = scroll_event_handler_body(surface);
    let (control_branch, _) = split_branches(handler);
    assert!(
        control_branch.contains("ev.delta-y > 0"),
        "scrolling down (delta-y > 0) must be the zoom-out direction — got: {control_branch}"
    );
    assert!(
        control_branch.contains("root.zoom-out-clicked()"),
        "Ctrl+Scroll down must fire the EXISTING zoom-out-clicked callback — the same one the \
         toolbar button fires, which main.rs already wires to set_canvas_zoom(zoomed_out(...)) — \
         got: {control_branch}"
    );
}

#[test]
fn ctrl_scroll_branch_consumes_the_event() {
    let window = read("ui/appwindow.slint");
    let surface = canvas_surface_block(&window);
    let handler = scroll_event_handler_body(surface);
    let (control_branch, _) = split_branches(handler);
    assert!(
        control_branch.trim_end().ends_with("accept"),
        "the Ctrl branch must end by accepting the event — otherwise it would also fall through \
         to scroll-view's panning while zooming — got: {control_branch}"
    );
    assert!(
        !control_branch.contains("reject"),
        "the Ctrl branch must never reject — got: {control_branch}"
    );
}

/// The other half of the seam: a plain Scroll (no Ctrl) must change nothing about zoom, and must
/// leave the event for `scroll-view` to keep panning with — precisely what an ABSENT handler would
/// already do here, which is how this exact wheel input reaches the canvas today.
#[test]
fn plain_scroll_branch_touches_neither_zoom_callback_and_rejects() {
    let window = read("ui/appwindow.slint");
    let surface = canvas_surface_block(&window);
    let handler = scroll_event_handler_body(surface);
    let (_, else_branch) = split_branches(handler);
    assert!(
        !else_branch.contains("zoom-in-clicked") && !else_branch.contains("zoom-out-clicked"),
        "a plain Scroll must not call either zoom callback — canvas-zoom must be untouched by it — \
         got: {else_branch}"
    );
    // `else_branch` runs to the END of the already brace-matched handler body, so unlike
    // `control_branch` (which stops right before its own closing `}`) it still carries the
    // else-block's own closing brace at the tail. Strip exactly that one delimiter — not part of
    // the statement itself — before checking the branch is nothing but `reject`.
    let else_statement = else_branch
        .trim_end()
        .strip_suffix('}')
        .unwrap_or_else(|| panic!("the else branch must close its own brace — got: {else_branch}"))
        .trim();
    assert_eq!(
        else_statement, "reject",
        "a plain Scroll's else branch must be exactly `reject` — the same outcome an absent \
         handler already produces, so it keeps falling through to scroll-view's own panning \
         unchanged — got: {else_branch}"
    );
}
