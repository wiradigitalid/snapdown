---
type: rules
scope: component
component: sharing
status: draft
created: "2026-08-22"
updated: "2026-08-22"
---

# Business Rules — sharing

Rules binding **only** this component. Ids continue the global sequence.

## Rules

| id | Rule | Binds | Source | Status |
| --- | --- | --- | --- | --- |
| BR-86 | The publish confirmation names the Bundle and says that publishing cannot be recalled once the review has been fetched. | `sharing` | FR-23 · UC-20 | active |
| BR-87 | Publishing is refused, naming what is missing, when no web service address or no publish credential is configured. | `sharing` | FR-23 · UC-20 | active |
| BR-88 | Publishing is refused, naming the file, when any of the Bundle's images is missing from the Vault. | `sharing` | FR-23 · BR-13 | active |
| BR-89 | A Publication's content becomes reachable only after every file has landed. A partially uploaded Publication never resolves. | `sharing` | FR-23 · BR-19 | active |
| BR-90 | A slug is generated once, on a Bundle's first publish, and reused for every republish of that Bundle. | `sharing` | BR-21 · AD-8 · FR-23 | active |
| BR-91 | A published Bundle is retrievable as raw Markdown at an explicit path, without content negotiation. | `sharing` | FR-24 · UC-21 | active |
| BR-92 | Image references in a published document resolve relative to that document's own URL. | `sharing` | FR-24 · NFR-8 | active |
| BR-93 | A browser asking for HTML at a Publication URL gets a rendering of the same Markdown, never different content. | `sharing` | FR-24 · AD-9 | active |
| BR-94 | Unpublishing removes the Markdown and the images from the service, not only the mapping. | `sharing` | FR-25 · UC-22 | active |
| BR-95 | Unpublishing a Bundle that is not published is harmless and says so. | `sharing` | FR-25 · UC-22 | active |
| BR-96 | An unpublish is confirmed against the service before the Bundle is shown as unpublished. | `sharing` | BR-20 · FR-25 | active |
| BR-97 | A Publication's recorded failure is cleared only by a confirmed outcome — a successful unpublish, or a successful republish. | `sharing` | BR-20 · FR-26 | active |
| BR-98 | Publication state is visible in the Bundle list without opening the Bundle, together with when it was published. | `sharing` | FR-26 · UC-23 | active |
| BR-99 | Publication state shown without a confirmation from the service is labelled as last known, not as current. | `sharing` | FR-26 · UC-23 | active |
| BR-100 | The Reviewer is told when a Publication URL was copied. | `sharing` | FR-26 · UC-23 | active |
| BR-101 | Nothing the service stores or serves carries a Library identifier. | `sharing` | AD-8 | active |
| BR-102 | The service keeps no access log pairing a slug with a client address beyond what it needs to operate. | `sharing` | NFR-15 · cross-cutting.md § Logging | active |

## Retired

None yet.
