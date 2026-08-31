# 01: Remove the number badges from the annotation toolbar buttons

**What to build:** The Marker, Shape, Callout, Blur, Arrow, and Text buttons in the Editor's
annotation toolbar currently each show a small numeric badge ("1" through "6") in the corner. These
badges imply a keyboard shortcut that does not exist anywhere in the app — no key handler switches
the active tool on digit press, only mouse clicks do. Remove the badges so the toolbar stops
promising a feature that isn't there. The Crop button, which already has no badge, is the reference
for how the other six should look and space out once theirs are gone.

If the badge-rendering path in the shared icon-button component ends up with no remaining caller
once these six are cleared, remove that dead code path too rather than leaving it unused.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] None of the seven annotation toolbar buttons (Marker, Shape, Callout, Blur, Arrow, Text, Crop)
      render a numeric badge
- [ ] Button spacing and alignment across the toolbar strip is unaffected by the removal, in both
      light and dark theme
- [ ] Any existing test asserting the presence of these badges is updated to match; no keyboard
      shortcut behavior is added as part of this ticket
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace` all pass
