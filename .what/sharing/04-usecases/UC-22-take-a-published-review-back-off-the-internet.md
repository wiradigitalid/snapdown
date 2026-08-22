---
type: uc
id: UC-22
component: sharing
satisfies: [FR-25]
critical: false
created: "2026-08-22"
---

# UC-22 — I take a published review back off the internet

Not `critical`: it reduces exposure rather than creating it. What makes it the most carefully specified
flow in the product is the opposite risk — reporting a success it did not achieve.

## Trigger

The Reviewer chooses to unpublish a Bundle, or deletes a published Bundle, which unpublishes it as part
of the same act (BR-23).

## Precondition

The Bundle has a live Publication. A web service address and a publish credential are configured.

## Main Flow

1. The Reviewer chooses to unpublish.
2. Snapdown asks the service to remove the Publication's Markdown and its images.
3. The service removes them.
4. Snapdown immediately asks the service whether that slug is still served.
5. The service answers that it is not.
6. Snapdown records the Publication as ended and clears any previous failure.
7. The Bundle shows as unpublished. Fetching the old URL from anywhere returns nothing, and returns it
   the same way an unknown slug does.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| 1 | The Bundle is not published | Harmless. Snapdown says so and does nothing (BR-95) |
| 1 | The Reviewer is deleting the Bundle | This whole flow runs first. If it does not complete, the deletion does not happen (BR-23, UC-12) |
| 7 | The Reviewer publishes that Bundle again later | A new Publication under the **same** slug, because the slug belongs to the Bundle (BR-90). The slug is never given to a different Bundle (BR-22) |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| 2 | The host is unreachable, or TLS fails | Leaves the Publication marked **published** and records the failure. It does not clear the flag and retry quietly (BR-20, BR-96) | The Bundle still shown as published, the failure named, a retry offered. Never told it is private when it may not be |
| 2 | The service rejects the credential | The same sticky outcome, with the reason distinguished from an unreachable host | The Bundle still shown as published, and the credential named as the problem |
| 3 | The service removes some files and not others | Detected at step 5, which still finds the slug served. The Publication stays marked published | The Bundle still shown as published, the failure named. Retrying is safe: removal is idempotent |
| 5 | The service reports the slug removed but still serves it — a cache, a stale replica | Not detectable from here. This is the one failure this flow cannot catch, and it is why the service must have no cache in front of it — a deployment constraint recorded in the devops repository, not code | The Bundle shown as unpublished while it may still be served. The single worst outcome in the product, and the only one with no in-product guard |
| 4 | The reconcile call itself fails | Treated the same as an unconfirmed unpublish: the Publication stays marked published | The Bundle still shown as published, with the failure named |
| 6 | The local record cannot be written after the service confirmed removal | The Publication is gone but still recorded as live. Reported, and a reconcile offered. The conservative direction — the Reviewer is told it may still be public when it is not, rather than the reverse | A warning, and reconcile offered |

## Outcome

On success: the Publication's content is gone from the service, its URL resolves to nothing, and the
refusal is indistinguishable from a slug that never existed. The Bundle is intact in the Library and
can be published again at the same URL.

On any failure short of that: the Bundle is still shown as published, carrying the reason. That is the
correct outcome, not a degraded one.

## Business Rules

BR-7, BR-20, BR-22, BR-23, BR-24, BR-90, BR-94, BR-95, BR-96, BR-97, BR-99, and NFR-13, NFR-15 as the
properties the flow relies on.
