---
topic: wdi-autopilot run log — mandate DEC-023
artifact: .control/decisions/DEC-023-autopilot-mandate-r5-bug-107-and-three-ready-specs.md
updated: 2026-09-05T00:00
---

## Resume

Mandate: `DEC-023`. Parameters at `.control/registry/decisions.yaml` → `DEC-023.mandate`
(`from_gate: G5`, `scope: {fr: all, defects: [BUG-107]}`, `parked: [ad-n]`, `smoke_test: agent`,
`loop: 10m`, `expires: 2026-09-12`).

- Iteration: 10, commit (pending, this write) is the boundary — HEAD after merging tickets 01
  (Ctrl+Scroll zoom) and 04 (copy-on-save), both independently re-verified (full `cargo fmt`/
  `clippy -D warnings`/`test --workspace --no-fail-fast`, green, coordinator-run each time)
- Run branch: `autopilot/DEC-023`. **PR #47 (draft)**: https://github.com/wiradigitalid/snapdown/pull/47.
  Pushing this iteration's boundary next (six real merges since the first push).
- Stopped at: **Capacity** (about to dispatch the remaining three post-testing-polish tickets — not a
  stop yet, see below).
- Done, all independently re-verified by the coordinator (full suite re-run from scratch each time, not
  taken on any builder's report):
  - `BUG-107` (`3bbde0e`, iteration 6)
  - `canvas-zoom-clipboard-paste` bookkeeping — already shipped, corrected (`1bee143`, iteration 4)
  - Post-testing-polish ticket 06, About-tab icon (`FR-27`)
  - `editor-virtual-desktop-focus` ticket 01 — feasibility finding: no public API to switch the ACTIVE
    Virtual Desktop; `SetForegroundWindow` (+`AttachThreadInput`) brings the switch along as a documented
    side effect instead of depending on the undocumented `IVirtualDesktopManagerInternal`. Actual desktop
    switch unverified by hands-on testing — flagged, not silently claimed.
  - Post-testing-polish ticket 01, Ctrl+Scroll zoom (`FR-34`) — reuses the shipped `zoomed_in`/`zoomed_out`
    callbacks exactly, plain Scroll still rejects/falls through unchanged.
  - Post-testing-polish ticket 04, copy-on-save (`FR-10`/`FR-12`/`FR-40`) — both new call sites reuse
    Copy Markdown's own two functions, gated correctly on save success only.
- In flight: none — about to dispatch tickets 02, 03, 05 (see Next).
- Blocked: —
- Parked: —
- Next: push this boundary (updates PR #47, re-triggers CI on the new head). Dispatch the three
  remaining post-testing-polish tickets in parallel, each in its own worktree — 02 (marker focus/
  tooltip), 03 (second Assemble button + filmstrip alignment, a/b already resolved to (a)), 05 (bulk
  reclaim space, `AD-2` write-ordering + a `wdi-product` follow-up once green). None block another.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
| --- | --- | --- | --- | --- | --- |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — scope | Scope narrowed to `defects: [BUG-107]`, excluding `BUG-7/23/28/37/57/61` | Carrying all seven open defects forward as `DEC-019` did | Re-running six iterations that only re-derive an owner-only conclusion already on record | `DEC-023` row in `decisions.yaml`, `DEC-023-....md` |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — validators | Treated the 6 baseline-matched reds as green | Blocking preflight on them | None — `DEC-021` already accepts these as permanent | preflight page (this ledger) |
| Iter 4, commit `1bee143` | canvas-zoom-clipboard-paste tickets | Both tickets closed as `done` with no code change — features already on `origin/main` since 2026-09-04 | Re-implementing a feature that already exists (both builders independently found this; verified again by the coordinator, not taken on trust) | None — verified via `git merge-base --is-ancestor` against `origin/main`, not against a builder's claim | `.scratch/canvas-zoom-clipboard-paste/{spec,issues/01,issues/02}.md` |
| Iter 4 | `BUG-107` review | Deferred review — builder still live, caught a file mid-edit with a deliberate mutation-check stub in place | Committing or fixing the file myself while the builder was still writing to it | Racing a live builder's own edits, corrupting its in-progress work | — (no commit made) |
| Iter 6, commit `3bbde0e` | `BUG-107` fix — crop-remap semantics | A Marker outside the new bounds is deleted (not clamped); a box annotation is clipped to what survives; an Arrow/Callout-tail is clamped onto the new edge unless its whole bbox misses | Clamping every kind uniformly (simpler, but misrepresents where a Marker's single point actually was) | An owner who wanted uniform clamping would see markers silently vanish instead — reported here so it's checkable, not a silent choice | `crates/snapdown-core/src/domain/finding.rs`, `.control/registry/defects.yaml` (`BUG-107` `fix:`) |
