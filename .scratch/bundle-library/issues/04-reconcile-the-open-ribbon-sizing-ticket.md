# 04: Reconcile the open ribbon-sizing ticket with the action-vocabulary rework

**Type:** task
**Status:** resolved
**Blocked by:** None (can start immediately)

## Question

An already-open ticket,
`.scratch/ribbon-action-button-sizing/issues/01-equalize-ribbon-action-buttons-to-60x44.md`, asks for
**Assemble, Copy and Share** to be equalised to 60×44px. Two of those three buttons are being deleted
by this map's settled vocabulary rework:

- **Share** (`appwindow.slint:1949`) — deleted. It has *zero* Rust behind it: no handler, no stub,
  not even a `println!`. It is currently excused in `DELIBERATELY_UNHANDLED`
  (`apps/desktop/tests/test_ui_callbacks_reach_rust.rs:69-72`) on the grounds that publishing is
  frozen by `DEC-005`. That entry must go with the button, and the test has a ratchet that will fail
  if it is left behind.
- **Assemble** (`appwindow.slint:1905`) — deleted from the ribbon. It acts on the filmstrip
  selection, not on the canvas, and the filmstrip footer already carries an Assemble button in the
  right place (`appwindow.slint:2447`), beside the "N Findings" readout with a visible enabled
  state. The context-menu entry ("Assemble selected…", `:1439`) stays too, so Assemble keeps two
  doors.
- **Copy** (`appwindow.slint:1927`) — kept, renamed **Copy Image**. It copies the burned image of
  the active Finding (`copy_burned_image`, `main.rs:1275-1335`), never Markdown, and the new name
  says so — matching the two context menus that already say "Copy image".

Sizing three buttons where one will remain is wasted work, so decide and act:

- Close or rewrite the ribbon-sizing ticket. If rewritten, it becomes a sizing/layout question about
  the ribbon's *remaining* right-hand group, not the old trio.
- Record the outcome so nobody picks up the stale ticket in the meantime.

This is a task, not a decision: the vocabulary itself is already settled on the map. What is left is
tidying the tracker so the two efforts do not collide.

## Answer

Resolved 2026-08-31. The ribbon-sizing ticket was **rewritten rather than closed**, because a real
layout question survives the removals: the ribbon's right-hand group drops from three buttons to
one, and the surviving button's 52px width was chosen to sit beside a wider Assemble that will no
longer be there.

`.scratch/ribbon-action-button-sizing/issues/01-equalize-ribbon-action-buttons-to-60x44.md` now:

- records what was superseded and why, naming all three buttons and their fates;
- asks only that the **remaining** button be sized and placed so the group reads as intentional;
- carries `Status: blocked` on the vocabulary rework, so nobody sizes a group that is still about to
  change shape;
- adds an acceptance criterion that the `share-bundle-clicked` row be removed from
  `DELIBERATELY_UNHANDLED`, since that test has a ratchet which fails if the excuse outlives the
  button.

The filename still reads `...-equalize-...-60x44`, which no longer describes its contents. Left as
is deliberately: renaming it would break any reference already pointing at that path, and the
ticket's own first line states the supersession.
