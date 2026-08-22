---
type: uc
id: UC-17
component: agent-access
satisfies: [FR-19]
critical: true
created: "2026-08-22"
---

# UC-17 — I let the agent in front of me read my reviews

`critical`: it grants a process the Reviewer does not control the ability to read images that may
contain personal data, and a disclosure cannot be recalled.

## Trigger

The Reviewer wants the coding agent in the next window to read a review, and chooses to issue or copy
an Access Key.

## Precondition

Snapdown is running. At least one Bundle exists — a key with nothing to read is legal but pointless.
The Windows credential store is reachable.

## Main Flow

1. The Reviewer opens the agent access panel.
2. Snapdown shows whether a key is currently valid, and when it was issued.
3. The Reviewer chooses to issue a key, or to copy the one already valid.
4. On issuing: Snapdown generates a key, revokes any key already valid in the same act, stores the
   key in the Windows credential store and its hash in the Library, and puts the key on the clipboard.
5. Snapdown says the key was copied.
6. The Reviewer pastes it into the agent conversation and tells the agent to read the Snapdown bundle.
7. The agent can now list Bundles and read one, which is UC-18.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| 3 | A key is already valid | The Reviewer copies it again rather than issuing a new one, so an agent already holding it is not cut off (BR-73) |
| 4 | A key was already valid and the Reviewer issues anyway | The previous key stops working immediately, in the same act. There is never an instant with two valid keys (BR-16) |
| 2 | No key has ever been issued | The panel says so plainly, and issuing is the only offered action |
| 6 | The Reviewer pastes the key somewhere else by mistake | Nothing in this product can detect it. Revoking is the whole remedy, and it is one action away (UC-19) |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| 4 | The credential store refuses to write | The key is not issued at all. No hash is stored, so nothing half-granted exists | The panel naming what failed. Any previously valid key is untouched and still works |
| 3 | The credential store cannot return the existing key for a re-copy | Says the key cannot be recovered and offers to issue a new one instead | The existing key still valid for the agent already holding it, and a clear next step |
| 4 | The Library write fails after the credential store write | Removes the key from the credential store, so no key exists that authorisation would not recognise | Nothing granted, and a message saying the key could not be issued |
| 4 | The clipboard write fails | The key is issued and valid; only the copy failed. The panel says so and offers to copy again | A valid key that the Reviewer can still copy. Not reissued, because reissuing would revoke what was just created |
| 1 | The Local API is not listening — the port could not be bound | The panel shows agent access as unavailable and does not offer to issue a key, because a key with no server is a false grant | The reason named. Capture and the Editor are unaffected |

## Outcome

Exactly one Access Key is valid, the Reviewer has it on the clipboard, and no previous key works. An
agent given it can read Bundles until the Reviewer revokes. Nothing else in the Library — no unbundled
Finding, no Note, no Setting — is reachable with it.

## Business Rules

BR-14, BR-15, BR-16, BR-17, BR-73, BR-74, BR-77, BR-78, BR-79, and NFR-9 as the property the grant
relies on.
