//! `FR-34` is REACHABLE, not merely built — the same convention `test_annotation_wiring.rs`
//! establishes and `codebase-conventions-guide.md` names: a component that compiles and is
//! instantiated nowhere, or a callback with no real handler behind it, is this repository's
//! signature failure (`BUG-4`, `BUG-5`, `BUG-6`, `BUG-72`).
//!
//! Two things this file asserts:
//!
//! 1. the zoom controls are instantiated in `appwindow.slint`, and `canvas-viewport` — the actual
//!    viewport every annotation and Marker is drawn against — reads the zoom property in its size
//!    expression, not a decorative property nothing consumes;
//! 2. each of the three zoom callbacks has a real Rust handler that changes `canvas-zoom`, not a
//!    stub.

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// The same source with every run of whitespace collapsed to one space.
///
/// Needed because `rustfmt` decides where a method chain breaks, and a guard a reformat can turn
/// red is a guard nobody keeps — the same reason `test_annotation_wiring.rs` carries this helper.
fn flat(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The source with `//` comment lines removed, same as `test_annotation_wiring.rs`'s helper.
///
/// Needed here because a real doc comment sits between two statements this file asserts are
/// adjacent — without stripping it, `flat()` alone would fold the comment's own words into the gap
/// and the exact-adjacency check below would never match.
fn code_only(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_three_zoom_callbacks_are_declared() {
    let window = read("ui/appwindow.slint");
    assert!(
        window.contains("callback zoom-in-clicked();"),
        "zoom-in-clicked must be declared on AppWindow"
    );
    assert!(
        window.contains("callback zoom-out-clicked();"),
        "zoom-out-clicked must be declared on AppWindow"
    );
    assert!(
        window.contains("callback zoom-reset-clicked();"),
        "zoom-reset-clicked must be declared on AppWindow"
    );
}

#[test]
fn canvas_zoom_is_a_window_property_the_toolbar_and_rust_can_both_reach() {
    let window = read("ui/appwindow.slint");
    assert!(
        window.contains("in-out property <float> canvas-zoom: 1.0;"),
        "canvas-zoom must be an AppWindow property Rust can read and set, defaulting to natural size"
    );
}

/// The transform actually reaches the viewport everything else is drawn against. A `canvas-zoom`
/// property that compiles and is read by nothing would be exactly `BUG-4`'s shape again.
#[test]
fn canvas_viewport_size_is_driven_by_canvas_zoom() {
    let window = flat(&read("ui/appwindow.slint"));
    assert!(
        window.contains(
            "width: root.active-image.width * 1px * root.canvas-zoom; height: root.active-image.height * 1px * root.canvas-zoom;"
        ),
        "canvas-viewport's own width/height must multiply the natural-size expression by \
         root.canvas-zoom — every annotation and Marker below it reads a fraction of THIS \
         rectangle, so this one binding is what makes zoom reach them all"
    );
}

#[test]
fn the_three_zoom_buttons_are_mounted_in_the_canvas_action_group() {
    let window = flat(&read("ui/appwindow.slint"));
    assert!(
        window.contains("clicked => { root.zoom-out-clicked(); }"),
        "a control must fire zoom-out-clicked"
    );
    assert!(
        window.contains("clicked => { root.zoom-reset-clicked(); }"),
        "a control must fire zoom-reset-clicked"
    );
    assert!(
        window.contains("clicked => { root.zoom-in-clicked(); }"),
        "a control must fire zoom-in-clicked"
    );
}

/// The handler body for each zoom callback actually changes `canvas-zoom` — not a stub. This is
/// deliberately a body-content check, not just a "handler exists" check: a `println!`-only body
/// would satisfy `test_ui_callbacks_reach_rust.rs`'s reachability test as `KNOWN_STUBS`-eligible,
/// but zoom is exactly the feature being built here, so nothing on this path may be a stub.
fn handler_body(main_rs: &str, handler: &str) -> String {
    let at = main_rs
        .find(&format!("{handler}("))
        .unwrap_or_else(|| panic!("{handler} must exist in main.rs"));
    let end = main_rs[at..]
        .find("\n    });")
        .map(|offset| at + offset)
        .unwrap_or_else(|| (at + 400).min(main_rs.len()));
    flat(&main_rs[at..end])
}

#[test]
fn zoom_in_and_zoom_out_handlers_call_the_pure_step_functions() {
    let main_rs = read("src/main.rs");
    assert!(
        handler_body(&main_rs, "on_zoom_in_clicked").contains("set_canvas_zoom(zoomed_in("),
        "on_zoom_in_clicked must compute the next value with zoomed_in and push it back"
    );
    assert!(
        handler_body(&main_rs, "on_zoom_out_clicked").contains("set_canvas_zoom(zoomed_out("),
        "on_zoom_out_clicked must compute the next value with zoomed_out and push it back"
    );
}

#[test]
fn zoom_reset_handler_sets_exactly_natural_size() {
    let main_rs = read("src/main.rs");
    assert!(
        handler_body(&main_rs, "on_zoom_reset_clicked").contains("set_canvas_zoom(1.0)"),
        "on_zoom_reset_clicked must set canvas-zoom to exactly 1.0 — \"natural size\""
    );
}

/// Opening a different Finding must not leave a stale zoom level behind — view state belongs to
/// the view being looked at.
#[test]
fn loading_a_finding_resets_zoom_to_natural_size() {
    let main_rs = flat(&code_only(&read("src/main.rs")));
    assert!(
        main_rs.contains(
            "window.set_active_finding_id(f.id.clone().into()); window.set_canvas_zoom(1.0);"
        ),
        "load_active_detail must reset canvas-zoom to 1.0 right after setting the active Finding, \
         so a Finding never opens still zoomed from whatever was viewed before it"
    );
}
