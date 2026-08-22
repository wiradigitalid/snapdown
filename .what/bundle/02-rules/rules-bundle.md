---
type: rules
scope: component
component: bundle
status: draft
created: "2026-08-22"
updated: "2026-08-22"
---

# Business Rules — bundle

Rules binding **only** this component. A rule that turns out to bind a second one is promoted to
`.what/business-rules.md`, never copied.

Ids continue the global sequence.

## Rules

| id | Rule | Binds | Source | Status |
| --- | --- | --- | --- | --- |
| BR-57 | A Bundle's name appears as the composed document's heading. | `bundle` | FR-10 · UC-9 | active |
| BR-58 | The Findings appear in the Bundle in the order the Reviewer selected them. | `bundle` | FR-10 · UC-9 | active |
| BR-59 | Composing does not remove the Findings it used from the Library. | `bundle` | FR-10 · UC-9 | active |
| BR-60 | A Bundle holds at least one Finding. A Bundle of zero is never created. | `bundle` | FR-10 | active |
| BR-61 | A Bundle may hold exactly one Finding, and nothing about it is special-cased. | `bundle` | FR-10 · OQ-16 | active |
| BR-62 | Every image the composed document references exists as that Bundle's own copy before the Bundle is listed. | `bundle` | BR-2 (AD-2) · FR-10 | active |
| BR-63 | A Bundle's images carry that Finding's Markers, drawn at the stored image's own dimensions. | `bundle` | FR-10 · AD-3 · AD-4 | active |
| BR-64 | Bundles are listed newest first. | `bundle` | FR-11 · UC-10 | active |
| BR-65 | Opening a Bundle shows what was composed, not a live view of the Findings as they are now. | `bundle` | BR-10 · FR-11 · UC-10 | active |
| BR-66 | A Bundle whose Markdown file is missing is shown as broken. It is never omitted from the list. | `bundle` | FR-11 | active |
| BR-67 | Copying a Bundle puts its Markdown on the clipboard exactly, with no added wrapper and the same relative image paths as the file. | `bundle` | FR-12 · UC-11 | active |
| BR-68 | The Reviewer is told when a copy succeeded. | `bundle` | FR-12 · UC-11 | active |
| BR-69 | Deleting a Bundle asks once and names the Bundle. | `bundle` | BR-6 · FR-14 · UC-12 | active |
| BR-70 | Deleting a Bundle does not delete the Findings it was composed from, unless the Reviewer chose that in the same confirmation. | `bundle` | FR-14 · UC-12 | active |
| BR-71 | A Bundle is never renamed. A different name means a different Bundle. | `bundle` | BR-11 · PRD §6.2 | active |
| BR-72 | The Findings in a Bundle are never reordered after composition. | `bundle` | BR-11 · PRD §6.2 | active |

## Retired

None yet.
