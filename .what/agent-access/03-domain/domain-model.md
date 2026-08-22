---
type: model
component: agent-access
layer: conceptual
created: "2026-08-22"
updated: "2026-08-22"
---

# Model — agent-access

Conceptual. No column types, no storage shape.

## Entities

| Entity | What it is | Identified by |
| --- | --- | --- |
| AccessKey | The Reviewer's grant of read access to an agent on this machine: a secret they copy out and paste in, and can take back | Its own opaque identifier |

One entity, and that is the whole domain of this component. Everything else it does is a read of
`bundle`'s entities across a process boundary, which is a surface rather than a thing.

## Relationships

- One Library has **at most one** AccessKey that is currently valid. Issuing a new one ends the
  previous one in the same act (BR-16).
- An AccessKey grants reads over **every** Bundle in the Library, and over nothing else. It is not
  scoped to a Bundle, and there is no per-Bundle or per-agent key.
- An AccessKey has **no** relationship to a Finding. BR-14 keeps unbundled Findings invisible on this
  surface, so the key cannot reach one.
- The MCP Bridge holds a copy of the key for the life of one process and stores nothing. There is no
  entity for that copy, deliberately: anything with a lifetime longer than the process would make
  revocation eventual instead of immediate (AD-5).

## State Lifecycle

An AccessKey has two states and one transition, and the transition is one-way.

| From | To | Trigger | Who may |
| --- | --- | --- | --- |
| — | valid | The Reviewer issues a key. Any key already valid becomes revoked in the same act | Reviewer |
| valid | revoked | The Reviewer revokes it, or issues a new one | Reviewer |

A revoked key never becomes valid again. There is no renewal and no re-activation: the only way
forward is a new key, which is what makes "exactly one valid at a time" a statement about the present
rather than a rule someone has to maintain.

No state can be entered and not left, and there is no expiry — revocation is the r2 control, and an
expiry transition would need a clock inside the authorisation path.

## Invariants

1. At most one AccessKey is valid at any moment. → BR-16
2. Issuing a key and revoking the previous one are one operation. There is no instant with two valid
   keys, and none with zero when the Reviewer asked for one. → BR-16
3. A revoked key never becomes valid again. → FR-22
4. A revoke takes effect on the next request. No cache, no session, no grace period. → NFR-13
5. The key itself is never stored in the Library. Only a hash of it is kept, and the key lives in the
   operating system's credential store. → cross-cutting.md § Secrets
6. The key never appears in a log line, a Bundle, a published document, or this repository.
   → cross-cutting.md § Secrets, § Logging
7. Every read this key authorises is over a Bundle. No Finding, Note, Marker, or Setting is reachable
   with it. → BR-14
8. Nothing this key authorises writes. → BR-15, AD-5
9. A request with no key and a request with a revoked key are refused with different codes, and both
   are distinguishable from an empty result. → BR-17, AD-7
