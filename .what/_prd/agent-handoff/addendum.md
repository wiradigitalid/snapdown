---
type: addendum
parent: prd
initiative: agent-handoff
status: draft
created: "2026-08-22"
updated: "2026-08-22"
---

# Addendum — PRD: Agent Handoff

Depth that earned a place beside the PRD but would derail it. Nothing here is a promise, and nothing
here may be cited as a design.

## Rejected alternatives

| Option | Why it lost |
| --- | --- |
| Write Bundles into a folder the agent already reads, and skip both paths | Simplest thing that works, and it makes the whole Library standing-readable by whatever runs in that repository. The brief's fourth constraint forbids exactly that, and the Access Key exists so that access is an act rather than a state. |
| Give the agent standing MCP access with no key | One less step per session, and the agent can read every Capture the Reviewer has ever taken, on a machine full of personal data, for as long as it is configured. The key is the whole control. |
| Let the agent trigger a Capture over MCP | Turns Snapdown into a remote-control surface for whatever holds the key. The value is the Reviewer's judgement at capture time, which an agent cannot supply, so the feature would cost the entire security posture and buy nothing. |
| One process: expose MCP over HTTP from the desktop app, no Bridge | Fewer moving parts wherever the agent can speak HTTP MCP with a bearer token. It fails for every client that only knows how to launch a stdio server, which is most of them today. The Bridge is a compatibility shim and should be deleted when it stops being needed. |
| Bundle the Local API's key into the Bridge's configuration file | Then the key is standing again, written on disk, and the paste ceremony is theatre. The Bridge holding no key between runs is what makes revoke mean something. |
| Sync the whole Library to the web service | The Reviewer would never have to think about publishing, and every screenshot they ever took would be on a server. Rejected on the brief's privacy constraint, and it also makes capture depend on connectivity. |
| Publish to a third-party service — a gist, an object store, a paste site | No host to run, no domain to buy. It also means the Reviewer's screenshots live in an account they do not control, under someone else's retention policy, and "unpublish" becomes a promise a third party keeps. Rejected. |
| Real authentication on published Bundles — accounts and sign-in | The correct answer for a product with users. This has one Reviewer and readers that are agents, so an account is a credential the agent has to be given anyway. The unlisted slug plus an optional read token is the same security with none of the machinery. Recorded as OQ-8, because it is an accepted risk rather than an obviously right call. |
| Serve published Bundles as HTML only | An agent then has to parse a page to find the review. Raw Markdown at the same URL is the primary representation and the rendering is the courtesy, not the other way round. |
| An index page listing the Reviewer's Publications | Convenient, and it turns one unguessable slug into one guessable entry point for all of them. NFR-15 forbids it. |
| Expire the Access Key automatically after N hours | Good hygiene, and it breaks a long review session in the middle for no gain the revoke button does not already give. Deferred, not rejected — noted in §6.2. |
| Expose unbundled Findings over MCP so the agent sees work in progress | Removes the composition step, which is the step where the Reviewer decides what the agent should look at. Without it the agent reads everything, including the four Findings about an unrelated concern. |

## Options weighed

### How an agent on this machine reaches the Library

Criteria fixed before scoring: works with a stdio-only MCP client; no secret written to disk; access
revocable in one action; no second copy of the Library.

| Shape | stdio clients | No secret on disk | One-action revoke | No second copy |
|---|---|---|---|---|
| Bridge + loopback API, key pasted per session | yes | yes | yes | yes |
| Bridge + loopback API, key in the Bridge's config | yes | no | partly | yes |
| HTTP MCP straight from the desktop app | no | yes | yes | yes |
| Export Bundles to a watched folder | yes | yes | no | no |

The first row is the only one that satisfies all four, which is why the design has two executables
instead of one. The third row is strictly better wherever it works and should replace the first if
stdio-only clients stop mattering.

### Access control on a published Bundle

Criteria: an agent can use it with no setup beyond a URL; it cannot be enumerated; the Reviewer can
end it; it needs no account.

An unguessable slug satisfies all four, with one honest weakness: the URL is a bearer credential, so
anywhere the Reviewer pastes it becomes somewhere the Bundle can be read from. A read token in a
header would fix that and would also mean the agent needs setup beyond a URL, which was the first
criterion. The slug is the r2 answer; the token is designed for so that adding it is not a rewrite.

### What the publish payload contains

Only the composed Bundle — its Markdown and the reduced images it references. Not the source
Findings, not their Notes as separate records, not Marker coordinates. A Publication is a copy of a
document, which is what makes FR-25's deletion complete and what makes OQ-16's answer "nothing" when
the source Findings are later deleted.

## Mechanism and transport

Not a design. The SDD owns that, and a builder MUST NOT follow this section.

- The Local API and the MCP Bridge are two processes joined by the Access Key. The Bridge is a thin
  translator with no state; anything it caches becomes a way for a revoke to be ignored.
- The refusal for "no key" and the response for "empty Library" have to be distinguishable, because an
  agent that reads an empty list will report to the Reviewer that there is nothing there. That is a
  requirement about error shape, not about status codes, and FR-20 states it as behaviour.
- Publishing wants to be a single upload that either completes or leaves nothing, because FR-23
  promises no partial state on the service. That points at content being staged and then made
  reachable, rather than files appearing one at a time under a live slug.
- Republishing the same Bundle at the same slug means the slug belongs to the Bundle, not to the act
  of publishing.
- The service's whole state being one directory is what makes NFR-14 checkable and what keeps the
  deployment story to one binary and one folder.
- The desktop side needs to store where the service is and a credential to publish with. That
  credential is a secret and has the same handling rules as the Access Key.

## Sizing

Nothing sized. Wave sizing happens at G4 and G5 against the story list.

One figure recorded because it bounds the publish payload: a Bundle of five Findings at the shipped
Quality Budget is on the order of one megabyte of images plus a few kilobytes of Markdown. That is
small enough that streaming, chunking, and resumable upload are all unnecessary in r2, and large
enough that a synchronous publish needs a visible progress state. Source: the Quality Budget default
recorded in the `capture-to-markdown` addendum, not a measurement of a real Bundle.

## Personas and research detail

The reader that shapes this initiative is not a person. Two of its properties do all the work:

- It reads text far more reliably than it navigates. Hence raw Markdown as the primary
  representation, and hence the rendering being an afterthought rather than the product.
- It has no memory of the conversation the Reviewer had about access. Hence errors that say what is
  wrong — "a key is required" rather than an empty list — because a misleading response becomes a
  confident wrong report to the Reviewer.

The human property that shapes it: the Reviewer's agents are not all in one place, and the same review
has to reach both without being prepared twice. That single fact is why there are two capabilities
here instead of one.

No external research was run for this PRD and `_bmad-output/` holds no run folder for it.
