# RTR-W8: Capture becomes real (Wave W8 Retrospective)

## Wave Information

- **Wave**: W8
- **Title**: Capture becomes real
- **Size**: `L` — retrospective required, not advisory
- **Release**: r3
- **PRD**: `capture-to-markdown`
- **FR**: `FR-1`, `FR-2`, `FR-4`, `FR-8`
- **Status**: Completed
- **Date Closed**: registry row read `status: open` from 2026-08-24 until 2026-09-04, when
  `wdi-autopilot` corrected it against git history — all six stories had merge commits weeks earlier
  and nobody had updated the row
- **Delivered Stories**: all six — W8-S1, S2, S3, S4, S5, S6

## How this wave started

`BUG-14`, found in the last hour of `W6`: `generate_placeholder_image` wrote a seventeen-byte fake PNG
— signature, width, height, quality byte, no `IHDR`/`IDAT`/`IEND` — and no screen-capture crate or
image codec existed anywhere in the workspace. Every test in the repository asserted dimensions, never
pixels, so the arithmetic was correct and applied to nothing for six waves. `RTR-W6` named it the
finding that mattered more than that wave, and predicted a wave of its own would be needed to fix it.
This is that wave.

## Scope delivered

1. **W8-S1** — the `snapdown-capture` crate, and a real grab of the requested region. First
   third-party dependency this workspace ever added: a screen-capture crate plus an image codec,
   chosen and recorded in the story's own commit.
2. **W8-S2** — a real decode, scale and encode, replacing `reduce_image`'s "downscaled payload
   simulation" comment with an actual pipeline. `compute_reduced_dimensions_for_pair` and the `Auto`
   resolution from `W6-S4` were kept exactly as written — the arithmetic was never wrong, only what it
   was applied to.
3. **W8-S3** — Markers drawn onto the image, not described beside it. `burn_markers` stopped writing a
   fake header with ordinals and coordinates appended as raw bytes, and started actually drawing.
4. **W8-S4** — the Note at capture time, the second half of `UC-1` (`BUG-18`): the overlay had never
   asked for one, and `FR-2` was entirely unmet until this story.
5. **W8-S5** — the expensive half of the wave: every test that asserted a PNG signature and dimensions
   was converted to decode the actual bytes. `test_golden_markdown.rs`'s golden file was regenerated
   from real output rather than continuing to prove byte-identity of a fabrication.
6. **W8-S6** — `BUG-19`: the burn `W8-S3` built was called from nowhere in the running application.
   Wired the call, and closed `BUG-20` (a corrupt source silently accepted) and `BUG-21` (the composed
   Markdown referencing the Finding's clean image instead of the Bundle's burned copy) in the same
   story, both found while tracing the call path rather than trusting the register.

## What this wave got right

**It converted the test surface before trusting any of the six stories' own claims.** `W8-S5` is
placed after the three implementations precisely so that "images are real now" would be proven by a
suite that could actually fail, not asserted by the same kind of dimension-only test that hid `BUG-14`
for six waves in the first place.

**Every defect found while building was found by tracing the real call path, not by reading the
register.** `BUG-19`, `BUG-20` and `BUG-21` were all caught this way, continuing the pattern `RTR-W6`
named — that the register is a claim about code at a moment and goes stale silently unless someone
reads the code instead.

## What this wave got wrong

**The wave's own closing bookkeeping never happened.** All six stories merged — `W8-S1` through
`W8-S6`, each with its own merge commit — and the wave row in `waves.yaml` sat at `status: open` for
roughly eleven days afterward. Nobody caught it because nothing re-read the registry against git
history until `wdi-autopilot`'s mandate did, on 2026-09-04, while checking whether a candidate backlog
row was genuinely unbuilt. The same shape as `BUG-12`'s lesson in `AGENTS.md`: a registry row is a
claim about code at a moment, and it goes stale exactly this quietly.

**This retrospective itself is twelve days late**, for the same reason — `V19` requires it for a
closed size-`L` wave and nothing enforced that requirement while the row still read `open`.

## The finding that matters more than the wave

**The dimension-only test pattern this wave exists to fix had already cost six waves once.** `W8-S5`'s
own commentary is explicit that the danger was never that a fake-image test would fail — it's that it
would *keep passing* while proving nothing, exactly as it did from `W2` through `W6`. The positive
obligation this wave leaves behind, `every_image_producing_path_decodes_its_own_output`, is the
control that failing forward on a smaller version of the same mistake could not be verified: a
mechanical prohibition against magic-byte-only assertions would itself be a magic-byte-only test,
asserting a copy of its own input. The obligation is a real test, not a lint, for exactly that reason.

## Carried forward

- `NFR-1` (overlay visible within 200 ms with three monitors attached) and `NFR-2` (saving returns
  focus within 500 ms) became measurable for the first time once capture was real, and neither was
  scheduled here — both need an instrument this project's UI verification has failed to provide four
  times running (`OQ-24`).
- The choice of screen-capture crate and image codec, made and recorded in `W8-S1`'s own commit, is
  the first third-party dependency this workspace has ever taken on and is worth a `DEC-` if it hasn't
  already had one written.
- Registry staleness (this wave's own closing bookkeeping) is now a pattern worth a standing check,
  not just a lesson recorded after the fact each time it's found.

## Register at close

Not reconstructed here. `BUG-14`, `BUG-18`, `BUG-19`, `BUG-20` and `BUG-21` were all opened and closed
within this wave's own stories, per each story's own commit history; a full register snapshot was not
taken at the time because the wave's `status` was never flipped to prompt one. Reading `defects.yaml`
directly for their current status is more reliable than a count reconstructed twelve days after the
fact.
