---
type: rules
scope: global
status: draft
created: "2026-08-22"
updated: "2026-08-22"
---

# Business Rules — Snapdown

Rules binding more than one Product Component. A rule that binds only one lives in that component's
`02-rules/rules-<pc>.md`, from `mode: outline` up.

Every rule here is checkable and states no mechanism. Where a rule exists because an `AD-N` forbids
the alternative, the `AD-N` is named as its source — the invariant is the reason, this is the
behaviour a reviewer checks.

## Rules

| id | Rule | Binds | Source | Status |
| --- | --- | --- | --- | --- |
| BR-1 | A Marker's number is the number of its line in the Note. There is never a Marker without a line, or a numbered line without a Marker. | `finding`, `bundle`, `agent-access`, `sharing` | AD-1 · FR-8 | active |
| BR-2 | Marker numbers run from 1 upward with no gaps. Removing one renumbers every Marker after it, and its line with it. | `finding`, `bundle` | FR-8 · UC-5 | active |
| BR-3 | A Marker's comment may be empty. Its numbered line still exists. | `finding`, `bundle` | FR-8 | active |
| BR-4 | A Note may be empty. A Finding with no words is still a Finding. | `finding`, `bundle` | FR-2 · FR-7 | active |
| BR-5 | Deleting a Finding, a Bundle, or a BundleItem deletes its files. If a file cannot be removed, nothing is removed and the Reviewer is told which file refused. | `finding`, `bundle` | AD-2 · FR-13 · FR-14 | active |
| BR-6 | Every destructive action is confirmed exactly once, and the confirmation states what will go and how many of them. | `finding`, `bundle`, `sharing` | FR-13 · FR-14 · FR-23 · UC-7 · UC-12 | active |
| BR-7 | Nothing is soft-deleted. There is no bin, no archive, and no state in which a deleted thing is still readable. | `finding`, `bundle`, `sharing` | BG-5 · AD-2 | active |
| BR-8 | An image is reduced once, when it is captured. No later step re-encodes or re-scales it. | `finding`, `bundle`, `sharing` | AD-4 · FR-4 | active |
| BR-9 | A change to the Quality Budget applies only to Captures taken after it. No stored image is ever re-encoded. | `finding`, `settings` | FR-5 · UC-13 | active |
| BR-10 | A Bundle is a snapshot. Editing a Finding, its Note, or its Markers after composition changes nothing in a Bundle that already holds it. | `bundle`, `finding`, `agent-access`, `sharing` | AD-9 · FR-10 | active |
| BR-11 | A Bundle is never edited in place. A change means composing a new Bundle. | `bundle`, `agent-access`, `sharing` | AD-9 · OQ-12 | active |
| BR-12 | One Finding may belong to several Bundles. Each Bundle keeps its own image copy. | `bundle`, `finding` | FR-10 · FR-13 | active |
| BR-13 | Composition refuses, naming the Finding, if any selected Finding's image file is missing. It never writes a Bundle with a broken image reference. | `bundle`, `finding` | AD-2 · FR-10 · UC-9 | active |
| BR-14 | Only a Bundle is ever readable by an agent. An unbundled Finding is invisible on every agent-facing surface. | `agent-access`, `sharing`, `bundle` | FR-20 · FR-24 | active |
| BR-15 | Every agent-facing surface is read-only. None of them creates, changes, or deletes anything. | `agent-access`, `sharing` | AD-5 | active |
| BR-16 | Exactly one Access Key is valid at a time. Issuing a new one revokes the previous one immediately. | `agent-access` | FR-19 · FR-22 | active |
| BR-17 | A refusal is always distinguishable from an empty result. "No Access Key" and "no Bundles" are never the same answer. | `agent-access`, `sharing` | AD-7 · FR-20 | active |
| BR-18 | Nothing leaves the machine unless the Reviewer confirmed a publish on a named Bundle. | all | AD-6 · NFR-11 | active |
| BR-19 | A publish that fails leaves nothing readable on the service, and leaves the Bundle unpublished locally. | `sharing` | FR-23 | active |
| BR-20 | An unpublish that fails leaves the Bundle marked published. The Reviewer is never told something is private when it may not be. | `sharing` | FR-25 · FR-26 | active |
| BR-21 | Publishing a Bundle that is already published replaces its content at the same URL. A second URL is never created for one Bundle. | `sharing` | FR-23 · AD-8 | active |
| BR-22 | A Publication slug is never reused for a different Bundle, including after an unpublish. | `sharing` | AD-8 · FR-25 | active |
| BR-23 | Deleting a published Bundle unpublishes it as part of the same action. | `bundle`, `sharing` | FR-14 · FR-25 | active |
| BR-24 | An unknown slug, a revoked slug, and a slug that never existed are refused identically. | `sharing` | NFR-15 | active |
| BR-25 | Only reduced images are ever transmitted. An unreduced capture never leaves the machine, because none is kept. | `sharing`, `finding` | AD-4 · NFR-12 | active |
| BR-26 | A Snapdown action bound to a hotkey that is unavailable is reported at the moment of binding, and again at startup if registration fails. It is never left silently broken. | `settings`, `finding` | FR-17 · NFR-7 · UC-15 | active |
| BR-27 | No two Snapdown actions share one hotkey combination. | `settings` | FR-17 | active |
| BR-28 | Capture works before anything is configured. A default Vault location is used until the Reviewer chooses one. | `settings`, `finding` | FR-16 · UC-14 | active |
| BR-29 | Changing the Vault location either moves every existing file or moves none. | `settings`, `finding`, `bundle` | AD-2 · FR-16 | active |
| BR-30 | Timestamps are UTC everywhere they are stored or transmitted. Local time exists only in what a person is shown. | all | cross-cutting.md § Timestamps | active |

## Retired

None yet. A retired rule keeps its id, states what replaced it, and is never deleted — documents
still cite it.
