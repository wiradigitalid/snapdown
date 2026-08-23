---
type: uc
id: UC-13
component: settings
satisfies: [FR-5]
critical: false
created: "2026-08-23"
---

# UC-13 — I decide how much picture quality a screenshot is worth

## Trigger

The Reviewer opens Settings, either because a stored image looked worse than they wanted or because a
Vault is growing faster than they expected.

## Precondition

Snapdown is running and the Editor is open. A Quality Budget is in effect — the Reviewer's choice, or
the shipped `Auto` (`BR-111`).

## Main Flow

1. The Reviewer opens Settings and finds the Quality Budget group without scrolling (`FR-29`).
2. Snapdown shows which of the five named budgets is in effect, one line saying what it is for, and
   the size, dimensions and budget of the most recently stored Finding.
3. The Reviewer picks a different named budget.
4. Snapdown stores the named state and its resolved pair as one write (`BR-116`) and applies it
   immediately; there is no Save.
5. The Reviewer takes a Capture to see the effect.
6. Snapdown reduces that Capture under the new budget, stores the resolved pair with the Finding
   (`NFR-18`), and Settings shows the new size, dimensions and budget name.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| 2 | No Capture has been taken yet | The readout says so. The control still works — a budget can be chosen before there is anything to compare |
| 3 | The Reviewer opens **Advanced** and edits a resolved value | The budget moves to `Custom` in the same interaction, visibly (`BR-117`). They never leave `Auto` without seeing it |
| 3 | The Reviewer picks `Auto` while on `Custom` | The edited pair is abandoned and derivation resumes. Advanced shows what `Auto` last resolved, and those numbers are a readout, not a stored choice |
| 6 | The captured region is small | `Auto` resolves a different pair than it would for a full screen (`BR-104`). A test finding them identical is a failing test |
| 6 | Findings captured earlier | Untouched. Nothing is ever re-encoded (`BR-9`) |

## Failure Flows

| Condition | What happens |
|---|---|
| An Advanced value is outside its sane range | Refused at the point of entry, naming the range. The budget does not move to `Custom` on a refused value |
| The settings store cannot be written | The group shows its own failure and keeps the previous budget. The other four groups continue to work — a partial failure is scoped to where it happened |
| The store cannot be read at all | The group is inert, says so, and offers Retry. Nothing is created over (`BR-118`) |

## Postcondition

Exactly one of five named budgets is in effect (`BR-103`). Every Capture taken afterwards carries the
pair actually applied to it. Nothing already stored has changed.

## Note on this use case

Its promise changed under `DEC-004` after r1 shipped. Before, this was "set two numbers": a maximum
long edge and an encoder quality, typed into two fields. Those numbers still exist one level down and
the Reviewer can still reach them — but they were never answerable. § 8 of the PRD records that 1600
px "has not been measured", and a value the team cannot defend is not one a Reviewer can judge.

`OQ-3` is not closed by the change; it is restated. The open question stops being *what is the right
constant* and becomes *is `Auto`'s output legible at its smallest*, and step 6 above is where a
Reviewer would notice if it is not.
