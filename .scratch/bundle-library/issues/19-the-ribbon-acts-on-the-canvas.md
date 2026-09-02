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

**Status:** done

- [x] Share and ribbon Assemble no longer exist in the ribbon; the filmstrip footer's Assemble and the
      context-menu Assemble both still work. `appwindow.slint`'s ribbon Group 3 dropped from three
      hand-rolled `Rectangle` buttons to one; the footer `SdActionButton` at (post-edit) line 2407 and
      the finding-menu `assemble` entry at line 1438 are untouched.
- [x] The `share-bundle-clicked` callback is gone from the UI and from the excused list; the
      callback-reachability ratchet passes in both directions. The `callback share-bundle-clicked();`
      declaration, its `sh-touch` `TouchArea`, and its `DELIBERATELY_UNHANDLED` row in
      `test_ui_callbacks_reach_rust.rs` are all deleted together.
- [x] The remaining button reads Copy Image, keeps its behaviour and its toast, and its icon is
      coloured like its label in both themes. Label text changed to `"Copy Image"`; the
      `clicked => { root.copy-image-clicked(); }` wiring and `copy_burned_image`'s toast text are
      untouched; `colorize: Theme.text-on-accent` on the icon (the `f7ce2e0` pattern) is kept.
- [x] The action group is sized and placed per the design-system guide so it reads as designed on its
      own, in both themes; the design-system test passes. Width raised from 52px to 84px (room for the
      longer label without clipping), icon raised from 15px to 16px to match the accent weight the
      button now carries alone; `Theme.radius-sharp`, the `accent-primary`/`hover`/`pressed` triple and
      the accent-tinted shadow are unchanged, so both themes still come from `theme.slint`.
      `test_design_system.rs` has no ribbon-specific assertion; it passes because nothing it checks
      changed.
- [x] `.scratch/ribbon-action-button-sizing/issues/01` is marked done with a pointer to this ticket.
- [x] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0. Verified 2026-09-02: fmt 0, clippy 0
      (`-D warnings`), test 0 — 41 test binaries, every `test result: ok`, 0 failed.
- [ ] **Look at:** not run in this session — a background implementer run, verified by the automated
      suite only. `target/release/Snapdown.exe` was not rebuilt or launched; the owner should do this
      pass before treating the visual result as confirmed.
