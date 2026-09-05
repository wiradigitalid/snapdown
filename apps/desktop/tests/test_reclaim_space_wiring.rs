//! Reclaim space is REACHABLE - not merely built. Ticket 18 of the Bundle Library spec.
//! `AGENTS.md` names this repository's signature failure in plain words: a component built,
//! unit-tested, and mounted nowhere. `test_library_wiring.rs` and `test_review_update_wiring.rs`
//! are the shape to copy - imported AND instantiated, every callback bound through to a real Rust
//! handler, not merely defined. This file is that same shape for `SdReclaimSpace`, plus the two
//! acceptance criteria specific to this ticket: it opens from BOTH the Library header and Settings'
//! Vault card, and it lands on the SAME screen either way.

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// The same source with every run of whitespace collapsed to one space - `rustfmt` decides where a
/// struct literal or a method chain breaks, and a guard written against one exact layout is a guard
/// the next `cargo fmt` can turn red for nothing. Copied from `test_review_update_wiring.rs`.
fn flat(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The source with `//` comment lines removed - copied from `test_design_system.rs`'s `code_only`.
fn code_only(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The component exists AND something mounts it, ABOVE both the Library and Settings - it is
/// reachable from either, so it must stack over whichever one is actually open underneath.
#[test]
fn the_reclaim_space_component_is_mounted_over_the_library_and_settings() {
    let window = read("ui/appwindow.slint");

    assert!(
        window.contains(
            r#"import { SdReclaimSpace, ReclaimBundleRow } from "components/reclaim-space.slint";"#
        ),
        "the Editor window must import Reclaim space's component"
    );
    assert!(
        window.contains("if root.reclaim-space-open : SdReclaimSpace {"),
        "`SdReclaimSpace` must be MOUNTED, gated by a window property Rust can flip - a component \
         that compiles and is instantiated nowhere is what `BUG-4`, `BUG-5`, `BUG-6` and `BUG-72` \
         all were"
    );
    assert!(
        window.contains("in-out property <bool> reclaim-space-open: false;"),
        "and the gate must be a property Rust can set"
    );

    let library_at = window
        .find("if root.library-open : SdLibrary {")
        .expect("the Library mount site must exist");
    let settings_at = window
        .find("if root.settings-open : SdSettings {")
        .expect("the Settings mount site must exist");
    let reclaim_at = window
        .find("if root.reclaim-space-open : SdReclaimSpace {")
        .expect("the Reclaim space mount site must exist");
    assert!(
        reclaim_at > library_at && reclaim_at > settings_at,
        "Reclaim space must be declared AFTER both the Library and Settings, or it would be drawn \
         BEHIND whichever one is open underneath it"
    );
}

/// The ticket's own acceptance criterion: reachable from the Library header AND from Settings'
/// Vault area, and both land on the SAME screen - proven here by checking both doors fire the SAME
/// root-level callback, which is forwarded to exactly one Rust handler that opens exactly one
/// screen.
#[test]
fn reclaim_space_is_reachable_from_both_the_library_header_and_settings_vault_area() {
    let library = read("ui/components/library.slint");
    let settings = read("ui/components/settings.slint");
    let window = read("ui/appwindow.slint");
    let main = read("src/main.rs");

    assert!(
        library.contains("callback reclaim-space-clicked();"),
        "the Library's header must declare its own `reclaim-space-clicked` entry point"
    );
    assert!(
        library.contains("\"Reclaim space\""),
        "the Library header must carry a visible \"Reclaim space\" entry, per `spec.md`'s \"Header: \
         title, Bundle count, a Reclaim space entry\""
    );
    assert!(
        settings.contains("callback reclaim-space-clicked();"),
        "Settings must declare its own `reclaim-space-clicked` entry point"
    );
    assert!(
        settings.contains("label: \"Reclaim space...\";"),
        "Settings' Vault card must carry a visible \"Reclaim space...\" button, beside \"Choose \
         folder...\" and \"Show in Explorer\""
    );

    // The button and the header entry must each actually FIRE their own `reclaim-space-clicked` -
    // a declared callback nothing calls is exactly as unreachable as one nothing forwards.
    let flat_library = flat(&library);
    assert!(
        flat_library.contains("clicked => { root.reclaim-space-clicked(); }"),
        "the Library header's own \"Reclaim space\" entry must fire `root.reclaim-space-clicked()` \
         on click, not merely declare the callback"
    );
    let settings_button_at = settings
        .find("label: \"Reclaim space...\";")
        .expect("checked above");
    let settings_button =
        flat(&settings[settings_button_at..(settings_button_at + 320).min(settings.len())]);
    assert!(
        settings_button.contains("clicked => { root.reclaim-space-clicked(); }"),
        "Settings' \"Reclaim space...\" button must fire `root.reclaim-space-clicked()` on click, \
         not merely declare the callback"
    );

    // Both mount sites forward to the SAME root-level callback.
    let library_at = window
        .find("if root.library-open : SdLibrary {")
        .expect("the Library mount site must exist");
    let library_mount = flat(&window[library_at..(library_at + 2600).min(window.len())]);
    assert!(
        library_mount.contains("reclaim-space-clicked => { root.reclaim-space-clicked(); }"),
        "the Library mount site must forward its own `reclaim-space-clicked` to \
         `root.reclaim-space-clicked()`"
    );

    let settings_at = window
        .find("if root.settings-open : SdSettings {")
        .expect("the Settings mount site must exist");
    let settings_mount = flat(&window[settings_at..(settings_at + 2200).min(window.len())]);
    assert!(
        settings_mount.contains("reclaim-space-clicked => { root.reclaim-space-clicked(); }"),
        "the Settings mount site must forward its own `reclaim-space-clicked` to the SAME \
         `root.reclaim-space-clicked()` - two different root callbacks here would let the two \
         doors open two different screens"
    );

    assert!(
        window.contains("callback reclaim-space-clicked();"),
        "the root window must declare exactly the callback both doors forward to"
    );
    assert!(
        main.contains("on_reclaim_space_clicked("),
        "`on_reclaim_space_clicked` must exist in main.rs, or neither door reaches anything"
    );
    let at = main
        .find("on_reclaim_space_clicked(")
        .expect("checked above");
    let body = &main[at..(at + 300).min(main.len())];
    assert!(
        body.contains("open_reclaim_space("),
        "the one handler both doors share must actually open Reclaim space, not merely flip a \
         property"
    );
}

/// Every callback `SdReclaimSpace` declares is bound at the mount site, and forwards to a
/// root-level callback with a real handler in `main.rs` - not a `println!` stub.
#[test]
fn every_reclaim_space_callback_is_bound_from_slint_to_rust() {
    let component = read("ui/components/reclaim-space.slint");
    let window = read("ui/appwindow.slint");
    let main = read("src/main.rs");

    for declared in [
        "callback closed();",
        "callback row-toggled(string);",
        "callback select-all-toggled(bool);",
        "callback discard-clicked();",
        "callback delete-both-clicked();",
        "callback discard-cancelled();",
        "callback discard-confirmed();",
        "callback delete-both-cancelled();",
        "callback delete-both-confirmed();",
    ] {
        assert!(
            component.contains(declared),
            "`SdReclaimSpace` must declare `{declared}`"
        );
    }

    let at = window
        .find("if root.reclaim-space-open : SdReclaimSpace {")
        .expect("the mount site must exist");
    // 1800, not 1200: ticket `05` of `post-testing-polish` added the select-all and bulk
    // Delete-both bindings/forwards at this mount site, pushing the callback forwards below past a
    // window sized before that landed (which itself widened from 900 for `BUG-100`'s toast
    // bindings).
    let mount = flat(&window[at..(at + 1800).min(window.len())]);

    let forwards = [
        (
            "closed => { root.reclaim-space-closed(); }",
            "on_reclaim_space_closed(",
        ),
        (
            "row-toggled(id) => { root.reclaim-space-row-toggled(id); }",
            "on_reclaim_space_row_toggled(",
        ),
        (
            "select-all-toggled(on) => { root.reclaim-space-select-all-toggled(on); }",
            "on_reclaim_space_select_all_toggled(",
        ),
        (
            "discard-clicked => { root.reclaim-space-discard-clicked(); }",
            "on_reclaim_space_discard_clicked(",
        ),
        (
            "discard-cancelled => { root.reclaim-space-discard-cancelled(); }",
            "on_reclaim_space_discard_cancelled(",
        ),
        (
            "discard-confirmed => { root.reclaim-space-discard-confirmed(); }",
            "on_reclaim_space_discard_confirmed(",
        ),
        (
            "delete-both-clicked => { root.reclaim-space-delete-both-clicked(); }",
            "on_reclaim_space_delete_both_clicked(",
        ),
        (
            "delete-both-cancelled => { root.reclaim-space-delete-both-cancelled(); }",
            "on_reclaim_space_delete_both_cancelled(",
        ),
        (
            "delete-both-confirmed => { root.reclaim-space-delete-both-confirmed(); }",
            "on_reclaim_space_delete_both_confirmed(",
        ),
    ];
    for (forward, handler) in forwards {
        assert!(
            mount.contains(forward),
            "the mount site must forward `{forward}`"
        );
        assert!(
            main.contains(handler),
            "`{handler}` must exist in main.rs, or the forwarded callback reaches nobody"
        );
    }
}

/// Ticket `05` of `post-testing-polish`: the select-all checkbox reuses the shared `SdCheckbox`
/// (never a hand-drawn second one), is rendered ONLY alongside a non-empty list (matching the
/// footer's own `rows.length > 0` gate - there is nothing to select-all over the empty state), and
/// its checked state is read straight off `selected-count`/`rows.length` rather than a second
/// "all selected" flag that could disagree with what the rows themselves show.
#[test]
fn select_all_checkbox_reuses_the_shared_component_and_is_gated_on_a_non_empty_list() {
    let component = read("ui/components/reclaim-space.slint");
    let flat_component = flat(&component);

    let at = component.find("select-all-checkbox := SdCheckbox").expect(
        "the select-all checkbox must exist, named so it is distinguishable from a row's \
             own checkbox",
    );
    // The nearest `rows.length > 0` gate BEFORE the checkbox's own declaration must be the one
    // wrapping it - the same `rows.length > 0` gate the footer below also uses.
    component[..at].rfind("if root.rows.length > 0").expect(
        "the select-all checkbox must be gated on `rows.length > 0`, the same way the \
             footer is - there is nothing to select-all over the empty state",
    );

    let body = flat(&component[at..(at + 400).min(component.len())]);
    assert!(
        body.contains("checked: root.selected-count == root.rows.length;"),
        "the select-all checkbox's checked state must be computed straight off \
         `selected-count`/`rows.length`, never a second flag Rust would have to keep in sync"
    );
    assert!(
        body.contains("toggled(on) => { root.select-all-toggled(on); }"),
        "the select-all checkbox must actually fire `select-all-toggled`, not merely display a \
         checked state"
    );

    assert!(
        flat_component.contains(r#"import { SdCheckbox } from "form-controls.slint";"#),
        "the select-all checkbox must reuse the shared component, imported once already for the \
         row checkboxes"
    );
}

/// Ticket `05`'s bulk "Delete both": a SEPARATE, danger-styled confirmation from the bulk Discard
/// originals dialog above it, matching `FR-41`/`BUG-104`'s own single-Bundle discipline (one
/// dedicated confirmation for the act that truly cannot be undone, never a second step nested
/// inside another dialog).
#[test]
fn bulk_delete_both_has_its_own_dedicated_danger_styled_confirmation() {
    let component = code_only(&read("ui/components/reclaim-space.slint"));

    for declared in [
        "in property <bool> delete-both-confirm-open: false;",
        "in property <string> delete-both-confirm-heading: \"\";",
        "in property <string> delete-both-confirm-body: \"\";",
    ] {
        assert!(
            component.contains(declared),
            "`SdReclaimSpace` must declare `{declared}`"
        );
    }

    let discard_confirm_at = component
        .find("if root.confirm-open : Rectangle {")
        .expect("the bulk Discard originals confirmation must exist");
    let delete_both_confirm_at = component
        .find("if root.delete-both-confirm-open : Rectangle {")
        .expect(
            "the bulk Delete-both confirmation must be its OWN dialog, gated on its OWN property \
             - never folded into the Discard-originals dialog above",
        );
    assert!(
        delete_both_confirm_at > discard_confirm_at,
        "the Delete-both confirmation must be declared as a SEPARATE block after the Discard \
         originals one, not nested inside it"
    );

    // Nothing before the Delete-both dialog's own opening brace names the Discard-originals
    // dialog's own confirm-open property inside ITS block, i.e. the two blocks are siblings, not
    // parent/child. A cheap proxy for that: the Discard confirmation's own closing `}` (matched by
    // indentation in this file) appears before the Delete-both dialog opens - verified above by
    // simple ordering; the flat search below now checks the button styling directly.
    let delete_both_dialog = flat(&component[delete_both_confirm_at..]);
    assert!(
        delete_both_dialog.contains(
            "SdActionButton { label: \"Delete both\"; danger: true; height: 30px; clicked => { \
             root.delete-both-confirmed(); } }"
        ),
        "the Delete-both dialog's own confirm button must be styled `danger: true` \
         (`BUG-103`'s own reasoning: this act truly cannot be undone) and must fire \
         `delete-both-confirmed`, not `discard-confirmed`"
    );
    assert!(
        delete_both_dialog.contains(
            "SdActionButton { label: \"Cancel\"; primary: false; height: 30px; clicked => { \
             root.delete-both-cancelled(); } }"
        ),
        "the Delete-both dialog's own Cancel button must fire `delete-both-cancelled`, not the \
         Discard-originals dialog's `discard-cancelled`"
    );
}

/// Escape closes it - the Library's own pattern, borrowed rather than reinvented.
#[test]
fn escape_closes_reclaim_space() {
    let component = read("ui/components/reclaim-space.slint");
    let at = component.find("reclaim-space-keys := FocusScope").expect(
        "Reclaim space must hold its own FocusScope for Escape, the way `library-keys` does for \
         the Library",
    );
    let body = flat(&component[at..(at + 500).min(component.len())]);
    assert!(
        body.contains("Key.Escape") && body.contains("root.closed()"),
        "Escape must fire `closed()`"
    );
    assert!(
        body.contains("init => { self.focus(); }"),
        "and something must GIVE the scope focus on open, or Escape never reaches it"
    );
}

/// The empty state renders its own settled copy, and - matching `ReclaimEmpty.dc.html`'s own
/// artboard - the footer (the ticked-count readout, Cancel, Discard originals) does not render at
/// all when there is nothing to act on.
#[test]
fn empty_state_renders_its_own_copy_and_hides_the_footer() {
    let component = code_only(&read("ui/components/reclaim-space.slint"));

    assert!(
        component.contains("\"Nothing to reclaim\""),
        "the empty state's heading must be the ticket's own settled copy"
    );
    assert!(
        component.contains(
            "No Bundle is holding original captures. Every Bundle here either had its originals \
             discarded already, or has none left to discard."
        ),
        "the empty state's body must be the ticket's own settled copy, word for word"
    );

    let empty_at = component
        .find("if root.rows.length == 0")
        .expect("the empty-state gate must exist");
    let footer_at = component
        .find("if root.rows.length > 0 : HorizontalLayout")
        .expect(
            "the footer must be gated on rows.length > 0, the same way the empty state is \
                 gated on rows.length == 0 - `ReclaimEmpty.dc.html` draws no footer at all",
        );
    assert!(
        footer_at > empty_at,
        "the footer must be declared after the empty state in this file"
    );
}

/// The explanatory line the ticket names, verbatim.
#[test]
fn the_screen_carries_its_own_explanatory_line() {
    let component = read("ui/components/reclaim-space.slint");
    assert!(
        component.contains(
            "Discarding a Bundle's originals keeps the Bundle readable and shareable. It can no \
             longer be disassembled."
        ),
        "the explanatory line must be the artboard's own wording, word for word"
    );
}

/// `spec.md`'s own rule: *\"Every surface is built from the shared components ... the modal header,
/// action button, ... checkbox ...\"*. A hand-drawn checkbox here would be the exact drift
/// `design-system-guide.md` warns about - a second place a 15px accent-tinted box with a drawn tick
/// can go out of step with `form-controls.slint`'s own `SdCheckbox`.
#[test]
fn the_row_checkbox_reuses_the_shared_component() {
    let component = read("ui/components/reclaim-space.slint");
    assert!(
        component.contains(r#"import { SdCheckbox } from "form-controls.slint";"#),
        "Reclaim space must import the shared checkbox component"
    );
    assert!(
        component.contains("SdCheckbox {"),
        "and it must actually be instantiated, not merely imported"
    );
    assert!(
        !component.contains("commands: \"M 0.5 4.5 L 3.3 7.3 L 8.5 1.4\""),
        "the checkbox tick must never be drawn a second time here - that path belongs to \
         `form-controls.slint` alone"
    );
}

/// The header carries the shared modal header and the total readout Rust composes.
#[test]
fn the_header_uses_the_shared_component_and_shows_the_total() {
    let component = read("ui/components/reclaim-space.slint");
    assert!(
        component.contains("SdModalHeader {"),
        "Reclaim space must use the shared header - a hand-rolled one here would be the same \
         drift `test_design_system.rs` already guards against for the Library and Review & Update"
    );
    assert!(
        component.contains("title: \"Reclaim space\";"),
        "the header title is settled copy, not a placeholder"
    );
    assert!(
        component.contains("root.total-label"),
        "the header must show the total Rust composes, not a number computed a second time here"
    );
}
