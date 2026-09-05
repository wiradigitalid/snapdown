---
topic: wdi-autopilot run log — mandate DEC-023
artifact: .control/decisions/DEC-023-autopilot-mandate-r5-bug-107-and-three-ready-specs.md
updated: 2026-09-05T00:00
---

## Resume

Mandate: `DEC-023`. Parameters at `.control/registry/decisions.yaml` → `DEC-023.mandate`
(`from_gate: G5`, `scope: {fr: all, defects: [BUG-107]}`, `parked: [ad-n]`, `smoke_test: agent`,
`loop: 10m`, `expires: 2026-09-12`).

- Iteration: 11, commit (pending, this write) is the boundary — HEAD after merging tickets 02
  (marker focus/tooltip) and 03 (second Assemble button + filmstrip alignment), both independently
  re-verified (full `cargo fmt`/`clippy -D warnings`/`test --workspace --no-fail-fast`, green,
  coordinator-run each time — ticket 03's builder looped on pause/resume notifications without
  committing, same shape as `BUG-107` earlier, so the coordinator ran its own verification directly in
  that worktree; the builder then also finished for real moments later, matching what the coordinator
  had already found)
- Run branch: `autopilot/DEC-023`. **PR #47 (draft)**: https://github.com/wiradigitalid/snapdown/pull/47.
  Pushing this iteration's boundary next.
- Stopped at: **Capacity** — one more builder (bulk reclaim space, ticket 05) still running; five of six
  post-testing-polish tickets are now merged.
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
  - Post-testing-polish ticket 02, marker focus/tooltip (`FR-8`/`UC-5`) — shared focus-target property
    consumed by `init` (fresh row) or `changed` via a mirror (existing row); drag excluded via a
    per-Marker guard measured in image pixels so it stays correct at any zoom level; tooltip bound
    directly to each iteration's own model row.
  - Post-testing-polish ticket 03, second Assemble button + filmstrip alignment (`FR-10`) — labelled
    MOVED (new door, same callback, same selection gate, option (a)) vs FIXED (unrelated
    `padding-bottom` centering-slack fix) per the ticket's own requirement to keep the two tellable
    apart.
- In flight: none. Every candidate from this mandate's scope is now merged.
  - Post-testing-polish ticket 05, bulk reclaim space (`FR-41`/`FR-42`/`BR-122`, plus new `FR-44`) —
    select-all + bulk Delete-both, `AD-2` ordering, shared-Finding dedup mutation-tested in a real
    store test. The builder correctly declined the ticket's literal instruction to widen `FR-42`
    (would have made `finding` write `[Bundle, BundleItem]`, violating `entity-one-writer`) and
    registered `FR-44` under `bundle` instead, deferring to `FR-42` — the same shape `FR-14` already
    uses for `FR-25`. Two already-stale `BUG-104`-era PRD sentences corrected in the same pass.
    Independently re-verified by the coordinator: full `fmt`/`clippy -D warnings`/`test --workspace`
    and `validate.py --check` all green, not taken on the builder's report.
- Blocked: —
- Parked: —
- Next: § The work table's "every FR in scope closed" applies. Moving to § Finish: smoke test, final
  `validate.py --generate` + `wdi-report` progress, raise `DEC-023` to `applied`, push, mark PR #47
  ready for review if CI is green, cancel the loop, final report.

Note for Finish: `Snapdown.exe` (PID 28564) is running, launched by ticket 05's own builder for its
manual "Look at" step — built from the `ticket-reclaim-bulk` worktree BEFORE tickets 02/03 merged
into it, so it is not representative of the final merged branch. It does not block this worktree's own
build (separate `target/` directory) but is flagged so the owner does not mistake it for the final
state.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
| --- | --- | --- | --- | --- | --- |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — scope | Scope narrowed to `defects: [BUG-107]`, excluding `BUG-7/23/28/37/57/61` | Carrying all seven open defects forward as `DEC-019` did | Re-running six iterations that only re-derive an owner-only conclusion already on record | `DEC-023` row in `decisions.yaml`, `DEC-023-....md` |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — validators | Treated the 6 baseline-matched reds as green | Blocking preflight on them | None — `DEC-021` already accepts these as permanent | preflight page (this ledger) |
| Iter 4, commit `1bee143` | canvas-zoom-clipboard-paste tickets | Both tickets closed as `done` with no code change — features already on `origin/main` since 2026-09-04 | Re-implementing a feature that already exists (both builders independently found this; verified again by the coordinator, not taken on trust) | None — verified via `git merge-base --is-ancestor` against `origin/main`, not against a builder's claim | `.scratch/canvas-zoom-clipboard-paste/{spec,issues/01,issues/02}.md` |
| Iter 4 | `BUG-107` review | Deferred review — builder still live, caught a file mid-edit with a deliberate mutation-check stub in place | Committing or fixing the file myself while the builder was still writing to it | Racing a live builder's own edits, corrupting its in-progress work | — (no commit made) |
| Iter 6, commit `3bbde0e` | `BUG-107` fix — crop-remap semantics | A Marker outside the new bounds is deleted (not clamped); a box annotation is clipped to what survives; an Arrow/Callout-tail is clamped onto the new edge unless its whole bbox misses | Clamping every kind uniformly (simpler, but misrepresents where a Marker's single point actually was) | An owner who wanted uniform clamping would see markers silently vanish instead — reported here so it's checkable, not a silent choice | `crates/snapdown-core/src/domain/finding.rs`, `.control/registry/defects.yaml` (`BUG-107` `fix:`) |
