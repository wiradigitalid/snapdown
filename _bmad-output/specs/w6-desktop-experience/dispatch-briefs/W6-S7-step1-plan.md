# W6-S7 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first — its
`## Code` section carries the verification commands, and its **first pitfall is what this story
exists to fix.**

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w6-desktop-experience/`
- `story_id`: `W6-S7`

Resolve everything else from `{spec_folder}/stories.yaml` and `{spec_folder}/SPEC.md`.

## Priority order — and this matters more than usual

This story is third in the wave and carries **two defects plus a surface rebuild**. Plan it in this
order, because if it runs long the first item is the one that must land:

### 1. `BUG-5` — critical. The Editor never renders a Finding's image

`MarkerLayer` is exported from `web/ui/src/index.ts` and **mounted nowhere**. Grep for `<MarkerLayer`
across `apps/desktop/src` and `web/ui/src`, excluding its own file and its tests: one hit, the export.

`web/ui/src/components/FindingsEditor.tsx` renders metadata, a Note `TextArea`, and — line 137 —
`{f.markers.length} markers` **as text**. `apps/desktop/src/components/FindingsView.tsx` contains no
`img`, no `convertFileSrc`, nothing.

So: **the screenshot the Note describes is not on screen.** Markers cannot be placed, moved, or
deleted, because there is no canvas to place them on. `FR-8` and `UC-5` are entirely unmet, and
`AD-1` — *Markers and Note lines are one sequence*, the invariant this whole product is built on —
has **no user interface**. `BG-1` says a note is unambiguously attached to the image it describes;
that attachment lives in the database and is invisible to the Reviewer.

**The tests that matter are composition tests.** `marker_layer.test.tsx` is already green — the
component is correct and that is precisely why nobody noticed. Plan tests that assert the image
element and the marker canvas are **present in the mounted Findings surface**, and that a click on
the canvas places a Marker and writes its Note line.

This repository has no composition test of any kind. These are the first.

### 2. `BUG-6` — high. The orphan report is unreachable

`OrphanReportView.tsx` exists, `orphan_report.test.tsx` is green, and nothing mounts it.
`App.tsx:23` reads `type NavigationTab = 'findings' | 'bundles' | 'agent-access' | 'settings'` — no
orphans route, tab, or entry point anywhere. `FR-15` and `UC-8` are unreachable, and `NFR-5` names
the orphan report as one of its two enforcement mechanisms, so half of that enforcement cannot be
opened.

`.how/finding/01-ux/DESIGN.md` places it as `LC-030` and offers it **from the Findings surface's
image-missing state**. That is the entry point to build.

### 3. The surface rebuild

Three regions per `.how/finding/01-ux/DESIGN.md`: the capture rail at `200px`, the canvas, the note
pane at `320px` with the Marker list beneath the Note. Every state — empty, loading,
nothing-selected, populated, image-missing, error.

Two things in it are not decoration:

- **The Marker list in the note pane is the keyboard path to every Marker.** Without it the
  accessibility floor is unreachable, because an image-only interaction cannot be operated from a
  keyboard, and Markers are load-bearing for `BG-1`.
- **A Marker with no Note line is reported in the note pane and NEVER annotated on the image**
  (`SCN-04`). The image is exported and read on another machine under another theme; an app-only
  state must not be burned into an artifact.

Panels take the height available. The shipped build uses fixed heights and leaves roughly a third of
the window dark beneath them.

## Documents that bind it

- `_bmad-output/specs/w6-desktop-experience/SPEC.md` and its `companions:` list
- `{spec_folder}/stories.yaml`, entry `W6-S7`
- `.how/finding/01-ux/DESIGN.md` — the layout and every state
- `.what/finding/04-usecases/EXPERIENCE.md` — behaviour and the accessibility floor
- `.what/finding/05-scenarios/SCN-04-the-note-line-deleted-without-its-marker.md` — **the asymmetry**
- `.what/finding/03-domain/state-machines.md` § 3 — the Marker sequence
- `.control/registry/defects.yaml` — `BUG-5`, `BUG-6`, and the systemic note under them

W6-S1 has landed: every colour is a token, a lint rule refuses a literal, and `EmptyState`,
`ErrorState`, `Badge`, `HotkeyChip`, `SegmentedControl` and a three-state `Toggle` exist in
`@snapdown/ui`. W6-S2 lands the editor shell. Use both; invent neither.

## The asymmetry, because it is the easiest thing here to get wrong

`SCN-04` requires two opposite behaviours and they are not symmetrical:

| The Reviewer does | What must happen |
|---|---|
| Deletes a **Marker** from the image | Its Note line is removed with it, and the remaining Markers renumber contiguously |
| Deletes a **numbered line** from the Note | The Marker **stays**, at its position, with its number. The note pane reports it as unbound |

The tempting implementations are both wrong: deleting the Marker to restore symmetry destroys
evidence the Reviewer did not ask to destroy, and renumbering the remaining lines silently repoints
them at the wrong badges — which is the exact defect `AD-1` exists to prevent, arrived at by trying
to satisfy `AD-1`.

The reason for the asymmetry: the **image** is edited through operations the product owns, so the
product keeps the collection consistent. The **Note** is free text, so it cannot.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal
  to escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation is **reported** and becomes a `DEC-`.
- **Verification is run, not assumed.** Every command in `AGENTS.md` § Code.
- **Never commit a captured screenshot or an accessibility-tree dump.** CI refuses them now. A
  failure there is a finding about the content, not about the check.
- **No scratch files in the commit.** A `temp_*.test.tsx` left behind is a review finding.
- **Do not push.**

## Done means

`_bmad-output/specs/w6-desktop-experience/stories/W6-S7-*.md` exists, carries an `<intent-contract>`,
and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
