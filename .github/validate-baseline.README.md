# Why each line is in `validate-baseline.txt`

The baseline is not a mute button. Every line has to be defensible, and a line nobody can defend is a
finding to fix rather than a row to add. Updated 2026-08-23.

## `V3` ×6 — six shipped use cases scheduled to no story

`UC-1`, `UC-2`, `UC-7`, `UC-8`, `UC-9`, `UC-12`.

**Permanent, and correct.** W2–W5 were closed without their story lists ever reaching `waves.yaml`.
Fourteen story files survive on disk and **not one contains a `UC-` reference** — the traceability was
never written, not lost. Ids, titles, components and status are recoverable; `satisfies:` is not.

`OQ-21` weighed reconstructing it and answered no: a registry assembled from plausible guesses reads
complete and is fiction. The recovery note at the head of `waves.yaml` records what the evidence
supports and what it does not.

**A green validator on these six would mean somebody guessed.** These lines stay.

## `V18` ×9 — nine W6 stories with no story file

**Temporary, and expected.** A story file is written by `wdi-build` Phase 3 Step 1, one story at a
time. `W6-S1`'s exists; the other nine do not yet. Each line leaves this file as its story is planned,
and the last one leaves when the wave closes.

**If a line is still here when W6 closes, that is a real finding** — it means a story was shipped
without a plan.

## `V24` ×2 — two citations inside a BMad package file

`.agent/skills/bmad-project-context/references/template.md` cites `src/lib/money.ts` and
`src/routes/webhooks.ts`. Both are **placeholders in an installed package's own example template**,
not claims about this product. The file is overwritten on every BMad update, so fixing it here would
be undone and would also be editing someone else's package.

## Removed 2026-08-23

Two `V25` lines — `mcp-bridge` and `web-api` missing headings in the code map. **Fixed, not excused.**
`.control/structure-codebase.md` now has a section for each; both are `built: true` containers whose
code we write, and the map owed them a heading.
