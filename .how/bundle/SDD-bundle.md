---
type: sdd
component: bundle
status: draft
created: "2026-08-22"
updated: "2026-08-23"
realizes: [UC-9, UC-10, UC-11, UC-12]
binds: [AD-1, AD-2, AD-9, AD-10]
reviewed:
  date: '2026-08-23'
  sha: '783a561'
  lenses: [structure, prose, edge-case-hunter]
---

# SDD — bundle

`mode: outline`. `Inherited Constraints` and `Failure Behaviour` are `[guarded]` sections and are
deliberately absent — the four `AD-N` in `binds` still hold, because an invariant does not stop
holding because a document is thin.

## Decision Summary · [outline]

This component is a composer and a store, and almost nothing else. Composition takes an ordered list
of Finding ids and a name, and produces three things in one unit of work: a `bundle` row holding the
composed Markdown, one `bundle_item` row per Finding, and one image file per Finding with that
Finding's Markers drawn into it.

Two choices cost the most to reverse.

**The composed Markdown is a column, not a rendering.** `bundle.markdown` holds the finished bytes,
and every handoff path — clipboard, Local API, publish — reads that column. The alternative, rendering
on demand from the Findings, is cheaper to write and makes AD-9 unenforceable: the moment a Note is
edited, the same Bundle produces different bytes on different days, and two agents reading "the same
review" disagree. Storing it is what makes a Bundle citable.

**Markers are burned at compose time, into a copy.** The Finding's own image stays clean, and each
`bundle_item` gets its own file with the badges drawn in at that image's stored dimensions. This is
what makes FR-8's repositioning possible at all — a badge already burned into the Finding's image
could never be moved — and it is why deleting a Bundle has several files to remove rather than one,
and why one Finding in three Bundles costs three image copies. That storage cost is accepted: it is
what lets a Bundle outlive the Finding it came from.

A Markdown file is also written next to the images, at `bundle.markdown_path`, holding the same bytes
as the column. The column is the source of truth; the file exists so that a Bundle is readable by
anything that opens a folder, which is the plainest form of FR-12.

## Structure · [outline]

Five Logical Components, all in `desktop-app`. Registered in `.control/registry/components.yaml`.

| LC | type | Responsibility |
| --- | --- | --- |
| LC-010 `bundle-composer` | service | The whole composition, as one transaction: read the Findings, burn the images, render the Markdown, write the rows and files. Refuses if any selected Finding's image is missing |
| LC-011 `markdown-writer` | service | Turns Findings plus their Notes and Markers into the document shape `cross-cutting.md` § Bundle Markdown shape defines. Pure, and the only place that shape exists |
| LC-012 `marker-burner` | service | Draws numbered badges into a copy of a Finding's image, at that image's own dimensions. Converts normalised coordinates to pixels here and nowhere else |
| LC-013 `bundle-store` | store | `bundle` and `bundle_item` rows, and the Bundle's own files through `vault-blobs`. Owns the delete-with-files transaction |
| LC-014 `bundles-editor` | ui-screen | The Bundle list, the Bundle detail view, compose, copy Markdown, delete |

```mermaid
graph TD
    LC014["LC-014 bundles-editor"] --> LC010["LC-010 bundle-composer"]
    LC014 --> LC013["LC-013 bundle-store"]
    LC010 --> LC011["LC-011 markdown-writer"]
    LC010 --> LC012["LC-012 marker-burner"]
    LC010 --> LC013
    LC010 --> LC004(["LC-004 finding-store<br/>finding, read-only"])
    LC012 --> LC005(["LC-005 vault-blobs<br/>finding"])
    LC013 --> LC005
    LC013 -.->|"ends a Publication<br/>on delete, BR-23"| LC020(["LC-020 publish-client<br/>sharing"])
```

`LC-011` is pure: no I/O, no clock, no filesystem. That is what makes AD-9's golden-file test possible
across all three handoff paths, and it is the one structural property in this component worth
defending.

Crossings out of this component: reads of `LC-004 finding-store` and `LC-005 vault-blobs`, both
read-only, and one call into `LC-020 publish-client` when a published Bundle is deleted. Nothing in
`finding` depends on anything here.

## Inherited Constraints · [guarded]

New at this gate: `bundle` was raised from an inherited `outline` to `deep` on 2026-08-23, so this
section had never been written. Quoted verbatim from `.how/_platform/ARCHITECTURE-SPINE.md`.

**AD-2 — A record and its files live or die together**
> Any operation that creates or removes a Finding, a Bundle, or a BundleItem MUST create or remove
> that record's files in the same unit of work, and MUST leave the prior state intact if any part of
> it fails. A record MUST NOT be committed before its files exist, and files MUST NOT be removed
> before the record is.

Reaches this component twice: composition writes image copies and the Markdown before the Bundle row
(`FR-10`, all-or-nothing), and deletion removes them with it (`FR-14`).

**AD-9 — One Bundle, one Markdown, byte-identical on every path**

The Markdown is composed once and **stored**, not regenerated. `bundle.markdown` is a column. That is
what makes clipboard, Local API, and published bytes identical without three code paths agreeing, and
it is why `BUG-1` damages the item list without damaging the document.

**AD-1 — Markers and Note lines are one sequence, not two**

Reaches this component as a **reader**. Composition renders the pairing into Markdown and burns the
badges into the image copies. A Finding whose sequence is ragged (`SCN-04`) composes as it is; this
component does not tidy another component's collection.

**AD-10 — Colour has exactly one authority, and every colour exists in both themes**

`[MISSING]` — `BundleView.tsx` carries `#f8fafc` (line 93), `#e0f2fe` / `#ffffff` (114), `#ffffff`
(137), `#f1f5f9` (173). All light-theme values on a surface that renders under either theme. This is
the panel the Reviewer saw as white-on-white.

## Failure Behaviour · [guarded]

Never written before. The boundary list is this component's rows in `inventory-screen.md` — 8, 9, 10 —
plus the two stores. `bundle` owns no endpoint in `inventory-api.md`; `agent-access` serves Bundles
over the Local API and owns those rows.

| Boundary | Other side is slow | Other side is absent | Other side is lying |
|---|---|---|---|
| **`LC-013` → `library.db`** | The surface renders its three regions and holds their shape. No assumed values | Reported with the file path; nothing is created over it | A row whose `markdown_path` names a file that is gone: the item is flagged, the Bundle still opens, Delete still works |
| **`LC-010` → `LC-005 vault-blobs`** | Composition shows progress and cannot be dismissed | Composition **refuses**, naming the Finding whose image is missing (`BR-13`). It never writes a Bundle with a broken reference | A write that reports success and produced no file is caught by the same all-or-nothing check that rolls composition back |
| **`LC-014` → the clipboard** | Not applicable; the write is synchronous | The failure is **reported**. A silent clipboard failure loses the primary handoff and the Reviewer would not know (`FR-12`) | A clipboard that accepts the write and holds something else is undetectable, and the product does not claim to detect it |
| **`LC-010` → `finding`** | Not applicable; in-process | A Finding deleted mid-composition fails the composition, all-or-nothing | **`BUG-1`.** A Finding deleted *after* composition cascades away the `bundle_item` row. The Bundle reports nothing and its item list is silently short |
| **`LC-022` → `sharing`** | The publish dialog shows its own progress | Frozen by `DEC-005`; the surface shows current state and gains no behaviour | Out of scope this release |

## ABCE · [deep]

### Boundary

| Object | What crosses it |
|---|---|
| `BundlesScreen` | Selection and the three actions |
| `ComposeDialog` | A name and a confirmed selection |
| `BundleStore` | Every Bundle and BundleItem read and write |
| `Clipboard` | The Markdown, out. The primary handoff (`FR-12`) |

### Control

| Object | Decides |
|---|---|
| `BundleComposer` | Order, naming, all-or-nothing across images, Markdown, and rows |
| `MarkdownWriter` | The exact bytes. Pure — no I/O, which is what makes `AD-9` testable by golden file |
| `MarkerBurner` | Badges onto the image copies |
| `BundleRemover` | Rows, image copies, and the Markdown file together (`AD-2`), plus the unpublish cascade (`BR-23`) |

`MarkdownWriter` being pure is the load-bearing choice: `AD-9` says the bytes are identical on every
path, and a pure function called once, whose output is stored, makes that true by construction rather
than by three code paths being kept in step.

### Entity

`Bundle` and `BundleItem`. A `BundleItem` is a **membership** — a position and the image copy written
for it — not a pointer. That distinction is exactly what `BUG-1` violates.

### Behaviour

Composition is the only operation whose cost scales with the selection, and it is the only one with a
progress state. Everything else is a read or a delete. Nothing here is scheduled or background.

## Evidence labels outstanding · [deep]

| Label | Claim | Disposition |
|---|---|---|
| `[MISSING]` | `bundle_item.finding_id` cascades on Finding deletion, contradicting `FR-13` | **`BUG-1`** — a defect, not planned work. The requirement predates the schema |
| `[MISSING]` | `BundleView.tsx` carries four light-theme literals on a surface rendered under either theme | Planned work — `AD-10`, `NFR-16`, `NFR-17` |
| `[MISSING]` | The Markdown preview is not distinguishable from a disabled input to a screen reader | Planned work — `NFR-16` |
| `[PARTIAL]` | Composition is all-or-nothing in code; whether a failure partway leaves image copies in the Vault was not verified | `wdi-question`, before G4 opens |

## Design Notes

- **`bundle.markdown` and the file at `markdown_path` are written from the same bytes in the same
  transaction.** If they can ever differ, the column wins and the file is a stale copy — but nothing
  is designed to let them differ, and a check that they match belongs in the deletion path rather
  than on every read.
- **`bundle_item.position` is dense and contiguous.** It is the selection order, and there is no
  reorder operation, so it is written once and never rewritten. Anything that makes it sparse has
  invented the second ordering mechanism the SRS rules out.
- **A Bundle of one Finding is an ordinary Bundle.** It is also how a single screenshot gets published
  (OQ-16). Nothing special-cases it.
- **Deleting a published Bundle calls `sharing` before removing anything.** If the unpublish is not
  confirmed, the deletion does not happen — BR-20 and BR-23 together, and the least comfortable
  consequence in this component, since it makes a local deletion depend on a network call.
- **The composer does not delete the Findings it consumed.** PRD open question 4 asks whether it
  should offer to; it does not in r1, and adding it later touches only `LC-014`.

---

## Slots

`01-ux/` — not written below `mode: deep`. Screens are rows 8–10 of `inventory-screen.md`.
`02-contracts/` — not written. This component owns no endpoint.
`03-integrations/` — `[guarded]`, and not applicable: no third party.
`04-components/`, `05-model/`, `06-flows/` — `[deep]` only.

## Open Items

- OQ-1 — whether a coding agent can open the relative image paths this component writes. It decides
  whether FR-12 is worth anything. `.control/questions/assumptions.md`.
- OQ-12 — recomposing in place of editing. `.control/questions/assumptions.md`.
- OQ-16 — whether a one-Finding Bundle is acceptable as the single-screenshot publish path.
  `.control/questions/assumptions.md`.
- RISK-5 — the composer becoming three composers. `.control/registry/risks.yaml`.
