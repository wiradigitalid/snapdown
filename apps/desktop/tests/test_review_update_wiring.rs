//! Review & Update is REACHABLE - not merely built. Ticket 13 of the Bundle Library spec built
//! locked mode; ticket 14 adds editing on top of it, and every one of its five new callbacks
//! (`edit-clicked`, `field-edited`, `save-clicked`, `cancel-clicked`, `discard-clicked`) gets the
//! same reachability proof locked mode's own callback already had. `AGENTS.md` names this
//! repository's signature failure in plain words: a component built, unit-tested, and mounted
//! nowhere. `test_annotation_wiring.rs`'s `the_annotation_component_is_mounted_on_the_canvas` is the
//! shape to copy - imported AND instantiated, not merely defined - and `test_library_wiring.rs` is
//! ticket 11's copy of it. This file is that same shape for `SdReviewUpdate`.

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
    // 600, not 400: `BUG-100` added `toast-text`/`toast-is-error` bindings before `closed =>`,
    // pushing it past a window sized for ticket 13/14 alone - widened rather than re-anchored, the
    // same fix `test_library_wiring.rs`'s own mount-site window needed once before.
    let mount = flat(&window[at..(at + 600).min(window.len())]);
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
    // 2000 chars, not 700: ticket 12/16's menu-target/menu-sealed/pending-* properties and
    // callbacks sit BEFORE `bundle-opened` in this block, widening it well past what a window
    // sized for ticket 13 alone assumed - a fixed window still beats scanning to a matching
    // brace for this file's purposes, so it is simply sized generously instead.
    let library_mount = flat(&window[library_at..(library_at + 2000).min(window.len())]);
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

/// Escape closes it when locked - the same pattern the Library and Assemble & Review both use,
/// borrowed rather than reinvented - and behaves exactly like Cancel while editing (ticket 14):
/// leaving must ask first when there is something to lose, whichever key tries to leave.
#[test]
fn escape_closes_locked_and_cancels_editing() {
    let component = read("ui/components/review-update.slint");
    let at = component.find("review-update-keys := FocusScope").expect(
        "Review & Update must hold its own FocusScope for Escape, the way `library-keys` does for \
         the Library",
    );
    let body = flat(&component[at..(at + 500).min(component.len())]);
    assert!(
        body.contains("Key.Escape") && body.contains("root.closed()"),
        "Escape must fire `closed()` when locked"
    );
    assert!(
        body.contains("root.cancel-clicked()"),
        "Escape must fire `cancel-clicked()` while editing, not bypass its confirmation: {body}"
    );
    assert!(
        body.contains("if (root.editing)"),
        "the two must be chosen by `editing`, not always the same one: {body}"
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

    // Ticket 14's editing path, checked the strongest way a source-reading test can: none of these
    // three functions even TAKES a `FindingStore` or an `AppContext` - `apply_review_update_field_edit`
    // and `review_update_edit_is_dirty` take only the parsed buffer and/or a `Bundle`, and
    // `save_review_update_edit` takes a `&dyn BundleStore` and a vault path. `BR-10`/`BR-11`'s "never
    // reads or writes a Finding" holds by construction for all three, not merely by their bodies
    // happening not to mention it.
    for (fn_name, signature_must_not_contain) in [
        (
            "apply_review_update_field_edit",
            ["FindingStore", "AppContext"],
        ),
        (
            "review_update_edit_is_dirty",
            ["FindingStore", "AppContext"],
        ),
        ("save_review_update_edit", ["FindingStore", "AppContext"]),
    ] {
        let needle = format!("fn {fn_name}(");
        let at = main
            .find(&needle)
            .unwrap_or_else(|| panic!("`{fn_name}` must exist in main.rs"));
        let signature_end = main[at..]
            .find(") -> ")
            .or_else(|| main[at..].find(") {"))
            .map(|i| at + i)
            .unwrap_or_else(|| panic!("`{fn_name}`'s signature never closes"));
        let signature = &main[at..signature_end];
        for forbidden in signature_must_not_contain {
            assert!(
                !signature.contains(forbidden),
                "`{fn_name}`'s own signature must not take a `{forbidden}` - ticket 14's editing \
                 path must be structurally unable to reach a Finding: {signature}"
            );
        }
        let body = rust_fn_body(&main, fn_name);
        assert!(
            !body.contains("finding_store"),
            "`{fn_name}` must never reference `finding_store`"
        );
    }
}

/// No affordance of any kind in LOCKED mode: no `SdTextField` (its own `flat` mode still lightens on
/// hover and shows a text cursor - `text-field.slint:87-102` - which is exactly the kind of
/// affordance locked mode must not offer), no `TouchArea` on an image, no "Fixed at compose" chip
/// (ticket 14 restricts that to edit mode), and no Edit/Preview control. Every `SdTextField` ticket
/// 14 DOES add must be gated by `root.editing` - checked by finding each occurrence's own enclosing
/// `if` condition and requiring it to mention `editing`, rather than forbidding the component from
/// the file outright the way ticket 13's version of this test did.
///
/// Checked against the CODE only (`//` comment lines stripped) - this file's own doc comments name
/// `SdTextField`, `TouchArea` and `IconButton` by name to explain why locked mode does not use them,
/// and an "absent" assertion run against the raw source would match its own explanation.
#[test]
fn locked_mode_declares_no_affordance() {
    let component = code_only(&read("ui/components/review-update.slint"));

    assert!(
        !component.contains("SdSegmented"),
        "there must be no Edit/Preview pair: locked already IS the preview (ticket 05's resolved \
         variant C)"
    );

    // Every `SdTextField {` in this file must sit inside an `if` condition that mentions `editing` -
    // ticket 14's whole guarantee that locked mode never renders one. Found by scanning backward
    // from each occurrence for the nearest `if ` keyword.
    for (at, _) in component.match_indices("SdTextField {") {
        let before = &component[..at];
        let if_at = before
            .rfind("if ")
            .expect("every `SdTextField` must sit inside an `if` condition");
        let condition_end = before[if_at..]
            .find(':')
            .map(|i| if_at + i)
            .unwrap_or(before.len());
        let condition = &before[if_at..condition_end];
        assert!(
            condition.contains("editing"),
            "an `SdTextField` whose enclosing condition does not mention `editing` would render in \
             locked mode too: {condition:?}"
        );
    }

    // The `Fixed at compose` chip must sit inside a condition that mentions `editing` too - ticket
    // 14 restricts it to edit mode outright.
    let chip_at = component
        .find("Fixed at compose")
        .expect("the `Fixed at compose` chip must exist somewhere (ticket 14)");
    let before_chip = &component[..chip_at];
    let chip_if_at = before_chip
        .rfind("if ")
        .expect("the chip must sit inside an `if` condition");
    assert!(
        before_chip[chip_if_at..].contains("editing"),
        "the `Fixed at compose` chip must be gated on `editing`"
    );

    // Every `TouchArea {` this file itself declares (not one owned internally by `SdTextField`,
    // `SdActionButton` or `SdModalHeader`): the scrim that swallows a miss-click, and the discard-
    // changes confirmation's own scrim + panel guard, borrowed from the Library's Disassemble/Delete
    // dialogs. Three, not one, now that ticket 14 added that confirmation - any more would be an
    // affordance nothing here asked for.
    let own_touch_areas = component.matches("TouchArea {").count();
    assert_eq!(
        own_touch_areas, 3,
        "review-update.slint itself must declare exactly three `TouchArea`s: the scrim, and the \
         discard-changes confirmation's own scrim and panel guard"
    );

    // No affordance on the image block specifically, beyond the `Fixed at compose` chip already
    // checked above.
    let image_block_at = component
        .find(r#"if block.kind == "image""#)
        .expect("the image block must exist");
    let image_block = flat(&component[image_block_at..(image_block_at + 900).min(component.len())]);
    assert!(
        !image_block.contains("TouchArea") && !image_block.contains("IconButton"),
        "no TouchArea/IconButton on the image - ticket 05 found none on an image in the compose \
         window either, so this is consistency, not a new restriction"
    );
}

/// The header carries the static provenance line and the `As composed`/`Editing` badge (ticket 14
/// gives it a second state rather than a second control); there is no Edit/Preview pair (checked
/// above), and the footer's Edit is a REAL control now - `enabled: false` is gone, because ticket 14
/// is the thing that used to be missing behind it.
#[test]
fn header_and_footer_match_the_spec() {
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
        component.contains("\"Editing\""),
        "the header must carry the `Editing` badge too (ticket 14)"
    );

    assert!(
        component.contains("label: \"Edit\";") && !component.contains("enabled: false;"),
        "the footer's Edit button must exist and be a real, enabled control now that ticket 14 gives \
         it something to do"
    );
    let edit_at = component.find("label: \"Edit\";").expect("checked above");
    let edit_button = flat(&component[edit_at..(edit_at + 150).min(component.len())]);
    assert!(
        edit_button.contains("root.edit-clicked()"),
        "Edit must actually fire `edit-clicked()`: {edit_button}"
    );

    assert!(
        component.contains("label: \"Close\";"),
        "the footer's locked-mode secondary action must be Close"
    );
    assert!(
        component.contains("label: \"Save\";") && component.contains("label: \"Cancel\";"),
        "the footer's editing-mode actions must be Save and Cancel"
    );
    // Save carries no `enabled:` binding at all - "always clickable once editing" (ticket 05's
    // amendment) means it is simply never disabled, not disabled-then-re-enabled by some condition.
    let save_at = component.find("label: \"Save\";").expect("checked above");
    let save_button = flat(&component[save_at..(save_at + 200).min(component.len())]);
    assert!(
        !save_button.contains("enabled:"),
        "Save must carry no `enabled:` binding of any kind: {save_button}"
    );
    assert!(
        save_button.contains("root.save-clicked()"),
        "Save must actually fire `save-clicked()`: {save_button}"
    );
}

/// `BUG-99`: neither the header badge nor the footer's action buttons had an explicit width, so
/// each sized itself from whichever of its two candidate strings happened to be showing -
/// "As composed"/"Editing", and "Close"/"Cancel" - which are different lengths. Toggling
/// locked/editing therefore visibly resized the badge (shifting the provenance `Text` beside it,
/// right next to the header's own Close button) and the whole footer (right-aligned, so the
/// secondary button's width change moved everything). Confirmed with a headless probe against a
/// minimal reproduction of the exact structure: without a fixed width, the badge/provenance pair's
/// on-screen position measurably changed between the two states; with a width measured off a
/// hidden reference `Text` holding the longer candidate, it did not move a pixel.
#[test]
fn the_badge_and_footer_buttons_have_a_fixed_width_that_does_not_change_between_modes() {
    let component = flat(&read("ui/components/review-update.slint"));

    assert!(
        component.contains("badge-reference := Text { text: \"As composed\""),
        "the badge must be sized off a hidden reference holding the LONGER candidate string \
         (\"As composed\"), not off whichever of the two is currently showing"
    );
    assert!(
        component.contains("width: badge-reference.preferred-width"),
        "the badge's own width must read from that fixed reference, not from `badge-text` (the \
         element whose text actually changes)"
    );

    assert!(
        component.contains("secondary-reference := Text { text: \"Cancel\""),
        "the footer's secondary button (Close/Cancel) must be sized off a hidden reference holding \
         the longer candidate (\"Cancel\")"
    );
    for label in ["\"Close\"", "\"Cancel\""] {
        let at = component
            .find(&format!("label: {label};"))
            .unwrap_or_else(|| panic!("the {label} button must exist"));
        let button = &component[at.saturating_sub(200)..at];
        assert!(
            button.contains("width: secondary-reference.preferred-width"),
            "the {label} button must use the shared secondary width: {button}"
        );
    }

    assert!(
        component.contains("primary-reference-edit := Text { text: \"Edit\"")
            && component.contains("primary-reference-save := Text { text: \"Save\""),
        "the footer's primary button (Edit/Save) must be sized off hidden references for BOTH of \
         its own candidate labels, taking the wider of the two"
    );
    for label in ["\"Edit\"", "\"Save\""] {
        let at = component
            .find(&format!("label: {label};"))
            .unwrap_or_else(|| panic!("the {label} button must exist"));
        let button = &component[at.saturating_sub(250)..at];
        assert!(
            button.contains(
                "max(primary-reference-edit.preferred-width, \
                              primary-reference-save.preferred-width)"
            ),
            "the {label} button must use the shared primary width (the max of both candidates), \
             not its own label's width: {button}"
        );
    }
}

/// Every callback ticket 14 added is bound at the mount site to a root-level callback with a real
/// handler in `main.rs` - the same reachability proof `every_review_update_callback_is_bound_from_
/// slint_to_rust` already gives locked mode's own `closed()`.
#[test]
fn every_ticket_14_callback_is_bound_from_slint_to_rust() {
    let component = read("ui/components/review-update.slint");
    let window = read("ui/appwindow.slint");
    let main = read("src/main.rs");

    for callback in [
        "callback edit-clicked();",
        "callback field-edited(string, int, int, string);",
        "callback save-clicked();",
        "callback cancel-clicked();",
        "callback discard-clicked();",
    ] {
        assert!(
            component.contains(callback),
            "`SdReviewUpdate` must declare `{callback}`"
        );
    }

    let at = window
        .find("if root.review-update-open : SdReviewUpdate {")
        .expect("the mount site must exist");
    let mount = flat(&window[at..(at + 900).min(window.len())]);

    for (forward, root_callback, handler) in [
        (
            "edit-clicked => { root.review-update-edit-clicked(); }",
            "callback review-update-edit-clicked();",
            "on_review_update_edit_clicked(",
        ),
        (
            "save-clicked => { root.review-update-save-clicked(); }",
            "callback review-update-save-clicked();",
            "on_review_update_save_clicked(",
        ),
        (
            "cancel-clicked => { root.review-update-cancel-clicked(); }",
            "callback review-update-cancel-clicked();",
            "on_review_update_cancel_clicked(",
        ),
        (
            "discard-clicked => { root.review-update-discard-clicked(); }",
            "callback review-update-discard-clicked();",
            "on_review_update_discard_clicked(",
        ),
    ] {
        assert!(
            mount.contains(forward),
            "the mount site must forward `{forward}`"
        );
        assert!(
            window.contains(root_callback),
            "`AppWindow` must declare `{root_callback}`"
        );
        assert!(
            main.contains(handler),
            "`{handler}` must exist in main.rs, or the forwarded callback reaches nobody"
        );
    }

    // `field-edited` carries arguments, so its forwarding line reads differently from the other
    // four - checked on its own.
    assert!(
        mount.contains(
            "root.review-update-field-edited(kind, finding-ordinal, marker-ordinal, text)"
        ),
        "the mount site must forward `field-edited`'s four arguments through by name: {mount}"
    );
    assert!(
        window.contains("callback review-update-field-edited(string, int, int, string);"),
        "`AppWindow` must declare `review-update-field-edited` with all four argument types"
    );
    assert!(
        main.contains("on_review_update_field_edited("),
        "`on_review_update_field_edited` must exist in main.rs"
    );

    // `editing` and `cancel-pending` are the two-way/one-way properties the whole mode switch and
    // the confirmation dialog stand on - both must actually be bound at the mount site, not merely
    // declared somewhere.
    assert!(
        mount.contains("editing: root.review-update-editing;"),
        "the mount site must bind `editing` to `AppWindow`'s own property: {mount}"
    );
    assert!(
        mount.contains("cancel-pending <=> root.review-update-cancel-pending;"),
        "`cancel-pending` must be bound TWO-WAY, so `SdReviewUpdate`'s own \"Keep editing\" button \
         can dismiss it without a Rust round trip: {mount}"
    );
}

/// The discard-changes confirmation itself: shown only when `cancel-pending` is set, "Keep editing"
/// clears it locally (no callback - a two-way property write is enough), and "Discard changes"
/// clears it AND fires `discard-clicked()`.
#[test]
fn the_discard_changes_confirmation_is_wired() {
    let component = code_only(&read("ui/components/review-update.slint"));

    let at = component
        .find("if root.cancel-pending : Rectangle {")
        .expect("the confirmation must be gated on `cancel-pending`");
    let dialog = flat(&component[at..(at + 2200).min(component.len())]);

    assert!(
        dialog.contains("label: \"Keep editing\";"),
        "the confirmation's cancel verb must keep what the act would destroy (the map's own \
         confirmation copy rule): {dialog}"
    );
    assert!(
        dialog.contains("label: \"Discard changes\";"),
        "the confirmation's confirm verb must be the act: {dialog}"
    );
    assert!(
        dialog.contains("root.discard-clicked()"),
        "\"Discard changes\" must fire `discard-clicked()`: {dialog}"
    );
}

/// Cancel's own decision lives in Rust, not in Slint: `on_review_update_cancel_clicked` must check
/// dirtiness (via `review_update_edit_is_dirty`) before deciding whether to show the confirmation or
/// return to locked immediately, and Save's failure path must leave `editing` on and the buffer
/// alive so a retry has something to retry with.
#[test]
fn cancel_decides_in_rust_and_a_failed_save_keeps_the_buffer_alive() {
    let main = read("src/main.rs");

    assert!(
        main.contains("on_review_update_cancel_clicked("),
        "`on_review_update_cancel_clicked` must exist"
    );
    let cancel_at = main.find("on_review_update_cancel_clicked(").unwrap();
    let cancel_body = flat(&main[cancel_at..(cancel_at + 1200).min(main.len())]);
    assert!(
        cancel_body.contains("review_update_edit_is_dirty("),
        "Cancel must ask the dirty predicate, not guess: {cancel_body}"
    );
    assert!(
        cancel_body.contains("set_review_update_cancel_pending(true)"),
        "a dirty buffer must raise the confirmation: {cancel_body}"
    );

    assert!(
        main.contains("on_review_update_save_clicked("),
        "`on_review_update_save_clicked` must exist"
    );
    let save_at = main.find("on_review_update_save_clicked(").unwrap();
    // Scanned as flat text rather than `rust_fn_body`, which matches braces from a top-level `fn` -
    // this is a closure passed to `on_review_update_save_clicked(`, not a `fn` of that name.
    //
    // 3600, not 3000: ticket 4 (`04-copy-markdown-on-save`) added a copy-on-save attempt plus its
    // own nested match INSIDE the Saved arm, which comes BEFORE the Err arm this test needs -
    // pushing the outer Err arm past the window this test previously used. Same reason the window
    // grew from 1800 to 3000 for `BUG-97` before it: widened rather than re-anchored, the same fix
    // `a_successful_save_refreshes_the_library_still_open_underneath` needed too.
    let save_handler = flat(&main[save_at..(save_at + 3600).min(main.len())]);
    let err_arm_at = save_handler
        .find("Err(message) =>")
        .expect("the Save handler must have an Err arm");
    let err_arm = &save_handler[err_arm_at..(err_arm_at + 200).min(save_handler.len())];
    assert!(
        !err_arm.contains("set_review_update_editing(false)") && !err_arm.contains("= None"),
        "a failed Save must not flip `editing` off or drop the buffer - BR-5's \"an unsaved edit \
         survives so it can be tried again\": {err_arm}"
    );
    assert!(
        err_arm.contains("toast(&win, message, true)"),
        "a failed Save must tell the Reviewer what refused: {err_arm}"
    );
}

/// `BUG-97`: the Library stays open the whole time Review & Update is on top of it (ticket 13's own
/// design), so a Save that actually wrote left the Library's OWN row list stale - ticket 15's
/// "edited <when>" suffix did not appear until the Reviewer closed and reopened the Library by hand,
/// since nothing re-ran `build_library_rows` after the write.
#[test]
fn a_successful_save_refreshes_the_library_still_open_underneath() {
    let main = read("src/main.rs");
    let save_at = main
        .find("on_review_update_save_clicked(")
        .expect("`on_review_update_save_clicked` must exist");
    // Wide enough to comfortably hold the Saved arm's own doc comment as well as its code - unlike
    // the sibling test above, which only needs the Err arm and stops well before this one. 3600,
    // not 3000, for the same reason the sibling test above widened to 3600: ticket 4's copy-on-save
    // code inside the Saved arm pushed the outer Err arm further out.
    let save_handler = flat(&main[save_at..(save_at + 3600).min(main.len())]);
    let saved_arm_at = save_handler
        .find("Ok(ReviewUpdateSaveOutcome::Saved) =>")
        .expect("the Save handler must have a Saved arm");
    let err_arm_at = save_handler[saved_arm_at..]
        .find("Err(message) =>")
        .map(|i| saved_arm_at + i)
        .expect("the Save handler must have an Err arm after the Saved arm");
    let saved_arm = &save_handler[saved_arm_at..err_arm_at];
    assert!(
        saved_arm.contains("get_library_open()") && saved_arm.contains("open_library("),
        "BUG-97: a successful Save must re-read the Library's rows when it is open underneath, or \
         the edited suffix and any other change stay invisible until closed and reopened by hand: \
         {saved_arm}"
    );
}
