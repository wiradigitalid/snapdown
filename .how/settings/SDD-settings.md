---
type: sdd
component: settings
status: draft
created: "2026-08-22"
updated: "2026-08-22"
realizes: [UC-13, UC-14, UC-15, UC-16]
binds: [AD-6]
reviewed:
  date: ""
  sha: ""
  lenses: []
---

# SDD — settings

Skeleton. `mode: catalog`. At `catalog` this **is** the finished state: G4 is skipped for this
component, and the code is written from the use case catalogue in `.what/settings/SRS-settings.md`, the
three inventories in `.how/_platform/`, and C4.

## Decision Summary · [outline]

Not written: `[outline]` is above this component's `mode`.

## Structure · [outline]

Not written for the same reason. For the record, and because `wdi-init` registered them rather than
this file: `settings` owns four Logical Components in `desktop-app` — `LC-025 settings-store`,
`LC-015 settings-screen`, `LC-026 startup-registrar`, and `LC-009 hotkey-registrar`. The last one is
here rather than in `finding` because this component owns the binding it registers; it raises a
capture-requested event that `finding` listens for.

## Design Notes

## Open Items
