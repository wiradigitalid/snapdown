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

- Iteration: 0 (preflight only — no work started yet)
- Run branch: `autopilot/DEC-019`, commit `8d3b85b` (preflight docs only). Isolated worktree at
  `D:\Developer\wiradigital.id\snapdown-autopilot-dec019` — every Door 2 iteration works there, not in
  the coordinator's own `main` checkout. Not yet pushed, no PR opened.
- Stopped at: Preflight complete, mandate accepted, worktree created. Loop starting.
- Blocked: —
- Parked: —
- Next: create the isolated worktree off `main` at the run branch `autopilot/DEC-019`, run
  `validate.py --generate`, then start Door 2's work table at "G4 clear for a candidate row" —
  no open `FR` exists, so the first runnable row is `BUG-2` (lowest id among the ten open defects in
  scope), via `wdi-systematic-debugging` before any fix is proposed. `BUG-7` is worked only for its
  agent-doable half (already `DONE` per its own `fix:` field) — do not attempt the history-scrub half.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight, commit 05e7b80 | wdi-autopilot Door 1 | Mandate accepted with `from_gate: G5`, `scope: {fr: all, defects: [10 open BUG ids]}`, `parked: [ad-n, sensitive]`, `smoke_test: agent`, `loop: 5m`, `expires: 2026-09-11` — scope widened beyond the default (no open FR exists; the owner asked for defects too, since `DEC-018`'s run already closed every FR) | Declining to start since `scope: all FR` alone is vacuous | — | `DEC-019`, `decisions.yaml` |
| Preflight | wdi-autopilot Door 1 | Confirmed via `git worktree list` that only the main checkout exists — no isolated worktree yet. Left worktree creation to iteration 1 rather than doing it inside preflight | Creating the worktree during preflight, before the mandate was actually accepted | If iteration 1 fails to create it, the run reports Capacity rather than silently working in the shared checkout | — (no edit; logged as what iteration 1 does first) |
| Preflight | wdi-autopilot Door 1 | Confirmed `cargo test --workspace` and `go test ./...` (in `apps/web-service`) both green before accepting the mandate, and confirmed `validate.py --check`'s 6 RED findings all match the existing `.github/validate-baseline.txt` fossils (`V6`×5, `V16`×1) named in `AGENTS.md` — no new red | Trusting a prior run's state without re-checking | If either had been red, preflight would have stopped rather than accepted the mandate | — |
