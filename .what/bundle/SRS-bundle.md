---
type: srs
component: bundle
status: draft
created: "2026-08-22"
updated: "2026-08-31"
satisfies: [FR-10, FR-11, FR-12, FR-14, FR-39, FR-40, NFR-8, NFR-19]
reviewed:
  date: '2026-08-23'
  sha: '783a561'
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
than no Bundle. And a Bundle's Markdown is composed **once, by the core, and stored**, so that every
handoff path serves the same authored document rather than each surface authoring its own.

It runs at `mode: deep` and `risk_accepted: low`. Deleting a Bundle removes files from disk and cannot
be undone, and a Bundle's images may hold anything that was on the Reviewer's screen. Since 2026-08-31
a Bundle's own title and notes can also be corrected after composition (`FR-40`), and the captures
behind it can be destroyed while it survives (`FR-41`, which belongs to `finding`).

**Corrected 2026-08-31, two claims.**

The second decision used to end *"so that the clipboard, the Local API, and a published page all serve
identical bytes — three handoff paths, one document."* Both halves of that were wrong.

*Identity is not what is promised.* `DEC-012` settles it: "`AD-9` guarantees that every handoff path
serves the **same authored document**: a path MAY substitute the base of a Bundle's image links so that
they resolve for its own reader, and MUST NOT change anything else — no re-ordering, no decoration, no
summarising, no editing of a single character the composer wrote. The substitution is made **by the
composer**, taking the base path as a parameter; a surface MUST NOT rewrite a document the composer has
already produced." `AD-9`'s own **Prevents** is what settled it: the harm named there is two readers
disagreeing about the same review, and a link that resolves for its own reader does not cause it.

*Two of the three named paths do not exist, and did not when this was written.* Verified in the code on
2026-08-31: the clipboard-Markdown path has no implementation at all — every clipboard call in the tree
is a bitmap write serving `FR-36`, and the desktop UI has no copy-markdown callback — and the Local API
does not exist, which is `BUG-59`'s own title. Only the published page runs. This document therefore
described a three-surface architecture for nine days while one surface existed, and the correction is
recorded rather than quietly made because the claim was load-bearing: it is cited in § Risks as the
reason a golden-file test is sufficient.

The mode sentence also used to read *"It runs at the global `mode: outline`"*. `components.yaml` raised
this component to an explicit `deep` on 2026-08-23 and the sentence was never updated.

## Why · [G3]

Because grouping is a decision, not a by-product. The Reviewer knows which five of their eleven
Findings are one concern; nothing else does. This component exists to capture that judgement and
freeze it into something citable — which is a different job from holding Findings, and would be a
different job even if both lived in one screen.

## Actor Register · [G3]

| Actor | Who they are | What they may do |
| --- | --- | --- |
| Reviewer | The person operating Snapdown. The only human actor and the only writer | Compose a Bundle from a selection, name it, list Bundles, open one, copy its Markdown, correct its title and its notes, export it as a PDF, and delete it with its files |

An agent reads Bundles, but never through this component: `agent-access` and `sharing` are the surfaces
that expose them, and both are read-only per AD-5.

**Amended 2026-08-31.** The Reviewer's rights used to end *"delete it with its files, and choose in the
same act whether its source Findings go too"*. That last clause is gone: `FR-14` no longer offers a
combined destroy, and destroying a Bundle's source Findings is now `FR-41`, whose use case belongs to
`finding` rather than here — the Reviewer reaches it from a Bundle, but what it destroys is a capture.
Correcting a title and exporting a PDF are the two rights added.

## UC Catalogue · [G3]

| id | Use case | Actor | Satisfies | critical |
| --- | --- | --- | --- | --- |
| UC-9 | I turn the findings I picked into one review document | Reviewer | FR-10 | no |
| UC-10 | I look back at the reviews I have already put together | Reviewer | FR-11 | no |
| UC-11 | I drop a whole review straight into the conversation I am having with my agent | Reviewer | FR-12 | no |
| UC-12 | I get rid of a review and everything in it | Reviewer | FR-14 | yes |
| UC-28 | I turn a review into something I can send to a person who does not have Snapdown | Reviewer | FR-39 | no |
| UC-29 | I fix a typo in a review I have already put together | Reviewer | FR-40 | no |

One of six is `critical`: UC-12 removes files from disk irreversibly, and it may also end a
Publication (BR-23), which is the one action in this component that reaches outside the machine.

UC-28 and UC-29 arrived on 2026-08-31 and neither is `critical`, which is worth stating because both
read like candidates. UC-28 writes a file, but it destroys nothing and sends nothing anywhere; its
closest sibling is UC-11, the clipboard handoff, which is also `no`. UC-29 overwrites stored text with
no revision history — irreversible in the same weak sense as UC-4, rewording a Note, which is `no`
too. Marking either one `yes` would put half this component's use cases under the label, and
`delivery-flow-guide.md` says a count past a third is a signal to derive again rather than a finding.

**Destroying a Bundle's source Findings is not in this catalogue.** It is `UC-30`, under `finding`.
The Reviewer reaches it from a Bundle, and the temptation is to file it here; the object destroyed is
a capture, and `bundle` has no authority to write one.

## Constraints · [G3]

| Constraint | Source |
| --- | --- |
| A Bundle's Markdown is composed once by the core and stored. Every handoff path serves that same authored document; a path may substitute the base of an image link so it resolves for its own reader, and may change nothing else | AD-9, DEC-012 |
| A Bundle's stored document is changed only by the composer writing it again over the Bundle's own copy. No surface edits it directly, and no change to a Bundle reads or writes a Finding | BR-11, DEC-012 |
| Editing a Finding, its Note, or its Markers changes nothing in a Bundle that already holds it | BR-10 |
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
- **Changing which Findings a Bundle holds.** The set is fixed at composition. Nothing here adds,
  removes, reorders or replaces one, and a different selection means a new Bundle. This is the part of
  the old "editing" prohibition that survives, and it is the part that was doing the work.
- **Reordering after composition.** Selection order is the ordering. A second mechanism would be a
  second source of truth.
- **Destroying the Findings a Bundle was composed from.** `FR-41` promises it, and it belongs to
  `finding` — `bundle` has no authority to write a Finding. This component only knows that a Bundle
  whose Findings are gone can no longer give them back (`BR-122`).
- **Publishing.** `sharing` owns the Publication; this component only knows that deleting a published
  Bundle must end one.
- **Exposing a Bundle to an agent.** `agent-access` and `sharing`.

**Three entries left this list on 2026-08-31**, each superseded by a promise that now exists. What
they said is recorded rather than deleted, because each was cited as a reason:

- *"**Editing a composed Bundle.** BR-11. A change means composing a new one."* Withdrawn by `FR-40`
  and by `BR-11`'s narrowing. What replaced it is narrower and is the second bullet above: the
  Finding **set** is fixed, but the Bundle's own title and notes are not.
- *"**Renaming a Bundle.** Out of MVP scope: a rename that does not rewrite the document's heading is
  a lie, and rewriting it contradicts BR-10."* Withdrawn by `FR-40`. Two things are worth separating
  here. The **objection** never applied to the design that replaced it: `FR-40` rewrites the heading
  when the title changes, so the rename it promises is not the lie this line guarded against — only
  the scope boundary ever bit. And the clause *"rewriting it contradicts BR-10"* was **wrong on its
  own terms** from the day it was written. `BR-10` says editing a *Finding* changes nothing in a
  Bundle that already holds it; it says nothing about a Bundle rewriting its own heading, and never
  did. It stood here since G3 as a reason nobody checked.
- *"**Exporting to anything but Markdown.**"* Withdrawn by `CAP-12` and `FR-39`. It was an MVP
  boundary, and it held for exactly as long as every reader of a Bundle was a machine.

## Prerequisite · [G3]

- `finding` must exist and hold at least one Finding with a present image file. CAP-4 declares
  `depends_on: [CAP-3]` for exactly this reason.
- A writable Vault location, supplied by `settings`.
- Nothing external.

## Success Signal · [G3]

Composing five selected Findings under a chosen name produces one Markdown file whose five Notes sit
under their own five images, in the selected order, with every image reference resolving and every
Marker badge matching its numbered line. Pasting the copied Markdown into a plain text editor yields
that same document, with every image link naming a location its reader can open. Correcting the
Bundle's title afterwards rewrites the document's heading and leaves its images and its source
Findings untouched. Deleting the Bundle leaves neither its Markdown nor its images in the Vault.

**Corrected 2026-08-31.** The third sentence used to read *"Pasting the copied Markdown into a plain
text editor yields that file unchanged."* Under `DEC-012` the clipboard may render image links against
a different base, so *unchanged* is no longer the signal — *the same document* is.

## Assumptions, Risks, and To Be Confirmed · [G3]

### Assumptions

- ~~Recomposing is acceptable in place of editing a Bundle's written Markdown — OQ-12.~~
  **Withdrawn 2026-08-31.** This assumption turned out false and `FR-40` is what replaced it. `OQ-12`
  is owed closure in place by `wdi-question`; it is struck through here rather than deleted, because
  this line is where the assumption was relied on.
- A coding agent handed a Markdown file with relative image paths can open those images — OQ-1. Still
  open, and **still untested**, because `FR-12` has no implementation: there is no text clipboard call
  anywhere in the tree. `FR-12`'s amendment of 2026-08-31 acts on the risk this assumption names
  rather than resolving the assumption — an absolute link removes the need for the agent to guess a
  base, but nothing has been run to confirm either way.

### Risks

- **The composer becoming three composers.** AD-9 exists because the pressure to render a Bundle
  slightly differently per surface is constant and each instance looks reasonable. `DEC-012` makes
  this risk sharper rather than smaller: a base-path substitution is now permitted, so the line
  between "the same document rendered for its reader" and "a second rendering" has to be held by
  judgement where it used to be held by a byte comparison. The guard is that the **composer** owns the
  substitution; a surface rewriting a finished document is still forbidden.

  **Corrected 2026-08-31.** This risk used to end *"The golden-file test across the three paths is the
  only thing that catches it."* That was not true when it was written. There is one golden-file test —
  `crates/snapdown-store/tests/test_golden_markdown.rs` — it pins the **stored** form, and it covers
  one path rather than three, because two of the three paths have no code. The absolute rendering has
  no guard at all yet, and writing one is owed with the implementation. A test asserting that a link
  *starts with* a drive letter is not that guard: this repository already spent five waves passing a
  fabricated image because the assertion checked a signature instead of reading the output back.
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
