---
type: mandate
id: DEC-023
status: applied
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

## Closed 2026-09-06

Every runnable row in scope is closed. `BUG-107` fixed and independently re-verified by the
coordinator (mutation-tested, decode-based tests, `430+` passing, `cargo fmt`/`clippy -D warnings`/
`test --workspace` all re-run from scratch by the coordinator, not taken on the builder's report).
`canvas-zoom-clipboard-paste` and `editor-virtual-desktop-focus`'s own tickets closed. `post-testing-polish`
was sized into six tickets by this mandate's own `to-tickets` pass (the ticket count `Cost` above flagged
as unknown); all six built, independently reviewed, and merged: Ctrl+Scroll zoom, marker note auto-focus
+ hover tooltip, a second Assemble entry point + a filmstrip alignment fix, copy-on-save, bulk reclaim
space (which also registered a new `FR-44` rather than widening `FR-42`, to respect `entity-one-writer`),
and the About tab's icon.

Landed on `autopilot/DEC-023` throughout — a single run branch, fourteen merge/feature commits from six
parallel-built ticket branches plus one direct commit for `BUG-107`, every ticket branch deleted after
its merge, one PR (`#47`). Pushed six times; Korpus Validation and Desktop CI both green on every push
checked.

Smoke test (`smoke_test: agent`) run against the real release build via `computer-use`, driven against
the Reviewer's live Vault with no destructive action taken: 4 of 9 candidates directly exercised and
confirmed live (zoom buttons, marker focus + tooltip, both Assemble buttons' selection gate, bulk
reclaim space's confirmation dialog including its correct shared-Finding dedup count); the rest stand on
their own independently-verified, several mutation-tested, automated coverage — Ctrl+Scroll could not be
simulated through the available tooling (no modifier-key support in the computer-use scroll command),
and `BUG-107`/copy-on-save/`editor-virtual-desktop-focus`'s actual desktop switch were left untested
live to avoid mutating real Vault data or because no interactive multi-desktop session was available.

`parked: [ad-n]` was never triggered — nothing in scope contradicted an `AD-N`. `wdi-report progress`
at close reads `promise progress: 0%` (0 of 33 counted RTM rows green) — a pre-existing corpus
measurement gap, not a finding about this run: RTM rows are broken at the ticket-linkage step because
the wave/ticket layer is retired and `.scratch/` tickets are ephemeral, cleared once their effort
closes. This mandate's actual delivery is verified through git history, independent code review, and
passing (often mutation-tested) automated tests instead, all named above.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | — |
| Source material | the owner's `/loop 10m /wdi-autopilot` invocation, 2026-09-05; the preflight page this session printed the same turn, confirmed via `AskUserQuestion`; `DEC-017`; `DEC-018`, `DEC-019` (precedent, both `applied`); `AGENTS.md` § Unattended runs |
