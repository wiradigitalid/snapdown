---
type: decision
id: DEC-021
status: accepted
touches: []
supersedes: null
superseded_by: null
created: "2026-09-05"
accepted_by: kodesh87, 2026-09-05
---

# DEC-021 — The six `refs-resolve`/`memlog-home` findings in `.github/validate-baseline.txt` are a permanent, accepted fossil

## Decision

The five `refs-resolve` findings against `DEC-001`, `DEC-002`, `DEC-005`, `DEC-007`, `DEC-013` (each
pointing to `CAP-7`, retired by `DEC-016`) and the one `memlog-home` finding against
`.control/memlog/agent-access.md` (its `artifact:` pointing to `.how/agent-access/SDD-agent-access.md`,
also deleted by `DEC-016`) MUST NOT be closed by editing those five applied/superseded `DEC-`s, and MUST
NOT be closed by rewriting that memlog. They stay red in `validate.py`, and stay listed in
`.github/validate-baseline.txt`, permanently — an accepted, correct state, not a backlog item.

## Why

`decision-guide.md` forbids both routes a "fix" would need: an applied (or superseded) `DEC-` MUST NOT
be edited except to record its own supersession, and a memlog is a run log — a record of what a session
did and why — that MUST NOT be rewritten to match the present. `DEC-001`, `DEC-002`, `DEC-005`, `DEC-007`
and `DEC-013` each cite `CAP-7` as a true fact about what they served *at the time they were written*;
editing that out to satisfy a validator that only exists to catch dangling references would falsify
history to please a tool built to protect it. The memlog is the same shape: it is `agent-access`'s own
run log, and `DEC-016` deleting the SDD it names is exactly what a memlog is supposed to survive
unedited.

`AGENTS.md` has carried this reasoning in prose since `DEC-016` (2026-09-04) and named its own gap in
writing: *"This boundary is not yet recorded as a `DEC-`. It SHOULD be, through `wdi-decision`."* This
decision closes that gap. The owner, asked in chat on 2026-09-05 whether "settle the six red
validators" meant recording this boundary or a structural fix (resurrecting tombstone/retired registry
entries so the references genuinely resolve), asked instead to follow whatever `wdi-method` itself
already prescribes — which is this: a recorded `DEC-`, not a structural change.

## Cost

- **The corpus is never fully green again**, and every future `wdi-autopilot` preflight's "validators
  green" check has to keep reading these six against the baseline rather than expecting zero red. That
  reading is now this `DEC-` rather than three paragraphs of `AGENTS.md`.
- **A seventh finding of the same shape — another retired capability or deleted document an applied
  `DEC-` or a memlog still names — is not automatically covered by this decision.** Each such case is
  its own instance of the same reasoning, added to the baseline citing this `DEC-` as precedent, the way
  `AGENTS.md` already describes for a new `V3` line.
- **A reader who trusts a green `validate.py --check` over the baseline file will misread this repo's
  state.** The baseline file, not a zero count, is what "clean" means here now.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | — |
| Source material | `AGENTS.md` § "Three validator checks are now fossils" (2026-09-04, updated by `DEC-016`); `.constitution/method/document/decision-guide.md`; the owner's chat instruction, 2026-09-05, to follow `wdi-method`'s own prescribed route rather than choose between the two options presented |
