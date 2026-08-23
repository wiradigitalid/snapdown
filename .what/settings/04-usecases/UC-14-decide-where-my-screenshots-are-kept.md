---
type: uc
id: UC-14
component: settings
satisfies: [FR-16]
critical: false
created: "2026-08-23"
---

# UC-14 — I decide where my screenshots are kept

## Trigger

The Reviewer wants Finding and Bundle files somewhere else — a project folder, a synced drive, a
different disk.

## Precondition

A Vault location is in effect: the Reviewer's choice, or the shipped default, so capture already works
on a fresh install (`BR-28`).

## Main Flow

1. The Reviewer opens Settings and sees the current Vault path.
2. The Reviewer clicks **Browse** and picks a folder in the native Windows picker. The path is never
   typed.
3. Snapdown validates the folder **by writing to it** (`BR-115`).
4. The Reviewer clicks **Apply**. This is the one Setting with an explicit Apply, because the next
   step moves files.
5. Snapdown counts the existing files and asks once whether to move them.
6. The Reviewer confirms.
7. Snapdown moves every file, or moves none (`BR-29`, `AD-2`).
8. The path field shows the new folder, and **Open in Explorer** proves it.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| 5 | The Vault is empty | No question is asked. There is nothing to move, and asking would be ceremony |
| 6 | The Reviewer declines the move | The new location takes effect for Captures from now on. Existing files stay where they are, and the orphan report (`FR-15`) is how they are found later. The confirmation says this before it is chosen, not after |
| 2 | The Reviewer cancels the picker | Nothing changes. The old path is still in effect and still shown |
| 2 | The Reviewer picks the folder already in effect | Apply is inert. No move is attempted and no question is asked |

## Failure Flows

| Condition | What happens |
|---|---|
| The chosen folder cannot be written to | Refused at step 3, at the moment of choosing, naming the folder. Not at the next Capture (`FR-16`) |
| The folder reports itself writable but the write fails | Same refusal. This is exactly why `BR-115` validates by writing rather than by inspecting permissions — a network drive, a synced folder, or a policy-locked path can all lie |
| The move fails partway | **Nothing moved.** One message says so plainly, and the Reviewer's files are where they were. A half-move is the state `BR-29` exists to forbid, because nothing left on disk would say which half was intended |
| The target runs out of space mid-move | Same as above. Space is checked before the move begins, and the check being passed does not remove the need for the rollback |
| A file is locked by another program mid-move | Same as above, naming the file |

## Postcondition

One Vault location is in effect. Either every file is in it, or the Reviewer declined the move and
knows they declined it. There is no state in which some files moved.

## Why this one has an Apply and the others do not

Every other Setting on this surface applies the moment it changes. This one may move every file the
Reviewer owns, and a two-step commit is the difference between a deliberate act and a stray click.
The inconsistency is deliberate and it is the only one.
