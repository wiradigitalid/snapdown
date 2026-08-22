---
type: rules
scope: component
component: finding
status: draft
created: "2026-08-22"
updated: "2026-08-22"
---

# Business Rules — finding

Rules binding **only** this component. A rule that turns out to bind a second one is promoted to
`.what/business-rules.md` through `wdi-blueprint`, never copied.

Ids continue the global sequence, which ends at BR-30 in the product-wide file.

## Rules

| id | Rule | Binds | Source | Status |
| --- | --- | --- | --- | --- |
| BR-31 | A selected region smaller than 8 × 8 pixels is refused. Nothing is stored and the overlay stays open. | `finding` | FR-1 · UC-1 | active |
| BR-32 | Escape at any point before the save discards the Capture. Nothing reaches the Vault and no row is written. | `finding` | FR-1 · FR-2 · UC-1 | active |
| BR-33 | The note field is focused the moment it appears, and the Capture can be saved without touching the mouse. | `finding` | FR-2 · UC-1 | active |
| BR-34 | Multi-line Note text is preserved verbatim, blank lines included. | `finding` | FR-2 · FR-7 | active |
| BR-35 | After a save, keyboard focus returns to the window that held it before the hotkey was pressed. | `finding` | FR-3 · UC-2 | active |
| BR-36 | The capture confirmation is a transient toast. It never takes focus, never needs dismissing, and states the running count of Findings. | `finding` | FR-3 · UC-2 | active |
| BR-37 | The Editor does not open after a Capture unless the Reviewer has turned that on. The shipped default is off. | `finding`, `settings` | FR-3 · OQ-9 | active |
| BR-38 | The Capture Overlay covers every connected monitor, and a region may be dragged on any of them. | `finding` | FR-1 · UC-1 | active |
| BR-39 | The selected region's pixel dimensions are shown while dragging. | `finding` | FR-1 | active |
| BR-40 | An image already within the Quality Budget's long edge is not upscaled. | `finding` | FR-4 | active |
| BR-41 | Reduction preserves aspect ratio. An image is never stretched, and never cropped to fit a budget. | `finding` | FR-4 | active |
| BR-42 | Reduction never delays the overlay closing. | `finding` | FR-4 · NFR-2 | active |
| BR-43 | Findings are listed newest first. | `finding` | FR-6 · UC-3 | active |
| BR-44 | A Finding captured while the Editor is open appears without the Editor being reopened. | `finding` | FR-6 | active |
| BR-45 | A Finding whose image file is missing is shown as broken. It is never omitted from the list. | `finding` | FR-6 · FR-15 · UC-8 | active |
| BR-46 | A Note edit persists without an explicit save action. | `finding` | FR-7 · UC-4 | active |
| BR-47 | The numbered lines belonging to Markers cannot be renumbered by hand. That ordering belongs to the Markers. | `finding` | FR-7 · FR-8 | active |
| BR-48 | A Marker can be repositioned without its number changing. | `finding` | FR-8 · UC-5 | active |
| BR-49 | Marker positions survive closing and reopening the Editor. | `finding` | FR-8 | active |
| BR-50 | Range selection and individual toggling both work, and the count of selected Findings is visible. | `finding` | FR-9 · UC-6 | active |
| BR-51 | Select-all and clear-selection are each one action, and the selection is cleared when an action on it completes. | `finding` | FR-9 · UC-6 | active |
| BR-52 | The orphan check runs at startup and can be run on demand. It never deletes anything on its own. | `finding` | FR-15 · UC-8 | active |
| BR-53 | A clean Vault reports itself as clean. Silence is not an answer. | `finding` | FR-15 · UC-8 | active |
| BR-54 | Deleting an unreferenced file the orphan report found is one action. | `finding` | FR-15 · UC-8 | active |
| BR-55 | A file that refuses to be deleted is reported by name, and the Finding stays. | `finding` | BR-5 · FR-13 · UC-7 | active |
| BR-56 | A Finding that belongs to a Bundle can still be deleted. The Bundle keeps its own image copy and stays readable. | `finding` | FR-13 · BR-12 | active |

## Retired

None yet. A retired rule keeps its id, states what replaced it, and is never deleted.
