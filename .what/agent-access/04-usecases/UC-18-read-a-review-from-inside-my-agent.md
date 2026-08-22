---
type: uc
id: UC-18
component: agent-access
satisfies: [FR-20, FR-21]
critical: false
created: "2026-08-22"
---

# UC-18 — I read a review from inside my agent instead of pasting it in

Written in the actor's own voice, and the actor is the Local coding agent. It is the one use case in
this product a non-human initiates.

## Trigger

The Reviewer has pasted an Access Key into the conversation and asked the agent to read a Snapdown
review.

## Precondition

The MCP Bridge is configured in the agent's MCP client. Snapdown is running. The pasted key is the
currently valid one.

## Main Flow

1. The agent hands the pasted key to the bridge.
2. The bridge holds it in memory for the life of this process, and confirms it has a key.
3. The agent asks for the list of Bundles.
4. The bridge asks Snapdown, with the key on the request, and returns each Bundle's name, its Finding
   count, and when it was composed.
5. The agent picks the Bundle the Reviewer meant and asks to read it.
6. The bridge returns that Bundle's Markdown, byte-identical to what the Reviewer would have copied.
7. The agent fetches the images the Markdown references, one at a time.
8. The agent now holds the review: every Note under the image it describes, with the numbered Markers
   matching the numbered lines.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| 3 | The Library holds no Bundles | An empty list, and HTTP 200. This is a successful answer, not a refusal, and the agent may say the Library is empty because that is true (BR-17) |
| 5 | The agent asks for a Bundle id that does not exist | `not_found`. It does not probe nearby ids |
| 7 | The agent reads only the Markdown and no images | Legitimate. The Markdown is the payload; the images support it |
| 7 | One image is missing from the Vault | That filename is unavailable; the rest of the read succeeded (BR-85) |
| 2 | The agent restarts, then asks again | It has no key: the bridge kept none (BR-81). It has to be given the key again |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| 3 | No key was ever handed over | `key_required`, as an error rather than an empty list. The agent tells the Reviewer to paste a key — it does not report the Library as empty (BR-17, BR-77) | The Reviewer told to paste a key. This distinction is the whole point of AD-7 |
| 3 | The key was revoked while the agent was working | `key_invalid`, distinct from `key_required`, so the agent can say the key expired rather than that it was never given one | The Reviewer told the key is no longer valid |
| 4 | Snapdown is not running | `unavailable`, immediately. The bridge does not hang and does not retry in a loop (BR-80) | The Reviewer told Snapdown is not running |
| 4 | Snapdown does not answer within ten seconds | `unavailable`, saying Snapdown is not responding. The agent's turn is not held open | The Reviewer told Snapdown is not responding |
| 4 | Something other than Snapdown is listening on the port | Treated as `unavailable`, because the reply lacks the response header the bridge requires. The agent is never fed a fabricated Bundle | The Reviewer told Snapdown is not responding |
| 7 | An image filename resolves outside its Bundle's folder | `bad_request`. Refused, and the refusal is logged at warning level (BR-84) | Nothing for the Reviewer. This is a defect in the caller or an attack, not their problem |

## Outcome

The agent holds one Bundle's Markdown and the images it references, without the Reviewer pasting any
content. It cannot see unbundled Findings, cannot see Notes outside a Bundle, and cannot change
anything. The key stays valid until the Reviewer revokes it, so the agent can read again later in the
same session.

## Business Rules

BR-14, BR-15, BR-17, BR-77, BR-78, BR-79, BR-80, BR-81, BR-82, BR-83, BR-84, BR-85.
