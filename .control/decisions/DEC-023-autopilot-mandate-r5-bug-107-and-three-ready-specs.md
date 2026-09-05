---
type: mandate
id: DEC-023
status: accepted
touches: [.control/memlog/autopilot-DEC-023.md]
supersedes: null
superseded_by: null
created: "2026-09-05"
accepted_by: kodesh87, 2026-09-05
---

# DEC-023 — Mandate: close BUG-107 and the three ready-for-agent scratch specs, unattended, checking in only at the smoke test

## Decision

From 2026-09-05 until this mandate's `expires`, `wdi-autopilot` acts with the Product Owner's authority
at every checkpoint of WDI Method and the wrapped BMad skills — accepting and applying a `DEC-`, setting
`mode`/`risk_accepted`, answering a skill's interview, holding a gate — without asking, for the work this
mandate's preflight found runnable: defect `BUG-107` (a cropped Finding's Markers/annotations are not
remapped into the cropped coordinate space), and the three `.scratch/` specs already at
`ready-for-agent` — `canvas-zoom-clipboard-paste`, `editor-virtual-desktop-focus`, `post-testing-polish`.

No `FR` is open (`DEC-018`'s run already closed `FR-1`..`FR-43`). Of the seven defects `defects.yaml`
still shows `status: open`, six — `BUG-7`, `BUG-23`, `BUG-28`, `BUG-37`, `BUG-57`, `BUG-61` — are
deliberately **excluded** from this mandate's scope: each one's own `fix:`/`note:` field already names
the remainder as the owner's call (a git-history rewrite, a design-direction study naming "Graphite", an
ordering choice across a nine-item backlog, or a surface with no Slint caller left to fix). Re-attempting
them would only re-derive what `DEC-019`'s eight iterations already found. Each defect is diagnosed via
`wdi-systematic-debugging` before any fix is proposed, per `AGENTS.md` § Bugs, decisions, questions.

Every act lands as one row in the ledger at `.control/memlog/autopilot-DEC-023.md`. The one checkpoint
kept for the owner is the smoke test at close (`smoke_test: agent`), exercising each closed item's PRD
proof of done or reproduction steps, and nothing before it stops for a question except what `parked`
names.

Its parameters — `from_gate` · `scope` · `parked` · `smoke_test` · `loop` · `expires` — live on this
decision's row in `decisions.yaml` under `mandate:`, per the fixed shape `type: mandate` carries.

## Why

The owner asked for this by running `/loop 10m /wdi-autopilot` in chat on 2026-09-05, then confirmed the
preflight page's proposed scope, parked list, smoke-test route, loop interval, and expiry as-is
(`AskUserQuestion` → "Confirm and start"). The narrowed defect scope (`BUG-107` only, not all seven open
rows) is this preflight's own finding, not the owner's explicit instruction: reading every open defect's
`fix:`/`note:` in full showed six of them already flagged, in their own words, as the owner's decision to
make — carrying them forward as "open, awaiting an autopilot attempt" a second time would have spent this
run's iterations re-discovering `DEC-019`'s own conclusion. `DEC-017` is the standing authority this
mandate draws on, the same as `DEC-018` and `DEC-019`.

## Cost

- **Six open defects stay unattempted by this run.** `BUG-7`, `BUG-23`, `BUG-28`, `BUG-37`, `BUG-57`,
  `BUG-61` remain `status: open` in `defects.yaml`, each still waiting on the owner decision its own row
  names. Not a gap this run introduces — `DEC-019` reached the identical conclusion — but worth stating so
  a reader of this decision alone knows it was deliberate, not an oversight.
- **`DEC-019`'s own "Closed" section calls `BUG-57` and `BUG-61` `status: fixed`, and `defects.yaml`
  disagrees** — both rows still read `status: open`, each with an explicit note saying so
  ("`status` stays `open`", for `BUG-57`; "ordering... belongs to the owner", for `BUG-61`). `DEC-019` is
  `applied` and MUST NOT be edited to correct this. `defects.yaml` is treated as authoritative here — it is
  the more specific, more recently reasoned record — and is what this mandate's scope was drawn from. The
  mismatch itself is reported at Finish, not silently carried forward a second time.
- **A ticket count is not yet known** for `post-testing-polish` (spec `ready-for-agent`, not yet run
  through `to-tickets`) — the first iteration's own `to-tickets` pass sizes it, per the granularity table
  in `delivery-flow-guide.md`, and that answer is a ledger row, not a re-plan.
- **`parked: [ad-n]`** (the method default) is narrower than `DEC-019`'s `[ad-n, sensitive]` — nothing in
  this mandate's scope reads as sensitive on its face (a crop-remap bug fix, a zoom/paste feature, a focus
  fix, an eleven-item UI-polish backlog already scoped and confirmed by the owner on 2026-09-03), so the
  wider category was not carried forward without a reason to.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | — |
| Source material | the owner's `/loop 10m /wdi-autopilot` invocation, 2026-09-05; the preflight page this session printed the same turn, confirmed via `AskUserQuestion`; `DEC-017`; `DEC-018`, `DEC-019` (precedent, both `applied`); `AGENTS.md` § Unattended runs |
