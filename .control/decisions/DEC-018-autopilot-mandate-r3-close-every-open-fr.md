---
type: mandate
id: DEC-018
status: applied
touches:
  - .control/memlog/autopilot-2026-09-04.md
  - .scratch/bundle-library/spec.md
  - .scratch/bundle-library/issues/10-the-composer-reads-its-own-document.md
  - .scratch/bundle-library/issues/11-the-library-opens-and-lists-bundles.md
  - .scratch/bundle-library/issues/12-hand-a-bundle-over-from-its-row.md
  - .scratch/bundle-library/issues/13-open-a-bundle-locked-in-review-and-update.md
  - .scratch/bundle-library/issues/14-edit-and-save-a-bundle.md
  - .scratch/bundle-library/issues/15-an-edited-bundle-says-so.md
  - .scratch/bundle-library/issues/16-disassemble-a-bundle-or-delete-a-sealed-one.md
  - .scratch/bundle-library/issues/17-discard-originals-and-delete-both.md
  - .scratch/bundle-library/issues/18-reclaim-space.md
  - .control/registry/requirements.yaml
  - .what/bundle/SRS-bundle.md
  - .how/bundle/SDD-bundle.md
  - .how/_platform/inventory-screen.md
  - .control/registry/components.yaml
  - .what/finding/SRS-finding.md
  - .control/registry/waves.yaml
  - .control/reports/RTR-W8.md
  - .gitattributes
  - .claude/skills/to-spec/SKILL.md
  - .claude/skills/to-tickets/SKILL.md
  - .claude/skills/implement-spec/SKILL.md
supersedes: null
superseded_by: null
created: "2026-09-04"
accepted_by: kodesh87, 2026-09-04
---

# DEC-018 — Mandate: carry every open FR to closed, unattended, checking in only at the smoke test

## Decision

From 2026-09-04 until this mandate's `expires`, `wdi-autopilot` acts with the Product Owner's authority
at every checkpoint of WDI Method and the wrapped BMad skills — accepting and applying a `DEC-`, setting
`mode`/`risk_accepted`, answering a skill's interview, holding a gate — without asking, for every FR still
open across this product, starting with the `bundle-library` spec's frontier (tickets 10–18) and then the
FR-30..43 backlog that has never been scheduled to a wave. Every such act lands as one row in the ledger at
`.control/memlog/autopilot-2026-09-04.md`. The one checkpoint kept for the owner is the smoke test at
close — it is run by the owner, not the agent, and nothing before it stops for a question.

Its parameters — `from_gate` · `scope` · `parked` · `smoke_test` · `loop` · `expires` — live on this
decision's row in `decisions.yaml` under `mandate:`, per the fixed shape `type: mandate` carries.

## Why

The owner confirmed this in chat on 2026-09-04, after reading the preflight page: *"seluruh FR dikerjakan,
telusuri agar selesai semua, dokumen kerjakan otomatis, smoke_test manusia (di akhir saja), loop 10m,
selebihnya saya ikut."* — every FR gets worked, chase it until all of it is done, documents are kept
current automatically, the smoke test is run by a human and only at the end, the loop runs every 10
minutes, and the remaining settings shown in the preflight page (`from_gate: G5`, `scope: all`,
`parked: []`, `expires` 7 days out) stand as proposed. This mandate is the record `wdi-autopilot`'s own
process requires before the loop may start, and `DEC-017` is the standing authority it draws on.

## Cost

- **Ratification comes after the fact**, same as `DEC-017` already accepts for this whole product: a
  `DEC-` this run opens and applies on the owner's behalf is frozen before it is read.
- **`smoke_test: owner`** means no proof-of-done is exercised by the agent at all during the run — every
  FR's PRD proof of done becomes a test script the owner is handed at close, not a result already checked.
  An FR reported closed here is closed on the strength of its ticket tests and code, not on anyone having
  used the feature.
- **`scope: all` reaches FR-30 through FR-43**, none of which has ever been scheduled to a wave, plus
  whatever `wdi-reconcile` finds already drifted (e.g. `specs.yaml`'s `W8` reading `status: open` while
  `crates/snapdown-capture` already exists in the tree — first thing the loop checks, not assumed here).
  The full size of that backlog is not yet estimated; `wdi-report` intent `estimate` is where that number
  is produced, in the first iteration.
- **`parked: []`** means nothing stops for the owner mid-run, including a promise-level ambiguity or a
  contradiction with an `AD-N` — those get decided and logged, not queued.
- **This mandate does not itself say anything ran unattended before it was accepted.** Everything before
  this point in the session — the preflight page, the questions it answered — was run with the owner
  present and answering directly; the unattended posture begins now, at `accepted`.

## Closed 2026-09-04

Every `FR` this product's two requirements registries name (`FR-1`..`FR-43`, `FR-19`..`FR-26`) now traces
to a closed wave, a closed spec, or an explicit withdrawal — the ledger at
`.control/memlog/autopilot-2026-09-04.md` (20 iterations) is the full record. `parked` stayed empty the
whole run: nothing was deferred to the owner mid-flight. `smoke_test: owner` means nothing below was
exercised by the agent as "working" — every closed `FR`'s proof of done is the test script in the final
report, for the owner to run.

**Landed via a consolidated branch, not `autopilot/r3` itself.** The mandate ran on `autopilot/r3`, but
that branch diverged from `main` before an earlier PR (bundle-library, #38) merged, and each dispatched
builder pushed its own branch off `main` rather than off `autopilot/r3` — by iteration 20 that was four
open PRs (#39, #40, #41, plus `autopilot/r3` itself with no PR). The owner asked for one PR only. This
file, the ledger, and every other file `touches:` names were therefore copied onto a fifth branch,
`release/r3-mandate-dec018`, built by merging the three worker branches together, reapplying the
bookkeeping fixes this coordinator made directly on `autopilot/r3` (they didn't survive the merges,
since the worker branches shared no history with `autopilot/r3`'s post-#38 state), and adding what the
merge itself required (`RTR-W8`, `.gitattributes`). That branch is PR #42 — the one PR. `autopilot/r3`
stays as the run's original working branch and is not itself merged anywhere.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | — |
| Source material | the owner's chat instruction, 2026-09-04, quoted above; the preflight page this session printed the same turn; `DEC-017`; `AGENTS.md` § Unattended runs |
