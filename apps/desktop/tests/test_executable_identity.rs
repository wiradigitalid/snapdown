//! `FR-27` promises three names: the tray and the installed executable are `Snapdown`, and the
//! workspace window titles itself `Snapdown Editor`. Its own consequence promises "a test asserts
//! the three against one source" - and until 2026-09-02 exactly one of the three was asserted here.
//! That is how `DEC-007`'s rewrite from Tauri to Slint dropped the Editor's name in silence and
//! nothing noticed for days: `BUG-89`.
//!
//! The one source is the crate's `[[bin]]` name. **Nothing below restates the product's name.** The
//! other two names are derived from it, so renaming the product in `Cargo.toml` makes every
//! assertion here demand the new name at every site, instead of going quietly green on a copy of an
//! old input - `AGENTS.md`: "a test that asserts a literal is a test that cannot fail."

use std::fs;
use std::path::PathBuf;

fn desktop_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let path = desktop_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// Every `[[bin]]` name declared by this crate, in declaration order.
fn declared_bin_names() -> Vec<String> {
    let mut names = Vec::new();
    let mut in_bin = false;
    for line in read("Cargo.toml").lines() {
        let trimmed = line.trim();
        if trimmed == "[[bin]]" {
            in_bin = true;
            continue;
        }
        if in_bin && trimmed.starts_with('[') {
            in_bin = false;
        }
        if in_bin && trimmed.starts_with("name") {
            if let Some((_, val)) = trimmed.split_once('=') {
                names.push(val.trim().trim_matches('"').trim_matches('\'').to_string());
            }
        }
    }
    names
}

/// The single source the other two names are measured against.
fn product_name() -> String {
    let names = declared_bin_names();
    assert_eq!(
        names.len(),
        1,
        "FR-27 rests on there being one executable to name; found {names:?}"
    );
    names.into_iter().next().unwrap()
}

/// Every double-quoted literal in a Slint or Rust source, in order of appearance.
fn string_literals(source: &str) -> Vec<String> {
    source
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

#[test]
fn desktop_crate_declares_exactly_one_binary_named_snapdown() {
    assert_eq!(
        declared_bin_names(),
        vec!["Snapdown"],
        "AD-11 / BR-121: Desktop app must declare exactly one binary named 'Snapdown'"
    );
}

#[test]
fn the_tray_wears_the_executables_own_name() {
    let source = read("src/tray.rs");
    let tooltip = source
        .split_once(".with_tooltip(\"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(name, _)| name.to_string())
        .expect("tray.rs must set a tooltip; FR-27 names the tray as one of the three");

    assert_eq!(
        tooltip,
        product_name(),
        "FR-27: the tray is the product itself, so its tooltip is the executable's own name"
    );
}

#[test]
fn the_workspace_window_titles_itself_the_editor() {
    let source = read("ui/appwindow.slint");
    let after = source
        .split_once("export component AppWindow inherits Window {")
        .expect("appwindow.slint must declare AppWindow")
        .1;
    let title = after
        .split_once("title:")
        .and_then(|(_, rest)| rest.split_once('"'))
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(name, _)| name.to_string())
        .expect("AppWindow must set a title");

    assert_eq!(
        title,
        format!("{} Editor", product_name()),
        "FR-27: the workspace window titles itself '<product> Editor'. DEC-003 puts the tray and \
         the Editor in one process, so the window's own name is the only thing telling the \
         Reviewer which persona they are looking at"
    );
}

#[test]
fn no_label_in_the_editor_window_wears_the_bare_product_name() {
    let product = product_name();
    let source = read("ui/appwindow.slint");

    // `no-frame: true` - the window draws its own titlebar, so the label a Reviewer actually reads
    // is a `Text` inside the file rather than the OS title. Both must say the same thing, and this
    // is the assertion that catches the one BUG-89 left behind at `:1630`.
    let offenders = string_literals(&source)
        .into_iter()
        .filter(|lit| lit == &product)
        .count();

    assert_eq!(
        offenders, 0,
        "FR-27: no label in the Editor window may be the bare product name '{product}'. This \
         window belongs to the Editor persona; the tray is what carries the bare name. The capture \
         overlay's own title is unaffected because it names itself past the product name."
    );
}
