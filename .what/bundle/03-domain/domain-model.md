---
type: model
component: bundle
layer: conceptual
created: "2026-08-22"
updated: "2026-08-22"
---

# Model — bundle

Conceptual. No column types, no storage shape.

## Entities

| Entity | What it is | Identified by |
| --- | --- | --- |
| Bundle | A named group of Findings, frozen once into one review document that can be handed to an agent. Not a view over the Findings — a snapshot of them | Its own opaque identifier |
| BundleItem | The membership of one Finding in one Bundle: where it sits in the reading order, and the image copy written for it with its Markers drawn in | Its Bundle and its Finding together |

## Relationships

- One Bundle is made of **one or more** BundleItems. A Bundle with none is not a Bundle and is never
  created.
- One BundleItem belongs to **exactly one** Bundle and refers to **exactly one** Finding.
- One Finding may appear in **zero, one, or several** Bundles, each with its own BundleItem and its
  own image copy (BR-12).
- One Bundle has **exactly one** composed Markdown document, and that document is the Bundle's
  content rather than a rendering of it (AD-9).
- One Bundle has **zero or one** Publication, owned by `sharing`. This component knows only that
  deleting a published Bundle must end one (BR-23).
- A BundleItem refers to a Finding but does not depend on it surviving. Deleting the Finding leaves the
  Bundle readable, because the image copy is the Bundle's own.

## State Lifecycle

A Bundle has no status of its own. It is composed, and after that it is either present or gone —
there is no draft, no editing state, and no revision, because BR-11 forbids editing and BR-10 makes it
a snapshot.

The one thing that looks like a state and is not: whether a Bundle is published. That belongs to its
Publication, owned by `sharing`, and asking the Bundle would be asking the wrong entity.

## Invariants

1. A Bundle holds at least one BundleItem. → FR-10
2. A Finding appears at most once in one Bundle. → BR-12
3. BundleItem positions within one Bundle are a contiguous reading order with no gaps and no ties.
   → FR-10
4. A Bundle's composed document is written once and never changed. A change means a new Bundle.
   → BR-10, BR-11, AD-9
5. Every image a Bundle's document references exists as that Bundle's own copy, at the same dimensions
   as the Finding's image it was copied from. → BR-13, AD-3, AD-4
6. Composition refuses, naming the Finding, if any selected Finding's image is missing. A Bundle with
   a broken image reference is never written. → BR-13
7. A Bundle is created together with its document and its image copies, and removed together with all
   of them. If any file cannot be removed, nothing is removed. → BR-5, AD-2
8. Editing a Finding, its Note, or its Markers changes nothing in a Bundle that already holds it.
   → BR-10
9. Deleting a Bundle that has a live Publication ends that Publication in the same act. If it cannot,
   the deletion does not happen. → BR-23, BR-20
