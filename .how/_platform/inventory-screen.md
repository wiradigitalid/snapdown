---
type: inventory
kind: screen
scope: _platform
status: draft
created: "2026-08-22"
updated: "2026-08-22"
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

The system tray menu is not a screen. It is the desktop shell's own affordance for opening rows 4 and
12 and for quitting, and it holds no state of its own.

Row 12 carries four use cases because Settings is one screen with four sections, not four screens.
Splitting it into four rows would promise navigation the product does not have. Row 13 is separate
because it belongs to a different component and a different release.

Rows 14 and 15 are the same route in two states. They are listed separately because the refused state
is a promise — NFR-15 requires it to be identical for an unknown, a revoked, and a never-issued slug —
and a promise with no row is a promise nobody checks.

No row is owned by `_platform`.

## Findings

None — `derived_from: plan`, and there is no code to derive from yet.
