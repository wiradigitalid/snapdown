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
