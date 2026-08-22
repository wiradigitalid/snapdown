---
type: srs
component: agent-access
status: draft
created: "2026-08-22"
updated: "2026-08-22"
satisfies: [FR-19, FR-20, FR-21, FR-22, NFR-9]
reviewed:
  date: "2026-08-22"
  sha: 1a67115
  lenses: [structure, prose, edge-case-hunter]
---

# SRS — agent-access

## Decision Summary · [G3]

This component is the door an agent on this machine reads Bundles through, and the key that opens it.
The Reviewer copies an Access Key and pastes it into an agent conversation; from then until they
revoke it, that agent can list Bundles, read one Bundle's Markdown, and fetch its images. Before the
paste, nothing is readable. After the revoke, nothing is readable again.

Three decisions do the work. There is **exactly one key at a time**, so issuing a new one is also a
revocation and the Reviewer never has to reason about a set. The **bridge holds no key between runs**,
so revocation is immediate rather than eventual. And a **refusal is never an empty result**: an agent
told "no key" must not report to the Reviewer that their Library is empty.

`mode: guarded`, `risk_accepted: low`. The key gates images that may contain anything that was on the
Reviewer's screen, across a process boundary this product does not control, and a disclosure cannot be
taken back.

## Why · [G3]

Because access is an act, not a state. Every other way to let an agent read the Library — a watched
folder, a config file with a key in it, standing MCP access — makes the whole Library readable by
whatever is configured, for as long as it is configured. This component exists so that granting is
something the Reviewer does and can undo, and it is the only component whose reason for existing is a
thing it refuses.

## Actor Register · [G3]

| Actor | Who they are | What they may do |
| --- | --- | --- |
| Reviewer | The person operating Snapdown. The only writer | Issue an Access Key, copy it, see whether one is valid and when it was issued, and revoke it |
| Local coding agent | An AI coding agent running on the Reviewer's machine, configured with the MCP Bridge | With a valid key: list Bundles, read one Bundle's Markdown, fetch that Bundle's images. Nothing else, and nothing that writes |

The agent is a real actor here — it is the only component in the product where a non-human initiates a
use case. It is not a variant of the Reviewer: what it may do is a strictly smaller, read-only set.

## UC Catalogue · [G3]

| id | Use case | Actor | Satisfies | critical |
| --- | --- | --- | --- | --- |
| UC-17 | I let the agent in front of me read my reviews | Reviewer | FR-19 | yes |
| UC-18 | I read a review from inside my agent instead of pasting it in | Local coding agent | FR-20, FR-21 | no |
| UC-19 | I take the agent's access away again | Reviewer | FR-22 | no |

One of three is `critical`: issuing a key grants a process the Reviewer does not control the ability to
read images that may contain personal data, and that disclosure cannot be recalled. UC-18 is not
marked — it is the exercise of an access already granted, and marking both would make two thirds of
this catalogue critical, which the definition forbids.

## Constraints · [G3]

| Constraint | Source |
| --- | --- |
| Every surface here is read-only. No route and no MCP tool creates, changes, or deletes anything | AD-5, BR-15 |
| The Local API binds `127.0.0.1` only, and every request without the currently valid key is refused. Keys are compared in constant time | NFR-9 |
| Exactly one Access Key is valid at a time; issuing a new one revokes the previous one immediately | BR-16 |
| A revoke takes effect on the next request. No cache and no grace period | NFR-13 |
| Only Bundles are readable. An unbundled Finding is invisible here | BR-14 |
| A refusal is always distinguishable from an empty result, by code | AD-7, BR-17 |
| Every failure crossing a process boundary uses the envelope in `cross-cutting.md` | AD-7 |
| A Bundle's Markdown is served as the exact stored bytes; nothing is re-rendered on the way out | AD-9 |
| The MCP Bridge persists no key and no Library content between runs | AD-5, cross-cutting.md § Configuration |
| The key never appears in a log line, a Bundle, a published document, or this repository. `library.db` holds only its hash | cross-cutting.md § Secrets |
| No image is re-encoded on the way out | AD-4 |

## Non-Goals · [G3]

- **Anything writable.** No MCP tool and no route that changes a Note, a Marker, a Bundle, or takes a
  Capture. AD-5 makes a new write here a violation rather than a feature.
- **Reaching another machine.** The Local API is loopback only. Off-machine access is `sharing`.
- **Exposing unbundled Findings.** BR-14.
- **Composing, editing, or deleting a Bundle.** `bundle` owns all three.
- **More than one key.** No key set, no per-agent keys, no scopes.
- **Key expiry.** Deferred: revocation is the r2 control, and an expiry policy needs a clock in the
  authorisation path.
- **Being an MCP server itself.** The desktop app serves HTTP on loopback; the Bridge is what speaks
  MCP. Merging them is the deferred simplification in the spine.

## Prerequisite · [G3]

- `bundle` must exist and hold at least one Bundle. CAP-7 declares `depends_on: [CAP-4]`.
- The desktop app must be running for the Bridge to reach anything. When it is not, the Bridge answers
  `unavailable` rather than hanging — FR-21.
- The agent's MCP client must be configured with the Bridge executable. That configuration is the
  Reviewer's, done once, and it grants nothing on its own.
- Nothing external.

## Success Signal · [G3]

The Reviewer clicks *Copy access key*, pastes it into a coding agent, and the agent names the Findings
in a Bundle and describes what is in their images — with no Bundle content pasted by hand. Clicking
*Revoke* makes the agent's next call fail with a reason a person can act on. Before the paste, and
after the revoke, the agent can read nothing and says so rather than reporting an empty Library.

## Assumptions, Risks, and To Be Confirmed · [G3]

### Assumptions

- The Reviewer prefers pasting a key per session over the agent holding standing access — OQ-6.
- A coding agent handed Markdown with relative image references can fetch those images through the
  Bridge's image tool — OQ-1.

### Risks

- **The agent misreporting a refusal.** BR-17 is the mitigation and it is not sufficient on its own: a
  Bridge that maps a refusal to an empty tool result would satisfy the letter of the rule and break
  its purpose. The behaviour has to be tested from the agent's side, not the API's.
- **A key ending up on disk.** The Bridge receives the key through a tool call and must hold it in
  memory only. Any convenience that persists it — a cache file, a shell history, a log line — turns
  per-session access into standing access and makes revocation cosmetic.
- **Loopback is not a boundary on its own.** Any process on the machine can reach `127.0.0.1`. The key
  is the actual control, which is why NFR-9 requires it on every request rather than on a session.
- **Image bytes through MCP.** An agent fetching several images in one turn can consume more context
  than the Reviewer intended. The product has no way to stop it, and it is the strongest argument for
  the Quality Budget defaults being conservative.

### To Be Confirmed

- Whether the Local API and the Bridge should collapse into one process for MCP clients that speak
  HTTP directly. PRD open question 2, deferred in the spine.

## Gate Checklist · [G3]

| Question | Answer |
| --- | --- |
| ★ Is every use case title a sentence a user would say? | Yes. UC-18 is written in the agent's voice, which is the honest reading of a non-human actor's use case |
| ★ Any `FR` with no use case? | No. FR-19, FR-20, FR-21, FR-22 all have one |
| ★ Do the inventories and this catalogue describe one system? | Yes. Table 7, endpoints 1–8, screen 13 |
| Actor list: is one missing, or are two the same person? | Two actors, and they are genuinely different: one writes and one may only read |
| Does every `AD-N` here name a concrete failure that crosses components? | AD-5, AD-7, and AD-9 all do — each is shared with `sharing` |
| Which business rule am I not sure is right? | BR-14. Exposing only Bundles is right, and it means an agent cannot see work in progress, which the Reviewer may eventually want |
| Is there a term I have to guess the meaning of? | No |

## Design Reference · [G3]

Paired with `.how/agent-access/SDD-agent-access.md`.

Binding invariants: **AD-4** (no re-encoding on the way out), **AD-5** (every surface outside the
desktop process is read-only), **AD-7** (one error envelope), **AD-9** (one Bundle, one Markdown). No
applied `DEC-` binds this component yet.

---

## Slots

`02-rules/rules-agent-access.md` — written at G4, `mode: guarded`.
`03-domain/domain-model.md` — written at G3, present.
`04-usecases/` — at most three full flows at `guarded`, written at G4.
`05-scenarios/` — not written below `mode: deep`.

## Open Items

- OQ-1 — whether an agent can fetch the images a Bundle references.
  `.control/questions/assumptions.md`.
- OQ-6 — a per-session key over standing access. `.control/questions/assumptions.md`.
