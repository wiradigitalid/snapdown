//! WCAG contrast over `theme.slint`, which is the palette that actually ships.
//!
//! `NFR-17` puts colour in one file, and its stated enforcement is a lint plus `contrast.test.ts`.
//! Both cover `web/ui/src/styles/tokens.css` — and `web/ui` builds nothing in the active workspace
//! (`OQ-27`). The Slint surfaces are the whole product's UI and nothing checked them, which is the
//! gap `DEC-009`'s study found and this file closes.
//!
//! Two things this file deliberately does NOT do:
//!
//! - it does not invent thresholds. Where WCAG has no requirement - a decorative card border, for
//!   instance - there is no assertion. An earlier draft asserted 1.5:1 for `border-strong` on
//!   `bg-card`, reported a failure at 1.39, and the failure was in the invented bar rather than in
//!   the palette. A gate that reports noise gets ignored, and then it protects nothing;
//! - it does not lower a real bar to reach green. Four pairings fail AA today. They are listed as
//!   EXCEPTIONS with their measured ratio and the defect that owns them, so this file passes now,
//!   fails if any of them gets worse, and fails when a defect is closed without updating it.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A token's dark and light values, as `#rrggbb`.
type Pair = (String, String);

/// Parses `out property <color> name: is-dark ? #dark : #light;` out of `theme.slint`.
///
/// Tokens that are not a plain pair of opaque hex literals - anything with `.with-alpha()` - are
/// skipped rather than guessed at. Alpha contrast depends on what is behind it, which this file has
/// no way to know; the overlay's translucent tokens are the deliberately theme-invariant group and
/// are covered by their own reasoning in `DEC-009`, not by a ratio.
fn theme_tokens() -> HashMap<String, Pair> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/theme.slint");
    let source =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"));

    let mut tokens = HashMap::new();
    for line in source.lines() {
        let line = line.trim();
        if line.starts_with("//") {
            continue;
        }
        let Some(rest) = line.strip_prefix("out property <color>") else {
            continue;
        };
        let Some((name, expr)) = rest.split_once(':') else {
            continue;
        };
        let name = name.trim().to_string();
        // Strip a trailing `; // comment`.
        let expr = expr.split("//").next().unwrap_or("").trim();
        let expr = expr.strip_suffix(';').unwrap_or(expr).trim();

        if let Some((dark, light)) = expr
            .strip_prefix("is-dark ?")
            .and_then(|t| t.split_once(':'))
        {
            let (dark, light) = (dark.trim(), light.trim());
            if is_hex(dark) && is_hex(light) {
                tokens.insert(name, (dark.to_string(), light.to_string()));
            }
        } else if is_hex(expr) {
            tokens.insert(name, (expr.to_string(), expr.to_string()));
        }
    }

    assert!(
        tokens.len() >= 20,
        "only {} tokens parsed out of theme.slint - the parser has fallen behind the file's syntax, \
         which would make every assertion below pass by vacuity",
        tokens.len()
    );
    tokens
}

fn is_hex(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

fn channel(c: u8) -> f64 {
    let c = f64::from(c) / 255.0;
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn luminance(hex: &str) -> f64 {
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).expect("checked by is_hex");
    0.2126 * channel(byte(1)) + 0.7152 * channel(byte(3)) + 0.0722 * channel(byte(5))
}

/// WCAG 2.1 relative-contrast ratio.
fn ratio(a: &str, b: &str) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// `(foreground, background, required_ratio, why)`.
///
/// 4.5 is AA for normal text. 3.0 is AA for large text and for non-text UI contrast - used here for
/// a focus ring, whose visibility is what makes keyboard use possible, and for placeholder text,
/// which sits between content and decoration.
const PAIRS: &[(&str, &str, f64, &str)] = &[
    ("text-primary", "bg-app", 4.5, "body text on the window"),
    ("text-primary", "bg-card", 4.5, "body text on a panel"),
    (
        "text-primary",
        "bg-subtle",
        4.5,
        "body text on a readout strip",
    ),
    (
        "text-secondary",
        "bg-app",
        4.5,
        "labels and section headings",
    ),
    ("text-secondary", "bg-card", 4.5, "labels on a panel"),
    ("text-muted", "bg-app", 4.5, "timestamps and dimensions"),
    ("text-muted", "bg-card", 4.5, "timestamps on a panel"),
    ("text-dim", "bg-app", 3.0, "placeholder text"),
    ("text-dim", "bg-card", 3.0, "placeholder text on a panel"),
    (
        "text-on-accent",
        "accent-primary",
        4.5,
        "every primary button's label",
    ),
    (
        "text-on-accent",
        "accent-hover",
        4.5,
        "a primary button's label under the pointer",
    ),
    (
        "text-on-accent",
        "accent-pressed",
        4.5,
        "a primary button's label while pressed",
    ),
    ("border-focus", "bg-app", 3.0, "the focus ring"),
    ("border-focus", "bg-card", 3.0, "the focus ring on a panel"),
    ("accent-active", "bg-card", 4.5, "an active tool's label"),
    // Added when `accent-capture` became a token. Its inline value was #e11d48, which measures
    // 4.29 on `bg-app` and misses AA for a 10px label - found by asserting it rather than by anyone
    // looking at it.
    (
        "accent-capture",
        "bg-app",
        4.5,
        "the Capture action's label",
    ),
    (
        "accent-capture",
        "bg-card",
        4.5,
        "the Capture action's label on a panel",
    ),
    ("semantic-success", "bg-card", 4.5, "the Editing tag"),
    ("semantic-error", "bg-card", 3.0, "a Marker pin"),
];

/// Pairings that fail today, with the ratio measured on 2026-08-27 and the defect that owns them.
///
/// This list is a ratchet, not an excuse. A pairing may appear here only with a measured number and
/// an open defect, the assertions below fail if any of them gets WORSE, and the last assertion in
/// this file fails if an entry is left here after its defect is closed.
const EXCEPTIONS: &[(&str, &str, &str, f64, &str)] = &[
    ("dark", "text-on-accent", "accent-primary", 3.20, "BUG-54"),
    ("dark", "text-on-accent", "accent-hover", 2.59, "BUG-54"),
    // `accent-pressed` is NOT here: white on the dark theme's #3a72e0 measures 4.50, which clears
    // AA by a hair. It was listed on the first draft and `every_recorded_exception_still_actually_
    // fails` rejected it immediately - the ratchet's release check earning its place on its first run.
    ("light", "text-muted", "bg-app", 4.34, "BUG-54"),
    ("light", "text-dim", "bg-app", 2.34, "BUG-54"),
    ("dark", "text-dim", "bg-card", 2.99, "BUG-54"),
    ("light", "text-dim", "bg-card", 2.56, "BUG-54"),
];

fn exception(theme: &str, fg: &str, bg: &str) -> Option<f64> {
    EXCEPTIONS
        .iter()
        .find(|(t, f, b, _, _)| *t == theme && *f == fg && *b == bg)
        .map(|(_, _, _, measured, _)| *measured)
}

#[test]
fn theme_tokens_meet_wcag_aa_or_a_recorded_exception() {
    let tokens = theme_tokens();
    let mut failures: Vec<String> = Vec::new();

    for (fg, bg, want, why) in PAIRS {
        let (Some(fgv), Some(bgv)) = (tokens.get(*fg), tokens.get(*bg)) else {
            failures.push(format!(
                "{fg} on {bg}: one of these tokens is missing from theme.slint, so the pairing that \
                 covers \"{why}\" is unchecked"
            ));
            continue;
        };

        for (theme, f, b) in [("dark", &fgv.0, &bgv.0), ("light", &fgv.1, &bgv.1)] {
            let got = ratio(f, b);
            match exception(theme, fg, bg) {
                // A known failure. It must not get worse, and 0.01 of slack absorbs the rounding in
                // the recorded figure rather than any real change.
                Some(recorded) if got + 0.01 >= recorded => {}
                Some(recorded) => failures.push(format!(
                    "{theme} {fg} on {bg} ({why}): {got:.2}, WORSE than the {recorded:.2} recorded \
                     against BUG-54. A known-bad pairing may not be allowed to degrade"
                )),
                None if got >= *want => {}
                None => failures.push(format!(
                    "{theme} {fg} on {bg} ({why}): {got:.2}, needs {want}. Either fix the token or \
                     add it to EXCEPTIONS with a measured ratio and an open defect - not by \
                     lowering the requirement"
                )),
            }
        }
    }

    assert!(
        failures.is_empty(),
        "theme.slint fails WCAG AA on {} pairing(s):\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

/// The ratchet has to release. An exception left behind after its defect is fixed is how a list like
/// this stops meaning anything.
#[test]
fn every_recorded_exception_still_actually_fails() {
    let tokens = theme_tokens();

    for (theme, fg, bg, recorded, defect) in EXCEPTIONS {
        let want = PAIRS
            .iter()
            .find(|(f, b, _, _)| f == fg && b == bg)
            .map(|(_, _, w, _)| *w)
            .unwrap_or_else(|| {
                panic!("{fg} on {bg} is an exception to a pairing that is not listed")
            });

        let (fgv, bgv) = (
            tokens.get(*fg).expect("exception names a real token"),
            tokens.get(*bg).expect("exception names a real token"),
        );
        let got = if *theme == "dark" {
            ratio(&fgv.0, &bgv.0)
        } else {
            ratio(&fgv.1, &bgv.1)
        };

        assert!(
            got < want,
            "{theme} {fg} on {bg} now measures {got:.2} and passes its {want} requirement, but is \
             still listed as an exception owned by {defect} (recorded at {recorded:.2}). Remove the \
             entry and close the defect"
        );
    }
}

/// The one pairing this file deliberately does not assert, recorded so it is not re-added.
///
/// `border-strong` on `bg-card` measures 1.39 dark and 1.48 light. An earlier draft asserted 1.5:1
/// and reported both as failures - but WCAG's 3:1 non-text requirement covers boundaries ESSENTIAL to
/// identifying a control, and a card's outline is not one. The 1.5 was invented, and inventing a bar
/// then reporting a failure against it is how a gate earns the right to be ignored.
///
/// `border-focus` IS asserted, at 3.0, because a focus ring is exactly the essential case.
#[test]
fn a_decorative_border_is_not_held_to_a_text_requirement() {
    assert!(
        !PAIRS
            .iter()
            .any(|(fg, bg, _, _)| *fg == "border-strong" && *bg == "bg-card"),
        "border-strong on bg-card must not be asserted: WCAG has no requirement for a decorative \
         boundary, and the 1.5:1 bar an earlier draft used here was invented"
    );
    assert!(
        PAIRS
            .iter()
            .any(|(fg, bg, want, _)| *fg == "border-focus" && *bg == "bg-app" && *want == 3.0),
        "the focus ring must still be asserted at 3:1 - that is the case WCAG's non-text \
         requirement is actually about"
    );
}

/// Colour lives in `theme.slint` and nowhere else, and this is what says so for the Slint surfaces.
///
/// `NFR-17` puts colour in one file and names a lint as the enforcement - a lint that covers
/// `web/ui/src/styles/tokens.css`, which builds nothing in the active workspace (`OQ-27`). The
/// Slint files had no equivalent, and on 2026-08-27 nine literals were found across them: a close
/// button's hover red, a white glyph, the canvas ground twice, two shadows, a filmstrip card ground,
/// and the Capture label at #e11d48 - which turned out to miss AA on `bg-app` and was only measured
/// because turning it into a token brought it into the gate above.
///
/// A literal is not merely untidy: it exists in exactly one theme. Every one of those nine painted
/// the same pixels in dark mode as in light.
#[test]
fn colour_lives_only_in_theme_slint() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let mut files = vec![root.join("appwindow.slint")];
    let components = root.join("components");
    let mut component_files: Vec<_> = fs::read_dir(&components)
        .unwrap_or_else(|e| panic!("Failed to list {components:?}: {e}"))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "slint"))
        .collect();
    component_files.sort();
    assert!(
        !component_files.is_empty(),
        "no component .slint files found - this test would then only cover one file"
    );
    files.append(&mut component_files);

    let mut offenders: Vec<String> = Vec::new();
    for path in &files {
        let source =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"));
        for (number, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            // Only the colour syntax, and only outside a comment on the same line.
            let code = trimmed.split("//").next().unwrap_or("");
            let mut rest = code;
            while let Some(at) = rest.find('#') {
                let after = &rest[at + 1..];
                let hex: String = after
                    .chars()
                    .take_while(|c| c.is_ascii_hexdigit())
                    .collect();
                if hex.len() == 3 || hex.len() == 6 || hex.len() == 8 {
                    offenders.push(format!(
                        "{}:{}: #{hex}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        number + 1
                    ));
                }
                rest = &after[hex.len()..];
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "{} colour literal(s) outside theme.slint. Add a token to theme.slint - with a comment          saying why, if it is deliberately theme-invariant - and reference it instead:
  {}",
        offenders.len(),
        offenders.join("
  ")
    );
}
