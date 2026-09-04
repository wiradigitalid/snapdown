---
topic: wdi-autopilot run log — mandate DEC-019
artifact: .control/decisions/DEC-019-autopilot-mandate-r4-close-every-open-defect.md
updated: 2026-09-04T00:00
---

## Resume

Mandate: `DEC-019`. Parameters at `.control/registry/decisions.yaml` → `DEC-019.mandate`
(`from_gate: G5`, `scope: {fr: all, defects: [BUG-2, BUG-7, BUG-23, BUG-28, BUG-37, BUG-57, BUG-60,
BUG-61, BUG-77, BUG-106]}`, `parked: [ad-n, sensitive]`, `smoke_test: agent`, `loop: 5m`,
`expires: 2026-09-11`).

- Iteration: 1, commit `1d00b53`
- Run branch: `autopilot/DEC-019`. Isolated worktree at
  `D:\Developer\wiradigital.id\snapdown-autopilot-dec019` is the coordinator's own — every Door 2
  iteration works there, not in the `main` checkout. Not yet pushed, no PR opened.
- Stopped at: **Capacity** — two builders dispatched this iteration, both still running. Nothing more
  started until at least one reports, to avoid a third worktree racing a merge these two will need.
- Blocked: —
- Parked: —
- In flight:
  - `fix/bug-60-startup-failure-visibility`, worktree `D:\Developer\wiradigital.id\snapdown-fix-bug-60`,
    agent `a91901bc93997d2bc` — `BUG-60` (startup failure visibility: native dialog, refuse/label the
    in-memory fallback, replace the three `unwrap`s). Code + defects.yaml only.
  - `docs/bug-2-withdraw-sharing-reader`, worktree `D:\Developer\wiradigital.id\snapdown-bug-2-decision`,
    agent `a10eb218007e6b57a` — `BUG-2` (withdraw the frozen `sharing` reader-SPA promise via a new
    `DEC-`, accepted under `DEC-019`, applied across the PRD/EXPERIENCE/inventory/`components.yaml`).
    Docs only, no code.
- Next: read both agents' reports when they land. Independently verify each (re-run
  `cargo test --workspace --no-fail-fast` for `BUG-60`'s branch; re-run `validate.py --generate` for
  `BUG-2`'s branch and diff against the known baseline) before merging either into `autopilot/DEC-019`.
  Merge whichever reports first; the two touch disjoint files (code vs. docs) so order shouldn't
  matter, but confirm no surprise overlap before merging the second. Then continue Door 2's work table:
  no open `FR` exists, so the next runnable defects are `BUG-57` (needs a diagnosis pass — 2 of its 3
  stubs may already be resolved by the merged bundle-library work, unverified), `BUG-77` (small, has a
  named preferred fix), and `BUG-106` (Crop tool — a real feature build, isolated to the crop
  interaction + a new store op). `BUG-23`, `BUG-28`, `BUG-37` are not independently actionable this run
  (BUG-23: blocked behind DEC-005, no Slint publish entry point to fix; BUG-28: remaining latency work
  is explicitly the owner's call per its own `fix:`; BUG-37: needs a `wdi-ux` look-and-feel study naming
  Graphite, not a code fix) — leave open with their existing notes, do not close them, do not attempt
  the design/ordering calls they defer. `BUG-7` is worked only for its agent-doable half (already
  `DONE`); do not attempt the history-scrub half under any circumstance.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight, commit 05e7b80 | wdi-autopilot Door 1 | Mandate accepted with `from_gate: G5`, `scope: {fr: all, defects: [10 open BUG ids]}`, `parked: [ad-n, sensitive]`, `smoke_test: agent`, `loop: 5m`, `expires: 2026-09-11` — scope widened beyond the default (no open FR exists; the owner asked for defects too, since `DEC-018`'s run already closed every FR) | Declining to start since `scope: all FR` alone is vacuous | — | `DEC-019`, `decisions.yaml` |
| Preflight | wdi-autopilot Door 1 | Confirmed via `git worktree list` that only the main checkout exists — no isolated worktree yet. Left worktree creation to iteration 1 rather than doing it inside preflight | Creating the worktree during preflight, before the mandate was actually accepted | If iteration 1 fails to create it, the run reports Capacity rather than silently working in the shared checkout | — (no edit; logged as what iteration 1 does first) |
| Preflight | wdi-autopilot Door 1 | Confirmed `cargo test --workspace` and `go test ./...` (in `apps/web-service`) both green before accepting the mandate, and confirmed `validate.py --check`'s 6 RED findings all match the existing `.github/validate-baseline.txt` fossils (`V6`×5, `V16`×1) named in `AGENTS.md` — no new red | Trusting a prior run's state without re-checking | If either had been red, preflight would have stopped rather than accepted the mandate | — |
| Iter 1, commit 1d00b53 | wdi-autopilot Door 2, routing table "Every FR in scope closed" reached instantly since no FR is open — moved to the defect half of scope | Read all 10 open defect rows in full before dispatching anything. Found most are not plain code bugs: `BUG-2` needs a promise decision (forced compliant answer by `DEC-005`'s freeze), `BUG-23`/`BUG-28`/`BUG-37` explicitly defer to the owner or are blocked, `BUG-61` is a backlog whose ordering is the owner's call (but its inventory note was stale against the merged bundle-library work), `BUG-57`/`BUG-77`/`BUG-106` are more straightforwardly actionable. Triaged before dispatching anything, rather than working the list in id order blind | Diagnosing `BUG-2` (lowest id) first via `wdi-systematic-debugging` as originally planned in preflight, which would have wasted a diagnosis pass on a defect that was never a code bug | If this triage is wrong for any row, the row stays open and the next iteration re-reads it — nothing was closed on the strength of the triage alone | — (no edit; logged as this iteration's plan) |
| Iter 1, commit 1d00b53 | wdi-autopilot Door 2 | Re-verified `BUG-61` against the current tree before doing anything else with it: confirmed by grepping call sites that `list_bundles`, `get_bundle`, `delete_bundle`, and `update_bundle_markdown` (renamed `update_bundle_name_and_markdown`) are now all called from `main.rs`, that `SqliteAccessKeyStore` no longer exists (`DEC-016`), and that `reorder_markers`/`update_finding_image`/the orphan-report surface genuinely still have no caller. Corrected the row's note rather than closing it — the row's own `fix:` leaves ordering the remainder to the owner | Trusting the 2026-09-01 note, which predates the bundle-library merge (PR #38, 2026-09-02) and DEC-016's removal (PR #40) | If the grep-based verification missed a caller, the next iteration's own read of the row would catch it before treating any of these as still-open work | `.control/registry/defects.yaml` (BUG-61 note) |
| Iter 1 | wdi-autopilot Door 2 | Dispatched `BUG-60` (code fix, own worktree/branch) and `BUG-2` (docs-only decision + apply, own worktree/branch) in parallel — disjoint file sets (Rust code vs. `.what`/`.how`/`.control` documents), so no shared-write risk | Dispatching more than two at once, or dispatching `BUG-106`/`BUG-77` (which both touch `appwindow.slint`) alongside `BUG-60` without first checking for overlap in `apps/desktop/src/main.rs` | If the two dispatches turn out not as disjoint as expected, the merge step (next iteration) is where that surfaces, before either lands on `autopilot/DEC-019` | agents `a91901bc93997d2bc` (BUG-60), `a10eb218007e6b57a` (BUG-2) — no files changed yet, pending their reports |
