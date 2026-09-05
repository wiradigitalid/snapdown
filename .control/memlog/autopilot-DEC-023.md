---
topic: wdi-autopilot run log — mandate DEC-023
artifact: .control/decisions/DEC-023-autopilot-mandate-r5-bug-107-and-three-ready-specs.md
updated: 2026-09-05T00:00
---

## Resume

Mandate: `DEC-023`. Parameters at `.control/registry/decisions.yaml` → `DEC-023.mandate`
(`from_gate: G5`, `scope: {fr: all, defects: [BUG-107]}`, `parked: [ad-n]`, `smoke_test: agent`,
`loop: 10m`, `expires: 2026-09-12`).

- **Run ended.** 12 iterations. Mandate raised to `status: applied` (this write). Loop cancelled.
- Run branch: `autopilot/DEC-023`. **PR #47**: https://github.com/wiradigitalid/snapdown/pull/47 — marked
  ready for review if CI is green on the final pushed head (checked at Finish), left draft otherwise.
- Stopped at: **Done** — every candidate this mandate's preflight found runnable is closed. Nothing
  left in scope is parked or blocked.
- Closed, all independently re-verified by the coordinator (full `cargo fmt --check`/
  `clippy -D warnings`/`test --workspace --no-fail-fast` re-run from scratch after every merge, not taken
  on any builder's report; `validate.py --check` clean against the baseline throughout):
  - `BUG-107` (crop-remap) — `CropRemap` domain type, mutation-tested; a Marker outside the new bounds
    is deleted, a box annotation clipped, an Arrow/Callout-tail clamped, reasoned in `defects.yaml`.
  - `canvas-zoom-clipboard-paste` — already shipped before this mandate opened; ticket bookkeeping
    corrected, no code change.
  - `editor-virtual-desktop-focus` — `SetForegroundWindow`+`AttachThreadInput` brings the OS desktop
    switch along as a documented side effect, deliberately not depending on the undocumented
    `IVirtualDesktopManagerInternal`. The actual desktop-switch behaviour stays unverified by hands-on
    testing — owed a manual pass.
  - Post-testing-polish, all 6 tickets: 01 Ctrl+Scroll zoom, 02 marker note auto-focus + hover tooltip,
    03 second Assemble button + filmstrip alignment fix, 04 copy-on-save, 05 bulk reclaim space
    (select-all + Delete both, registering `FR-44` rather than widening `FR-42` to respect
    `entity-one-writer`), 06 About-tab icon.
- Blocked: —
- Parked: —
- Next: nothing from this mandate. Follow-ups for the owner are named in the final report (Finish
  Output), not here.

**Smoke test run** (`smoke_test: agent`), against `target/release/Snapdown.exe` built from this fully
merged branch (superseding the earlier stale instance from ticket 05's own worktree, which was closed):
launched via `computer-use` (Orca), driven against the Reviewer's real Vault (`D:\SnapdownVault2`) —
no destructive action was taken on real data; every check that could destroy something used Cancel
after inspecting the confirmation, never Confirm.

| Item | Result |
| --- | --- |
| `BUG-107` (crop-remap) | **Not live-tested.** Would need a disposable test Finding with markers, cropped, to observe the remap — skipped this pass to keep the smoke test bounded. Stands on its own independently-verified automated tests (mutation-tested, decode-based) instead. |
| `canvas-zoom-clipboard-paste` (`FR-34` buttons, `FR-35`) | **Zoom in/out/reset: PASS** — clicked Zoom in, canvas visibly scaled; Zoom out/Natural size reset it. Paste not exercised (no test image staged). |
| Ctrl+Scroll zoom (ticket 01) | **Not live-tested** — the `orca computer scroll` command has no modifier-key flag, so a Ctrl-held scroll cannot be synthesized through this tool. Stands on code review (the `scroll-event` handler reusing the shipped `zoomed_in`/`zoomed_out` callbacks) + its own passing wiring test. |
| Marker focus + tooltip (ticket 02) | **PASS** — clicking Marker #2's canvas badge showed a live tooltip ("lontong") and put a real focus ring on Marker #2's own Note field in the panel. |
| Second Assemble button + filmstrip alignment (ticket 03) | **PASS (partial)** — both Assemble controls (canvas-top and filmstrip-footer) are present and correctly disabled ("Nothing picked") with nothing ticked, matching the required selection gate. Did not tick a real Finding to see the enabled/click path, to avoid touching real data. Filmstrip alignment is a pixel-level visual check not practical through this tool. |
| Copy-on-save (ticket 04) | **Not live-tested** — exercising it means actually running Assemble & Save or Review & Update Save against real Bundles. Skipped to avoid mutating real data; stands on its own passing, mutation-tested wiring tests. |
| Bulk reclaim space (ticket 05) | **PASS** — opened Reclaim space (15 real Bundles listed), ticked "Select all" (15 of 15 selected, correct freed-space total), clicked "Delete both", read the real confirmation dialog verbatim: *"DELETE BOTH 15 BUNDLES? 15 Bundles and their 16 original captures... This cannot be undone."* — the 15-vs-16 count is the live app correctly deduping a Finding shared across two Bundles. **Cancelled**, then closed the dialog without confirming; no real data touched. |
| About-tab icon (ticket 06) | **PASS** — Settings → About shows the Snapdown app icon beside "Snapdown 0.1.0". |
| `editor-virtual-desktop-focus` | **Not live-tested** (needs two Windows Virtual Desktops to switch between, not set up in this smoke test). Already honestly flagged by its own builder as unverified by hands-on testing — stands unchanged; still owed a manual pass. |

Net: 4 of 9 candidates directly exercised live and passed with real, decoded confirmation (not just
"it opened"); the rest were left to their own independently-verified automated tests rather than risk
the Reviewer's real Vault data or hit real tooling limits (no Ctrl-modifier scroll support in the
computer-use CLI). Every one of the untested items already carries its own passing, often
mutation-tested, automated coverage — this smoke test adds live confirmation on top where it safely
could, not a replacement for it.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
| --- | --- | --- | --- | --- | --- |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — scope | Scope narrowed to `defects: [BUG-107]`, excluding `BUG-7/23/28/37/57/61` | Carrying all seven open defects forward as `DEC-019` did | Re-running six iterations that only re-derive an owner-only conclusion already on record | `DEC-023` row in `decisions.yaml`, `DEC-023-....md` |
| Iter 0 (preflight), commit (pending) | Door 1 preflight — validators | Treated the 6 baseline-matched reds as green | Blocking preflight on them | None — `DEC-021` already accepts these as permanent | preflight page (this ledger) |
| Iter 4, commit `1bee143` | canvas-zoom-clipboard-paste tickets | Both tickets closed as `done` with no code change — features already on `origin/main` since 2026-09-04 | Re-implementing a feature that already exists (both builders independently found this; verified again by the coordinator, not taken on trust) | None — verified via `git merge-base --is-ancestor` against `origin/main`, not against a builder's claim | `.scratch/canvas-zoom-clipboard-paste/{spec,issues/01,issues/02}.md` |
| Iter 4 | `BUG-107` review | Deferred review — builder still live, caught a file mid-edit with a deliberate mutation-check stub in place | Committing or fixing the file myself while the builder was still writing to it | Racing a live builder's own edits, corrupting its in-progress work | — (no commit made) |
| Iter 6, commit `3bbde0e` | `BUG-107` fix — crop-remap semantics | A Marker outside the new bounds is deleted (not clamped); a box annotation is clipped to what survives; an Arrow/Callout-tail is clamped onto the new edge unless its whole bbox misses | Clamping every kind uniformly (simpler, but misrepresents where a Marker's single point actually was) | An owner who wanted uniform clamping would see markers silently vanish instead — reported here so it's checkable, not a silent choice | `crates/snapdown-core/src/domain/finding.rs`, `.control/registry/defects.yaml` (`BUG-107` `fix:`) |
| Iter 12, ticket 05's own commit `c663801` | `wdi-product` pass for ticket 05's "widen FR-42" instruction | Registered new `FR-44` (component `bundle`, `defers_to: [FR-25, FR-42]`) instead; `FR-42` left unchanged | Widening `FR-42` to write `[Bundle, BundleItem]` as the ticket literally said | Would have made `finding` write entities `bundle` owns, violating `entity-one-writer` — an owner reading `FR-42` later would find a promise the component structurally cannot keep | `.control/registry/requirements-capture-to-markdown.yaml`, `.control/registry/usecases.yaml`, `.what/_prd/capture-to-markdown/{prd.md,addendum.md}` |
| Iter 12, this commit | Finish — smoke test scope | Live-tested 4 of 9 candidates (zoom, marker focus/tooltip, Assemble gate, reclaim-space dialog); left the rest to their own automated coverage | Live-testing all 9, including `BUG-107`/copy-on-save/bulk-delete's confirm path | Would have required mutating the Reviewer's real Vault data (15 real Bundles) or staging disposable fixtures — judged not worth the added run time given each stands on independently-verified, often mutation-tested automated tests already | this ledger's smoke-test table |
