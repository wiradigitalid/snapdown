---
type: uc
id: UC-7
component: finding
satisfies: [FR-13]
critical: true
created: "2026-08-22"
---

# UC-7 — I throw away the findings I am done with

`critical`: the image files leave the disk and cannot be recovered.

## Trigger

The Reviewer selects one or more Findings in the Editor and chooses to delete them.

## Precondition

The Editor is open. At least one Finding is selected. The Vault is reachable — a deletion is never
attempted against a Vault that cannot be read.

## Main Flow

1. The Reviewer selects the Findings they are done with.
2. The Reviewer chooses to delete them.
3. Snapdown asks once, naming how many Findings will go and saying that their image files go too.
4. The Reviewer confirms.
5. Snapdown removes every selected Finding's image file from the Vault.
6. Snapdown confirms every file is gone.
7. Snapdown removes the Findings, their Notes, and their Markers from the Library.
8. The Editor shows the remaining Findings, with the selection cleared.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| 4 | The Reviewer declines | Nothing is removed. The selection stays as it was (FR-13) |
| 5 | A selected Finding's file is already gone | Treated as success for that file. The goal is that it is not there, and refusing would leave the Reviewer unable to delete a Finding whose file someone else removed |
| 5 | A selected Finding belongs to a Bundle | Deleted anyway. The Bundle keeps its own image copy and stays readable (BR-12, BR-56) |
| 1 | The Reviewer selects all Findings | Allowed. Step 3 names the full count, which is the whole safety |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| 5 | A file refuses to be deleted — held open by another process | Abandons the whole deletion. Not one file is removed and not one row is removed (BR-5) | A dialog naming which files refused, and every selected Finding still present. Nothing is half gone |
| 6 | A file reports deleted but is still present | Treated as a refusal, and the deletion is abandoned as above | The same: nothing removed, the files named |
| 7 | The store write fails after the files are already gone | Cannot be rolled back — the files are gone. The rows stay, so the Findings appear as broken rather than vanishing | Findings shown as broken, and the orphan report offering to remove the rows. This is the one reachable inconsistent state, and it is the recoverable direction on purpose |
| 3 | The Vault cannot be read at all | Refuses the deletion before asking for confirmation | A message naming the Vault path, with an action that opens Settings |

## Outcome

The selected Findings are gone from the Library and their image files are gone from the Vault. Nothing
is archived, nothing is recoverable, and the Vault holds no file that the Library no longer points at.
Any Bundle that held one of them is untouched and still readable.

## Business Rules

BR-5, BR-6, BR-7, BR-12, BR-55, BR-56, and NFR-5 as the invariant the whole flow exists to keep.
