---
type: decision
id: DEC-017
status: superseded
touches: []
supersedes: null
superseded_by: DEC-022
created: "2026-09-04"
---

# DEC-017 — An unattended run holds the owner's authority over every human checkpoint in the method

## Decision

In a session the owner opened with the unattended-run prompt, the agent acts with the Product Owner's
authority at every point where WDI Method or a wrapped BMad skill would otherwise stop for a person:
it accepts and applies a `DEC-`, edits an `AD-N`, sets `mode` and `risk_accepted`, answers a skill's
interview or check-in, and reports a gate — without asking. Every such act is logged as one row in
`.scratch/unattended/owner-questions.md`, and the owner reverses any of them afterwards by the method's
normal means: a superseding `DEC-`, an edited document, a changed registry row.

## Why

The owner asked for it, in chat, on 2026-09-04: *"sebenarnya semua wdi-method agar unattended oleh
agent, apa bisa dianulir, agar diberikan otoritas"* — after first asking that the whole backlog be built
*"unattended ... jangan meminta respon-respon dari manusia ... kalau ada concern, catat, dan putuskan
sendiri, saya ingin review hasil akhirnya"*. The method's checkpoints exist so that a human makes each
call; the owner has chosen to make those calls **after** the work, from a review queue, instead of
during it, and to review the finished application rather than each decision. This decision records
that choice so that no rule of the method has to be patched in the package-owned tree to honour it.

## Cost

- **Ratification comes after the fact.** A `DEC-` the run accepts and applies is frozen before the
  owner has read it. Reversing it costs a superseding `DEC-` and a second edit of every document it
  touched, where a rejection at `draft` would have cost nothing.
- **The corpus can be written to match what was built.** Documents already trail the code here by
  rule; under this decision they are also *ratified* by the same party that wrote the code. The review
  queue is the only place that difference is visible, so a run that forgets to log a row hides a
  decision entirely.
- **`risk_accepted_by` names a run, not a person.** `high-risk-named` still passes, but the name it
  finds is `unattended run under DEC-017, <date>` — the owner has to read the queue to know a risk was
  accepted on their behalf.
- **This holds for one kind of session only.** A session opened any other way keeps every checkpoint;
  a run has to be able to point at the prompt it was opened with.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | — |
| Source material | the owner's chat instructions, 2026-09-04, quoted above; `AGENTS.md § Unattended runs`; `.constitution/project/unattended-authority.md` |
