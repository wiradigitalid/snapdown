# 06: Show the product's icon on Settings' About tab

**What to build:** Add Snapdown's own icon to the existing "SNAPDOWN" card in Settings' About tab
(`settings.slint:901`), beside or above the version line. Confirm at build time whether Slint's
`@image-url` loads `apps/desktop/assets/app-icon.ico` directly, or whether a `.svg`/`.png` export of the
same mark needs to be added to `assets/icons/` first — a builder decision either way.

**Blocked by:** None (can start immediately). Smallest, most isolated ticket in this spec — touches
only `apps/desktop/ui/settings.slint` and possibly one new asset file.

**Status:** done

Realizes `FR-27`'s home (the tab that names the product now also shows it); `BR-121` governs the
product's name elsewhere and is not itself touched by this ticket.

## Seam

Component/wiring test: the About tab's `SdCard` instantiates an image element bound to the icon asset
(existence check on the actual rendered element, not a comment or a decorative property nothing reads).

## Acceptance

- [ ] Snapdown's icon appears on the About tab's "SNAPDOWN" card
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] **Look at:** open Settings → About in the real build, confirm the icon renders crisply at the
      chosen size
