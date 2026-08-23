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

Two front ends. Rows 1–13 are the React webview inside `desktop-app`; rows 14–15 are `web-ui` in the
reader's browser. A desktop surface that is a window rather than a route has `—` for its route.

`No` is stable. A new row takes the next number; a removed one keeps its number with
`status: removed`.

## Rows

| No | Screen | Route | Owning component | Actor | UC served |
| --- | --- | --- | --- | --- | --- |
| 0 | Editor shell | — (the window frame itself) | `settings` | Reviewer | UC-24, UC-25 |
| 1 | Capture Overlay | — (one transparent window per monitor) | `finding` | Reviewer | UC-1, UC-2 |
| 2 | Capture note field | — (anchored to the selected region) | `finding` | Reviewer | UC-1 |
| 3 | Capture confirmation toast | — (transient, never takes focus) | `finding` | Reviewer | UC-2 |
| 4 | Editor — Findings | `/findings` | `finding` | Reviewer | UC-3, UC-4, UC-6 |
| 5 | Finding detail with Marker canvas | `/findings/:id` | `finding` | Reviewer | UC-4, UC-5 |
| 6 | Delete Findings confirmation | `/findings/delete` (modal) | `finding` | Reviewer | UC-7 |
| 7 | Orphan report | `/findings/orphans` | `finding` | Reviewer | UC-8 |
| 8 | Editor — Bundles | `/bundles` | `bundle` | Reviewer | UC-10, UC-11, UC-23 |
| 9 | Compose Bundle | `/bundles/compose` (modal) | `bundle` | Reviewer | UC-9 |
| 10 | Bundle detail | `/bundles/:id` | `bundle` | Reviewer | UC-11, UC-12, UC-23 |
| 11 | Publish and unpublish a Bundle | `/bundles/:id/publish` (modal) | `sharing` | Reviewer | UC-20, UC-22 |
| 12 | Settings | `/settings` | `settings` | Reviewer | UC-13, UC-14, UC-15, UC-16 |
| 13 | Settings — Agent access | `/settings/agent-access` | `agent-access` | Reviewer | UC-17, UC-19 |
| 14 | Published Bundle reader | `/b/:slug` | `sharing` | Remote coding agent, Reviewer | UC-21 |
| 15 | Publication not available | `/b/:slug` (the refused state) | `sharing` | Remote coding agent, Reviewer | UC-22 |

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