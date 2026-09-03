//! The Library is REACHABLE, not merely built. Ticket 11 of the Bundle Library spec.
//!
//! `AGENTS.md` names this repository's signature failure in plain words: a component built,
//! unit-tested, and mounted nowhere. `test_annotation_wiring.rs`'s
//! `the_annotation_component_is_mounted_on_the_canvas` is the shape to copy - imported AND
//! instantiated, not merely defined - and this file is that shape for `SdLibrary`.

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// The same source with every run of whitespace collapsed to one space - `rustfmt` decides where a
/// method chain or a struct literal breaks, and a guard written against one exact layout is a guard
/// the next `cargo fmt` can turn red for nothing. Copied from `test_annotation_wiring.rs`.
fn flat(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The component exists AND something mounts it.
#[test]
fn the_library_component_is_mounted_over_the_editor() {
    let window = read("ui/appwindow.slint");

    assert!(
        window
            .contains(r#"import { SdLibrary, LibraryBundleRow } from "components/library.slint";"#),
        "the Editor window must import the Library component"
    );
    assert!(
        window.contains("if root.library-open : SdLibrary {"),
        "`SdLibrary` must be MOUNTED, gated by a window property Rust can flip - a component that \
         compiles and is instantiated nowhere is what `BUG-4`, `BUG-5`, `BUG-6` and `BUG-72` all \
         were"
    );
    assert!(
        window.contains("in-out property <bool> library-open: false;"),
        "and the gate must be a property Rust can set - the same shape `bundle-preview-open` \
         already uses for Assemble & Review"
    );
}

/// Every callback `SdLibrary` declares is bound at the mount site, and every one of those forwards
/// to a root-level callback with a real handler in `main.rs` - not a `println!` stub.
#[test]
fn every_library_callback_is_bound_from_slint_to_rust() {
    let library = read("ui/components/library.slint");
    let window = read("ui/appwindow.slint");
    let main = read("src/main.rs");

    let at = window
        .find("if root.library-open : SdLibrary {")
        .expect("the mount site must exist");
    // 2400, not 900, not 1800: ticket 16 landed the same mount site's `in`-property forwards
    // (`menu-target`/`menu-sealed`/`menu-x`/`menu-y`/the four `pending-*`) ABOVE the callback
    // forwards, pushing ticket 12's two callbacks further into the block than a 900-char window
    // reaches, and ticket 17 then added three more `in`-property forwards and five more callback
    // forwards on top of that, past 1800 - `every_row_menu_callback_is_bound_from_slint_to_rust`
    // below already uses 2400 for the same reason, over the same block.
    let mount = flat(&window[at..(at + 2400).min(window.len())]);

    for (slint_callback, root_callback, rust_handler) in [
        (
            "callback closed();",
            "root.library-closed();",
            "on_library_closed",
        ),
        (
            "callback try-again-clicked();",
            "root.library-try-again-clicked();",
            "on_library_try_again_clicked",
        ),
        (
            "callback open-file-location-clicked();",
            "root.library-open-file-location-clicked();",
            "on_library_open_file_location_clicked",
        ),
        // Ticket 12: the row's two everyday actions, both carrying the Bundle id.
        (
            "callback copy-markdown-clicked(string);",
            "root.library-copy-markdown-clicked(id);",
            "on_library_copy_markdown_clicked",
        ),
        (
            "callback bundle-open-file-location-clicked(string);",
            "root.library-bundle-open-file-location-clicked(id);",
            "on_library_bundle_open_file_location_clicked",
        ),
    ] {
        assert!(
            library.contains(slint_callback),
            "`SdLibrary` must declare `{slint_callback}`"
        );
        assert!(
            mount.contains(root_callback),
            "the mount site must forward it: `{root_callback}` not found in the `if \
             root.library-open` block"
        );
        assert!(
            main.contains(&format!("{rust_handler}(")),
            "`{rust_handler}` must exist in main.rs, or the forwarded callback reaches nobody"
        );
    }

    // `library-clicked` is the Editor-side half of the round trip (`FR-28`): the icon that opens
    // it, not a callback `SdLibrary` declares itself.
    assert!(
        main.contains("on_library_clicked("),
        "the ribbon's Library icon must have a real handler"
    );
}

/// The row menu's destructive group (ticket 16, and ticket 17's Discard originals/Delete both on
/// top of it): every callback `SdLibrary` declares for it is forwarded at the mount site to a
/// root-level callback with a real handler in `main.rs`. The same shape
/// `every_library_callback_is_bound_from_slint_to_rust` already proves for ticket 11's three - this
/// is that shape for the eleven ticket 16 and ticket 17 add between them. `BUG-104` retired
/// `delete-both-requested` (Delete both is a direct row-menu action now, answered by
/// `on_library_row_menu_action` itself, not a separate callback), which is why this is eleven
/// and not twelve.
#[test]
fn every_row_menu_callback_is_bound_from_slint_to_rust() {
    let library = read("ui/components/library.slint");
    let window = read("ui/appwindow.slint");
    let main = read("src/main.rs");

    let at = window
        .find("if root.library-open : SdLibrary {")
        .expect("the mount site must exist");
    let mount = flat(&window[at..(at + 2400).min(window.len())]);

    for (slint_callback, root_callback, rust_handler) in [
        (
            "callback row-menu-requested(string, length, length);",
            "root.library-row-menu-requested(id, x, y);",
            "on_library_row_menu_requested",
        ),
        (
            "callback row-menu-dismissed();",
            "root.library-row-menu-dismissed();",
            "on_library_row_menu_dismissed",
        ),
        (
            "callback row-menu-action(string, string);",
            "root.library-row-menu-action(action, id);",
            "on_library_row_menu_action",
        ),
        (
            "callback disassemble-cancelled();",
            "root.library-disassemble-cancelled();",
            "on_library_disassemble_cancelled",
        ),
        (
            "callback disassemble-confirmed(string);",
            "root.library-disassemble-confirmed(id);",
            "on_library_disassemble_confirmed",
        ),
        (
            "callback delete-cancelled();",
            "root.library-delete-cancelled();",
            "on_library_delete_cancelled",
        ),
        (
            "callback delete-confirmed(string);",
            "root.library-delete-confirmed(id);",
            "on_library_delete_confirmed",
        ),
        (
            "callback discard-cancelled();",
            "root.library-discard-cancelled();",
            "on_library_discard_cancelled",
        ),
        (
            "callback discard-confirmed(string);",
            "root.library-discard-confirmed(id);",
            "on_library_discard_confirmed",
        ),
        (
            "callback delete-both-cancelled();",
            "root.library-delete-both-cancelled();",
            "on_library_delete_both_cancelled",
        ),
        (
            "callback delete-both-confirmed(string);",
            "root.library-delete-both-confirmed(id);",
            "on_library_delete_both_confirmed",
        ),
    ] {
        assert!(
            library.contains(slint_callback),
            "`SdLibrary` must declare `{slint_callback}`"
        );
        assert!(
            mount.contains(root_callback),
            "the mount site must forward it: `{root_callback}` not found in the `if \
             root.library-open` block"
        );
        assert!(
            main.contains(&format!("{rust_handler}(")),
            "`{rust_handler}` must exist in main.rs, or the forwarded callback reaches nobody"
        );
    }

    // The seven in-properties Rust drives the menu and the confirmations with must also be
    // forwarded, or Rust's live sealed/unsealed read and the confirmation copy never reach the
    // screen.
    for prop in [
        "menu-target: root.library-menu-target;",
        "menu-sealed: root.library-menu-sealed;",
        "pending-disassemble: root.library-pending-disassemble;",
        "pending-delete: root.library-pending-delete;",
        "pending-bundle-name: root.library-pending-bundle-name;",
        "pending-bundle-finding-count: root.library-pending-bundle-finding-count;",
        "pending-discard: root.library-pending-discard;",
        "pending-discard-warning: root.library-pending-discard-warning;",
        "pending-delete-both: root.library-pending-delete-both;",
    ] {
        assert!(
            mount.contains(prop),
            "the mount site must forward `{prop}`, or Rust's writes to it never reach `SdLibrary`"
        );
    }
}

/// The verb is read LIVE, in Slint's own source too: `row-menu` must be a ternary keyed on
/// `menu-sealed` - a property ONLY Rust ever writes (asserted by
/// `bundle_is_sealed_reads_live_never_a_cached_answer` in `main.rs`) - never a value this component
/// computes for itself from `rows` or any other client-side state.
///
/// `ticket 17` moved the sealed/unsealed choice from a per-field ternary on one shared array to two
/// named `[MenuEntry]` models (`sealed-row-menu` / `unsealed-row-menu`), because Slint cannot infer
/// `[MenuEntry]` for an inline `cond ? [...] : [...]` literal - `row-menu` itself is still the one
/// ternary this comment's title promises, just over the two named models rather than two inline
/// arrays. `row-menu` stays one property, not two: ticket 12's Copy Markdown/Open file location rows
/// and the destructive group both live inside each model, in the order `spec.md`'s "Menu order and
/// the two states" settles.
#[test]
fn the_destructive_menu_entry_is_keyed_on_the_rust_supplied_sealed_flag() {
    let library = flat(&read("ui/components/library.slint"));

    assert!(
        library.contains(
            "private property <[MenuEntry]> row-menu: root.menu-sealed ? root.sealed-row-menu : \
             root.unsealed-row-menu;"
        ),
        "`row-menu` must switch on `menu-sealed` between the two named models"
    );

    let at = library
        .find("private property <[MenuEntry]> sealed-row-menu")
        .expect("the sealed-row-menu property must exist");
    let sealed_body = &library[at..(at + 400).min(library.len())];
    assert!(
        sealed_body.contains(r#"{ action: "delete-bundle", label: "Delete…","#),
        "the sealed model must offer Delete… and only the destructive act - no Disassemble…, no \
         Discard originals…: {sealed_body}"
    );
    assert!(
        !sealed_body.contains("disassemble-bundle") && !sealed_body.contains("discard-originals"),
        "a sealed Bundle must never offer Disassemble… or Discard originals…"
    );

    let at = library
        .find("private property <[MenuEntry]> unsealed-row-menu")
        .expect("the unsealed-row-menu property must exist");
    // Widened from 700 to 1000 for `BUG-104`'s third entry (`delete-both-bundle`) - see this
    // file's own note on why these windows are fixed-character rather than to-the-brace.
    let unsealed_body = &library[at..(at + 1000).min(library.len())];
    assert!(
        unsealed_body.contains(r#"{ action: "disassemble-bundle", label: "Disassemble…","#),
        "the unsealed model must offer Disassemble…: {unsealed_body}"
    );
    assert!(
        unsealed_body.contains(r#"action: "discard-originals-bundle","#)
            && unsealed_body.contains(r#"label: "Discard originals…","#),
        "and Discard originals…, per ticket 17: {unsealed_body}"
    );
    assert!(
        unsealed_body.contains(r#"action: "delete-both-bundle","#)
            && unsealed_body.contains(r#"label: "Delete both…","#),
        "and Delete both…, per `BUG-104`'s reversal - a dedicated row now, not a link inside the \
         Disassemble confirmation: {unsealed_body}"
    );
    assert!(
        !unsealed_body.contains("\"delete-bundle\""),
        "an unsealed Bundle must never offer Delete… - that is the sealed-only verb: {unsealed_body}"
    );
}

/// `BUG-104` reversed `spec.md`'s original "never a menu row" rule on the owner's explicit request -
/// this is the structural half of that reversal, over the WHOLE file rather than just the unsealed
/// model above, so a future edit that adds a third model or moves the entry cannot silently drop
/// the one row this act depends on for reachability.
#[test]
fn delete_both_appears_as_a_dedicated_menu_row() {
    let library = read("ui/components/library.slint");
    assert!(
        library.contains("action: \"delete-both-bundle\""),
        "\"delete-both-bundle\" must be a `MenuEntry` action - `BUG-104` made Delete both reachable \
         directly from the row menu, not only from inside the Disassemble confirmation"
    );
}

/// The overflow button and a right-click on the row both open the SAME menu - the filmstrip's own
/// gesture, matched here rather than a second menu implementation being drawn.
#[test]
fn the_overflow_button_and_a_right_click_both_open_the_row_menu() {
    let library = flat(&read("ui/components/library.slint"));

    assert!(
        library.contains("overflow-touch := TouchArea"),
        "the overflow button must carry a TouchArea - ticket 11 deliberately left it undecorated"
    );
    assert!(
        library.contains(
            "clicked => { root.row-menu-requested(row.id, self.absolute-position.x + \
             self.mouse-x, self.absolute-position.y + self.mouse-y); }"
        ),
        "the overflow button's click must fire row-menu-requested with the row's own id and a \
         window-coordinate position"
    );
    assert!(
        library.contains("PointerEventKind.down && ev.button == PointerEventButton.right"),
        "the row itself must catch a right-click - `row-touch`'s own pointer-event handler"
    );
    assert!(
        library.contains(
            "root.row-menu-requested(row.id, self.absolute-position.x + self.mouse-x, \
             self.absolute-position.y + self.mouse-y);"
        ),
        "the right-click handler must open the same menu the overflow button does"
    );
}

/// `SdContextMenu` is reused, not redrawn: the row menu is built from the shared component, exactly
/// as the map's exploration notes instruct (§6, "reuse this, don't draw a second menu").
#[test]
fn the_row_menu_reuses_the_shared_context_menu_component() {
    let library = read("ui/components/library.slint");
    assert!(
        library.contains(r#"import { SdContextMenu, MenuEntry } from "context-menu.slint";"#),
        "`SdLibrary` must import the shared context menu component"
    );
    assert!(
        library.contains("if root.menu-target != \"\" : SdContextMenu {"),
        "the row menu must be an `SdContextMenu` instance, gated by `menu-target`"
    );
}

/// All four confirmations carry the settled copy from `spec.md`'s "The four confirmations": the
/// Bundle named in quotes, what comes back (or that nothing does), and "This cannot be undone." -
/// plus the house cancel/confirm verbs, "Cancel" and the act itself. Ticket 16 built Disassemble and
/// Delete; ticket 17 adds Discard originals and Delete both to the same standard.
///
/// `BUG-101` corrected the cancel verb from "Keep it"/"Keep them" (naming what the act would
/// destroy) to a plain "Cancel", on the owner's decision recorded in `spec.md`'s own "Corrected
/// 2026-09-03" note beside "The four confirmations" - a Reviewer testing the feature found the
/// object-naming verb less immediately recognisable as a cancel action than a proper reading of
/// the (deliberately non-generic) original intended.
#[test]
fn all_four_confirmations_carry_the_settled_copy() {
    let library = flat(&read("ui/components/library.slint"));

    assert!(
        library.contains("text: \"DISASSEMBLE \\\"\" + root.pending-bundle-name + \"\\\"?\";"),
        "the Disassemble heading must name the Bundle in quotes"
    );
    assert!(
        library.contains("available to assemble again, with their notes and markers intact."),
        "the Disassemble body must say what comes back, per spec.md's settled wording"
    );
    assert!(
        library.contains("text: \"DELETE \\\"\" + root.pending-bundle-name + \"\\\"?\";"),
        "the Delete heading must name the Bundle in quotes"
    );
    assert!(
        library.contains("Its original captures were discarded earlier, so nothing comes back."),
        "the Delete body must say nothing comes back, per spec.md's settled wording"
    );
    assert!(
        library.contains(
            "text: \"DISCARD ORIGINALS FROM \\\"\" + root.pending-bundle-name + \"\\\"?\";"
        ),
        "the Discard originals heading must name the Bundle in quotes"
    );
    assert!(
        library.contains(
            "This Bundle keeps its own copies and stays readable, but it can no longer be \
             disassembled."
        ),
        "the Discard originals body must say the Bundle stays readable, per spec.md's settled \
         wording"
    );
    assert!(
        library.contains("text: \"DELETE BOTH \\\"\" + root.pending-bundle-name + \"\\\"?\";"),
        "the Delete both heading must name the Bundle in quotes"
    );
    assert!(
        library.contains("Nothing comes back to the filmstrip."),
        "the Delete both body must say nothing comes back, per spec.md's settled wording"
    );

    for cannot_be_undone in [
        "available to assemble again, with their notes and markers intact.\" + \" This cannot be undone.\"",
        "Its original captures were discarded earlier, so nothing comes back.\" + \" This cannot be undone.\"",
        "This cannot be undone.\";",
    ] {
        assert!(
            library.contains(cannot_be_undone),
            "every confirmation must end \"This cannot be undone.\": {cannot_be_undone}"
        );
    }

    assert_eq!(
        library.matches("label: \"Cancel\";").count(),
        4,
        "all four confirmations' cancel verb must read \"Cancel\" (`BUG-101`)"
    );
    assert!(
        library.contains("label: \"Disassemble\";"),
        "the Disassemble confirmation's confirm verb must be the act itself"
    );
    assert!(
        library.contains("label: \"Delete\";"),
        "the Delete confirmation's confirm verb must be the act itself"
    );
    assert!(
        library.contains("label: \"Discard\";"),
        "the Discard originals confirmation's confirm verb must be the act itself"
    );
    assert!(
        library.contains("label: \"Delete both\";"),
        "the Delete both confirmation's confirm verb must be the act itself"
    );
}

/// The Discard originals confirmation names any other Bundle that shares one of the Findings about
/// to be discarded (`BR-12`/`BR-122`) - the ONLY place that consequence is said, per `spec.md`. The
/// warning sentence is Rust's own live computation (`pending-discard-warning`), inserted only when
/// non-empty; this component never derives it.
#[test]
fn the_discard_confirmation_makes_room_for_the_other_bundle_warning() {
    let library = flat(&read("ui/components/library.slint"));
    assert!(
        library.contains("in property <string> pending-discard-warning: \"\";"),
        "`pending-discard-warning` must be an `in` property Rust alone writes"
    );
    assert!(
        library.contains(
            "(root.pending-discard-warning == \"\" ? \"\" : \" \" + root.pending-discard-warning)"
        ),
        "the Discard originals body must splice in the warning only when Rust supplied one"
    );
}

/// `BUG-104`: Delete both is reachable directly from the row menu now, through the exact same
/// `row-menu-action` path Disassemble and Discard originals already use - not through a button
/// nested inside the Disassemble confirmation (that button is gone; `delete_both_appears_as_a_dedicated_menu_row`
/// proves the menu half). This proves the Rust-side reachability: the "delete-both-bundle" action
/// reaching `on_library_row_menu_action` sets `library_pending_delete_both`, the same property that
/// opens the Delete both dialog, and the dialog's own confirm button still fires
/// `delete-both-confirmed`.
#[test]
fn delete_both_is_reached_directly_from_the_row_menu() {
    let library = flat(&read("ui/components/library.slint"));
    let main = read("src/main.rs");

    assert!(
        !library.contains("delete-both-button := SdActionButton"),
        "the old second-step button inside the Disassemble confirmation must be gone - `BUG-104` \
         replaced it with a direct row-menu entry"
    );
    assert!(
        !main.contains("on_library_delete_both_requested"),
        "the retired `delete-both-requested` handler must not still exist in main.rs"
    );

    let at = main
        .find("main_window.on_library_row_menu_action(")
        .expect("the row-menu-action handler must exist");
    let handler = &main[at..(at + 2000).min(main.len())];
    assert!(
        handler.contains("\"delete-both-bundle\" => win.set_library_pending_delete_both(id)"),
        "the row-menu-action handler must answer \"delete-both-bundle\" by opening the Delete both \
         confirmation, the same way it answers disassemble-bundle/discard-originals-bundle: {handler}"
    );

    let library_src = read("ui/components/library.slint");
    let delete_both_at = library_src
        .find("if root.pending-delete-both != \"\" : Rectangle {")
        .expect("the Delete both confirmation must exist");
    let delete_both_block =
        &library_src[delete_both_at..(delete_both_at + 2200).min(library_src.len())];
    assert!(
        delete_both_block
            .contains("clicked => { root.delete-both-confirmed(root.pending-delete-both); }"),
        "the Delete both dialog's own confirm button must fire `delete-both-confirmed`"
    );
}

/// `BUG-103`: a genuinely irreversible confirm (Delete, Discard, Delete both) read identically to
/// Disassemble - the one confirm among the four that is NOT final, since it gives the Findings back.
/// `danger: true` must be on exactly the three final ones, and on no others, or the colour stops
/// meaning "cannot be undone" and starts meaning nothing.
#[test]
fn only_the_three_irreversible_confirmations_carry_the_danger_flag() {
    let library = read("ui/components/library.slint");

    for (label, needle) in [
        (
            "Delete",
            "label: \"Delete\";\n                        height: 30px;\n                        danger: true;",
        ),
        (
            "Discard",
            "label: \"Discard\";\n                        height: 30px;\n                        danger: true;",
        ),
        (
            "Delete both",
            "label: \"Delete both\";\n                        height: 30px;\n                        danger: true;",
        ),
    ] {
        assert!(
            library.contains(needle),
            "the {label} confirm button must carry `danger: true` right after its `height`, per \
             `BUG-103`"
        );
    }

    // Disassemble gives the Findings back - the one confirm among the four that is not final -
    // and Cancel is never destructive at all. Neither may carry the flag.
    let disassemble_at = library
        .find("if root.pending-disassemble != \"\" : Rectangle {")
        .expect("the Disassemble confirmation must exist");
    let delete_at = library
        .find("if root.pending-delete != \"\" : Rectangle {")
        .expect("the Delete confirmation must exist");
    let disassemble_block = &library[disassemble_at..delete_at];
    assert!(
        !disassemble_block.contains("danger: true"),
        "Disassemble is reversible-ish (the Findings come back) and must stay the plain primary \
         colour, not the danger one"
    );
}

/// The stub that only printed a line must be gone from the ratchet's excuse list, and the handler
/// that replaced it must do real work - `AppContext`, not `println!` alone.
#[test]
fn the_library_stub_has_left_the_known_stubs_list() {
    let ratchet = read("tests/test_ui_callbacks_reach_rust.rs");
    assert!(
        !ratchet.contains(r#""library-clicked","#),
        "`library-clicked` must be removed from `KNOWN_STUBS` now that it opens a real screen - \
         `a_handler_that_only_prints_is_recorded_as_a_stub`'s release half fails otherwise"
    );

    let main = read("src/main.rs");
    let at = main
        .find("on_library_clicked(")
        .expect("the handler must exist");
    let body = &main[at..(at + 300).min(main.len())];
    assert!(
        body.contains("open_library("),
        "the handler must actually open the Library, not merely flip a property and print"
    );
}

/// Escape closes it - the same pattern Assemble & Review uses, borrowed rather than reinvented.
#[test]
fn escape_closes_the_library() {
    let library = read("ui/components/library.slint");
    let at = library.find("library-keys := FocusScope").expect(
        "the Library must hold its own FocusScope for Escape, the way `preview-keys` does \
                 for Assemble & Review",
    );
    let body = flat(&library[at..(at + 500).min(library.len())]);
    assert!(
        body.contains("Key.Escape") && body.contains("root.closed()"),
        "Escape must fire `closed()`"
    );
    assert!(
        body.contains("init => { self.focus(); }"),
        "and something must GIVE the scope focus on open, or Escape never reaches it"
    );
}

/// Closing touches nothing the Editor owns. `on_library_closed` may only flip `library_open` back
/// to false - any editor-state setter here would mean the round trip is not free, which is exactly
/// what user story 3 promises ("lie over the Editor rather than replace it").
#[test]
fn closing_the_library_does_not_touch_editor_state() {
    let main = flat(&read("src/main.rs"));
    let at = main
        .find("main_window.on_library_closed(")
        .expect("the handler must exist");
    let body = &main[at..(at + 250).min(main.len())];

    assert!(
        body.contains("set_library_open(false)"),
        "closing must flip the Library's own gate back off"
    );
    for editor_setter in [
        "set_active_finding_id",
        "set_filmstrip_items",
        "set_active_image",
        "set_canvas_scroll",
    ] {
        assert!(
            !body.contains(editor_setter),
            "`on_library_closed` must not touch `{editor_setter}` - the Editor's canvas, selection \
             and scroll are what user story 3 promises stay exactly as left, and the only way this \
             source-reading test can vouch for that is confirming nothing here reaches for them"
        );
    }
}

/// The row order is the store's own. `build_library_rows` must not re-sort what `list_bundles`
/// already hands back newest-composed-first (`bundle_store.rs`'s own `ORDER BY composed_at DESC`).
#[test]
fn build_library_rows_does_not_re_sort_the_store_s_order() {
    let main = read("src/main.rs");
    let at = main
        .find("fn build_library_rows(")
        .expect("the function must exist");
    let body = &main[at..(at + 700).min(main.len())];
    assert!(
        !body.contains(".sort"),
        "the store's own order must be preserved, not re-derived here"
    );
}

/// The relative-time humaniser is hand-written, and this asserts it stayed that way rather than a
/// crate quietly changing what `Cargo.toml` was checked for before writing it.
#[test]
fn relative_time_is_hand_written_not_a_crate() {
    let cargo = read("Cargo.toml");
    for absent in ["chrono-humanize", "timeago"] {
        assert!(
            !cargo.contains(absent),
            "no `{absent}` dependency was found when this was written, and the ladder in \
             `relative_time` was hand-rolled on that basis - if `{absent}` was added since, \
             `relative_time` should very likely be deleted in favour of it instead of the two \
             coexisting"
        );
    }
    assert!(
        read("src/main.rs").contains("fn relative_time("),
        "the hand-written ladder must exist"
    );
}

/// `BUG-91`: a row's hover icons flickered on and off the moment the cursor actually moved onto one
/// of them. `row-touch` covers the whole row; the icon group is a later sibling painted on top, so
/// once its own TouchAreas overlapped the cursor position, Slint stopped delivering hover to
/// `row-touch` - and gating the icon group's existence on `row-touch.has-hover` ALONE (via `if`) tore
/// the group down the instant that happened, which put its TouchAreas out of the hit path, handed
/// hover back to `row-touch`, and rebuilt the group again: a flicker loop.
///
/// Confirmed empirically during diagnosis with a headless `PointerMoved` sweep across a minimal
/// reproduction of this exact structure (outer TouchArea + a later sibling holding its own
/// TouchAreas): `row-touch.has-hover` alone toggled on every 5px step once the sweep entered the
/// icon group's bounds, while OR-ing in the icons' own `has-hover` (which Slint DOES set correctly
/// for whichever element currently wins the hit-test) held steady with zero flips for the same
/// sweep, and correctly went false again once the cursor left the row entirely. See `BUG-91` in
/// `defects.yaml` for the full sweep output.
///
/// This crate has no `[lib]` target, so no test here can drive a live `SdLibrary` instance the way
/// that headless probe did - every wiring test in this file reads source text instead, and this one
/// is no exception. It is a narrower guarantee than the probe gave: it cannot prove the runtime is
/// flicker-free, only that the specific fix that was proven flicker-free is still there. Watch for a
/// change to the icon group's `visible` condition, an `if` reappearing in its place, or new hover
/// state being layered on without an accompanying OR term - none of the three would page anyone.
#[test]
fn the_row_hover_icons_are_gated_by_visible_ored_across_every_touch_area_not_by_an_if_on_row_touch_alone(
) {
    let library = flat(&read("ui/components/library.slint"));

    assert!(
        !library.contains("if row-touch.has-hover : HorizontalLayout"),
        "BUG-91: gating the icon group's very existence on `row-touch.has-hover` alone is the \
         exact shape that flickered - `if` tears the group down the instant hover moves onto one \
         of its own TouchAreas, which is what caused the loop"
    );

    let visible_at = library
        .find("icon-actions := HorizontalLayout {")
        .map(|start| &library[start..])
        .and_then(|rest| rest.find("visible:").map(|i| &rest[i..]))
        .expect("the icon group must be a named, always-instantiated HorizontalLayout with its own `visible` binding");
    let condition_end = visible_at
        .find(';')
        .expect("the `visible` binding must end with `;`");
    let condition = &visible_at[..condition_end];

    for must_appear in [
        "row-touch.has-hover",
        "copy-touch.has-hover",
        "reveal-touch.has-hover",
        "overflow-touch.has-hover",
    ] {
        assert!(
            condition.contains(must_appear),
            "BUG-91: the icon group's `visible` condition must OR in every one of its own \
             TouchAreas' `has-hover` alongside `row-touch`'s, or hovering directly onto whichever \
             one is left out reproduces the exact same flicker for that icon - missing: \
             `{must_appear}` in {condition:?}"
        );
    }
}

/// The overflow button's three dots are fixed-size children of a `HorizontalLayout` with no explicit
/// height - Slint fills that layout to its 28px parent, and without `cross-axis-alignment: center`
/// the dots sat pinned to the TOP of that space instead of centred in the button, a one-line CSS-
/// flexbox-shaped miss that was easy not to notice while `BUG-91`'s flicker was still making the
/// whole hover state hard to look at for more than a moment.
#[test]
fn the_overflow_buttons_dots_are_centred_on_both_axes() {
    let library = flat(&read("ui/components/library.slint"));
    let at = library
        .find("overflow-touch := TouchArea")
        .expect("the overflow button's TouchArea must exist");
    // The dots' HorizontalLayout is declared just BEFORE its own TouchArea sibling in the same
    // Rectangle - search backward from there rather than forward, so this does not accidentally
    // match a later, unrelated `alignment: center;` elsewhere in the file.
    let before = &library[..at];
    let layout_at = before
        .rfind("HorizontalLayout {")
        .expect("the overflow button's dots must sit in their own HorizontalLayout");
    let window = &before[layout_at..];
    assert!(
        window.contains("cross-axis-alignment: center"),
        "the dots' HorizontalLayout must centre its cross (vertical) axis too, not just the main \
         (horizontal) one - `alignment: center` alone leaves fixed-size children pinned to the top"
    );
}
