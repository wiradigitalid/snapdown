# W6-S2 · Step 2 — BUILD

Implement the story. Commit locally. **Never push** — the coordinator pushes.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 2. Read `AGENTS.md` first — its
`## Code` section carries the verification commands and the pitfalls, and the first pitfall there is
this story's whole reason for existing.

Run `bmad-build-auto` with the spec file path:

`_bmad-output/specs/w6-desktop-experience/stories/W6-S2-the-editor-shell-persona-naming-the-navigation-rail-one-executable-and-the-overlay-mount.md`

Its frontmatter reads `status: ready-for-dev`. This step ends when it reads `status: done`.

## Priority order

**`BUG-4` first. It is critical and it outranks the rest of the story.**

The capture path does not work today: `capture.rs:106` opens the overlay window at
`index.html?overlay=true`, `main.tsx` ignores `window.location.search` and mounts `<App />`
unconditionally, so pressing the capture hotkey shows Settings. `FR-1`, `FR-2`, `UC-1`, `UC-2` unmet.

**The test that matters asserts the MOUNT DECISION from the URL**, not that `CaptureOverlay` renders.
The latter already exists and already passes — which is exactly why four waves missed this.

Then the shell: `LC-028`, the rail, the persona title, the one-executable build assertion.

## What W6-S1 already landed — use it, do not reinvent it

Every colour is a token and a lint rule refuses a literal. `SegmentedControl`, `HotkeyChip`,
`EmptyState`, `ErrorState`, `Badge`, and a three-state `Toggle` exist in `@snapdown/ui`.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal to
  escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation from an SDD or an `AD-N` is **reported** and becomes a `DEC-`.
- **Verification is run, not assumed.** Every command in `AGENTS.md` § Code, all of it green. A green
  `korpus.yml` is not proof the code compiles.
- **Never commit a captured screenshot or an accessibility-tree dump.** This repository is public and
  CI now refuses them. A failure there is a finding about the content, not about the check — do not
  weaken the guard.
- **Do not push.**

## Done means

The spec's frontmatter reads `status: done`, every verification command is green, and the work is
committed on this worktree's branch.

Report `worker_done` with `--outcome succeeded` and `--files-modified`, or `--outcome failed` with the
blocking reason. Do not encode failure only in prose.
