# W6-S3 · Step 2 — BUILD

The plan is done and approved. Implement it.

Read `AGENTS.md` first. Run `bmad-build-auto` with the spec path:

`_bmad-output/specs/w6-desktop-experience/stories/W6-S3-two-columns-packed-by-content-height-four-groups-visible-at-1024x720.md`

The spec is complete and its `<intent-contract>` is the owner's. **Do not edit anything inside it.**
The token values, the column arrangement, the geometry calculation and the three tests with their
regression modes are all already written there. This step ends when its frontmatter reads
`status: done`.

## What the plan found, so you do not have to look for it

The current layout is `display: grid; grid-template-columns: 1fr 1fr;` with **stretched rows**, and
`VaultSection` and `HotkeySection` stacked full-width beneath. That is why the `General` group is more
than half empty while `Hotkeys` is cut off below the fold — the grid stretches every group to match
its row-mate, spending height where there is nothing to show.

The replacement is two independent flex column stacks with `align-items: flex-start`, each group
packed to its own content height.

| Column A | Column B |
|---|---|
| Startup | Quality Budget |
| Vault folder | Hotkeys |

## Three tokens, and they go where every other token goes

`--settings-group-gap`, `--settings-column-min`, `--settings-row-height`, defined in
`.how/settings/01-ux/DESIGN.md` with their values.

**Colour lives in exactly one file and so do these**: `web/ui/src/styles/tokens.css`. `W6-S1` landed a
lint rule that refuses a colour literal anywhere else, and the same discipline applies to a layout
constant a test needs to read.

`--settings-row-height: 32px` exists so a group's height is **countable in advance**. That is what
makes the fit at 1024×720 assertable rather than eyeballed.

## The tests must fail when the layout regresses

The plan states the regression mode for each one. Keep them:

- **fit at minimum size** — parses the three tokens from `tokens.css`, computes both columns at
  1024×720 with the 200px rail, asserts the taller column plus padding stays under 720. Regressing to
  the stacked layout pushes it past 760 and the assertion fails.
- **no group stretched** — asserts the two columns are separate containers and the Startup group's
  height is its own content, not Quality Budget's. Replacing them with a stretched grid fails it.
- **agent access is not a Settings group** — asserts the surface renders exactly four configuration
  groups and zero Agent Access elements.

**Do not hardcode a pixel total.** `contrast.test.ts` in `web/ui/src/test/` is the pattern: it parses
`tokens.css` and derives its expectations, and it was verified by mutation. A test that copies the
input cannot fail — this wave already caught that once.

## What this story does NOT own

The **contents** of three of the four groups belong to later stories: Quality Budget is `W6-S4`,
`Run at Windows startup`'s behaviour is `W6-S5`, and the hotkey rows are `W6-S6`. This story owns the
**frame those groups sit in**. If the boundary turns out to be unclear anywhere, report it rather than
deciding it.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **Verification is run, not assumed.** All of `AGENTS.md` § Code — the web half matters most here:
  `web/ui` typecheck, lint and test, then `apps/desktop` typecheck, lint, test and build. Four traps
  are recorded there; the newest is that `cmd; echo "EXIT=$?"` makes the harness report 0 whatever
  `cmd` did.
- **A green unit test does not mean the component is reachable.** Grep for `<ComponentName` across
  `apps/desktop/src` and `web/ui/src` before closing, excluding its own file and its tests.
- **Write UTF-8, no BOM, keep trailing newlines.** No scratch files in the commit — `W6-S9` left a
  `test_ro.rs` at the repo root. Never a captured screenshot. **Do not push.**
- **Set the frontmatter to `status: done` when you are finished.**

## Done means

`npm --prefix web/ui run typecheck && lint && test` and
`npm --prefix apps/desktop run typecheck && lint && test && build` all exit **0**, the three named
tests execute, and the spec's frontmatter reads `status: done`.

Report `worker_done` with `--outcome succeeded`, or `--outcome failed` with the blocking reason.
