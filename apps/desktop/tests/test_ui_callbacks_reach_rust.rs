//! Every callback the UI declares must reach Rust, or be declared unreachable on purpose.
//!
//! This is the cheapest useful form of the composition test class `OQ-23` asks for, and it exists
//! because of this repository's signature failure: a part is built, unit-tested, and joined to
//! nothing. `BUG-4`, `BUG-5`, `BUG-6`, the `EmptyState` sweep and `BUG-19` are all that shape, and on
//! 2026-08-27 six more instances were found in one file — `marker-placed`, `delete-marker-clicked`,
//! `assemble-bundle-clicked`, `copy-image-clicked`, `share-bundle-clicked` and `paste-clicked` were
//! all declared in `appwindow.slint`, and nothing in `main.rs` listened to any of them. Seven tool
//! buttons and a fully drawn Assemble tile did nothing at all, and every test in the repository was
//! green.
//!
//! `V12` does not help: it checks that an `LC` is REGISTERED, not that it is REACHED.
//!
//! Two things this file asserts, and the second matters as much as the first:
//!
//! 1. a declared callback has a handler;
//! 2. a handler is not a `println!` stub. A stub is worse than a gap, because from the UI it is
//!    indistinguishable from a working action.

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// The callbacks `appwindow.slint` declares, as kebab-case names.
fn declared_callbacks() -> Vec<String> {
    let ui = read("ui/appwindow.slint");
    let mut names: Vec<String> = Vec::new();
    for line in ui.lines() {
        let line = line.trim();
        if line.starts_with("//") {
            continue;
        }
        let rest = line
            .strip_prefix("callback ")
            .or_else(|| line.strip_prefix("pure callback "));
        let Some(rest) = rest else {
            continue;
        };
        let Some(name) = rest.split(['(', ' ', ';']).next() else {
            continue;
        };
        if !name.is_empty() && !names.iter().any(|n| n == name) {
            names.push(name.to_string());
        }
    }

    assert!(
        names.len() >= 15,
        "only {} callbacks parsed out of appwindow.slint — the parser has fallen behind the file's \
         syntax, which would make the assertions below pass by vacuity",
        names.len()
    );
    names
}

/// Callbacks with no Rust handler on purpose, each with the reason it is not a defect.
///
/// A name may sit here only while the feature behind it is genuinely absent. It is not a place to
/// park something that was forgotten — that is the state this whole file exists to make visible.
///
/// `paste-clicked` left this list on `FR-35`: `on_paste_clicked` now reads the Windows clipboard
/// through `paste_clipboard_image` and turns what it finds into a new Finding.
const DELIBERATELY_UNHANDLED: &[(&str, &str)] = &[];

/// Handlers that exist and do nothing but print, each with what is actually missing behind it.
///
/// These are worse than an absent handler and that is why they are listed separately: the button
/// looks live, the click is accepted, and the Reviewer is told nothing.
const KNOWN_STUBS: &[(&str, &str)] = &[(
    "bundles-drawer-clicked",
    "the drawer is always open; nothing toggles it",
)];

fn snake(name: &str) -> String {
    format!("on_{}", name.replace('-', "_"))
}

#[test]
fn every_ui_callback_has_a_rust_handler_or_is_declared_unreachable() {
    let main_rs = read("src/main.rs");
    let mut missing: Vec<String> = Vec::new();

    for name in declared_callbacks() {
        let handler = snake(&name);
        let handled = main_rs.contains(&format!("{handler}("));
        let excused = DELIBERATELY_UNHANDLED.iter().any(|(n, _)| *n == name);

        match (handled, excused) {
            (true, false) | (false, true) => {}
            (false, false) => missing.push(format!(
                "{name}: declared in appwindow.slint, fired by the UI, and `{handler}` appears \
                 nowhere in main.rs. Either handle it or add it to DELIBERATELY_UNHANDLED with the \
                 reason the feature is absent"
            )),
            // The release half of the ratchet. An excused callback that has since been wired must
            // leave the list, or the list stops describing anything.
            (true, true) => missing.push(format!(
                "{name} is handled by `{handler}` but is still listed as DELIBERATELY_UNHANDLED. \
                 Remove the entry"
            )),
        }
    }

    assert!(
        missing.is_empty(),
        "{} UI callback(s) do not reach Rust:\n  {}",
        missing.len(),
        missing.join("\n  ")
    );
}

#[test]
fn a_handler_that_only_prints_is_recorded_as_a_stub() {
    let main_rs = read("src/main.rs");
    let mut surprises: Vec<String> = Vec::new();

    for name in declared_callbacks() {
        let handler = snake(&name);
        let Some(at) = main_rs.find(&format!("{handler}(")) else {
            continue;
        };

        // The handler's body, up to the end of the registration. Long enough to see what it does and
        // short enough not to run into the next one.
        let body_end = main_rs[at..]
            .find("\n    });")
            .or_else(|| main_rs[at..].find("\n    );"))
            .map(|offset| at + offset)
            .unwrap_or_else(|| (at + 400).min(main_rs.len()));
        let body = &main_rs[at..body_end];

        let only_prints = body.contains("println!")
            && !body.contains("_store")
            && !body.contains("load_")
            && !body.contains("toast(")
            && !body.contains("set_");
        let recorded = KNOWN_STUBS.iter().any(|(n, _)| *n == name);

        if only_prints && !recorded {
            surprises.push(format!(
                "{name}: `{handler}` does nothing but print. A button that accepts a click and \
                 reports nothing is worse than one that is not wired — record it in KNOWN_STUBS \
                 with what is missing behind it, or implement it"
            ));
        }
        if !only_prints && recorded {
            surprises.push(format!(
                "{name}: `{handler}` now does real work but is still listed in KNOWN_STUBS. Remove \
                 the entry"
            ));
        }
    }

    assert!(
        surprises.is_empty(),
        "{} handler(s) disagree with KNOWN_STUBS:\n  {}",
        surprises.len(),
        surprises.join("\n  ")
    );
}

/// The two lists above are documentation, and documentation with no reason in it rots fastest.
#[test]
fn every_excused_callback_states_why() {
    for (name, reason) in DELIBERATELY_UNHANDLED.iter().chain(KNOWN_STUBS.iter()) {
        assert!(
            reason.len() >= 30,
            "{name}'s reason is too short to be one: {reason:?}"
        );
    }
}
