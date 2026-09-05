---
topic: wdi-autopilot run log — mandate DEC-023
artifact: .control/decisions/DEC-023-autopilot-mandate-r5-bug-107-and-three-ready-specs.md
updated: 2026-09-05T00:00
---

## Resume

Mandate: `DEC-023`. Parameters at `.control/registry/decisions.yaml` → `DEC-023.mandate`
(`from_gate: G5`, `scope: {fr: all, defects: [BUG-107]}`, `parked: [ad-n]`, `smoke_test: agent`,
`loop: 10m`, `expires: 2026-09-12`).

- Iteration: 5, commit `b165239` (post-testing-polish sized into 6 tickets) is the boundary so far
- Run branch: `autopilot/DEC-023`, isolated worktree at `.claude/worktrees/autopilot+DEC-023`. No PR
  opened yet — the first push (at the first defect/spec close with real code on it) opens it as a draft.
- Stopped at: **Capacity** — 5 builders now in flight across 5 worktrees; no more dispatched this
  iteration, reviewing/merging is next.
- Done: `canvas-zoom-clipboard-paste` (`FR-34`, `FR-35`) — already shipped to `origin/main` before this
  mandate opened, ticket bookkeeping corrected, merged (`1bee143`). See iteration 4 decisions row.
- `.scratch/post-testing-polish/` sized into 6 tickets this iteration (`b165239`), none blocking another.
  Diagnosis in ticket 01 found the shipped zoom already handles placement-under-zoom correctly by
  construction (`appwindow.slint:2313`'s `mouse-x / parent.width` is zoom-invariant) — only Ctrl+Scroll
  wiring is a real gap. Ticket 03's open a/b question resolved to (a), the spec's own stated default.
- In flight:
  - `BUG-107` — this worktree directly, branch `autopilot/DEC-023`. `CropRemap` domain logic verified
    correct by hand (worked-example tests, arithmetic checked). Store-layer wiring was mid-edit with a
    deliberate mutation-check stub (`return Ok(())` before the real body) — sent the builder a direct
    status check + instructions to finish the mutation check, remove the stub, and report back with the
    final green suite and commit SHA, rather than keep passively waiting through repeated pause/resume
    notifications (3 so far, ~225k tokens). Do not touch this file until it reports back for real.
  - `editor-virtual-desktop-focus` ticket 01 — `ticket-vdesktop-focus` / `autopilot/DEC-023-vdesktop-focus`.
    Diagnosing Windows Virtual-Desktop switching feasibility before implementing.
  - Post-testing-polish ticket 01 (Ctrl+Scroll zoom) — `ticket-zoom-scroll` / `autopilot/DEC-023-zoom-scroll`.
  - Post-testing-polish ticket 04 (copy-on-save) — `ticket-copy-on-save` / `autopilot/DEC-023-copy-on-save`.
  - Post-testing-polish ticket 06 (about-tab icon) — `ticket-about-icon` / `autopilot/DEC-023-about-icon`.
- Blocked: —
- Parked: —
- Next: wait for genuine completions (not a paused mid-task notification) on all five, independently
  review each diff in full before trusting any report, re-run the full suite, then merge serially. After
  that: post-testing-polish tickets 02 (marker focus/tooltip), 03 (second Assemble button), 05 (bulk
  reclaim space) — deferred this iteration to keep review load manageable, not because they're blocked.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
| --- | --- | --- | --- | --- | --- |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — scope | Scope narrowed to `defects: [BUG-107]`, excluding `BUG-7/23/28/37/57/61` | Carrying all seven open defects forward as `DEC-019` did | Re-running six iterations that only re-derive an owner-only conclusion already on record | `DEC-023` row in `decisions.yaml`, `DEC-023-....md` |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — validators | Treated the 6 baseline-matched reds as green | Blocking preflight on them | None — `DEC-021` already accepts these as permanent | preflight page (this ledger) |
| Iter 4, commit `1bee143` | canvas-zoom-clipboard-paste tickets | Both tickets closed as `done` with no code change — features already on `origin/main` since 2026-09-04 | Re-implementing a feature that already exists (both builders independently found this; verified again by the coordinator, not taken on trust) | None — verified via `git merge-base --is-ancestor` against `origin/main`, not against a builder's claim | `.scratch/canvas-zoom-clipboard-paste/{spec,issues/01,issues/02}.md` |
| Iter 4 | `BUG-107` review | Deferred review — builder still live, caught a file mid-edit with a deliberate mutation-check stub in place | Committing or fixing the file myself while the builder was still writing to it | Racing a live builder's own edits, corrupting its in-progress work | — (no commit made) |
