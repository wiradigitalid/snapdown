---
topic: wdi-autopilot run log — mandate DEC-023
artifact: .control/decisions/DEC-023-autopilot-mandate-r5-bug-107-and-three-ready-specs.md
updated: 2026-09-05T00:00
---

## Resume

Mandate: `DEC-023`. Parameters at `.control/registry/decisions.yaml` → `DEC-023.mandate`
(`from_gate: G5`, `scope: {fr: all, defects: [BUG-107]}`, `parked: [ad-n]`, `smoke_test: agent`,
`loop: 10m`, `expires: 2026-09-12`).

- Iteration: 0 (preflight only — mandate just accepted, no iteration has run yet)
- Run branch: `autopilot/DEC-023`, created in an isolated worktree at
  `.claude/worktrees/autopilot+DEC-023`. No PR opened yet — the first push (at the first spec/defect
  close) opens it as a draft.
- Stopped at: n/a (preflight just finished)
- Blocked: —
- Parked: —
- Next: iteration 1 — pick the first runnable row. Candidates found at preflight, no ranking done yet:
  `BUG-107` (crop doesn't remap Markers/annotations), `.scratch/canvas-zoom-clipboard-paste/` (spec + 3
  tickets, all `ready-for-agent`), `.scratch/editor-virtual-desktop-focus/` (1 ticket `ready-for-agent`),
  `.scratch/post-testing-polish/` (spec `ready-for-agent`, not yet run through `to-tickets`)

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
| --- | --- | --- | --- | --- | --- |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — scope | Scope narrowed to `defects: [BUG-107]`, excluding `BUG-7/23/28/37/57/61` | Carrying all seven open defects forward as `DEC-019` did | Re-running six iterations that only re-derive an owner-only conclusion already on record | `DEC-023` row in `decisions.yaml`, `DEC-023-....md` |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — validators | Treated the 6 baseline-matched reds as green | Blocking preflight on them | None — `DEC-021` already accepts these as permanent | preflight page (this ledger) |
