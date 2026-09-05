---
type: decision
id: DEC-022
status: accepted
touches: []
supersedes: DEC-017
superseded_by: null
created: "2026-09-05"
accepted_by: kodesh87, 2026-09-05
---

# DEC-022 — Unattended authority is held through a `wdi-autopilot` mandate, not a generic "unattended-run" session type

## Decision

The owner's authority over the method's human checkpoints during an unattended run is held **only**
through an accepted `wdi-autopilot` mandate (`type: mandate` in `decisions.yaml`, its own
`accepted_by`/`expires`/`scope`/`parked`, one ledger per mandate at
`.control/memlog/autopilot-<mandate-id>.md`). This supersedes `DEC-017`'s broader "a session the owner
opened with the unattended-run prompt" framing and its `.scratch/unattended/owner-questions.md` log
path — neither the generic session type nor that log path was ever built.

## Why

`DEC-017` (2026-09-04) authorized "an unattended run" as a session-level property, logging each
owner-delegated act as a row in `.scratch/unattended/owner-questions.md`, and cited
`.constitution/project/unattended-authority.md` as its source. Neither path exists anywhere in this
repository — they were never built. What was built and has since run to completion twice, both
`applied` (`DEC-018`, then `DEC-019`), is `wdi-autopilot`'s own mechanism: a mandate accepted once by
the owner by name and date, with its own scope and parked list and expiry, and a per-mandate ledger the
owner reviews in parallel rather than a shared cross-session log file. This is a planning assumption
that turned out void in the way `wdi-decision` exists for — the document named a mechanism that was
never the one the code ended up building, and the mechanism that shipped and proved itself is what the
standing authority decision should actually name. The owner, asked in chat on 2026-09-05 for the
direction of this supersession (renew/strengthen `DEC-017`'s authority vs. narrow/revoke it), asked to
follow `wdi-method`'s own guidance rather than choose a side — read here as: correct the record to what
actually shipped, not a change in how much authority a mandate holds.

## Cost

- **`DEC-017` stays on record as `superseded`, pointing here** — its own text is not edited, per
  `decision-guide.md`; anyone who lands on it next reads `superseded_by: DEC-022` and comes here for the
  mechanism that actually governs an unattended run today.
- **A future unattended-run mechanism other than a `wdi-autopilot` mandate has no standing authority of
  its own.** If the method ever adds a second unattended shape, it needs its own `DEC-`, the same way
  this one replaces `DEC-017` rather than being read as covering it by extension.
- **This does not change anything about how a mandate is opened, accepted, or run** — `wdi-autopilot`'s
  own Door 1/Door 2 mechanics, `mandate-accept`'s validation chain, and the ledger shape are unchanged.
  This decision only retires the never-built alternative `DEC-017` named and points the standing
  authority at the mechanism that replaced it.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | — |
| Source material | `DEC-017` (2026-09-04, being superseded); `DEC-018` and `DEC-019` (both `applied`, the two runs proving the mandate mechanism); `wdi-autopilot`'s own `SKILL.md`; the owner's chat instruction, 2026-09-05, to follow `wdi-method`'s own guidance on the direction of the supersession |
