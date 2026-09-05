---
topic: wdi-autopilot run log — mandate DEC-023
artifact: .control/decisions/DEC-023-autopilot-mandate-r5-bug-107-and-three-ready-specs.md
updated: 2026-09-05T00:00
---

## Resume

Mandate: `DEC-023`. Parameters at `.control/registry/decisions.yaml` → `DEC-023.mandate`
(`from_gate: G5`, `scope: {fr: all, defects: [BUG-107]}`, `parked: [ad-n]`, `smoke_test: agent`,
`loop: 10m`, `expires: 2026-09-12`).

- Iteration: 6, commit `3bbde0e` (`BUG-107` fix, independently verified) is the boundary so far
- Run branch: `autopilot/DEC-023`, isolated worktree at `.claude/worktrees/autopilot+DEC-023`. No PR
  opened yet — next push (queued for the next iteration boundary) opens it as a draft.
- Stopped at: **Capacity** — `BUG-107` genuinely finished and independently verified this iteration; the
  other four builders are still running. No new work dispatched.
- Done this iteration:
  - `BUG-107` — genuinely complete, not just reported. Independently re-verified by the coordinator (not
    taken on the builder's word): read the full diff, confirmed the mutation-check stub was actually
    removed (`grep` for it returns nothing), re-ran `cargo fmt --all -- --check` / `cargo clippy
    --workspace --all-targets -- -D warnings` / `cargo test --workspace --no-fail-fast` myself from
    scratch — all exit 0, 430+ tests passed, zero failures. `validate.py --generate` after: still only
    the 6 baseline-matched reds, no regression. `defects.yaml`'s `BUG-107` row is `status: fixed` with
    the domain decision (drop Markers, clip boxes, clamp Arrow/Callout-tail endpoints) reasoned in full.
    Already on `autopilot/DEC-023` directly (`3bbde0e`) — no separate merge needed.
- Also done (iteration 4): `canvas-zoom-clipboard-paste` — already shipped to `origin/main` before this
  mandate opened, ticket bookkeeping corrected, merged (`1bee143`).
- In flight:
  - `editor-virtual-desktop-focus` ticket 01 — `ticket-vdesktop-focus` / `autopilot/DEC-023-vdesktop-focus`.
  - Post-testing-polish ticket 01 (Ctrl+Scroll zoom) — `ticket-zoom-scroll` / `autopilot/DEC-023-zoom-scroll`.
  - Post-testing-polish ticket 04 (copy-on-save) — `ticket-copy-on-save` / `autopilot/DEC-023-copy-on-save`.
  - Post-testing-polish ticket 06 (about-tab icon) — `ticket-about-icon` / `autopilot/DEC-023-about-icon`.
- Blocked: —
- Parked: —
- Next: wait for genuine completions on all four, independently review each diff in full before trusting
  any report, re-run the full suite, then merge serially and push (opens the draft PR — first code
  actually lands on this iteration boundary). After that: post-testing-polish tickets 02 (marker
  focus/tooltip), 03 (second Assemble button), 05 (bulk reclaim space) — still queued, not blocked.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
| --- | --- | --- | --- | --- | --- |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — scope | Scope narrowed to `defects: [BUG-107]`, excluding `BUG-7/23/28/37/57/61` | Carrying all seven open defects forward as `DEC-019` did | Re-running six iterations that only re-derive an owner-only conclusion already on record | `DEC-023` row in `decisions.yaml`, `DEC-023-....md` |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — validators | Treated the 6 baseline-matched reds as green | Blocking preflight on them | None — `DEC-021` already accepts these as permanent | preflight page (this ledger) |
| Iter 4, commit `1bee143` | canvas-zoom-clipboard-paste tickets | Both tickets closed as `done` with no code change — features already on `origin/main` since 2026-09-04 | Re-implementing a feature that already exists (both builders independently found this; verified again by the coordinator, not taken on trust) | None — verified via `git merge-base --is-ancestor` against `origin/main`, not against a builder's claim | `.scratch/canvas-zoom-clipboard-paste/{spec,issues/01,issues/02}.md` |
| Iter 4 | `BUG-107` review | Deferred review — builder still live, caught a file mid-edit with a deliberate mutation-check stub in place | Committing or fixing the file myself while the builder was still writing to it | Racing a live builder's own edits, corrupting its in-progress work | — (no commit made) |
| Iter 6, commit `3bbde0e` | `BUG-107` fix — crop-remap semantics | A Marker outside the new bounds is deleted (not clamped); a box annotation is clipped to what survives; an Arrow/Callout-tail is clamped onto the new edge unless its whole bbox misses | Clamping every kind uniformly (simpler, but misrepresents where a Marker's single point actually was) | An owner who wanted uniform clamping would see markers silently vanish instead — reported here so it's checkable, not a silent choice | `crates/snapdown-core/src/domain/finding.rs`, `.control/registry/defects.yaml` (`BUG-107` `fix:`) |
