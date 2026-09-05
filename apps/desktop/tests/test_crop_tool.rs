//! The Crop tool is REACHABLE, not merely a highlighted icon. `BUG-106`.
//!
//! Before this file the Crop `IconButton` set `root.active-tool-index = 6` - the same mechanism
//! every other tool uses to mark itself active and swap its cursor - and nothing on the canvas ever
//! read that index to draw or apply anything. `AGENTS.md` names this repository's signature
//! failure in plain words: a thing exists, compiles, and is reachable by nobody. `test_annotation_
//! wiring.rs` and `test_capture_interaction.rs` are the shape copied here: a callback declared AND
//! wired to a real Rust handler, a drag gesture with a real threshold, and no colour literal.

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// The same source with every run of whitespace collapsed to one space - `rustfmt` decides where a
/// method chain or a struct literal breaks, and a guard pinned to one exact layout is a guard the
/// next `cargo fmt` can turn red for nothing. Copied from `test_annotation_wiring.rs`.
fn flat(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The source with `//` comment lines removed, so a guard asserting a string is ABSENT is not
/// satisfied by the prose explaining why it was removed.
fn code_only(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The callback exists, and the canvas's own drag starts and ends it - the two halves `BUG-4`,
/// `BUG-5`, `BUG-6` and `BUG-72` all lacked at once: a declared surface with nothing wired under it.
#[test]
fn the_crop_drag_is_wired_from_press_to_release_on_the_canvas() {
    let window = code_only(&read("ui/appwindow.slint"));
    let flat_window = flat(&window);

    assert!(
        flat_window.contains("callback crop-applied(float, float, float, float);"),
        "the canvas must declare a callback for a finished crop drag, the same shape \
         `annotation-drawn` already uses for the five annotation tools"
    );

    // A press starts it, gated to the Crop tool's own index - not folded into the annotation
    // tools' `>= 1 && <= 5` branch, which is a different state machine feeding a different
    // callback.
    assert!(
        flat_window.contains("if (root.active-tool-index == 6) {")
            && flat_window.contains("root.crop-dragging = true;"),
        "a press on the canvas while the Crop tool is active must start a crop drag"
    );

    // A release ends it and fires the callback - but only past the same "a click is not a
    // selection" threshold every other drag tool on this canvas already enforces.
    assert!(
        flat_window.contains("if (ev.kind == PointerEventKind.up && root.crop-dragging) {"),
        "a release must be able to end the crop drag"
    );
    assert!(
        flat_window.contains("private property <bool> crop-drag-is-real:"),
        "a real drag must be a named condition, not an inline number in one branch - the same rule \
         `a_gesture_that_is_not_a_drag_draws_nothing` already holds the annotation tools to"
    );
    let up_at = window
        .find("if (ev.kind == PointerEventKind.up && root.crop-dragging) {")
        .expect("checked above");
    let up_block = flat(&window[up_at..(up_at + 900).min(window.len())]);
    assert!(
        up_block.contains("if (root.crop-drag-is-real) {")
            && up_block.contains("root.crop-applied("),
        "the release must guard the callback on a real drag, or a bare click on the Crop tool \
         would apply a zero-sized crop"
    );

    // The pointer must actually move the selection while it is held, or the drag has a start and
    // an end with nothing shown in between.
    assert!(
        flat_window.contains("if (root.crop-dragging) {")
            && flat_window.contains(
                "root.crop-x2 = Math.max(0.0, Math.min(1.0, self.mouse-x / parent.width));"
            ),
        "the crop selection must follow the pointer while the drag is held"
    );
}

/// The visual preview exists, is gated the same way the drag state is, and takes its colour from
/// the token file rather than a literal - `NFR-17`, and the standing rule this repository already
/// pays for once in `test_capture_interaction.rs`.
#[test]
fn the_crop_selection_preview_is_visible_and_uses_a_theme_token() {
    let window = code_only(&read("ui/appwindow.slint"));

    assert!(
        window.contains("if root.crop-dragging && root.crop-drag-is-real : Rectangle {"),
        "the drag must be shown on screen while it is happening, gated on a REAL drag so a click \
         draws nothing"
    );
    let at = window
        .find("if root.crop-dragging && root.crop-drag-is-real : Rectangle {")
        .expect("checked above");
    let block = &window[at..(at + 800).min(window.len())];
    assert!(
        block.contains("Theme.annotation-select-ring"),
        "the crop preview must take its border from the canvas's own selection-ring token, not a \
         literal - a new colour for one more rectangle is exactly how the overlay accumulated five \
         different alphas of black before `NFR-17`"
    );
    assert!(
        !block.contains('#'),
        "no hex literal may sit inside the crop preview's own block"
    );
}

/// A crop is not an annotation. It must never be foldable into `root.annotations`, the model every
/// `AnnotationItem` on the canvas repeats over.
#[test]
fn a_crop_never_becomes_an_annotation() {
    let window = flat(&code_only(&read("ui/appwindow.slint")));

    assert!(
        !window.contains(r#"root.annotation-drawn("crop""#),
        "the Crop tool must not be routed through `annotation-drawn` - `FR-30`/`FR-32`'s own \
         non-goals say an annotation produces no Markdown and lives in `root.annotations`; a crop \
         replaces the image itself and belongs nowhere in that model"
    );
}

/// The Rust side: a real handler, not a stub, and it reaches the store operation that actually
/// crops the Vault's file rather than only updating the database row.
#[test]
fn on_crop_applied_reaches_a_real_crop_operation() {
    let main = flat(&read("src/main.rs"));

    assert!(
        main.contains("main_window.on_crop_applied(move |x1, y1, x2, y2| {"),
        "`crop-applied` must have a real Rust handler - `test_ui_callbacks_reach_rust.rs` is the \
         mechanical half of this; this is the behavioural half"
    );

    assert!(
        main.contains("fn crop_finding_image("),
        "the handler must reach a named store operation, not inline Vault and database calls that \
         a second caller could not reuse or a reviewer could not find"
    );

    let at = main.find("fn crop_finding_image(").expect("checked above");
    let body = &main[at..(at + 2200).min(main.len())];
    assert!(
        body.contains(".vault_store .read_blob(image_path)")
            || body.contains(".vault_store.read_blob(image_path)"),
        "the operation must read the FINDING'S CURRENT bytes out of the Vault before cropping them"
    );
    assert!(
        body.contains("ImageReducer::crop_image("),
        "the actual crop must go through the pipeline's own decode-crop-encode function, not a \
         second hand-rolled one"
    );
    assert!(
        body.contains(".finding_store .update_finding_image(") || body.contains(".finding_store.update_finding_image("),
        "the crop must end by calling `update_finding_image` - the port this repository already had \
         with no caller anywhere in the app, `BUG-61`'s exact shape one level up"
    );

    // Every fallible write on this path must be handled, not swallowed with `let _ =` - `AGENTS.md`
    // names that pattern a defect rather than a style, and this path performs three of them: the
    // Vault write of the cropped bytes, the database update, and the best-effort delete of the
    // pre-crop file.
    assert!(
        !body.contains("let _ ="),
        "no fallible write inside the crop operation may be swallowed with `let _ =`"
    );
    assert!(
        body.contains("write_blob(&new_rel_path, &cropped_bytes)")
            && (body.contains(".map_err(") || body.contains("if let Err(")),
        "the Vault write of the cropped bytes must have its Result handled"
    );
    assert!(
        body.contains("if let Err(e) = ctx") && body.contains("update_finding_image("),
        "the database update's Result must be checked - a failure here must be reported, not \
         assumed to have succeeded"
    );
    assert!(
        body.contains("delete_blob(image_path)") && body.contains("if let Err(e) ="),
        "even the best-effort cleanup delete must have its Result read, so a failure is at least \
         logged rather than silently discarded"
    );
}

/// The two normative pitfalls `AGENTS.md` names for any new fallible-write path in this repository:
/// a Result an invariant depends on must not be swallowed, and the database row must never be left
/// naming a file whose dimensions do not match it.
#[test]
fn a_failed_database_update_does_not_leave_the_file_already_overwritten() {
    let main = flat(&read("src/main.rs"));
    let at = main.find("fn crop_finding_image(").expect("checked above");
    let body = &main[at..(at + 2200).min(main.len())];

    // The cropped bytes must land at a NEW path, never overwriting `image_path` directly - so a
    // database update that fails after the Vault write still leaves the Finding's row pointing at
    // the OLD, still-correct file.
    assert!(
        body.contains("let new_rel_path = format!("),
        "the cropped bytes must be written to a NEW Vault path, not over `image_path` - overwriting \
         it would mean a failed database update leaves the row's own dimensions describing a file \
         that no longer matches them"
    );
    assert!(
        body.contains("write_blob(&new_rel_path, &cropped_bytes)"),
        "the write must target that new path"
    );

    // The pre-crop file is deleted only AFTER the database points at the new one - found as the
    // ORDER of the two calls in the source, not merely that both exist.
    let update_at = body
        .find("update_finding_image(")
        .expect("the database update must exist");
    let delete_at = body
        .find("delete_blob(image_path)")
        .expect("the old file's cleanup must exist");
    assert!(
        update_at < delete_at,
        "the pre-crop file must be deleted AFTER the database row is repointed at the new one, or a \
         crash between the two leaves the row naming a file that has already been deleted"
    );
}
