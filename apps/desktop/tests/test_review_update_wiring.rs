//! Review & Update, locked mode, is REACHABLE - not merely built. Ticket 13 of the Bundle Library
//! spec. `AGENTS.md` names this repository's signature failure in plain words: a component built,
//! unit-tested, and mounted nowhere. `test_annotation_wiring.rs`'s
//! `the_annotation_component_is_mounted_on_the_canvas` is the shape to copy - imported AND
//! instantiated, not merely defined - and `test_library_wiring.rs` is ticket 11's copy of it. This
//! file is that same shape for `SdReviewUpdate`.

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// The same source with every run of whitespace collapsed to one space - `rustfmt` decides where a
/// method chain or a struct literal breaks, and a guard written against one exact layout is a guard
/// the next `cargo fmt` can turn red for nothing. Copied from `test_annotation_wiring.rs` /
/// `test_library_wiring.rs`.
fn flat(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The source with `//` comment lines removed - copied from `test_design_system.rs`'s `code_only`.
/// An "absent" assertion below would otherwise match its OWN explanatory comment (this file's own
/// doc comments name `SdTextField`, `TouchArea` and `IconButton` by name to say why they are not
/// used), which makes the comment unwritable.
fn code_only(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extracts the body of a top-level Rust function by matching braces, starting from `fn <name>(` -
/// robust to the function's actual length, unlike a fixed-width slice.
fn rust_fn_body<'a>(source: &'a str, fn_name: &str) -> &'a str {
    let needle = format!("fn {fn_name}(");
    let start = source
        .find(&needle)
        .unwrap_or_else(|| panic!("`{fn_name}` must exist in main.rs"));
    let open = source[start..]
        .find('{')
        .map(|i| start + i)
        .unwrap_or_else(|| panic!("`{fn_name}` has no body"));
    let mut depth = 0i32;
    for (offset, ch) in source[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[open..open + offset + 1];
                }
            }
            _ => {}
        }
    }
    panic!("`{fn_name}`'s body is never closed");
}

/// The component exists AND something mounts it, ABOVE the Library rather than in place of it.
#[test]
fn the_review_update_component_is_mounted_over_the_library() {
    let window = read("ui/appwindow.slint");

    assert!(
        window.contains(
            r#"import { SdReviewUpdate, ReviewUpdateBlock } from "components/review-update.slint";"#
        ),
        "the Editor window must import Review & Update's component"
    );
    assert!(
        window.contains("if root.review-update-open : SdReviewUpdate {"),
        "`SdReviewUpdate` must be MOUNTED, gated by a window property Rust can flip - a component \
         that compiles and is instantiated nowhere is what `BUG-4`, `BUG-5`, `BUG-6` and `BUG-72` \
         all were"
    );
    assert!(
        window.contains("in-out property <bool> review-update-open: false;"),
        "and the gate must be a property Rust can set"
    );

    // Mounted AFTER the Library block, so it stacks ABOVE it (Slint draws later siblings on top) -
    // never replacing it, which is what keeps the Library's own scroll position alive underneath.
    let library_at = window
        .find("if root.library-open : SdLibrary {")
        .expect("the Library mount site must exist");
    let review_update_at = window
        .find("if root.review-update-open : SdReviewUpdate {")
        .expect("the Review & Update mount site must exist");
    assert!(
        review_update_at > library_at,
        "Review & Update must be declared AFTER the Library, or it would be drawn BEHIND it"
    );
}

/// Every callback `SdReviewUpdate` declares is bound at the mount site, and forwards to a root-level
/// callback with a real handler in `main.rs` - not a `println!` stub.
#[test]
fn every_review_update_callback_is_bound_from_slint_to_rust() {
    let component = read("ui/components/review-update.slint");
    let window = read("ui/appwindow.slint");
    let main = read("src/main.rs");

    assert!(
        component.contains("callback closed();"),
        "`SdReviewUpdate` must declare `closed()`"
    );

    let at = window
        .find("if root.review-update-open : SdReviewUpdate {")
        .expect("the mount site must exist");
    let mount = flat(&window[at..(at + 400).min(window.len())]);
    assert!(
        mount.contains("root.review-update-closed();"),
        "the mount site must forward `closed()` to `root.review-update-closed()`"
    );
    assert!(
        main.contains("on_review_update_closed("),
        "`on_review_update_closed` must exist in main.rs, or the forwarded callback reaches nobody"
    );

    // The Library-side half of the round trip: a row click forwards to a root-level callback with
    // its own real handler, which is what actually opens this window.
    let library = read("ui/components/library.slint");
    assert!(
        library.contains("callback bundle-opened(string);"),
        "`SdLibrary` must declare `bundle-opened(string)`"
    );
    let library_at = window
        .find("if root.library-open : SdLibrary {")
        .expect("the Library mount site must exist");
    let library_mount = flat(&window[library_at..(library_at + 700).min(window.len())]);
    assert!(
        library_mount.contains("bundle-opened(id) => { root.library-bundle-clicked(id); }"),
        "the Library mount site must forward `bundle-opened(id)` to `root.library-bundle-clicked(id)`"
    );
    assert!(
        main.contains("on_library_bundle_clicked("),
        "`on_library_bundle_clicked` must exist in main.rs, or the forwarded callback reaches nobody"
    );
    let at = main
        .find("on_library_bundle_clicked(")
        .expect("checked above");
    let body = &main[at..(at + 300).min(main.len())];
    assert!(
        body.contains("open_review_update("),
        "the handler must actually open Review & Update, not merely flip a property"
    );
}

/// Escape closes it - the same pattern the Library and Assemble & Review both use, borrowed rather
/// than reinvented.
#[test]
fn escape_closes_review_update() {
    let component = read("ui/components/review-update.slint");
    let at = component.find("review-update-keys := FocusScope").expect(
        "Review & Update must hold its own FocusScope for Escape, the way `library-keys` does for \
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

/// Closing Review & Update touches nothing the Library or the Editor own - only its own gate and its
/// own blocks. This is what makes "Close returns to the Library with its scroll position intact"
/// true by construction: the Library's `SdLibrary` instance is never torn down or rebuilt.
#[test]
fn closing_review_update_does_not_touch_library_or_editor_state() {
    let main = read("src/main.rs");
    let body = rust_fn_body(&main, "close_review_update");

    assert!(
        body.contains("set_review_update_open(false)"),
        "closing must flip Review & Update's own gate back off"
    );
    for other_setter in [
        "set_library_open",
        "set_library_state",
        "set_library_rows",
        "set_active_finding_id",
        "set_filmstrip_items",
        "set_active_image",
        "set_canvas_scroll",
    ] {
        assert!(
            !body.contains(other_setter),
            "`close_review_update` must not touch `{other_setter}` - the Library's own scroll \
             position, and everything the Editor owns underneath it, must survive exactly as left, \
             and the only way this source-reading test can vouch for that is confirming nothing here \
             reaches for them"
        );
    }
}

/// `review_update_doc_blocks` and `open_review_update` never touch the Finding store, source and
/// signature both - the reachability/binding check ticket 13's own acceptance criteria ask for,
/// which is what lets a sealed Bundle (`BR-11`) open exactly like an unsealed one.
#[test]
fn review_update_never_binds_to_the_finding_store() {
    let main = read("src/main.rs");

    let blocks_body = rust_fn_body(&main, "review_update_doc_blocks");
    assert!(
        !blocks_body.contains("finding_store"),
        "`review_update_doc_blocks` must never reference `ctx.finding_store` - it is built entirely \
         from `MarkdownSerializer::parse_bundle_document`'s read of the stored document"
    );
    assert!(
        blocks_body.contains("parse_bundle_document("),
        "`review_update_doc_blocks` must be built from the composer's own parse of the stored \
         document, not a second rendering path"
    );

    let open_body = rust_fn_body(&main, "open_review_update");
    assert!(
        !open_body.contains("finding_store"),
        "`open_review_update` must never reference `ctx.finding_store` either"
    );
}

/// No affordance of any kind in locked mode: no `SdTextField` (its own `flat` mode still lightens on
/// hover and shows a text cursor - `text-field.slint:87-102` - which is exactly the kind of
/// affordance this mode must not offer), no `TouchArea` on an image, no "Fixed at compose" chip (that
/// belongs to ticket 14's edit mode only), and no Edit/Preview control.
///
/// Checked against the CODE only (`//` comment lines stripped) - this file's own doc comments name
/// `SdTextField`, `TouchArea` and `IconButton` to explain why they are absent, and an "absent"
/// assertion run against the raw source would match its own explanation.
#[test]
fn locked_mode_declares_no_affordance() {
    let component = code_only(&read("ui/components/review-update.slint"));

    assert!(
        !component.contains("SdTextField"),
        "locked mode must not use `SdTextField` anywhere - even flat and read-only, it lightens and \
         shows a text cursor on hover"
    );
    assert!(
        !component.contains("Fixed at compose"),
        "the `Fixed at compose` chip belongs to ticket 14's edit mode, not to locked mode"
    );
    assert!(
        !component.contains("SdSegmented"),
        "there must be no Edit/Preview pair: locked already IS the preview (ticket 05's resolved \
         variant C)"
    );

    // Exactly ONE `TouchArea` is legitimate in this file: the scrim, which swallows a miss-click.
    // The FocusScope needs none of its own, and every button goes through `SdActionButton` /
    // `SdModalHeader`, which own their `TouchArea` internally - so THIS file itself must declare no
    // more than the scrim's.
    let own_touch_areas = component.matches("TouchArea {").count();
    assert_eq!(
        own_touch_areas, 1,
        "review-update.slint itself must declare exactly one `TouchArea` (the scrim) - any more is \
         an affordance this locked mode must not have"
    );

    // No affordance on the image block specifically.
    let image_block_at = component
        .find(r#"if block.kind == "image""#)
        .expect("the image block must exist");
    let image_block = flat(&component[image_block_at..(image_block_at + 700).min(component.len())]);
    assert!(
        !image_block.contains("TouchArea") && !image_block.contains("IconButton"),
        "no TouchArea/IconButton on the image - ticket 05 found none on an image in the compose \
         window either, so this is consistency, not a new restriction"
    );
}

/// The header carries the static provenance line and the `As composed` badge; there is no
/// Edit/Preview pair (checked above) and the footer's `Edit` is disabled, per the hand-off's own
/// choice: rendered, not omitted, and never enabled ahead of ticket 14.
#[test]
fn header_and_footer_match_the_locked_spec() {
    let component = read("ui/components/review-update.slint");

    assert!(
        component.contains("title: \"Review & Update\";"),
        "the header title is settled copy (ticket 05's answer), not a placeholder"
    );
    assert!(
        component.contains("root.provenance"),
        "the header must show the provenance line Rust composes"
    );
    assert!(
        component.contains("\"As composed\""),
        "the header must carry the `As composed` badge"
    );
    assert!(
        !component.contains("\"Editing\""),
        "no `Editing` state exists yet - that is ticket 14's, and rendering it now would be a \
         control with nothing behind it"
    );

    assert!(
        component.contains("label: \"Edit\";") && component.contains("enabled: false;"),
        "the footer's Edit button must exist and be disabled - it does nothing until ticket 14, and \
         the map's own rule forbids a control that does"
    );
    assert!(
        component.contains("label: \"Close\";"),
        "the footer's secondary action must be Close"
    );
}
