# W6-S3 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w6-desktop-experience/`
- `story_id`: `W6-S3`

## This defect is photographed, not argued

`AUDIT-4` ran the first properly built binary on 2026-08-24 and shot the Settings panel. Two things
are visible in one image:

- **A vertical scrollbar down the right edge**, with the `Hotkeys` group cut off at the window's
  bottom. `FR-29` requires Settings to fit at **1024×720 without scrolling**.
- **The `General` group is more than half empty** while that cut-off is happening.

That pairing is the whole defect. The layout stretches each group to match its neighbour's height, so
space is spent where there is nothing to show and withheld where there is. The owner asked for this
surface to be dense and asked specifically not to have to scroll it.

## What the design already specifies — do not redesign it

`.how/settings/01-ux/DESIGN.md` § `Settings (LC-015)` opens with the rule in bold:

> **Two columns, packed by content height, never stretched.**

and gives the arrangement:

| Column A | Column B |
|---|---|
| Startup | Quality Budget |
| Vault folder | Hotkeys |

Its three tokens are already defined and are the ones to use:

- `--settings-group-gap` — `var(--space-4)`, between groups in a column
- `--settings-column-min` — `380px`, below which the two columns become one
- `--settings-row-height` — `32px`, one control row, **so a group's height is countable in advance**

That last one is the point of the token. A layout whose heights can be counted is a layout whose fit
at 1024×720 can be asserted rather than eyeballed.

## The third test is not layout, and it is the easiest thing here to break by accident

```
vitest::agent_access_is_a_primary_surface_and_not_a_settings_group
```

**Agent access is a primary surface reachable from the navigation rail** (`FR-28`, and `LC-028`
owns the rail). It is not a group inside Settings. Rebuilding this panel into four groups is exactly
the moment somebody folds a fifth thing into it because it looks like configuration.

## The other two tests

```
vitest::all_four_settings_groups_are_visible_at_the_minimum_window_size
vitest::no_group_is_stretched_to_match_a_neighbours_height
```

**Neither may be written as a test that cannot fail**, and this surface makes that easy to do wrong.
Asserting that a CSS class is present proves nothing about fit; asserting a hardcoded pixel total
proves nothing when the token changes. `contrast.test.ts` in `web/ui/src/test/` is the pattern to
follow — it parses `tokens.css` and derives its expectations, and it was verified by mutation. Say in
the plan how each of these two tests fails when the layout regresses.

`--settings-column-min: 380px` and `--settings-row-height: 32px` are the inputs a countable assertion
can be built from.

## Boundaries

- `W6-S1` landed the colour tokens and the lint rule that refuses a literal anywhere outside
  `web/ui/src/styles/tokens.css`. **Every colour goes through a token**, and the four deliberately
  theme-invariant groups are the only exception and already live in that file.
- `W6-S2` landed `LC-028`, the editor shell and the navigation rail. Use it; do not rebuild it.
- The Quality Budget group's **contents** are `W6-S4`, `Run at Windows startup`'s behaviour is
  `W6-S5`, and the hotkey rows are `W6-S6`. This story owns the **frame those four groups sit in**,
  not what is inside them. Leave their internals to their own stories and say so if the boundary
  turns out to be unclear anywhere.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation is **reported** and becomes a `DEC-`.
- **Verification is run, not assumed.** All of `AGENTS.md` § Code — the web half matters most here.
  Four traps are recorded there; the newest is that `cmd; echo "EXIT=$?"` makes the harness report 0
  whatever `cmd` did.
- **A green unit test does not mean the component is reachable.** Before closing any story that adds
  a component, grep for `<ComponentName` across `apps/desktop/src` and `web/ui/src`, excluding its own
  file and its tests. Four components once shipped mounted nowhere in this repository.
- **Write UTF-8, no BOM.** No scratch files in the commit, never a captured screenshot, and **do not
  push.**

## Done means

`_bmad-output/specs/w6-desktop-experience/stories/W6-S3-*.md` exists, carries an `<intent-contract>`,
and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
