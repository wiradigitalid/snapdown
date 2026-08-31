---
type: inventory
kind: screen
scope: _platform
status: draft
created: "2026-08-22"
updated: "2026-08-23"
derived_from: plan
verified: ""
---

# Inventory — screens

Two front ends. Rows 1–13 are the desktop UI inside `desktop-app` — a React webview until `DEC-007`,
native Slint since; rows 14–15 are `web-ui` in the reader's browser, unaffected by that decision. A
desktop surface that is a window rather than a route has `—` for its route.

`No` is stable. A new row takes the next number; a removed one keeps its number with
`status: removed`.

## Derivation finding, 2026-08-31 — the Route column is a fossil, and this is not hand work

`inventory.py` was run against `.constitution/project/inventory-readers.py` and reports **15
plan-versus-code gaps** on this inventory. They are one finding wearing two shapes, and neither is
patched over here, because the script's own rule is that a plan-versus-code gap is routed to the skill
that owns its side and MUST NOT be fixed by editing the other side.

**Every desktop route in the table below describes a router this product does not have.** `/findings`,
`/bundles`, `/bundles/:id`, `/settings`, `/settings/agent-access` and the rest are React Router paths.
`DEC-007` moved the desktop app off React onto Slint, and Slint has no routes. The derivation reports
each of them as *"planned but not read in code"*, which is accurate: there is nothing to read.

**Every per-screen component file the plan expects is absent.** The reader looks for
`apps/desktop/ui/screens/findings-view.slint`, `screens/bundle-view.slint`,
`screens/settings-view.slint`, `screens/orphan-report-view.slint`, `screens/agent-access-view.slint`,
`components/confirm-dialog.slint` and `components/publish-dialog.slint`. None exists. The shipped
desktop UI is one file — `apps/desktop/ui/appwindow.slint` — so the row-per-file structure this
inventory assumes was never built. Two `web-ui` rows are absent for the same reason
(`PublishedBundleReader.tsx`, `PublicationNotFound.tsx`).

**What this does NOT mean.** It is not evidence that the screens are unreachable. The Capture Overlay,
the Findings list, the Bundle list, the compose modal and Settings all exist and run; they live inside
one Slint file rather than behind a route. This is a stale **plan**, not a missing product — the
opposite of the `BUG-4`/`BUG-5` class, where a component existed and nothing mounted it.

**Re-planning this table against Slint is owed and is deliberately NOT done in this pass.** It is
entangled with work already in flight: the Bundle Library screen is being designed on the wayfinder map
at `.scratch/bundle-library/`, where ticket 01 settled the Library's shape and tickets 03 and 05 are
still open. Re-cutting rows 8 and 10 now would be guessing at the answer those tickets exist to
produce. The honest state is a table that says what it planned, beside a finding that says what runs.

`verified:` therefore stays empty. It MUST NOT be filled until the rows describe Slint.

## Rows & UI Design Assets Mapping

| No | Screen | Route | Owning component | Actor | UC served | Permanent HTML Asset (.how Layer) |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | Editor shell | — (the window frame itself) | `settings` | Reviewer | UC-24, UC-25 | [`.how/finding/01-ux/assets/01-studio-workspace.html`](../finding/01-ux/assets/01-studio-workspace.html) |
| 1 | Capture Overlay | — (one transparent window per monitor) | `finding` | Reviewer | UC-1, UC-2 | [`.how/finding/01-ux/assets/02-capture-overlay.html`](../finding/01-ux/assets/02-capture-overlay.html) |
| 2 | Capture note field | — (anchored to the selected region) | `finding` | Reviewer | UC-1 | [`.how/finding/01-ux/assets/02-capture-overlay.html`](../finding/01-ux/assets/02-capture-overlay.html) |
| 3 | Capture confirmation toast | — (transient, never takes focus) | `finding` | Reviewer | UC-2 | [`.how/finding/01-ux/assets/01-studio-workspace.html`](../finding/01-ux/assets/01-studio-workspace.html) |
| 4 | Editor — Findings | `/findings` | `finding` | Reviewer | UC-3, UC-4, UC-6 | [`.how/finding/01-ux/assets/01-studio-workspace.html`](../finding/01-ux/assets/01-studio-workspace.html) |
| 5 | Finding detail with Marker canvas | `/findings/:id` | `finding` | Reviewer | UC-4, UC-5 | [`.how/finding/01-ux/assets/01-studio-workspace.html`](../finding/01-ux/assets/01-studio-workspace.html) |
| 6 | Delete Findings confirmation | `/findings/delete` (modal) | `finding` | Reviewer | UC-7 | [`.how/bundle/01-ux/assets/05-saved-bundles-drawer.html`](../bundle/01-ux/assets/05-saved-bundles-drawer.html) |
| 7 | Orphan report | `/findings/orphans` | `finding` | Reviewer | UC-8 | [`.how/finding/01-ux/assets/01-studio-workspace.html`](../finding/01-ux/assets/01-studio-workspace.html) |
| 8 | Editor — Bundles | `/bundles` | `bundle` | Reviewer | UC-10, UC-11, UC-23, UC-28, UC-30 | [`.how/bundle/01-ux/assets/05-saved-bundles-drawer.html`](../bundle/01-ux/assets/05-saved-bundles-drawer.html) |
| 9 | Compose Bundle | `/bundles/compose` (modal) | `bundle` | Reviewer | UC-9 | [`.how/bundle/01-ux/assets/04-bundle-assembly-modal.html`](../bundle/01-ux/assets/04-bundle-assembly-modal.html) |
| 10 | Bundle detail | `/bundles/:id` | `bundle` | Reviewer | UC-11, UC-12, UC-23, UC-28, UC-29 | [`.how/bundle/01-ux/assets/04-bundle-assembly-modal.html`](../bundle/01-ux/assets/04-bundle-assembly-modal.html) |
| 11 | Publish and unpublish a Bundle | `/bundles/:id/publish` (modal) | `sharing` | Reviewer | UC-20, UC-22 | [`.how/bundle/01-ux/assets/04-bundle-assembly-modal.html`](../bundle/01-ux/assets/04-bundle-assembly-modal.html) |
| 12 | Settings — General & Quality | `/settings` | `settings` | Reviewer | UC-13, UC-14, UC-15, UC-16 | [`.how/settings/01-ux/assets/06a-settings-general.html`](../settings/01-ux/assets/06a-settings-general.html)<br>[`.how/settings/01-ux/assets/06b-settings-hotkeys.html`](../settings/01-ux/assets/06b-settings-hotkeys.html)<br>[`.how/settings/01-ux/assets/06d-settings-about.html`](../settings/01-ux/assets/06d-settings-about.html) |
| 13 | Settings — Agent access | `/settings/agent-access` | `agent-access` | Reviewer | UC-17, UC-19 | [`.how/settings/01-ux/assets/06c-settings-agent-bridge.html`](../settings/01-ux/assets/06c-settings-agent-bridge.html) |
| 14 | Published Bundle reader | `/b/:slug` | `sharing` | Remote coding agent, Reviewer | UC-21 | [`.how/bundle/01-ux/assets/04-bundle-assembly-modal.html`](../bundle/01-ux/assets/04-bundle-assembly-modal.html) |
| 16 | Reclaim space | — (a desktop surface; PLAN ONLY, no code and no route) | `finding` | Reviewer | UC-31 | _none yet_ |
| 15 | Publication not available | `/b/:slug` (the refused state) | `sharing` | Remote coding agent, Reviewer | UC-22 | [`.how/bundle/01-ux/assets/04-bundle-assembly-modal.html`](../bundle/01-ux/assets/04-bundle-assembly-modal.html) |

Row 0 is numbered 0 rather than 16 deliberately. Numbers here are stable and a new row normally takes
the next one — but the shell is not a new surface, it is the frame every other row has always been
drawn inside, and it was missing rather than added. Numbering it 16 would place the frame after the
things it contains. This is the one exception, and it is not a precedent: rows 16 onward take the next
number as usual.

The system tray menu is not a screen. It is the desktop shell's own affordance for opening rows 4 and
12 and for quitting, and it holds no state of its own.

Row 12 carries four use cases because Settings is one screen with four sections, not four screens.
Splitting it into four rows would promise navigation the product does not have. That reasoning was
right and is now load-bearing: `FR-29` requires all five groups visible at the minimum window size,
and the alternative considered at G2 — a sub-navigation of five groups — was rejected precisely
because it would have satisfied the requirement by hiding four of them. Row 13 is separate because it
belongs to a different component and a different release.

Row 0 carries `UC-24` and `UC-25` — knowing what you have opened, and reaching every surface from
every surface. It is owned by `settings`, and § 4.7 of the PRD carries the argument: `settings`
already owns the container-level Logical Components, and the frame is machinery of the same kind.
`UC-26` (seeing everything a screen offers) is listed against **no** row, and that is a real
irregularity rather than a tidy exception. Every other use case in this product is served by a screen;
this one is a property that every screen must have. Listing it on all sixteen rows would make the
column unreadable and would still not be a claim any one row could be checked against.

It is recorded here rather than resolved: `FR-29` is checkable — at the minimum window size, nothing
is discovered only by scrolling — and the check belongs to a test over every screen, not to a row in
this table. If a later pass finds this unsatisfying, the question is whether the inventory needs a
notion of a row-crossing use case, and that is a method question rather than a product one.

The system tray menu is still not a screen, and row 0 does not change that. The tray belongs to the
**Snapdown** persona and the shell to **Snapdown Editor** (`DEC-003`); the tray holds no state and
draws no surface.

Rows 14 and 15 are the same route in two states. They are listed separately because the refused state
is a promise — NFR-15 requires it to be identical for an unknown, a revoked, and a never-issued slug —
and a promise with no row is a promise nobody checks.

No row is owned by `_platform`.

## Findings

**This whole section is a 2026-08-23 snapshot of the pre-`DEC-007` Tauri webview, now stale in
substance, not just in file paths.** The desktop UI has since been rebuilt in Slint, and the
"screen gaps" table below cites `.tsx` files that no longer exist — some because the surface was
rebuilt (rows 0 and 2, confirmed current: `apps/desktop/ui/appwindow.slint` and its capture-note-field),
some because nothing has replaced them yet (`BUG-59`, `BUG-61`, both filed 2026-08-27, are the current
source of truth for what is and is not built). Treat the table as history explaining how these gaps
were first found, not as today's state — that is `BUG-61`'s job now.

**Corrected 2026-08-23.** An earlier version of this section stated that
`.constitution/project/inventory-readers.py` "has not been written". **That was wrong** — the file
exists and has since W1. What was true is that its readers barely read: `derive_api` returned nothing
behind a comment reading *"web-api will be built in W5"*, four waves stale, and `derive_screen`
emitted exactly one row. The engine then reported 34 rows as *planned but not read in code*, which
looks like drift and was a reader that never looked.

The readers were rewritten on 2026-08-23. The result:

| Inventory | Planned | Read from code | Gaps |
|---|---|---|---|
| db | 12 | 13 | 0 |
| api | 14 | 14 | 0 |
| screen | 16 | 11 | **5** |

The five screen gaps are real, and each names the file that is absent:

| Row | Screen | Missing |
|---|---|---|
| 0 | Editor shell | `apps/desktop/src/components/EditorShell.tsx` — inline in `App.tsx`, owned by nothing. **W6-S2** |
| 2 | Capture note field | `apps/desktop/src/components/CaptureNoteField.tsx` — inside `CaptureOverlay.tsx`. `LC-029` |
| 11 | Publish and unpublish a Bundle | `PublishDialog.tsx` — **does not exist.** `BUG-2` |
| 14 | Published Bundle reader | `PublishedBundleReader.tsx` — **does not exist.** `BUG-2` |
| 15 | Publication not available | `PublicationNotFound.tsx` — **does not exist.** `BUG-2` |

Rows 11, 14 and 15 are the serious ones. `HANDOVER.md` recorded all three as delivered in W5 and no
file behind any of them was ever written. What `GET /b/{slug}` actually returns is the stored Markdown
inside a bare `<pre>` with no stylesheet and no rendered images. For a coding agent that is arguably
enough; for the human reader those rows promise, it is not. `DEC-005` freezes `sharing`, so `BUG-2` is
registered and decided when that lifts — and it is a **promise** decision, not a task: withdrawing the
three rows may be the correct answer for a product whose reader is a machine.

Rows 7 and 9 — the orphan report and the Compose Bundle dialog — **do** have components
(`OrphanReportView.tsx`, `BundleComposer.tsx`) and now read cleanly. An earlier note here said they
had no build unit; what they lacked was an `LC` registration, which `wdi-ux` supplied as `LC-030` and
`LC-031`. The distinction matters: a screen with code and no `LC` is a bookkeeping gap, and a screen
with an `LC` and no code is `BUG-2`.

**This is what a working reader buys.** Three screens have been recorded as shipped since W5 and are
absent; nothing noticed for two waves, because `derived_from: plan` means nobody ever compared the
plan to the tree.