//! `BUG-93`: the toast Rectangle's own comment read "last declared, so it draws over everything" -
//! true the day it was written, false the moment the first overlay was added below it. By the time
//! this was found, five more had landed after it (the Library, Review & Update, Settings, Reclaim
//! space, and Assemble & Review), each painting over a toast that believed it was on top. A Reviewer
//! firing a toast (Copy Markdown's, for instance) while any one of those was open watched it appear
//! behind the Editor's own dimmed backdrop instead of over whatever they were actually looking at.
//!
//! This asserts the ordering property directly - the toast's own declaration must come AFTER every
//! overlay's - rather than re-checking the specific list of overlays that happened to exist when this
//! was written, which is exactly the kind of guard that silently stops meaning anything the next time
//! a new overlay is added below the list rather than below the toast.

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

#[test]
fn the_toast_is_declared_after_every_full_window_overlay_so_it_paints_on_top_of_all_of_them() {
    let window = read("ui/appwindow.slint");

    let toast_at = window
        .find(r#"if root.toast-text != "" : Rectangle {"#)
        .expect("the toast Rectangle must exist");

    for (overlay_name, needle) in [
        ("the Library", "if root.library-open : SdLibrary {"),
        (
            "Review & Update",
            "if root.review-update-open : SdReviewUpdate {",
        ),
        ("Settings", "if root.settings-open : SdSettings {"),
        (
            "Reclaim space",
            "if root.reclaim-space-open : SdReclaimSpace {",
        ),
        (
            "the Vault-move confirmation",
            r#"if root.pending-vault-folder != "" : Rectangle {"#,
        ),
        (
            "the delete-Finding confirmation",
            r#"if root.pending-delete-finding != "" : Rectangle {"#,
        ),
        (
            "the canvas/filmstrip context menu",
            r#"if root.menu-subject != "" : SdContextMenu {"#,
        ),
        (
            "Assemble & Review",
            "if root.bundle-preview-open : Rectangle {",
        ),
    ] {
        let overlay_at = window
            .find(needle)
            .unwrap_or_else(|| panic!("{overlay_name}'s own mount point must exist"));
        assert!(
            toast_at > overlay_at,
            "BUG-93: the toast must be declared AFTER {overlay_name} so it paints on top of it - \
             it is currently declared BEFORE, which means {overlay_name} paints over any toast \
             fired while it is open"
        );
    }
}
