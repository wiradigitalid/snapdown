---
type: uc
id: UC-20
component: sharing
satisfies: [FR-23]
critical: true
created: "2026-08-22"
---

# UC-20 — I put a review somewhere the agent on my server can reach it

`critical`: images that may contain anything from the Reviewer's screen go onto the public internet,
and an unpublish cannot recall what was already fetched.

## Trigger

The Reviewer selects a Bundle and chooses to publish it.

## Precondition

The Bundle exists and every image it references is present in the Vault. A web service address and a
publish credential are configured. For a single screenshot, the Reviewer has first composed a
one-Finding Bundle — publishing is per Bundle, always (BR-61, OQ-16).

## Main Flow

1. The Reviewer chooses to publish a Bundle.
2. Snapdown asks to confirm, naming the Bundle and saying that publishing cannot be recalled once the
   review has been fetched.
3. The Reviewer confirms.
4. Snapdown stages the Bundle's Markdown and every one of its images on the service, showing progress.
5. Snapdown confirms with the service that everything it sent is held, file by file.
6. Snapdown asks the service to make the Publication reachable.
7. Snapdown records the Publication: its slug, when it happened, and where.
8. The Bundle now shows as published, with its URL.
9. The Reviewer copies the URL and pastes it to the agent on their server, which then reads it (UC-21).

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| 1 | The Bundle is already published | The same slug is reused and its content is replaced. A URL already handed out keeps working and starts serving the new content (BR-21, BR-90) |
| 1 | The Bundle holds one Finding | Published like any other. This is how a single screenshot reaches a remote agent (BR-61) |
| 3 | The Reviewer declines | Nothing is staged and nothing leaves the machine |
| 8 | The Reviewer opens the URL in a browser | They get a rendering of the same Markdown the agent reads, never different content (BR-93) |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| 2 | No service address or no publish credential is configured | Refuses before reading anything from the Bundle, naming what is missing (BR-87) | A message with an action that opens Settings. Nothing left the machine |
| 4 | One of the Bundle's images is missing from the Vault | Refuses the publish, naming the file. A review with broken images is never published (BR-88) | The Bundle unpublished, and the missing file named |
| 4 | The host is unreachable, or TLS fails | Abandons the staged upload. The slug never resolves, so nothing partial is readable (BR-89) | The Bundle unpublished, and the reason named. Nothing readable on the service |
| 4 | The service rejects the credential | Reports that the service rejected the credential, not that the network failed | A message distinguishing a bad credential from an unreachable host |
| 5 | The service holds fewer files than were sent, or different byte counts | Abandons the publish rather than making a partial Publication reachable | The Bundle unpublished, and the mismatch named |
| 6 | The service cannot make the Publication reachable | Abandons it. Nothing resolves, and no `publication` row is written | The Bundle unpublished |
| 7 | The local record cannot be written after the service confirmed | The Publication is live but unrecorded. Snapdown says so and offers to reconcile, because the honest state is "this may be public and I have lost track of it" | A warning naming the slug, with reconcile offered. Never silence |

## Outcome

The Bundle is readable at an unlisted URL by anyone holding it, and by nothing else — no index, no
search, no enumeration. The Reviewer has the URL and can see when it was published. The Bundle itself
is unchanged, and the Findings behind it are untouched. Nothing else on the machine has left it.

## Business Rules

BR-18, BR-19, BR-21, BR-22, BR-24, BR-25, BR-61, BR-86, BR-87, BR-88, BR-89, BR-90, BR-93, BR-101, and
NFR-10, NFR-11, NFR-12 as the properties the act relies on.
