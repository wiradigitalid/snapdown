---
topic: wdi-autopilot run log — mandate DEC-023
artifact: .control/decisions/DEC-023-autopilot-mandate-r5-bug-107-and-three-ready-specs.md
updated: 2026-09-05T00:00
---

## Resume

Mandate: `DEC-023`. Parameters at `.control/registry/decisions.yaml` → `DEC-023.mandate`
(`from_gate: G5`, `scope: {fr: all, defects: [BUG-107]}`, `parked: [ad-n]`, `smoke_test: agent`,
`loop: 10m`, `expires: 2026-09-12`).

- Iteration: 9, commit (pending, this write) is the boundary so far — HEAD after merging tickets 06 and
  `editor-virtual-desktop-focus`, both independently re-verified (full `cargo fmt`/`clippy -D warnings`/
  `test --workspace --no-fail-fast`, green each time, run by the coordinator, not taken on either
  builder's report)
- Run branch: `autopilot/DEC-023`. **PR #47 (draft)**: https://github.com/wiradigitalid/snapdown/pull/47.
  CI on the first pushed head (`a135a04`): Korpus Validation passed, Desktop CI was still running as of
  the last check — not yet re-pushed with this iteration's two new merges.
- Stopped at: **Capacity** — two more builders (copy-on-save, Ctrl+Scroll zoom) still running.
- Done this iteration:
  - Post-testing-polish ticket 06 (About-tab icon, `FR-27`) — merged. `settings.slint`'s SNAPDOWN card
    now shows a 32×32 `Image` bound to the existing, already-proven `app-icon.png` asset (reused rather
    than the `.ico`, which Slint's asset pipeline can't decode). Wiring test confirms a real `Image`
    element exists in the card, not a comment.
  - `editor-virtual-desktop-focus` ticket 01 — merged. Feasibility finding: no public API exists to
    switch the ACTIVE Virtual Desktop (only `IVirtualDesktopManager`, query/move-only); switching
    normally needs the undocumented, per-build-fragile `IVirtualDesktopManagerInternal`, deliberately
    NOT taken on. Instead, `SetForegroundWindow` (backed by `AttachThreadInput` so it isn't refused by
    the foreground-lock heuristic) already brings the desktop switch along as a side effect — the same
    mechanism a taskbar click uses. All four "reopen the Editor" entry points now route through one
    shared function (`focus::bring_editor_to_foreground`), reachability-tested. The actual OS-level
    desktop switch is honestly flagged as unverified by hands-on testing (no interactive Windows session
    available to the builder) — a manual pass is still owed, named in the final report, not silently
    claimed as done.
- Also done (iteration 6): `BUG-107` (independently verified, `3bbde0e`).
  Also done (iteration 4): `canvas-zoom-clipboard-paste` bookkeeping (`1bee143`).
- In flight:
  - Post-testing-polish ticket 01 (Ctrl+Scroll zoom) — `ticket-zoom-scroll` / `autopilot/DEC-023-zoom-scroll`.
  - Post-testing-polish ticket 04 (copy-on-save) — `ticket-copy-on-save` / `autopilot/DEC-023-copy-on-save`.
- Blocked: —
- Parked: —
- Next: wait for genuine completions on both, independently review + full suite + merge, then push
  (updates PR #47's head, re-triggers CI). After that: post-testing-polish tickets 02 (marker
  focus/tooltip), 03 (second Assemble button), 05 (bulk reclaim space) — still queued, not blocked.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
| --- | --- | --- | --- | --- | --- |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — scope | Scope narrowed to `defects: [BUG-107]`, excluding `BUG-7/23/28/37/57/61` | Carrying all seven open defects forward as `DEC-019` did | Re-running six iterations that only re-derive an owner-only conclusion already on record | `DEC-023` row in `decisions.yaml`, `DEC-023-....md` |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — validators | Treated the 6 baseline-matched reds as green | Blocking preflight on them | None — `DEC-021` already accepts these as permanent | preflight page (this ledger) |
| Iter 4, commit `1bee143` | canvas-zoom-clipboard-paste tickets | Both tickets closed as `done` with no code change — features already on `origin/main` since 2026-09-04 | Re-implementing a feature that already exists (both builders independently found this; verified again by the coordinator, not taken on trust) | None — verified via `git merge-base --is-ancestor` against `origin/main`, not against a builder's claim | `.scratch/canvas-zoom-clipboard-paste/{spec,issues/01,issues/02}.md` |
| Iter 4 | `BUG-107` review | Deferred review — builder still live, caught a file mid-edit with a deliberate mutation-check stub in place | Committing or fixing the file myself while the builder was still writing to it | Racing a live builder's own edits, corrupting its in-progress work | — (no commit made) |
| Iter 6, commit `3bbde0e` | `BUG-107` fix — crop-remap semantics | A Marker outside the new bounds is deleted (not clamped); a box annotation is clipped to what survives; an Arrow/Callout-tail is clamped onto the new edge unless its whole bbox misses | Clamping every kind uniformly (simpler, but misrepresents where a Marker's single point actually was) | An owner who wanted uniform clamping would see markers silently vanish instead — reported here so it's checkable, not a silent choice | `crates/snapdown-core/src/domain/finding.rs`, `.control/registry/defects.yaml` (`BUG-107` `fix:`) |
