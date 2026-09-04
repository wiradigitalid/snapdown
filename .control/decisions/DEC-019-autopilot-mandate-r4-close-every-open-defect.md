---
type: mandate
id: DEC-019
status: accepted
touches: []
supersedes: null
superseded_by: null
created: "2026-09-04"
accepted_by: kodesh87, 2026-09-04
---

# DEC-019 — Mandate: carry every open FR and every open defect to closed, unattended, checking in only at the smoke test

## Decision

From 2026-09-04 until this mandate's `expires`, `wdi-autopilot` acts with the Product Owner's authority
at every checkpoint of WDI Method and the wrapped BMad skills — accepting and applying a `DEC-`, setting
`mode`/`risk_accepted`, answering a skill's interview, holding a gate — without asking, for every `FR`
still open (none, as of this mandate's preflight — `DEC-018`'s run already closed `FR-1`..`FR-43`) and
for every open row in `defects.yaml`: `BUG-2, BUG-7, BUG-23, BUG-28, BUG-37, BUG-57, BUG-60, BUG-61,
BUG-77, BUG-106`. Each defect is diagnosed via `wdi-systematic-debugging` before any fix is proposed, per
`AGENTS.md` § Bugs, decisions, questions. `BUG-7` is the one named exception: its own `fix:` field already
marks the agent-doable half done (the tracked screenshots were removed from the working tree); the
remaining half is a `git filter-branch` history scrub, which this mandate MUST NOT attempt — it is
owner-only regardless of `parked`, per the standing rule against agent-initiated destructive git history
rewrites.

Every such act lands as one row in the ledger at `.control/memlog/autopilot-DEC-019.md`. The one
checkpoint kept for the owner is the smoke test at close — it is run by the agent this time
(`smoke_test: agent`), exercising each closed `FR`'s PRD proof of done and each closed defect's own
reproduction, and nothing before it stops for a question except what `parked` names.

Its parameters — `from_gate` · `scope` · `parked` · `smoke_test` · `loop` · `expires` — live on this
decision's row in `decisions.yaml` under `mandate:`, per the fixed shape `type: mandate` carries.

## Why

The owner confirmed this in chat on 2026-09-04, after reading the preflight page, in three answers: scope
is *"seluruh FR, seluruh defect"* — every FR (none open) and every open defect; `parked` adds `sensitive`
to the default `ad-n`, on top of the standing rule that already keeps `BUG-7`'s history scrub owner-only;
and the session's permission mode is switched to bypass/accept-edits by the owner before the loop starts.
`smoke_test`, `loop`, `expires`, `from_gate`, the ledger path, and the run branch stood as the preflight
page proposed them. `DEC-017` is the standing authority this mandate draws on, the same as `DEC-018`.

## Cost

- **`scope` includes defects, not just FRs** — a shape `wdi-autopilot`'s own routing table does not name a
  row for. Each defect is worked as its own `wdi-systematic-debugging` → fix → review → merge cycle,
  reusing the same run-branch/one-PR discipline Door 2 already applies to spec work.
- **`BUG-7`'s open status will not be closed by this run.** Its agent-doable half is already done; the
  registry row stays `status: open` (or moves to a state naming the remaining owner-only half) until the
  owner performs the history scrub separately. Reported at Finish, not silently dropped.
- **`parked: [ad-n, sensitive]`** stops the run for the owner on any `AD-N` contradiction *and* on anything
  judged sensitive — wider than `DEC-018`'s empty `parked`, so more of this run returns to the owner
  mid-flight than the previous one did.
- **`smoke_test: agent`** means the agent itself exercises proof-of-done and defect repro steps at close,
  which `DEC-018` explicitly left to the owner; a defect this smoke test cannot actually exercise (e.g. one
  needing a real corrupt file on disk, per `BUG-60`) is named as a gap in the final report rather than
  claimed as tested.
- **Some of these defects may reveal a corpus/code mismatch, not just a code bug** (`BUG-61`'s title
  names promised surfaces with no implementation) — if a defect turns out to be an `FR` the registry marked
  closed while the feature does not exist, this run treats that as "code wrong, document was right" and
  reports it rather than silently reopening the FR without the owner's DEC-018-style visibility.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | — |
| Source material | the owner's chat instructions, 2026-09-04, quoted above; the preflight page this session printed the same turn; `DEC-017`; `DEC-018` (precedent, `applied`); `AGENTS.md` § Unattended runs |
