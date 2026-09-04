---
type: decision
id: DEC-002
status: superseded
touches:
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/c4-l2-containers.md
  - .how/_platform/inventory-api.md
  - .what/agent-access/SRS-agent-access.md
  - .what/agent-access/03-domain/domain-model.md
supersedes: null
superseded_by: DEC-016
created: "2026-08-22"
---

# DEC-002 — An agent reads the Library through a stateless MCP bridge and a key the Reviewer pastes

## Decision

Agent access to the Library is two processes joined by a secret the Reviewer hands over: the desktop
app serves a read-only HTTP interface bound to `127.0.0.1`, and a separate stateless `mcp-bridge`
executable speaks the Model Context Protocol to the agent. Exactly one Access Key is valid at a time,
it reaches the bridge only through a tool call, and the bridge persists neither the key nor any Library
content between runs.

## Why

Four things had to hold at once, and no single-process design holds all four: it must work with an MCP
client that can only launch a stdio server, which is most of them today; no secret may sit on disk;
access must end in one action; and there must not be a second copy of the Library anywhere.

The obvious alternatives each fail one of them, and they fail it invisibly. A key in the bridge's
configuration file satisfies everything except the second, and the moment it is on disk the paste
ceremony becomes theatre — "revoke" stops meaning anything, because the next bridge process reads the
key again. Standing MCP access with no key satisfies the first, third, and fourth, and hands whatever
is configured the ability to read every Capture the Reviewer has ever taken, on a machine full of
personal data. Exporting Bundles into a folder the agent already watches is the cheapest of all and
makes the Library permanently readable, which is what the brief's fourth constraint exists to forbid.

Two processes is therefore not a preference; it is the shape the constraints leave. The bridge is a
compatibility shim, and it should be deleted when stdio-only MCP clients stop mattering — the spine
records that as a deferred simplification rather than as a plan.

## Cost

- **Two executables to ship, configure, and version.** The Reviewer configures the bridge path in
  their agent's MCP settings once, and a version skew between bridge and desktop is a failure mode
  that would not exist with one process.
- **The key is pasted again every session.** The bridge holds it in memory only, so restarting the
  agent means pasting again. That friction is the feature, and it is still friction — OQ-6 records
  that it may not be wanted.
- **Loopback is not a boundary.** Any process on the machine can reach `127.0.0.1`, so the key is the
  only real control. It has to be checked on every request rather than once per session, which is a
  cost paid on every call.
- **An extra hop for image bytes.** Every image an agent reads crosses two process boundaries.
- **A refusal has to survive translation.** The bridge maps a Local API envelope onto an MCP tool
  error, and a bridge that flattened a refusal into an empty result would satisfy AD-7's letter and
  break its purpose. That has to be tested from the agent's side, not the API's.

## Alternatives

Required here: `agent-access` sits at `risk_accepted: low`.

| Option | Why not |
| --- | --- |
| One process — MCP over HTTP straight from the desktop app | Strictly better wherever it works, and it does not work for a client that can only launch a stdio server. Kept as the deferred simplification in the spine |
| The Access Key in the bridge's config file | Puts the secret on disk, which makes revocation cosmetic: the next bridge process reads the key again |
| Standing MCP access with no key at all | Hands whatever is configured every Capture the Reviewer has ever taken, indefinitely |
| Export Bundles to a folder the agent already reads | Cheapest, and it makes the Library permanently readable with no act of granting and nothing to revoke |
| A key set — one per agent, with scopes | Solves a problem this product does not have. One Reviewer, one machine; a set is state the Reviewer would have to reason about |
| An expiring key instead of an explicit revoke | Good hygiene, and it breaks a long review session in the middle. Revocation already closes the hole; deferred, not rejected |
| Exposing unbundled Findings as well | Removes the composition step, which is where the Reviewer decides what the agent should look at. The agent would then read the four Findings about an unrelated concern too |

## Reversal trigger

- The MCP clients the Reviewer actually uses all speak HTTP with a bearer token. Then the bridge is
  pure cost and the one-process shape wins.
- Pasting the key per session proves to be friction the Reviewer routes around — by keeping one agent
  session alive purely to avoid it, for instance. That reopens key lifetime, not the two-process
  shape.
- An agent is observed reporting an empty Library when the real answer was a refusal, more than once
  after the mapping is fixed. That reopens whether the bridge should translate errors at all, or pass
  them through untouched.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | `OQ-6` |
| Source material | `.what/_prd/agent-handoff/addendum.md` § Options weighed — the four-criteria table this decision is read off |
