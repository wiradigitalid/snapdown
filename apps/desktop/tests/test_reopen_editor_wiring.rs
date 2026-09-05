//! `.scratch/editor-virtual-desktop-focus/issues/01-reopening-snapdown-jumps-to-its-own-virtual-desktop.md`
//! asks for four different "reopen the Editor" entry points - the already-running double-click
//! early exit, the tray's Open Editor, its matching global hotkey, and reveal-after-capture - to
//! all route through ONE shared function, so a future change to what "bring the Editor to front"
//! means is made once.
//!
//! `focus::bring_editor_to_foreground`'s own call-order and no-op behaviour is covered by
//! `apps/desktop/src/focus.rs`'s own unit tests (`RecordingBackend`), which do not need a live
//! window. What compiling that function correctly cannot prove is that all four sites actually
//! CALL it - a shared function nobody reaches is exactly the "green tests, unreachable component"
//! shape `AGENTS.md` names as this repository's signature failure. This file is that reachability
//! check, in the same shape as `tests/test_annotation_wiring.rs`.

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// The source with `//` comment lines removed, so a guard asserting a string is ABSENT cannot
/// match the comment explaining why it was removed.
fn code_only(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `main.rs` is big; each block below is sliced out by its own distinctive anchor so a match found
/// deep in an unrelated part of the file cannot pass a check meant for one particular entry point.
fn slice_after(source: &str, anchor: &str, take_lines: usize) -> String {
    let after = source
        .split_once(anchor)
        .unwrap_or_else(|| panic!("main.rs must still contain the anchor: {anchor:?}"))
        .1;
    after
        .lines()
        .take(take_lines)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn reveal_editor_window_calls_the_one_shared_function() {
    let main = code_only(&read("src/main.rs"));
    let block = slice_after(&main, "fn reveal_editor_window(window: &AppWindow) {", 30);
    assert!(
        block.contains("focus::bring_editor_to_foreground("),
        "reveal_editor_window is the in-process half of the shared entry point - it must call \
         focus::bring_editor_to_foreground, not reimplement its own foregrounding logic"
    );
}

#[test]
fn the_already_running_double_click_exit_routes_through_the_shared_function() {
    let main = code_only(&read("src/main.rs"));
    let block = slice_after(&main, "Snapdown is already running.\");", 10);
    assert!(
        block.contains("focus::find_running_editor_window()"),
        "the double-click entry point shares no memory with the already-running instance, so it \
         must look up that instance's HWND by title before it can foreground anything"
    );
    assert!(
        block.contains("focus::bring_editor_to_foreground("),
        "and it must hand that HWND to the SAME shared function every other entry point uses - \
         this is what the ticket calls 'reopening the Editor ... it no longer silently exits with \
         no visible effect'"
    );
}

#[test]
fn the_trays_open_editor_calls_reveal_editor_window() {
    let main = code_only(&read("src/main.rs"));
    let block = slice_after(&main, "TrayAction::OpenEditor => {", 8);
    assert!(
        block.contains("reveal_editor_window(&win)"),
        "the tray's Open Editor is one of the ticket's four entry points and must route through \
         reveal_editor_window, not just win.show()/set_minimized(false) as it did before this \
         change - that pair alone is exactly the 'opens invisibly on a desktop nobody is looking \
         at' bug this ticket describes"
    );
}

#[test]
fn the_matching_global_hotkey_calls_reveal_editor_window() {
    let main = code_only(&read("src/main.rs"));
    let block = slice_after(&main, "HotkeyAction::OpenEditor => {", 10);
    assert!(
        block.contains("reveal_editor_window(&win)"),
        "the ticket requires the hotkey to 'behave identically' to the tray action, which means \
         also routing through reveal_editor_window rather than only show()/set_minimized(false)"
    );
}

#[test]
fn open_editor_after_capture_calls_reveal_editor_window() {
    let main = code_only(&read("src/main.rs"));
    let block = slice_after(
        &main,
        "let reveal = REVEAL_EDITOR_AFTER_CAPTURE.replace(false);",
        6,
    );
    assert!(
        block.contains("reveal_editor_window(&main)"),
        "'open Editor after capture' is the ticket's fourth entry point and must route through \
         reveal_editor_window exactly like the tray and hotkey paths"
    );
}
