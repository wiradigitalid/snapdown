# 04: Copy Markdown to clipboard automatically on a successful Bundle save

**What to build:** A successful Assemble & Save (`on_bundle_preview_confirmed`'s `Ok` arm) and a
successful Review & Update Save (`on_review_update_save_clicked`'s `Saved` arm) each also copy the
Bundle's Markdown to the clipboard — reusing the exact function `copy-markdown-clicked`'s own handler
already calls, not a second implementation. Only a *successful* save copies; a failed save (either
path) copies nothing and shows only its existing failure toast. The toast after either save says the
same thing Copy Markdown's own toast already says, including that the copied content carries absolute
image paths.

**Blocked by:** None (can start immediately). Touches `apps/desktop/src/main.rs` only, in the two named
handlers — independent of every other ticket in this spec.

**Status:** ready-for-agent

Realizes `FR-10`, `FR-12`, `FR-40`. See `.scratch/post-testing-polish/spec.md` Implementation Decisions
§ "Copy on save" for the full design.

## Seam

Assert the SAME function name (the existing clipboard-write function `copy-markdown-clicked`'s handler
calls) appears in all three call sites — the existing one plus the two new ones — not a re-description
of the clipboard write a third time. A failed-save test path asserts zero clipboard-write calls.

## Acceptance

- [ ] A successful Assemble & Save copies the Bundle's Markdown to the clipboard, via the same function
      Copy Markdown already calls
- [ ] A successful Review & Update Save (editing mode) does the same
- [ ] A failed save on either path copies nothing and shows only its existing failure toast
- [ ] The success toast on either save path matches Copy Markdown's own wording, including the
      absolute-path disclosure
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] **Look at:** confirm the copy-on-save toast doesn't feel like noise stacked on top of the
      existing save-success toast — not asked to fix, just worth a look (Testing Decisions in the spec
      names this explicitly)
