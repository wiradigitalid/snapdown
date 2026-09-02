# 01: Size the ribbon's remaining action button

**Superseded in scope on 2026-08-31.** This ticket originally asked for **Assemble, Copy and Share**
to be equalised at 60×44px. Two of those three buttons are being removed by the action-vocabulary
rework settled on `.scratch/bundle-library/map.md`, so equalising all three is wasted work:

- **Share** (`appwindow.slint:1949`) — removed. It has no Rust behind it at all: no handler, no
  stub, not even a `println!`. Its excuse row in `DELIBERATELY_UNHANDLED`
  (`apps/desktop/tests/test_ui_callbacks_reach_rust.rs:69-72`) must be deleted with it, or the
  test's ratchet fails.
- **Assemble** (`appwindow.slint:1905`) — removed from the ribbon. It acts on the filmstrip
  selection, not the canvas, and the filmstrip footer already carries an Assemble button beside the
  "N Findings" readout (`appwindow.slint:2447`). The context-menu entry (`:1439`) stays, so Assemble
  keeps two doors.
- **Copy** (`appwindow.slint:1927`) — kept, renamed **Copy Image**, since it copies the burned image
  of the active Finding (`copy_burned_image`, `main.rs:1275-1335`) and never Markdown. The name now
  matches the two context menus that already say "Copy image".

**What to build:** after the removals, the ribbon's right-hand action group holds a single button.
Give it a size and placement that looks deliberate on its own rather than like a gap where two
buttons used to be — the old 52px width was chosen to sit beside a wider Assemble that is no longer
there. No wording change beyond the Copy → Copy Image rename.

**Blocked by:** `.scratch/bundle-library/issues/19-the-ribbon-acts-on-the-canvas.md`, which IS the
action-vocabulary rework and also carries this ticket's sizing work; completing 19 marks this done.
The original wording follows. The action-vocabulary rework itself (removal of Share and ribbon Assemble, rename of
Copy). Do not size the group before those land — the layout question is meaningless until then.

**Status:** done, by ticket 19 — 2026-09-02

Completing `.scratch/bundle-library/issues/19-the-ribbon-acts-on-the-canvas.md` marked this ticket
done, per this file's own note above: "completing 19 marks this done." The ribbon's action group now
holds one button, `Copy Image`, 84px wide by 44px tall, sized to fit its longer label without clipping
rather than sitting inside the old 52px width chosen for a shorter "Copy" beside a wider Assemble.

- [x] The ribbon's remaining action button is sized and placed so the group reads as intentional in
      both light and dark theme
- [x] The `share-bundle-clicked` row is gone from `DELIBERATELY_UNHANDLED` and the callback-reachability
      test passes
- [x] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all pass — verified 2026-09-02, all three exit 0
