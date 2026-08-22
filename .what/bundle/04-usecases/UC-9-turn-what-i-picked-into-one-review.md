---
type: uc
id: UC-9
component: bundle
satisfies: [FR-10]
critical: false
created: "2026-08-22"
---

# UC-9 — I turn the findings I picked into one review document

## Trigger

The Reviewer has a selection of Findings in the Editor and chooses to compose a Bundle from it.

## Precondition

At least one Finding is selected. Every selected Finding's image file is present in the Vault. The
Vault is writable.

## Main Flow

1. The Reviewer selects the Findings that belong to one concern.
2. The Reviewer chooses to compose a Bundle.
3. Snapdown asks for a name.
4. The Reviewer names it.
5. Snapdown writes, as one act: a copy of each Finding's image with that Finding's Markers drawn in,
   one Markdown document in which each Note sits under its own image in the selected order, and the
   Bundle itself.
6. The new Bundle appears at the top of the Bundle list, with its name and its Finding count.
7. The Findings remain in the Library, unchanged and still selected-independent.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| 1 | Exactly one Finding is selected | An ordinary Bundle of one is composed. Nothing is special-cased, and this is how a single screenshot is later published (BR-61) |
| 1 | A selected Finding is already in another Bundle | Composed again, with its own image copy for this Bundle. One Finding may belong to several (BR-12) |
| 1 | A selected Finding has no Markers | Its image is copied with nothing drawn on it, and its Note has no numbered lines |
| 1 | A selected Finding has an empty Note | Its section heading falls back to "Finding N" rather than being blank (`cross-cutting.md` § Bundle Markdown shape) |
| 4 | The Reviewer gives a name a Bundle already has | Allowed. A name is a label, not an identifier; two Bundles may share one |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| 5 | A selected Finding's image file is missing | Refuses the whole composition and names the Finding. No Bundle with a broken image reference is ever written (BR-13) | The selection intact, and a message naming which Finding is broken. The orphan report can then resolve it |
| 5 | Burning a Marker into an image fails | Abandons the composition. Nothing is written — no rows, no files (BR-5) | The selection intact, and a message saying the Bundle could not be composed |
| 5 | The Vault runs out of space partway through the image copies | Abandons the composition and removes any copies already written | The selection intact, the Vault as it was, and a message naming the space |
| 5 | The store write fails after the files are written | Removes the files it wrote, then reports the failure | The Vault as it was. Nothing half-composed appears in the list |
| 3 | The Vault is not writable | Refuses before asking for a name | A message naming the Vault path, with an action that opens Settings |

## Outcome

One Bundle exists, holding its own copy of each selected Finding's image with the Markers drawn in,
and one Markdown document where every Note sits under the image it describes. It is a snapshot:
editing those Findings afterwards changes nothing in it. The Findings are all still in the Library.

## Business Rules

BR-5, BR-10, BR-11, BR-12, BR-13, BR-57, BR-58, BR-59, BR-60, BR-61, BR-62, BR-63, BR-64, and NFR-8
as the property the composed document has to have.
