---
type: scenario
id: SCN-04
component: finding
branches_from: UC-5
created: "2026-08-23"
---

# SCN-04 — A Note line deleted without its Marker

Branches from `UC-5`. It is the `Mismatched` transition in `state-machines.md` § 3, and it is the one
place where `AD-1` is easy to implement wrongly in a way that reads as correct.

## Setup

A Finding has three Markers and a Note reading:

```
1. the CTA drops below the fold at 1280
2. the summary row spacing is inconsistent
3. focus outline is invisible on this control
```

The Reviewer, editing the Note as free text, deletes line 2. They did not touch the image.

## The three things the product could do

| Option | What it costs |
|---|---|
| **Delete Marker 2 and renumber** | Destroys evidence the Reviewer did not ask to destroy. They may have deleted the line to rewrite it |
| **Renumber the remaining lines to 1 and 2** | Silently repoints line 2 — which was line 3 — at the wrong badge. This is precisely the defect `AD-1` exists to prevent, arrived at by trying to satisfy `AD-1` |
| **Show the mismatch** | Nothing is destroyed and nothing is silently repointed. The Reviewer sees a state they created |

The product does the third.

## What must happen

1. Marker 2 stays on the image, at its position, numbered 2.
2. The Note keeps the two lines the Reviewer left, numbered as they wrote them.
3. The **note pane** shows Marker 2 as having no line. The row for it says so.
4. The **image** shows nothing unusual. A badge is a badge.
5. Composing a Bundle from this Finding is **allowed**. The Markdown carries what is there.

## Why point 4 is not an oversight

An app-only state must never be burned into an exported image. The Bundle's images are read on
another machine, by an agent or a person who has no access to Snapdown's opinion about whether the
numbering is ragged. A "this marker has no line" annotation on the image would be a permanent artifact
of a temporary editing state.

## Why point 5 is not a hole

`FR-10` promises composition of what the Reviewer picked. A Finding whose numbering is ragged is still
an observation, and refusing to compose it would make a tidiness rule outrank the product's purpose.
The mismatch is visible before composing, in the note pane, which is where the Reviewer can still
choose to fix it.

## The reverse case

The Reviewer deletes Marker 2 from the image instead. Then `BR-5`'s renumbering applies: Marker 3
becomes 2, **and its line moves with it**, because removing a Marker is an operation over the one
ordered collection (`AD-1`). Line 2's text is removed with its Marker.

The two cases are asymmetric on purpose, and the asymmetry is the whole design: the **image** is
edited through operations the product owns, so the product keeps the collection consistent. The
**Note** is free text, so it is not. Making the Note a structured editor would restore symmetry and
would take the Reviewer's prose away from them.

## Tests this scenario names

- `finding::deleting_a_note_line_leaves_its_marker_in_place_and_numbered`
- `finding::deleting_a_note_line_does_not_renumber_the_remaining_lines`
- `finding::a_marker_with_no_line_is_reported_in_the_note_pane`
- `finding::a_marker_with_no_line_is_not_annotated_on_the_image`
- `finding::a_finding_with_a_ragged_sequence_can_still_be_composed_into_a_bundle`
- `finding::deleting_a_marker_removes_its_line_and_renumbers_contiguously`
