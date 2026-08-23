---
type: srs
component: bundle
status: draft
created: "2026-08-22"
updated: "2026-08-22"
satisfies: [FR-10, FR-11, FR-12, FR-14, NFR-8]
reviewed:
  date: '2026-08-23'
  sha: '4aabb93'
  lenses: [structure, prose, edge-case-hunter]
---

# SRS — bundle

## Decision Summary · [G3]

This component is the moment a review stops being a pile of Findings and becomes an artifact. The
Reviewer selects the Findings that belong to one concern, names the group, and this component writes
one Markdown document in which every Note sits under the image it describes, in the order they were
selected, with each Finding's Markers burned into its own copy of the image.

Two decisions carry the rest. A Bundle is a **snapshot**: editing a Finding afterwards changes nothing
in a Bundle that already holds it, because a Bundle that drifts from what was handed over is worse
than no Bundle. And a Bundle's Markdown is composed **once and stored**, so that the clipboard, the
Local API, and a published page all serve identical bytes — three handoff paths, one document.

It runs at the global `mode: outline` and `risk_accepted: low`. Deleting a Bundle removes files from
disk and cannot be undone, and a Bundle's images may hold anything that was on the Reviewer's screen.

## Why · [G3]

Because grouping is a decision, not a by-product. The Reviewer knows which five of their eleven
Findings are one concern; nothing else does. This component exists to capture that judgement and
freeze it into something citable — which is a different job from holding Findings, and would be a
different job even if both lived in one screen.

## Actor Register · [G3]

| Actor | Who they are | What they may do |
| --- | --- | --- |
| Reviewer | The person operating Snapdown. The only human actor and the only writer | Compose a Bundle from a selection, name it, list Bundles, open one, copy its Markdown, delete it with its files, and choose in the same act whether its source Findings go too |

An agent reads Bundles, but never through this component: `agent-access` and `sharing` are the surfaces
that expose them, and both are read-only per AD-5.

## UC Catalogue · [G3]

| id | Use case | Actor | Satisfies | critical |
| --- | --- | --- | --- | --- |
| UC-9 | I turn the findings I picked into one review document | Reviewer | FR-10 | no |
| UC-10 | I look back at the reviews I have already put together | Reviewer | FR-11 | no |
| UC-11 | I drop a whole review straight into the conversation I am having with my agent | Reviewer | FR-12 | no |
| UC-12 | I get rid of a review and everything in it | Reviewer | FR-14 | yes |

One of four is `critical`: UC-12 removes files from disk irreversibly, and it may also end a
Publication (BR-23), which is the one action in this component that reaches outside the machine.

## Constraints · [G3]

| Constraint | Source |
| --- | --- |
| A Bundle's Markdown is composed once by the core and stored; every handoff path serves those exact bytes | AD-9, BR-10, BR-11 |
| A record and its files are created and removed in one unit of work | AD-2, BR-5 |
| Composition refuses, naming the Finding, when a selected Finding's image file is missing | BR-13 |
| A Finding may belong to several Bundles, and each Bundle keeps its own image copy | BR-12 |
| Marker positions arrive normalised and are burned in at the image's stored dimensions; nothing is re-scaled | AD-3, AD-4, BR-8 |
| Deleting a published Bundle unpublishes it in the same action | BR-23 |
| The Markdown is CommonMark with relative image paths, in the shape `cross-cutting.md` defines | NFR-8, cross-cutting.md § Bundle Markdown shape |
| Findings, Notes, and Markers are read-only here — `finding` owns them | `components.yaml` → `owns` |
| The Vault location is read from `settings` | `components.yaml` → `owns` |

## Non-Goals · [G3]

- **Capturing, noting, or marking.** All three are `finding`.
- **Editing a composed Bundle.** BR-11. A change means composing a new one.
- **Renaming a Bundle.** Out of MVP scope: a rename that does not rewrite the document's heading is a
  lie, and rewriting it contradicts BR-10.
- **Reordering after composition.** Selection order is the ordering. A second mechanism would be a
  second source of truth.
- **Publishing.** `sharing` owns the Publication; this component only knows that deleting a published
  Bundle must end one.
- **Exposing a Bundle to an agent.** `agent-access` and `sharing`.
- **Exporting to anything but Markdown.**

## Prerequisite · [G3]

- `finding` must exist and hold at least one Finding with a present image file. CAP-4 declares
  `depends_on: [CAP-3]` for exactly this reason.
- A writable Vault location, supplied by `settings`.
- Nothing external.

## Success Signal · [G3]

Composing five selected Findings under a chosen name produces one Markdown file whose five Notes sit
under their own five images, in the selected order, with every image reference resolving and every
Marker badge matching its numbered line. Pasting the copied Markdown into a plain text editor yields
that file unchanged. Deleting the Bundle afterwards leaves neither its Markdown nor its images in the
Vault.

## Assumptions, Risks, and To Be Confirmed · [G3]

### Assumptions

- Recomposing is acceptable in place of editing a Bundle's written Markdown — OQ-12.
- A coding agent handed a Markdown file with relative image paths can open those images — OQ-1. If it
  cannot, FR-12 is worthless and `agent-access` becomes the only path.

### Risks

- **The composer becoming three composers.** AD-9 exists because the pressure to render a Bundle
  slightly differently per surface is constant and each instance looks reasonable. The golden-file
  test across the three paths is the only thing that catches it.
- **Burning Markers at the wrong scale.** AD-3 gives normalised coordinates and AD-4 forbids
  re-scaling, so the burn must happen at the stored image's own dimensions. Getting it wrong puts
  every badge slightly off, which is subtle enough to ship.
- **Partial deletion.** A Bundle holds a Markdown file and one image per BundleItem. BR-5 requires
  all-or-nothing across all of them, which is a harder guarantee than `finding`'s single file.
- **A published Bundle deleted while the service is unreachable.** BR-23 requires the unpublish, and
  BR-20 forbids reporting success. The honest outcome is refusing the deletion, and that has to be
  designed rather than discovered.

### To Be Confirmed

- Whether composing should offer to delete the Findings it consumed, mirroring FR-14's offer in the
  other direction. Left out of r1 deliberately; PRD open question 4.

## Gate Checklist · [G3]

| Question | Answer |
| --- | --- |
| ★ Is every use case title a sentence a user would say? | Yes, all four in the Reviewer's own voice |
| ★ Any `FR` with no use case? | No. FR-10, FR-11, FR-12, and FR-14 each have one |
| ★ Do the inventories and this catalogue describe one system? | Yes. Tables 4–5, screens 8–10, and no endpoint of its own — the surfaces that expose a Bundle belong to other components |
| Actor list: is one missing? | No. The agent is a reader of what this produces, not an actor here |
| Does every `AD-N` here name a concrete failure that crosses components? | AD-9 does, across three components. AD-2 and AD-3 are shared with `finding` |
| Which business rule am I not sure is right? | BR-23 — deleting a published Bundle unpublishes it. It is right, and it makes deletion depend on a network call, which is the least comfortable consequence in this document |
| Is there a term I have to guess the meaning of? | No |

## Design Reference · [G3]

Paired with `.how/bundle/SDD-bundle.md`.

Binding invariants: **AD-2** (a record and its files live or die together), **AD-3** (Marker
coordinates normalised), **AD-4** (no re-encoding after capture), **AD-9** (one Bundle, one Markdown,
every path). No applied `DEC-` binds this component yet.

---

## Slots

`02-rules/rules-bundle.md` — written at G4, `mode: outline`.
`03-domain/domain-model.md` — written at G3, present.
`04-usecases/` — at most three full flows at `outline`, written at G4.
`05-scenarios/` — not written below `mode: deep`.

## Open Items

- OQ-1 — whether a coding agent can open relative image paths. `.control/questions/assumptions.md`.
- OQ-12 — recomposing in place of editing. `.control/questions/assumptions.md`.
