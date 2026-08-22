---
type: model
component: finding
layer: conceptual
created: "2026-08-22"
updated: "2026-08-22"
---

# Model — finding

Conceptual. No column types, no storage shape. `.how/finding/` owns those.

## Entities

| Entity | What it is | Identified by |
| --- | --- | --- |
| Finding | One observation the Reviewer made: a captured region of the screen, together with what they said about it and where on it they pointed. The atomic unit of the product | Its own opaque identifier |
| Note | The Reviewer's prose about one Finding: a body, plus one numbered line for each Marker | The Finding it belongs to |
| Marker | A numbered badge the Reviewer placed on a Finding's image, and the numbered line in the Note that is the same thing as it | Its Finding and its number within that Finding |

## Relationships

- One Finding has **exactly one** Note. The Note may be empty; it is never absent.
- One Finding has **zero or more** Markers.
- One Marker belongs to **exactly one** Finding, and no Marker exists without one.
- One Marker **is** one numbered line of its Finding's Note. This is an identity, not an association:
  there is no Marker without a line and no numbered line without a Marker (AD-1, BR-1).
- One Finding has **exactly one** image, held in the Vault. The Finding exists only while its image
  does (AD-2).
- A Finding may be referenced by Bundles owned by `bundle`. This component does not know about that,
  and a Bundle's existence never changes a Finding.

## State Lifecycle

A Finding has no status. It exists or it does not, and there is no intermediate state the Reviewer can
observe — which is what makes AD-2 stateable as an invariant rather than a lifecycle.

The one transient condition worth naming is not a state of the Finding: during a Capture, the record
is committed a moment before its reduced image has finished being written, so that the save can return
focus inside NFR-2's 500 ms while NFR-3's encoding continues. A Finding in that condition is complete
as far as the domain is concerned; the Editor renders it as still arriving.

Cut for Note and Marker: neither changes status.

## Invariants

1. A Finding has exactly one Note, and exactly one image in the Vault. → BR-4, AD-2
2. A Marker's number is the number of its line in the Note. There is no Marker without a line and no
   numbered line without a Marker. → BR-1, AD-1
3. Marker numbers within one Finding run from 1 upward with no gaps. Removing one renumbers those
   after it, and their lines with them, as one operation. → BR-2
4. A Marker's position is a fraction of its image's width and height, in the closed range 0 to 1.
   → BR-2, AD-3
5. A Marker's comment may be empty. Its numbered line still exists. → BR-3
6. A Note's body may be empty. → BR-4
7. A Finding is created together with its image file, and removed together with it. If the file
   cannot be removed, nothing is removed. → BR-5, AD-2
8. A Finding's image is the reduced image. No unreduced version of it exists anywhere. → BR-8, AD-4
9. A Finding's image is never re-encoded or re-scaled after it is stored, including when the Quality
   Budget changes. → BR-9
