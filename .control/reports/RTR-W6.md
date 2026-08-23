# RTR-W6: The desktop experience (Wave W6 Retrospective)

## Wave Information

- **Wave**: W6
- **Title**: The desktop experience rework
- **Size**: `L` — retrospective required, not advisory
- **Release**: r3
- **PRD**: `capture-to-markdown`
- **FR**: `FR-5`, `FR-18`, `FR-27`, `FR-28`, `FR-29`
- **Status**: Completed
- **Date Closed**: 2026-08-24
- **Delivered Stories**: all eleven — W6-S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11

## How this wave started

It did not start from a plan. The owner ran the shipped product and wrote down what they saw: the app
called itself *Desktop*, the Vault folder had no Browse button, *Run at Windows startup* defaulted off,
the Quality Budget was two bare numbers, hotkeys could not be recorded by pressing keys, text was white
on white, and Settings had to be scrolled. They asked for G1–G4 to be re-run at depth and then for G5.

The re-run found more than the report did. Six defects were registered during it, two critical, and by
the wave's close the register had grown from six to fourteen.

## Scope delivered

1. **W6-S1** — every colour through a token, both themes, enforced by a lint rule that refuses a
   literal anywhere outside `tokens.css`. `EmptyState`, `ErrorState`, `Badge`, `HotkeyChip`,
   `SegmentedControl` and a three-state `Toggle` born in `@snapdown/ui`.
2. **W6-S2** — `LC-028` the editor shell: the `Snapdown Editor` persona, the vertical navigation rail
   replacing the tab row, and the fix for `BUG-4` — `main.tsx` now routes on `?overlay=true`.
3. **W6-S3** — Settings in two columns packed by content height, fitting at 1024×720 without
   scrolling. Three layout tokens that make a group's height countable in advance.
4. **W6-S4** — `DEC-004`: the Quality Budget as a named intent, with `Auto` deriving the pair from the
   captured region. Migration v7.
5. **W6-S5** — `SCN-02`'s four runs and `startup.registered`; plus `BUG-12`, the corrupt `library.db`
   that made the application vanish on launch.
6. **W6-S6** — hotkey rows: chip states, conflicts worded by their remedy, badges carrying a word and
   not only a colour.
7. **W6-S7** — the Findings surface rebuilt, and `BUG-5` and `BUG-6`: `MarkerLayer` and
   `OrphanReportView` finally mounted.
8. **W6-S8** — Bundles: the preview as the centre and as a read-only region, not a disabled input.
9. **W6-S9** — `BUG-1` and `BUG-9`: migration v6 drops the `finding_id` foreign key, and a failed
   unpublish aborts the delete instead of leaving a published copy live on the internet.
10. **W6-S10** — the Vault move reports a source file it could not remove, without failing a move that
    succeeded.
11. **W6-S11** — `BUG-11`: the application became buildable, and CI built it.

## What this wave got right

**It resequenced itself risk-first.** The original order was a linearisation of the dependency graph
that nobody had chosen. `BUG-1` corrupted the record of what was handed over and `BUG-9` could leave a
deleted Bundle live on the public internet; both sat behind six stories of UI work. Moving them
forward was the single best decision in the wave.

**It put `W6-S11` fourth, ahead of more visible value, for a reason that outranked its own severity.**
Until the product could be built, nothing in the wave could be verified in the product. That judgement
held: every story after it was checked against a real binary rather than a dev server.

**It wrote down what it could not prove.** `BUG-4`, `BUG-5` and `BUG-11` all closed as
`resolved-pending-product-verification` rather than `resolved`, and `AUDIT-4`'s addendum states plainly
which of its own claims its evidence does not support.

## What this wave got wrong

**Three story files arrived with a UTF-8 BOM** despite every brief forbidding it, and a BOM makes the
frontmatter parser report the story as having no status at all. It was caught each time only because
the corpus guard ran.

**Three stories needed a continuation dispatch** — W6-S4 and W6-S9 twice — because the worker did the
work and stopped before setting `status: done`. The engine rule that a step is judged from frontmatter
is correct, and the workers kept failing the last line of it.

**A dispatch omitted `--base-branch` and gave W6-S9's planner a checkout at `main`'s tip**, with no
wave context at all. It rebuilt `SPEC.md`, `stories.yaml`, `waves.yaml` and every brief as new files.
Only the story file was taken; the reconstructions were discarded. A self-contained brief is the only
reason that dispatch produced a usable plan.

## The finding that matters more than the wave

**`BUG-14`. Snapdown does not take screenshots.**

Found in the last hour of the wave, while `inventory.py` was flagging a missing *Capture note field*
and the capture path was read to check that claim. `generate_placeholder_image` writes seventeen bytes
— a PNG signature, the width, the height, the quality byte. No IHDR, no IDAT, no IEND. No
screen-capture dependency exists in the workspace, and no image codec either. The reduction pipeline
says so in its own comment: *"downscaled payload simulation"*.

**Every test in this repository asserts dimensions, never pixels.** The arithmetic is correct and is
applied to nothing. That is why six waves, twelve use cases and four UI audits went past it.

This wave fixed the experience around a core that is not there, and could not have noticed, because
nothing in its scope decoded an image either. Fixing it is a wave of its own.

## The pattern this project keeps repeating

Three times now, in three different shapes:

| Wave | Shape |
|---|---|
| Waves 1–5 | Four components built, unit-tested, and mounted nowhere. `BUG-4`, `BUG-5`, `BUG-6` |
| W6-S1 | A contrast test asserting a hardcoded copy of the token values, passing whatever `tokens.css` said |
| Waves 1–6 | An image pipeline asserting dimensions, with no image behind them. `BUG-14` |

**The unit is correct and nothing checks that the unit is connected to reality.** The composition-test
convention (`OQ-23`) answers the first shape. Mutation-testing a new assertion answers the second, and
was used on W6-S1's contrast test and W6-S3's layout test. **Nothing yet answers the third**, and
`BUG-14` is what the third costs.

## Carried forward

- **`BUG-14`** needs a wave of its own. The owner's call.
- `BUG-2`, `BUG-3`, `BUG-7`, `BUG-8`, `BUG-10` remain open and unrelated to this wave.
- `BUG-4`, `BUG-5`, `BUG-11` are `resolved-pending-product-verification`. The verification is now
  possible for the first time; `BUG-14` explains why the attempt failed.
- `OQ-22`, `OQ-23`, `OQ-24` remain the owner's.
- `OQ-24` is four for four: every dispatched UI verification has reported source instead of screen.
  The next attempt must be denied application code entirely.
- GitHub CI was disabled part-way through at the owner's request to remove wait time from a long
  autonomous run. Local verification covered the same set, including a byte-for-byte reproduction of
  the `korpus.yml` baseline comparison. **It must be re-enabled.**

## Register at close

| Status | Count |
|---|---|
| `resolved` | 4 |
| `resolved-pending-product-verification` | 3 |
| `open` | 7 |

Fourteen defects registered in total, eight of them found during this wave.
