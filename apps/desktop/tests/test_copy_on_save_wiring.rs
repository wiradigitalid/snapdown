//! Ticket 4 (`04-copy-markdown-on-save`, `post-testing-polish` spec): a successful Assemble & Save
//! (`on_bundle_preview_confirmed`'s `Ok` arm) and a successful Review & Update Save
//! (`on_review_update_save_clicked`'s `Saved` arm) each also copy the Bundle's Markdown to the
//! clipboard - reusing the exact function `on_library_copy_markdown_clicked`'s own handler already
//! calls (`bundle_markdown_for_clipboard` then `put_text_on_clipboard`), never a second
//! implementation.
//!
//! The spec's own Seam 1 entry for this ticket: "proven by asserting the SAME function name appears
//! in all three call sites, not by re-describing the clipboard write a third time." That is what
//! every test below checks - never a copy of the clipboard-write logic itself, only that the three
//! handlers reach the same two named functions, and that a failed save reaches neither.

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// The same source with every run of whitespace collapsed to one space - copied from
/// `test_review_update_wiring.rs` / `test_annotation_wiring.rs`. `rustfmt` decides where a method
/// chain or a match arm breaks, and a guard written against one exact layout is a guard the next
/// `cargo fmt` can turn red for nothing.
fn flat(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The body of a closure passed to `main_window.on_<callback>(move |...| { ... });`, matched by
/// braces from the first `{` after `on_<callback>(move` - robust to the closure's actual length,
/// unlike a fixed-width slice. Mirrors `test_review_update_wiring.rs`'s `rust_fn_body`, but anchored
/// on a callback registration rather than a top-level `fn`, since none of the three handlers this
/// file checks is one.
fn callback_body<'a>(source: &'a str, callback: &str) -> &'a str {
    let needle = format!("on_{callback}(move");
    let start = source
        .find(&needle)
        .unwrap_or_else(|| panic!("`on_{callback}` must be registered in main.rs"));
    let open = source[start..]
        .find('{')
        .map(|i| start + i)
        .unwrap_or_else(|| panic!("`on_{callback}`'s closure has no body"));
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
    panic!("`on_{callback}`'s closure body is never closed");
}

/// Splits a closure body that matches `write_bundle`'s/`save_review_update_edit`'s own shape - one
/// `match` whose SUCCESS arm this ticket touches comes textually before its failure arm - into
/// (success_arm, failure_arm), cut at the first occurrence of `failure_needle` after
/// `success_needle`.
fn split_arms<'a>(body: &'a str, success_needle: &str, failure_needle: &str) -> (&'a str, &'a str) {
    let success_at = body
        .find(success_needle)
        .unwrap_or_else(|| panic!("expected to find {success_needle:?} in: {body}"));
    let failure_at = body[success_at..]
        .find(failure_needle)
        .map(|i| success_at + i)
        .unwrap_or_else(|| {
            panic!("expected to find {failure_needle:?} after the success arm in: {body}")
        });
    (&body[success_at..failure_at], &body[failure_at..])
}

/// The two named functions every copy site must call, and only through their real names - never a
/// re-description of what they do.
const BUNDLE_MARKDOWN_FOR_CLIPBOARD: &str = "bundle_markdown_for_clipboard(";
const PUT_TEXT_ON_CLIPBOARD: &str = "put_text_on_clipboard(";

/// The baseline: Copy Markdown's own handler (ticket 12) is the one whose two function calls the
/// other two handlers must reuse. If this ever stopped calling them, the other two assertions in
/// this file would be checking against a moved target.
#[test]
fn copy_markdown_clicked_calls_bundle_markdown_for_clipboard_then_put_text_on_clipboard() {
    let main = read("src/main.rs");
    let body = flat(callback_body(&main, "library_copy_markdown_clicked"));

    assert!(
        body.contains(BUNDLE_MARKDOWN_FOR_CLIPBOARD),
        "Copy Markdown's handler must call `bundle_markdown_for_clipboard`: {body}"
    );
    assert!(
        body.contains(PUT_TEXT_ON_CLIPBOARD),
        "Copy Markdown's handler must call `put_text_on_clipboard`: {body}"
    );
    assert!(
        body.contains("toast(&win, COPY_MARKDOWN_TOAST, false)"),
        "Copy Markdown's own success toast must be the shared `COPY_MARKDOWN_TOAST` constant, not \
         a literal only this handler knows: {body}"
    );
}

/// Assemble & Save's `Ok` arm (a successful `write_bundle`) must call the SAME two functions Copy
/// Markdown's handler calls, and show the SAME toast wording - and its `Err` arm (a failed save)
/// must call neither, showing only its own existing failure toast.
#[test]
fn assemble_and_save_copies_on_success_and_only_on_success() {
    let main = read("src/main.rs");
    let body = callback_body(&main, "bundle_preview_confirmed");
    let (ok_arm, err_arm) = split_arms(body, "Ok(message) =>", "Err(message) =>");
    let ok_arm = flat(ok_arm);
    let err_arm = flat(err_arm);

    assert!(
        ok_arm.contains(BUNDLE_MARKDOWN_FOR_CLIPBOARD),
        "a successful Assemble & Save must call `bundle_markdown_for_clipboard`, the same function \
         Copy Markdown's handler calls - not a second implementation: {ok_arm}"
    );
    assert!(
        ok_arm.contains(PUT_TEXT_ON_CLIPBOARD),
        "a successful Assemble & Save must call `put_text_on_clipboard`, the same function Copy \
         Markdown's handler calls: {ok_arm}"
    );
    assert!(
        ok_arm.contains("toast(&win, COPY_MARKDOWN_TOAST, false)"),
        "a successful copy on this path must show Copy Markdown's own toast wording, via the same \
         shared constant, including the absolute-path disclosure: {ok_arm}"
    );

    assert!(
        !err_arm.contains(BUNDLE_MARKDOWN_FOR_CLIPBOARD)
            && !err_arm.contains(PUT_TEXT_ON_CLIPBOARD),
        "a FAILED Assemble & Save must copy nothing - the `Err` arm must call neither clipboard \
         function: {err_arm}"
    );
    assert!(
        err_arm.contains(r#"toast(&win, message, true)"#),
        "a failed Assemble & Save must still show only its own existing failure toast: {err_arm}"
    );
}

/// Review & Update's `Saved` arm (a successful `save_review_update_edit`) must call the SAME two
/// functions Copy Markdown's handler calls, and show the SAME toast wording on the branch where the
/// view refresh also succeeds - and the outer `Err` arm (a failed save) must call neither, showing
/// only its own existing failure toast.
#[test]
fn review_update_save_copies_on_success_and_only_on_success() {
    let main = read("src/main.rs");
    let body = callback_body(&main, "review_update_save_clicked");
    let (saved_arm, err_arm) = split_arms(
        body,
        "Ok(ReviewUpdateSaveOutcome::Saved) =>",
        "Err(message) =>",
    );
    let saved_arm = flat(saved_arm);
    let err_arm = flat(err_arm);

    assert!(
        saved_arm.contains(BUNDLE_MARKDOWN_FOR_CLIPBOARD),
        "a successful Review & Update Save must call `bundle_markdown_for_clipboard`, the same \
         function Copy Markdown's handler calls - not a second implementation: {saved_arm}"
    );
    assert!(
        saved_arm.contains(PUT_TEXT_ON_CLIPBOARD),
        "a successful Review & Update Save must call `put_text_on_clipboard`, the same function \
         Copy Markdown's handler calls: {saved_arm}"
    );
    assert!(
        saved_arm.contains("toast(&win, COPY_MARKDOWN_TOAST, false)"),
        "a successful copy on this path must show Copy Markdown's own toast wording, via the same \
         shared constant, including the absolute-path disclosure: {saved_arm}"
    );

    // The `NoChange` arm sits BEFORE the `Saved` arm in source order and is excluded by
    // `split_arms` starting its scan at `success_needle`; asserted directly here so a future
    // reordering cannot silently let this test start passing on the wrong arm.
    let no_change_at = body
        .find("Ok(ReviewUpdateSaveOutcome::NoChange) =>")
        .expect("the Save handler must have a NoChange arm");
    let saved_at = body
        .find("Ok(ReviewUpdateSaveOutcome::Saved) =>")
        .expect("the Save handler must have a Saved arm");
    assert!(
        no_change_at < saved_at,
        "this test's `split_arms` call assumes NoChange precedes Saved in source order"
    );
    let no_change_arm = flat(&body[no_change_at..saved_at]);
    assert!(
        !no_change_arm.contains(BUNDLE_MARKDOWN_FOR_CLIPBOARD)
            && !no_change_arm.contains(PUT_TEXT_ON_CLIPBOARD),
        "a no-op Save (nothing changed) must copy nothing either: {no_change_arm}"
    );

    assert!(
        !err_arm.contains(BUNDLE_MARKDOWN_FOR_CLIPBOARD)
            && !err_arm.contains(PUT_TEXT_ON_CLIPBOARD),
        "a FAILED Review & Update Save must copy nothing - the outer `Err` arm must call neither \
         clipboard function: {err_arm}"
    );
    assert!(
        err_arm.contains(r#"toast(&win, message, true)"#),
        "a failed Review & Update Save must still show only its own existing failure toast: \
         {err_arm}"
    );
}
