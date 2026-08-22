---
type: rules
scope: component
component: agent-access
status: draft
created: "2026-08-22"
updated: "2026-08-22"
---

# Business Rules — agent-access

Rules binding **only** this component. Ids continue the global sequence.

## Rules

| id | Rule | Binds | Source | Status |
| --- | --- | --- | --- | --- |
| BR-73 | The Access Key is shown once when it is issued, and can be re-copied for as long as it is valid. The Reviewer is never forced to reissue in order to recover it. | `agent-access` | FR-19 · UC-17 | active |
| BR-74 | Whether a key is currently valid, and when it was issued, is visible without issuing anything. | `agent-access` | FR-19 · UC-17 | active |
| BR-75 | Revoking when no key is valid is harmless and says so. | `agent-access` | FR-22 · UC-19 | active |
| BR-76 | Revoking changes, deletes, and unpublishes nothing. | `agent-access` | FR-22 · UC-19 | active |
| BR-77 | The refusal for a missing key and the refusal for a revoked key carry different codes. | `agent-access` | AD-7 · FR-20 | active |
| BR-78 | A request from anything other than this machine is refused, whatever key it carries. | `agent-access` | NFR-9 · FR-20 | active |
| BR-79 | The only route that works without a key reveals nothing about the Library — not a count, not a Vault path, not a version of its contents. | `agent-access` | NFR-9 · FR-20 | active |
| BR-80 | The bridge answers immediately when Snapdown is not running, saying so. It never hangs and never retries in a loop. | `agent-access` | FR-21 · UC-18 | active |
| BR-81 | The bridge keeps no copy of the Library and no key between runs. | `agent-access` | AD-5 · FR-21 | active |
| BR-82 | Listing Bundles returns each Bundle's name, its Finding count, and when it was composed — and nothing about the Findings themselves. | `agent-access` | FR-21 · BR-14 | active |
| BR-83 | The Markdown an agent reads is byte-identical to what *Copy Markdown* produces. | `agent-access` | AD-9 · FR-21 | active |
| BR-84 | An image filename that resolves outside its own Bundle's folder is refused. | `agent-access` | FR-20 | active |
| BR-85 | One missing image does not fail the Bundle read. The Markdown still serves and the missing file is named. | `agent-access` | FR-21 | active |

## Retired

None yet.
