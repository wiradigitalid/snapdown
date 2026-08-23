# W6-S1 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. Do not write application code. This step ends when the
story spec file exists with frontmatter `status: ready-for-dev`.

## Method position

This repo uses **WDI Method** (it wraps BMad). This work sits at **G5 Release**, wave **W6**, run
through `wdi-build` Phase 3 Step 1. Read `AGENTS.md` at the repo root first.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w6-desktop-experience/`
- `story_id`: `W6-S1`

Resolve everything else from `{spec_folder}/stories.yaml` and `{spec_folder}/SPEC.md`.

## The story

**W6-S1 — Every colour through tokens, both themes, enforced by a lint rule.**

It runs **alone and first**, because every other story in this wave writes UI and would otherwise each
invent its own colours again.

## Documents that bind it

Read these before planning. They are the contract; this brief is only the pointer.

- `_bmad-output/specs/w6-desktop-experience/SPEC.md` — the kernel and its `companions:` list
- `_bmad-output/specs/w6-desktop-experience/stories.yaml` — this story's `invoke_dev_with`
- `.how/_platform/design-system.md` — the tokens, the base elements, and their required states
- `.how/_platform/ARCHITECTURE-SPINE.md` — **AD-10**, quoted verbatim, is the invariant this story
  exists to make true
- `.how/_platform/cross-cutting.md` § Colour, theme, and contrast
- `web/ui/src/styles/tokens.css` — what exists today
- `.constitution/project/codebase-stack-guide.md` — the verification commands and their directories

## What the plan must cover

1. The tokens missing from `tokens.css`, **defined in both themes**: the four meaning pairs
   (`--color-success-bg`/`-text`, `--color-warning-*`, `--color-info-*`, `--color-neutral-*`),
   `--color-surface-sunken`, `--space-0`, `--radius-full`.
2. Replacing all **23 hex literals** under `apps/desktop/src/**/*.tsx` with token references. They are
   in `HotkeySection.tsx`, `BundleView.tsx`, `App.tsx` and their neighbours — find them, do not trust
   this list.
3. The base elements `design-system.md` names that do not exist yet: `SegmentedControl`, `HotkeyChip`,
   `EmptyState`, `ErrorState`, and **`Toggle`'s indeterminate state**.
4. The lint rule that refuses a colour literal outside the token file.
5. The contrast assertion, and the both-themes render test.

## Three things that are easy to get wrong

**Three token groups are theme-invariant ON PURPOSE** and must keep literal values while still living
in the token file, each with a comment saying why: `--color-marker*`, the capture overlay's scrim and
region ring, and `--canvas-checker`. They are drawn over the Reviewer's own screen content or over an
exported image that will be read on another machine under another theme, so this machine's theme is
the wrong reference for them. A lint rule that blindly refuses every literal will fight them.

**The contrast assertion checks every text element against ITS OWN background**, not against the page
background. Checking against the page is what would have passed the shipped build: the shell is dark,
the text is white, and the white panel in between is what nobody looked at.

**`Toggle`'s indeterminate state is load-bearing, not decoration.** W6-S5 needs it: `FR-18` requires
the startup control to reflect the real Windows registration and never a remembered intention, reading
that is asynchronous, and without a third state the control must guess. The shipped build guesses.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** When a test or build fails and the cause is not known,
  run `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the
  signal to escalate, not to try a fourth.
- **The corpus is not yours to change.** You MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
  A deviation from the SDD or an `AD-N` is **reported** and becomes a `DEC-` — never absorbed as a
  code patch.
- **Verification is run, not assumed.** The commands are in
  `.constitution/project/codebase-stack-guide.md`. A green `korpus.yml` is not proof the code
  compiles; they answer different questions.
- **Never commit a captured screenshot.** This repository is public and the brief forbids it.

## Done means

`_bmad-output/specs/w6-desktop-experience/stories/W6-S1-*.md` exists, carries an
`<intent-contract>`, and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec file path, or `--outcome failed` with the
blocking reason. Do not encode failure only in prose.
