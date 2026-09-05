//! Ticket 03 (`.scratch/post-testing-polish/issues/03-second-assemble-button-and-filmstrip-alignment.md`),
//! `FR-10`: a second, closer door to `assemble-bundle-clicked` near the canvas, plus an unrelated
//! visual fix to the filmstrip's own Assemble tile. Two different kinds of change, two test groups
//! below, kept apart so a failure names which one broke.
//!
//! The new button is not a new act: `AGENTS.md`'s "REACHABLE, not merely built" rule is about a
//! component with nowhere to draw from, and this is the opposite risk — a second control that looks
//! wired but fires nothing, or fires something that does not obey the same selection rule as every
//! existing Assemble door. Both are asserted directly against the source, the same shape
//! `test_annotation_wiring.rs` uses.

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// The same source with every run of whitespace collapsed to one space - `rustfmt` breaks a chain
/// wherever it likes, and a guard a reformat can turn red is a guard nobody keeps.
fn flat(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The new button lives in Group 3 (Right): Canvas Action, the ribbon row directly above the canvas -
/// "near the canvas's top-right" per the ticket - and it is INSTANTIATED there, not merely mentioned
/// in a comment recording the decision to add it.
#[test]
fn a_second_assemble_button_is_instantiated_beside_the_canvas() {
    let window = read("ui/appwindow.slint");
    let flat_window = flat(&window);

    let group_at = window
        .find("// Group 3 (Right): Canvas Action.")
        .expect("Group 3, the canvas-side ribbon group ticket 19 originally kept Assemble out of");
    let group = &window[group_at..(group_at + 6000).min(window.len())];

    assert!(
        group.contains("canvas-assemble-button := SdActionButton {"),
        "a second Assemble control must be INSTANTIATED in the canvas-side ribbon group, not just \
         referenced in a comment about the decision to add it"
    );
    assert!(
        group.contains(r#"label: "Assemble";"#),
        "and it must actually read \"Assemble\", the same word every other door uses"
    );

    // The instantiation must be the same distance as its ticked-selection gate below - close enough
    // together that they are plainly the same control, the way the filmstrip tile's own instantiation
    // and `enabled:` line sit a few lines apart.
    let button_at = flat_window
        .find("canvas-assemble-button := SdActionButton {")
        .expect("flattened source must still contain the instantiation");
    let button_body = &flat_window[button_at..(button_at + 400).min(flat_window.len())];

    assert!(
        button_body.contains("enabled: root.selected-finding-count > 0 ;")
            || button_body.contains("enabled: root.selected-finding-count > 0;"),
        "the new button must be gated by the SAME `selected-finding-count > 0` rule the filmstrip \
         tile and the context menu's `assemble` entry already use - reading option (a) from the \
         spec, not (b): nothing here ticks the active Finding on the Reviewer's behalf"
    );
    assert!(
        button_body.contains("clicked => { root.assemble-bundle-clicked(); }"),
        "and clicking it must fire the exact SAME `assemble-bundle-clicked` callback every other \
         Assemble door fires - a second door to the identical act, not a new one"
    );
}

/// The callback it fires already has a Rust handler (`test_ui_callbacks_reach_rust.rs` covers that
/// generally); this asserts the specific join this ticket depends on still holds: one handler, one
/// refusal path, for every door including the new one.
#[test]
fn the_callback_the_new_button_fires_is_handled_by_the_one_existing_assemble_handler() {
    let main = flat(&read("src/main.rs"));

    assert!(
        main.contains("main_window.on_assemble_bundle_clicked("),
        "the callback the new button fires must already be handled - this ticket adds a door, not a \
         second handler"
    );
    assert!(
        main.contains("fn prepare_bundle (") || main.contains("fn prepare_bundle("),
        "and that handler must still route through `prepare_bundle`, which is the one place the \
         ticked-selection refusal (\"Tick at least one Finding in the strip first.\") lives - the new \
         button must not carry a copy of that message"
    );
}

/// The filmstrip alignment fix. Plain visual defect, no behaviour change: the sticky Assemble tile
/// sits at `y: 12px`, unpadded, height `86px`. The filmstrip's own cards are the same fixed `86px`
/// tall, inside a `HorizontalLayout` of fixed height `110px` with `padding-top: 12px` and, before this
/// fix, no matching `padding-bottom` - so Slint centred the 86px-tall cards in the 98px left over
/// after the top inset alone (110 - 12), landing them 6px lower than the tile beside them. Declaring
/// `padding-bottom: 12px` too makes the available cross-axis space exactly 86px (110 - 12 - 12), the
/// card's own height, leaving no slack to centre away - which is what actually moves the row, not a
/// number chosen to look right in isolation.
#[test]
fn the_filmstrip_row_no_longer_has_slack_to_center_away_from_the_assemble_tile() {
    let window = read("ui/appwindow.slint");

    let scroll_at = window
        .find("filmstrip-scroll := ScrollView {")
        .expect("the filmstrip's ScrollView must exist");
    let scroll = &window[scroll_at..(scroll_at + 2200).min(window.len())];

    assert!(
        scroll.contains("height: 110px;") && scroll.contains("padding-top: 12px;"),
        "the filmstrip row's fixed height and top inset are the two numbers the tile beside it \
         already uses (12px margin, matched against the tile's own `y: 12px`) - this test must be \
         reading the right layout before asserting the fix"
    );
    assert!(
        scroll.contains("padding-bottom: 12px;"),
        "the row needs an explicit `padding-bottom: 12px` to match its own `padding-top`. Without it \
         the 86px-tall cards have 12px of slack (110 - 12 top inset - 86 card height) that Slint \
         spends centring them, landing the row 6px below the sticky Assemble tile it sits beside - a \
         plain visual misalignment, not a behaviour change"
    );

    // The card height and the tile height must still agree - the fix is the padding, not a new card
    // size that would happen to look aligned for a different reason.
    assert!(
        window.contains("width: 120px;\n                                        height: 86px;")
            || flat(&window).contains("width: 120px; height: 86px;"),
        "the filmstrip cards must still be 86px tall - the same height the sticky Assemble tile \
         already declares. Changing this number instead of the padding would be moving the defect, \
         not fixing it"
    );
}
