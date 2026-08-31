# 08: Grow Export PDF and Bundle rename from non-goals into promises

**Type:** task
**Status:** open
**Blocked by:** None (can start immediately)

## Question

Two things this map depends on are **explicit MVP non-goals** in the corpus. The owner decided on
2026-08-31 to grow both into scope rather than drop them or defer to r2. This ticket is that growth.

**What is written today** (verified verbatim; the word "PDF" appears **zero** times anywhere in
`.what/`, `.how/` or `.control/`):

- `.what/_prd/capture-to-markdown/prd.md:723` — *"Exporting a Bundle to anything but Markdown."*
- `.what/bundle/SRS-bundle.md:84` — *"**Exporting to anything but Markdown.**"*
- `.what/_prd/capture-to-markdown/prd.md:717` — *"Renaming a Bundle. Same reason; and a rename that
  does not rewrite the document's heading is a lie."*

**Why this cannot be stepped over.** `AGENTS.md` says the corpus is an input and that code wins over
documents — but that rule governs a document trailing *existing* code. There is no code for either
feature. These are deliberate scope boundaries still in force, so they are grown through the front
door. Both are written as **MVP/r1** boundaries rather than permanent bans (a sibling entry,
*"Searching or filtering the Library"*, even carries `[NOTE FOR PM] … revisit for r2`), so growth is
ordinary rather than exceptional.

**The route is `wdi-product` intent `update`.** It turns a non-goal into a promise and produces the
`FR-`. A `DEC-` cannot come first: its `serves:` field must name a `CAP-`, and no existing `CAP-`
covers export — CAP-4 is Bundles, the nearest but not this. Whether export needs a new `CAP-` is a
`wdi-blueprint` question that may surface here.

## What the growth has to say

**Export PDF.** A Bundle can be rendered to a PDF for a human to read and share. Everything needed
to write the `FR-` is already established in
[Research the PDF render engine](07-research-the-pdf-render-engine.md) — engine, licences, measured
costs, image handling, escaping — but keep the promise free of implementation: the `FR-` states what
the Reviewer gets, not that it is typst.

**Bundle rename.** The Bundle's title can be edited after composition. Note the PRD's stated
objection is **already satisfied** by this map's design and should be recorded as such: editing the
title block in the Review & Update window *does* rewrite the document's heading, so the "a rename
that does not rewrite the heading is a lie" concern does not apply. Only the scope boundary did.

## Afterwards

- `DEC-` for the PDF exporter's packaging becomes writable — but see ticket 07: that decision is
  deliberately deferred to the Export PDF effort, because the research reversed itself twice on it.
- The editable title in [Prototype the Review & Update window](05-prototype-the-review-and-update-window.md)
  stops being out of bounds.

Do not run `wdi-product` without the owner present — repo rules require a go-ahead per skill, and
this one rewrites the PRD.
