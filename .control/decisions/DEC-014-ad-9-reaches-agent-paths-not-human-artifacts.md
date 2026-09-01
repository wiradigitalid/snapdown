---
type: decision
id: DEC-014
status: accepted
serves: [CAP-12]
touches:
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .control/questions/answered.md
supersedes: null
superseded_by: null
created: "2026-09-01"
---

# DEC-014 — `AD-9` reaches the paths an agent reads, not an artifact made for a person

> **Accepted by the owner on 2026-09-01, by answering `OQ-34`.** They were shown `AD-9`'s Rule and its
> Prevents side by side, the three answers available, and what each cost — including that the third,
> leaving `AD-9` to reach a PDF unnarrowed, ends `FR-39` in the shape ticket 07 researched. They chose
> this one. The agent did **not** accept its own decision; what the agent got wrong is recorded under
> § Provenance, because it argued at the time that this answer needed no `DEC-` at all.

## Decision

`AD-9` governs the handoff paths **an agent reads** — the clipboard, MCP, and the published web copy.
It does **not** reach an artifact produced for a person to read away from Snapdown, and a PDF export
(`CAP-12`, `FR-39`) is such an artifact.

The boundary MUST be written into `AD-9` itself rather than held as an understanding. That is the
condition the owner attached to the answer, and it is the whole material effect of this decision.

Nothing else about `AD-9` moves. Its **Binds**, its **Prevents**, and every clause of the Rule that
governs an agent path stand exactly as `DEC-012` left them.

## Why

`AD-9`'s two halves stopped agreeing, and the disagreement was invisible until the spine's missing
`CAP-12` row was filled on 2026-08-31.

The **Rule** forbids a surface *re-rendering* a Bundle on the way out, in those words. A PDF is a
re-rendering by any ordinary reading, so the Rule alone forbids `FR-39` outright.

The **Prevents** names the harm precisely: *"the clipboard, MCP, and web paths drifting into three
renderings of one Bundle, so that two agents reading the same review disagree about it and nobody can
say which is right."* Every noun in that sentence is an agent path, and the harm is **two agents
disagreeing**. A PDF cannot cause it. Nothing reads a PDF back into a review; the agent path is
`Copy Markdown`, and it is untouched.

**The precedent is the argument, not a convenience.** `DEC-012` faced the same shape ten days earlier
— a Rule that over-claimed relative to its own Prevents — and settled it by letting Prevents decide.
Deciding the same way twice is consistency; deciding it the other way here would mean `AD-9`'s
Prevents governs when it is convenient and its Rule governs when it is not.

**Timing made it cheap.** The exporter is unwritten: no crate, no code, not one line. Answering after
it existed would have meant either undoing work or granting an exception to code already shipped,
which is how an invariant quietly stops being one.

## Cost

**`AD-9` no longer says anything about a PDF, and that is a real loss, not a technicality.** If Export
PDF ever drifts — reordering Findings, summarising notes, dropping an image — no `AD-N` forbids it.
The document a person reads and the document an agent reads could diverge without breaking any
invariant in the spine.

That exposure is accepted for a reason that is worth stating rather than assuming: the two documents
are produced by the same composer from the same stored Markdown, so divergence needs a deliberate act
to create. If `FR-39`'s design ever introduces a second authoring path, this decision's premise is
gone and it MUST be revisited — see § Reversal trigger.

**A second cost, smaller and certain.** `AD-9` is now an invariant with a stated exception, and an
invariant with an exception is harder to reason about than one without. The exception is written into
the `AD-N` precisely so that the cost is paid once, in the reading, rather than repeatedly by every
reader who checks the Rule and reaches the wrong conclusion.

## Alternatives

**Narrow `AD-9` a second time so a PDF becomes legal under it.** Rejected as the more honest-looking
answer that is actually less honest. It would have `AD-9` govern a path whose failure mode it cannot
describe: the Prevents would have to be widened to name a harm — a person misled by a bad PDF — that
has nothing to do with two agents disagreeing, and an invariant that prevents two unrelated things
prevents neither well.

**Leave `AD-9` reaching PDF, unnarrowed.** Rejected by the owner. It ends `FR-39` in the shape ticket
07 researched, and Export PDF would have to be redesigned as something that is not a rendering —
which, for a PDF, is close to a contradiction in terms.

**Answer it in the `CAP-12` row of the capability map instead of in `AD-9`.** Rejected on the spine's
own rule: an `AD-N`'s scope is not settled by a table that cites it. That row already says so in its
own words — *"It MUST NOT be answered by writing an `AD-N` into this row."*

## Reversal trigger

Any of these, and this decision MUST be re-opened rather than worked around:

- `FR-39`'s design introduces a **second authoring path** — anything that composes a PDF's content
  from the Findings rather than from the Bundle's stored Markdown. The premise under § Cost is then
  false and the exposure is no longer bounded.
- An **agent** is given the PDF as an input, by any surface. The Prevents' harm becomes reachable and
  the boundary drawn here is in the wrong place.
- A second human-readable export is added. One exception is a boundary; two is a pattern, and a
  pattern belongs in the Rule rather than in a list of exceptions.

## Provenance — the agent argued this needed no decision, and was wrong

When the three options were put to the owner, this one was presented as costing **no `DEC-`** — *"cukup
satu kalimat di `AD-9` yang menyebut batas jangkauannya"* — on the reasoning that `AD-9` had never
reached a PDF, so stating the boundary was a clarification rather than a change.

That reasoning does not survive contact with this project's own rules. Recording is optional
everywhere here **except** for a decision that contradicts an `AD-N`, and the distinction between
*narrowing* an invariant and *clarifying* that it never reached something is exactly the distinction
a reader cannot check a year later. `DEC-012` settles it by example: it recorded the identical move
rather than editing the spine in place, and it did so because editing an `AD-N` directly is how a
reversal happens with nobody deciding it.

The owner's answer is unchanged by this. What changed is that it lands as a decision with its cost
and its reversal trigger written down, instead of as a sentence appearing in the spine one day with
no author.

## Trace

`OQ-34`, raised 2026-08-31 by `wdi-review` while filling the spine's missing `CAP-12` row; answered
by the owner 2026-09-01 and closed into `.control/questions/answered.md`.

`FR-39` was blocked from being specced while this was open. That block lifts here.
