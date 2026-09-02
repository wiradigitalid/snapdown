# 19: The ribbon acts on the canvas — Share gone, Assemble gone, Copy becomes Copy Image

**What to build:** The Editor's ribbon action group holds three buttons today and only one of them
belongs there. **Share** is removed — it has nothing behind it, not even a stub, and a button that does
nothing is the toolbar-badge mistake in another place; its excuse row in the callback-reachability
ratchet goes with it, or that test fails. **Assemble** is removed from the ribbon — it acts on the
filmstrip's ticked selection, not on the canvas beside it, and the filmstrip footer already carries
Assemble in the right place; the context-menu entry stays, so Assemble keeps two doors. **Copy** is
renamed **Copy Image** — it copies the burned image of the Finding on the canvas and never Markdown,
and the two context menus already say "Copy image". Then size and place the one surviving button so
the group reads as intentional rather than as a gap where two buttons used to be; the old width was
chosen to sit beside a wider Assemble that is no longer there. This is the rewritten ribbon-sizing
ticket's work, and completing it marks that ticket done.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Share and ribbon Assemble no longer exist in the ribbon; the filmstrip footer's Assemble and the
      context-menu Assemble both still work
- [ ] The `share-bundle-clicked` callback is gone from the UI and from the excused list; the
      callback-reachability ratchet passes in both directions
- [ ] The remaining button reads Copy Image, keeps its behaviour and its toast, and its icon is
      coloured like its label in both themes
- [ ] The action group is sized and placed per the design-system guide so it reads as designed on its
      own, in both themes; the design-system test passes
- [ ] `.scratch/ribbon-action-button-sizing/issues/01` is marked done with a pointer to this ticket
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] **Look at:** the ribbon in both themes — one action button, Copy Image, sitting where a
      designer would have put it; press it and confirm the image lands on the clipboard; tick Findings
      and confirm Assemble still works from the filmstrip footer and from right-click
