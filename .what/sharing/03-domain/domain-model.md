---
type: model
component: sharing
layer: conceptual
created: "2026-08-22"
updated: "2026-08-22"
---

# Model — sharing

Conceptual. No column types, no storage shape.

## Entities

| Entity | What it is | Identified by |
| --- | --- | --- |
| Publication | The record that one Bundle was put somewhere readable from outside this machine: where it is served, when that happened, and whether it is still live | Its own opaque identifier |

One entity. What the web service holds is a **copy** of a Bundle's document and images, not a second
domain: it has a slug, some Markdown, and some image files, and it knows nothing about Findings,
Bundles, or the Library that produced them (AD-8).

## Relationships

- One Bundle has **zero or one** Publication. Publishing an already published Bundle replaces its
  content at the same slug rather than creating a second Publication (BR-21).
- One Publication belongs to **exactly one** Bundle, for its whole life.
- A Publication's slug is related to **nothing** — not the Bundle's identifier, not a Finding's, not
  any other slug. It is drawn independently from a cryptographically secure source (AD-8, NFR-10).
- A Publication refers to the Bundle's composed document and images, and serves copies of them. It
  never reaches back into `finding`: what the remote agent reads is the Bundle's snapshot, so deleting
  the source Findings changes nothing about a Publication.
- Deleting a Bundle ends its Publication in the same act (BR-23).

## State Lifecycle

| From | To | Trigger | Who may |
| --- | --- | --- | --- |
| — | live | The Reviewer confirms a publish on a named Bundle, and the upload completes | Reviewer |
| live | live | The Reviewer publishes the same Bundle again. Content is replaced at the same slug | Reviewer |
| live | ended | The Reviewer unpublishes, and the service confirms removal | Reviewer |
| live | live, with a recorded failure | The Reviewer unpublishes and the service cannot be reached or does not confirm | Reviewer |
| ended | — | Nothing. A slug is never re-served, for this Bundle or any other | — |

The fourth row is the one that matters most and is easiest to design away. An unpublish that did not
demonstrably succeed leaves the Publication **live**, carrying the failure, because the alternative is
telling the Reviewer something is private when it may still be served (BR-20). It is left by retrying
the unpublish, or by reconciling against the service — never by assuming.

A publish that fails never enters `live` at all: nothing readable is left on the service and the
Bundle stays unpublished (BR-19). There is no partial state.

`ended` is terminal. It does not mean the content was never read — an unpublish cannot recall a fetch,
and that is why publishing is `critical` in the use case catalogue.

## Invariants

1. A Bundle has at most one Publication, ever. → BR-21
2. A slug is unique across every Publication that has ever existed and is never reused, including
   after an unpublish. → BR-22, AD-8
3. A slug carries at least 128 bits of entropy from a cryptographically secure source, and is derived
   from no Library identifier. → NFR-10, AD-8
4. No Library identifier appears in a Publication's URL, in what the service serves, or in anything
   the service stores. → AD-8
5. A Publication comes into existence only through an act the Reviewer confirmed on a named Bundle.
   → BR-18, AD-6, NFR-11
6. A publish either makes the whole Bundle readable or leaves nothing readable. → BR-19
7. An unpublish that is not confirmed by the service leaves the Publication live with its failure
   recorded. → BR-20
8. What is served is the Bundle's exact composed bytes and its reduced images. Nothing is re-rendered
   or re-encoded on the way out. → AD-9, AD-4, NFR-12
9. An unknown slug, an ended one, and one that never existed are refused identically. → BR-24, NFR-15
10. Nothing the service exposes lists, searches, or enumerates Publications. → NFR-15
11. Nothing the service exposes writes to the Library. → BR-15, AD-5
