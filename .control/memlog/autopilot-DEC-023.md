---
topic: wdi-autopilot run log — mandate DEC-023
artifact: .control/decisions/DEC-023-autopilot-mandate-r5-bug-107-and-three-ready-specs.md
updated: 2026-09-05T00:00
---

## Resume

Mandate: `DEC-023`. Parameters at `.control/registry/decisions.yaml` → `DEC-023.mandate`
(`from_gate: G5`, `scope: {fr: all, defects: [BUG-107]}`, `parked: [ad-n]`, `smoke_test: agent`,
`loop: 10m`, `expires: 2026-09-12`).

- Iteration: 1, commit `88234e6` (mandate open/accept) is the boundary so far
- Run branch: `autopilot/DEC-023`, isolated worktree at `.claude/worktrees/autopilot+DEC-023`. No PR
  opened yet — the first push (at the first spec/defect close) opens it as a draft.
- Stopped at: **Capacity** — one builder subagent dispatched and in flight in this same worktree; a
  second concurrent builder needs its own worktree and was deliberately not started this iteration to
  avoid two agents committing in the same working tree at once.
- In flight: `BUG-107` — builder subagent dispatched (diagnose root cause → fix in
  `snapdown-core`/`snapdown-store` → regression test that decodes actual post-crop coordinates, not a
  count/existence check → `cargo fmt`/`clippy`/`test --workspace` green → commit directly to
  `autopilot/DEC-023`, no push, no PR). Independent review still owed before the commit is trusted —
  the builder is not its own reviewer.
- Blocked: —
- Parked: —
- Next: when the `BUG-107` builder returns — independently review its diff (read it in full, don't
  trust its self-report), re-run the full suite, then either accept the commit as-is or send it back for
  a fix. After `BUG-107` lands: `.scratch/canvas-zoom-clipboard-paste/` (spec + 3 tickets, all
  `ready-for-agent`), `.scratch/editor-virtual-desktop-focus/` (1 ticket `ready-for-agent`),
  `.scratch/post-testing-polish/` (spec `ready-for-agent`, not yet run through `to-tickets`) — each in
  its own worktree if run concurrently.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
| --- | --- | --- | --- | --- | --- |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — scope | Scope narrowed to `defects: [BUG-107]`, excluding `BUG-7/23/28/37/57/61` | Carrying all seven open defects forward as `DEC-019` did | Re-running six iterations that only re-derive an owner-only conclusion already on record | `DEC-023` row in `decisions.yaml`, `DEC-023-....md` |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — validators | Treated the 6 baseline-matched reds as green | Blocking preflight on them | None — `DEC-021` already accepts these as permanent | preflight page (this ledger) |
