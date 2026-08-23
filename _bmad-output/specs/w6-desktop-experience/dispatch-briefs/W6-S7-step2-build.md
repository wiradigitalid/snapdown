# W6-S7 · Step 2 — BUILD

Implement the story. Commit locally. **Never push** — the coordinator pushes.

## Method position

WDI Method, **G5 Release**, wave **W6**, `wdi-build` Phase 3 Step 2. Read `AGENTS.md` first — its
`## Code` section carries the verification commands, and **its first pitfall is what this story
exists to fix.**

Run `bmad-build-auto` with the spec file path:

`_bmad-output/specs/w6-desktop-experience/stories/W6-S7-findings-capture-rail-canvas-note-pane-and-every-state.md`

Its frontmatter reads `status: ready-for-dev`. This step ends when it reads `status: done`.

## Priority order — if the story runs long, item 1 is what must land

1. **`BUG-5`, critical.** Render the Finding's image, and mount `<MarkerLayer />` over it. Today the
   Editor never shows the screenshot at all: `MarkerLayer` is exported and mounted nowhere, and
   `FindingsEditor.tsx:137` prints `{f.markers.length} markers` as text. `FR-8` and `UC-5` are unmet
   and `AD-1` — the invariant the product is built on — has no user interface.
2. **`BUG-6`, high.** Give the orphan report an entry point. It exists, it is tested, and nothing
   mounts it.
3. The surface rebuild: capture rail, canvas, note pane, and every state.

## The tests that matter are composition tests, and this repo has none

`marker_layer.test.tsx` and `orphan_report.test.tsx` are already green. **The components are correct
and that is exactly why nobody noticed.** Assert that the `<img>` and the marker canvas are
**present in the mounted Findings surface**, and that a click on the canvas places a Marker and
writes its Note line. These are the first composition tests in the repository.

## The asymmetry — the easiest thing here to get wrong

| The Reviewer does | What must happen |
|---|---|
| Deletes a **Marker** from the image | Its Note line goes with it, and the rest renumber contiguously |
| Deletes a **numbered line** from the Note | The Marker **stays**, at its position, with its number. The note pane reports it unbound |

Both tempting shortcuts are wrong. Deleting the Marker to restore symmetry destroys evidence the
Reviewer did not ask to destroy. Renumbering the remaining lines silently repoints them at the wrong
badges — the exact defect `AD-1` exists to prevent, reached by trying to satisfy `AD-1`.

The reason: the **image** is edited through operations the product owns, so the product keeps the
collection consistent. The **Note** is free text, so it cannot. See `SCN-04`.

**A Marker with no Note line is reported in the note pane and NEVER annotated on the image.** The
image is exported and read on another machine under another theme; an app-only state must not be
burned into an artifact.

## What already landed — use it, invent nothing

`W6-S1`: every colour is a token, a lint rule refuses a literal, and `EmptyState`, `ErrorState`,
`Badge`, `HotkeyChip`, `SegmentedControl` and a three-state `Toggle` are in `@snapdown/ui`.
`W6-S2`: `LC-028 editor-shell` with the navigation rail, and `.nav-rail-item:focus-visible` in
`components.css` — the pattern to follow for any new interactive element.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal
  to escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation is **reported** and becomes a `DEC-`.
- **Verification is run, not assumed.** Every command in `AGENTS.md` § Code, all green.
- **No colour literal anywhere.** `AD-10`, and the lint rule enforces it.
- **Never suppress `focus-visible`.** An inline `outline: 'none'` beats every selector; that was the
  must-fix on `W6-S2`.
- **Write UTF-8.** A story spec came back in cp1252 on `W6-S2` and had to be normalised.
- **No scratch files in the commit**, and never a captured screenshot — CI refuses them.
- **Do not push.**

## Done means

The spec's frontmatter reads `status: done`, every verification command is green, and the work is
committed on this worktree's branch.

Report `worker_done` with `--outcome succeeded` and `--files-modified`, or `--outcome failed` with
the blocking reason.
