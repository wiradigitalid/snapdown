---
type: uc
id: UC-12
component: bundle
satisfies: [FR-14]
critical: true
created: "2026-08-22"
---

# UC-12 — I get rid of a review and everything in it

`critical`: files leave the disk and cannot be recovered, and if the Bundle is published this is also
the act that takes it off the internet.

## Trigger

The Reviewer chooses to delete a Bundle, from the Bundle list or from the Bundle's own view.

## Precondition

The Bundle exists. The Vault is reachable. If the Bundle is published, the web service is reachable —
otherwise the deletion cannot complete and step 3 says so.

## Main Flow

1. The Reviewer chooses to delete a Bundle.
2. Snapdown asks once, naming the Bundle, saying that its Markdown and its images go, and offering —
   in the same confirmation — to delete the Findings it was composed from as well.
3. The Reviewer confirms, with or without the Findings.
4. If the Bundle is published, Snapdown unpublishes it and waits for the service to confirm.
5. Snapdown removes the Bundle's image copies and its Markdown file from the Vault.
6. Snapdown confirms every file is gone.
7. Snapdown removes the Bundle and its memberships from the Library.
8. If the Reviewer chose it, Snapdown then deletes the source Findings, which is UC-7.
9. The Bundle list no longer holds it.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| 3 | The Reviewer declines | Nothing is removed and nothing is unpublished |
| 3 | The Reviewer confirms without the Findings | The Findings stay in the Library. This is the default (BR-70) |
| 4 | The Bundle was never published | Step 4 is skipped |
| 5 | One of the Bundle's files is already gone | Treated as success for that file, the same as in UC-7 |
| 8 | A source Finding is in another Bundle too | It is still deleted; that other Bundle keeps its own image copy and stays readable (BR-12, BR-56) |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| 4 | The unpublish is not confirmed by the service | Abandons the whole deletion. The Bundle stays, and stays marked published, because telling the Reviewer it is private when it may not be is the worse outcome (BR-20, BR-23) | The Bundle present and still marked published, with the failure named and the option to retry |
| 5 | A file refuses to be deleted | Abandons the deletion. Not one file and not one row is removed (BR-5) | A dialog naming which files refused. The Bundle still present |
| 7 | The store write fails after the files are gone | Cannot be rolled back. The rows stay, so the Bundle shows as broken rather than vanishing | The Bundle shown as broken, and the orphan report offering to remove the rows. The recoverable direction, on purpose |
| 8 | Deleting the source Findings fails | The Bundle is already gone and stays gone. The Findings stay, and the failure is reported separately | The Bundle deleted, the Findings still present, and a message saying so. Two acts, reported as two |

## Outcome

The Bundle is gone from the Library, its Markdown and image copies are gone from the Vault, and if it
was published its URL no longer resolves. The source Findings are gone too if the Reviewer said so,
and still there if they did not. Nothing is archived and nothing is recoverable.

## Business Rules

BR-5, BR-6, BR-7, BR-12, BR-20, BR-23, BR-56, BR-69, BR-70, and NFR-5 as the invariant the flow keeps.
