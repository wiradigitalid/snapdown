---
type: uc
id: UC-5
component: finding
satisfies: [FR-8]
critical: false
created: "2026-08-22"
---

# UC-5 — I point at three separate spots inside one screenshot

## Trigger

The Reviewer, looking at one Finding in the Editor, wants to point at particular places inside its
image rather than describe where they are in words.

## Precondition

The Finding exists and its image file is present. The Editor is open with that Finding selected.

## Main Flow

1. The Reviewer chooses to add a Marker and clicks the first spot on the image.
2. Snapdown places badge `1` there and creates line `1.` in that Finding's Note.
3. The Reviewer types the sub-comment for line `1`.
4. The Reviewer repeats steps 1 to 3 for the second and third spots, getting badges `2` and `3` and
   lines `2.` and `3.`.
5. The Reviewer drags badge `2` so it sits beside the control rather than on top of it.
6. Snapdown keeps badge `2` as number `2` and leaves line `2.` untouched.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| 3 | The Reviewer types nothing for a Marker | The numbered line still exists, empty. The badge is the point; the words are optional (BR-3) |
| 5 | The Reviewer drags a badge outside the image | The badge is refused at the edge and stays within the image. A position outside `[0,1]` is never stored |
| 6 | The Reviewer removes badge `2` | Its line goes with it, and badge `3` and line `3.` become `2`. No gap is left (BR-2) |
| 6 | The Reviewer closes and reopens the Editor | Every badge is where it was put, at the same number (BR-49) |
| 1 | The Reviewer adds a tenth Marker | Numbering continues. Nothing caps it, though a Finding needing ten is usually two Findings |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| 1 | The Finding's image file is missing | Refuses to open the Marker canvas and shows the Finding as broken (BR-45) | The Finding listed as broken, with the orphan report offering to delete it |
| 2 | The store write fails | Neither the badge nor the line is created. There is never one without the other (BR-1) | The image unchanged and the Note unchanged. A message saying the Marker could not be added |
| 6 | A renumber after a removal fails partway | Rolls the whole renumber back. Every badge and line keeps its previous number | The Marker still present, with a message saying it could not be removed. Numbering is still consistent |

## Outcome

The Finding's Note reads as a numbered list whose numbers are visible on the image, with no gaps and
no mismatch between a badge and its line. Nothing had to be described positionally, which is the whole
point of the use case.

## Business Rules

BR-1, BR-2, BR-3, BR-45, BR-47, BR-48, BR-49.
