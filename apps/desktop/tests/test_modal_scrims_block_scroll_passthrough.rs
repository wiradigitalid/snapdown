//! `BUG-94`: scrolling anywhere over the Library while it was open (or Review & Update, Reclaim
//! space, or Assemble & Review) could still pan or zoom the Editor's canvas underneath - the "ini kan
//! modal dialog, harusnya semua aktivitas terhenti di window dialog" report.
//!
//! Each of these is a full-window scrim `Rectangle` with a `TouchArea` whose only documented job was
//! to swallow a stray CLICK that misses the panel. A bare `TouchArea` rejects wheel input by default
//! (confirmed with a headless `PointerScrolled` dispatch against a minimal reproduction of exactly
//! this shape - a "canvas" TouchArea behind a scrim `TouchArea` with no `scroll-event` handler: the
//! canvas received the event), and a rejected event keeps walking backward through the hit-test to
//! whatever is behind it. Over any part of the modal that has nothing of its own to scroll - the
//! header, the padding, a `ScrollView` that has reached either end - that "whatever" was the canvas.
//! The same probe with `scroll-event(event) => { accept }` added to the scrim's `TouchArea` shows the
//! canvas receiving zero events.
//!
//! This crate has no `[lib]` target (see `test_library_wiring.rs`'s own note on the same limitation),
//! so this reads source text for the fix's presence rather than driving a live component - the
//! headless probe that found and proved the fix lived outside the repository, not as a shipped test.

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

fn flat(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every full-window modal scrim in the app, and the file it lives in - one row per overlay this
/// bug affected. Grown, not replaced, the day a ninth modal joins the other eight `BUG-93` named.
const MODAL_SCRIMS: &[(&str, &str)] = &[
    ("the Library", "ui/components/library.slint"),
    ("Review & Update", "ui/components/review-update.slint"),
    ("Reclaim space", "ui/components/reclaim-space.slint"),
    ("Assemble & Review", "ui/appwindow.slint"),
];

#[test]
fn every_modal_scrims_touch_area_explicitly_accepts_scroll_events() {
    for (name, file) in MODAL_SCRIMS {
        let source = flat(&read(file));
        // The scrim TouchArea always immediately follows its own "Swallows ... click" comment in
        // each of these files - anchor there so this cannot accidentally match some other,
        // unrelated `TouchArea` elsewhere in a large file like `appwindow.slint`.
        let comment_at = source.find("Swallows").unwrap_or_else(|| {
            panic!("{name} ({file}) must have its scrim's click-swallow comment")
        });
        let touch_area_at = source[comment_at..]
            .find("TouchArea {")
            .map(|i| comment_at + i)
            .unwrap_or_else(|| panic!("{name} ({file}) must declare the scrim's own TouchArea"));
        // Balanced-brace scan, not a naive `find('}')`: the fix nests a `scroll-event` handler's own
        // `{ accept }` inside the TouchArea's braces, so the FIRST `}` closes that inner block, not
        // the TouchArea itself - a naive search would truncate `block` before the closing `}` of
        // `{ accept }` ever appears in it, and the `contains` check below would then never match a
        // correctly-written fix.
        let body = &source[touch_area_at + "TouchArea {".len()..];
        let mut depth = 1i32;
        let mut end = None;
        for (i, ch) in body.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let end = end.unwrap_or_else(|| panic!("{name} ({file})'s scrim TouchArea must be closed"));
        let block = &body[..end];
        assert!(
            block.contains("scroll-event(event) => { accept }"),
            "BUG-94: {name} ({file})'s scrim TouchArea must explicitly accept scroll-event, or a \
             bare TouchArea's default rejection lets the wheel walk through to the canvas behind it \
             - found: {block:?}"
        );
    }
}
