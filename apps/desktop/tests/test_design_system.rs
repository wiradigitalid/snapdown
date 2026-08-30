//! One design system, composed from one component set.
//!
//! `.constitution/project/design-system-guide.md` opens by saying the owner raised design-system
//! inconsistency FIVE separate times in one week, and that each time the instance was fixed and the
//! cause was left. The sixth was the Settings screen: its header, its toggles and its preset row all
//! disagreed with the Assemble & Review modal that had shipped days earlier.
//!
//! The rule the guide states is the one asserted here: *"Two places showing the same kind of thing
//! MUST reference the same component, not the same numbers. Matching numbers drift the first time one
//! is touched."* These tests fail when a second copy of a treatment appears, which is the moment
//! drift becomes possible - not later, when it has already happened.

use std::fs;
use std::path::Path;

fn ui(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("ui")
        .join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// The source with `//` comment lines removed.
///
/// A guard that asserts a string is ABSENT will otherwise match the comment explaining why it was
/// removed - which makes the comment unwritable, and an unexplained removal is how the next reader
/// puts it back. This has now caught the same test twice.
fn code_only(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join(
            "
",
        )
}

/// Both modals wear the same header.
#[test]
fn every_modal_header_is_the_same_component() {
    let window = ui("appwindow.slint");
    let settings = ui("components/settings.slint");

    assert!(
        window.contains("SdModalHeader {"),
        "Assemble & Review must use the shared header"
    );
    assert!(
        settings.contains("SdModalHeader {"),
        "and so must Settings. It grew its own - 700-weight title, no icon, a Done button where the \
         close is - which is the drift the guide describes"
    );

    // Neither may hand-write one again. A 52px bar with a bottom hairline and a title is the header,
    // whatever it is called locally.
    for (name, source) in [("appwindow.slint", &window), ("settings.slint", &settings)] {
        let hand_written = source.matches("height: 52px;").count();
        let shared = source.matches("SdModalHeader {").count();
        assert!(
            hand_written <= shared,
            "{name} has {hand_written} 52px bar(s) and {shared} SdModalHeader(s). A hand-written \
             header is how the two came to differ in the first place"
        );
    }
}

/// The segmented treatment exists once.
#[test]
fn every_segmented_control_is_the_same_component() {
    let window = ui("appwindow.slint");
    let settings = ui("components/settings.slint");

    let uses = window.matches("SdSegmented {").count() + settings.matches("SdSegmented {").count();
    assert!(
        uses >= 4,
        "the segmented treatment appears in Edit/Preview, the Quality Budget presets, the font \
         family pair and the alignment triple - that is four, and only {uses} go through the \
         component"
    );

    // The hand-drawn versions are gone, each identified by the local touch-area id it needed.
    for gone in [
        "edit-tab :=",
        "code-tab :=",
        "fam-touch :=",
        "align-touch :=",
        "preset-touch :=",
    ] {
        assert!(
            !window.contains(gone) && !settings.contains(gone),
            "`{gone}` belonged to a hand-drawn segmented control. Four of them existed, and they had \
             already drifted in height, radius and label size"
        );
    }
}

/// A pill is a status chip, never a control.
#[test]
fn radius_pill_is_only_used_for_status_and_never_for_a_control() {
    // The guide: "Corner radius: `Theme.radius-sharp` for every control. `radius-pill` only for a
    // status chip, never a button." The first Settings build drew its toggles as pill-shaped
    // switches - the one control in the product with a shape nothing else has, which is exactly what
    // the owner reacted to.
    let controls = code_only(&ui("components/form-controls.slint"));
    assert!(
        controls.contains("border-radius: Theme.radius-sharp;"),
        "the checkbox must be radius-sharp like every other control"
    );
    assert!(
        !controls.contains("radius-pill"),
        "no control in `form-controls.slint` may be pill-shaped"
    );

    let settings = code_only(&ui("components/settings.slint"));
    // The `DEC-004` chip is the one legitimate pill on that screen, and the hotkey status dot.
    let pills = settings.matches("radius-pill").count();
    assert!(
        pills <= 2,
        "{pills} pills on the Settings screen. Only the DEC-004 chip and the registration dot \
         qualify - both are status, neither is a control"
    );
}

/// A checkbox and a tick, not a glyph.
#[test]
fn the_checkbox_draws_its_tick_rather_than_typing_it() {
    let controls = ui("components/form-controls.slint");
    assert!(
        controls.contains("commands: \"M 0.5 4.5 L 3.3 7.3 L 8.5 1.4\""),
        "the tick must be drawn. A bare glyph falls back to whatever the font has, and this product \
         has had to replace two of them for exactly that reason - the Marker delete affordance and \
         the modal close"
    );
}

/// Settings follows its own G3 design, which is where the answers came from.
#[test]
fn the_settings_screen_follows_the_designed_structure() {
    let settings = ui("components/settings.slint");

    // Horizontal tabs, as designed and as the inspector already does - not a left sidebar.
    assert!(
        settings.contains("height: 40px;") && settings.contains("HorizontalLayout {"),
        "the tabs belong in a horizontal row under the header"
    );
    // Each group in a card, not separated by bare dividers.
    assert!(
        settings.matches("SdCard {").count() >= 5,
        "every settings group belongs in a sunken card - `06a-settings-general.html` puts each one \
         in one, and the first build used dividers instead"
    );
    // The primary action in a footer, which is where the design has it.
    //
    // Labelled "Close", not "Done": Settings auto-saves every change as it happens (the caption
    // beside this button says so), so there was never a pending action for "Done" to finish.
    let footer = settings
        .rfind("label: \"Close\";")
        .expect("a Close action must exist");
    let header = settings
        .find("SdModalHeader {")
        .expect("the header must exist");
    assert!(
        footer > header,
        "Close belongs in the FOOTER, below the panels. It was in the top-right corner, where the \
         design and the other modal both put the close"
    );
}
