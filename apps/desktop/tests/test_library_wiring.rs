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
    let mount = flat(&window[at..(at + 900).min(window.len())]);

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

/// The row menu's destructive group (ticket 16): every callback `SdLibrary` declares for it is
/// forwarded at the mount site to a root-level callback with a real handler in `main.rs`. The same
/// shape `every_library_callback_is_bound_from_slint_to_rust` already proves for ticket 11's three -
/// this is that shape for the seven ticket 16 adds.
#[test]
fn every_row_menu_callback_is_bound_from_slint_to_rust() {
    let library = read("ui/components/library.slint");
    let window = read("ui/appwindow.slint");
    let main = read("src/main.rs");

    let at = window
        .find("if root.library-open : SdLibrary {")
        .expect("the mount site must exist");
    let mount = flat(&window[at..(at + 1800).min(window.len())]);

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

    // The four in-properties Rust drives the menu and the confirmations with must also be
    // forwarded, or Rust's live sealed/unsealed read and the confirmation copy never reach the
    // screen.
    for prop in [
        "menu-target: root.library-menu-target;",
        "menu-sealed: root.library-menu-sealed;",
        "pending-disassemble: root.library-pending-disassemble;",
        "pending-delete: root.library-pending-delete;",
        "pending-bundle-name: root.library-pending-bundle-name;",
        "pending-bundle-finding-count: root.library-pending-bundle-finding-count;",
    ] {
        assert!(
            mount.contains(prop),
            "the mount site must forward `{prop}`, or Rust's writes to it never reach `SdLibrary`"
        );
    }
}

/// The verb is read LIVE, in Slint's own source too: `destructive-menu` must be a ternary keyed on
/// `menu-sealed` - a property ONLY Rust ever writes (asserted by
/// `bundle_is_sealed_reads_live_never_a_cached_answer` in `main.rs`) - never a value this component
/// computes for itself from `rows` or any other client-side state.
#[test]
fn the_destructive_menu_entry_is_keyed_on_the_rust_supplied_sealed_flag() {
    let library = read("ui/components/library.slint");
    let at = library
        .find("private property <[MenuEntry]> destructive-menu")
        .expect("the destructive-menu property must exist");
    let body = flat(&library[at..(at + 400).min(library.len())]);

    assert!(
        body.contains("root.menu-sealed ? \"delete-bundle\" : \"disassemble-bundle\""),
        "the action must switch on `menu-sealed`"
    );
    assert!(
        body.contains("root.menu-sealed ? \"Delete…\" : \"Disassemble…\""),
        "the label must switch on `menu-sealed` too, and read exactly \"Disassemble…\" / \
         \"Delete…\" per spec.md's settled copy"
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

/// Both confirmations carry the settled copy from `spec.md`'s "The four confirmations": the Bundle
/// named in quotes, what comes back (or that nothing does), and "This cannot be undone." - plus the
/// house cancel/confirm verbs, "Keep it" and the act itself.
#[test]
fn both_confirmations_carry_the_settled_copy() {
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

    for cannot_be_undone in [
        "available to assemble again, with their notes and markers intact.\" + \" This cannot be undone.\"",
        "Its original captures were discarded earlier, so nothing comes back.\" + \" This cannot be undone.\"",
    ] {
        assert!(
            library.contains(cannot_be_undone),
            "every confirmation must end \"This cannot be undone.\": {cannot_be_undone}"
        );
    }

    assert_eq!(
        library.matches("label: \"Keep it\";").count(),
        2,
        "both confirmations' cancel verb must read \"Keep it\" - the object (the Bundle, or its \
         captures) rather than a generic \"Cancel\""
    );
    assert!(
        library.contains("label: \"Disassemble\";"),
        "the Disassemble confirmation's confirm verb must be the act itself"
    );
    assert!(
        library.contains("label: \"Delete\";"),
        "the Delete confirmation's confirm verb must be the act itself"
    );
}

/// `Discard originals…` is ticket 17's job, not this one's - the spec is explicit that it "is not
/// rendered yet". Asserted here so a future edit that adds it back in ahead of schedule is caught by
/// this file rather than discovered by a Reviewer.
///
/// Checked as a quoted STRING LITERAL - `"Discard originals` - never the bare phrase: this file's
/// own comments say why the row is absent yet ("ticket 17's Discard originals"), and a bare-phrase
/// check would fail on its own explanation. A menu label or an `action:` value is what would
/// actually render something; prose that mentions the future ticket is not that.
#[test]
fn discard_originals_is_not_rendered_by_this_ticket() {
    let library = read("ui/components/library.slint");
    assert!(
        !library.contains("\"Discard originals") && !library.contains("\"discard-originals\""),
        "`Discard originals…` is ticket 17's row to add, not ticket 16's - it must not appear as a \
         rendered label or menu action yet"
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
