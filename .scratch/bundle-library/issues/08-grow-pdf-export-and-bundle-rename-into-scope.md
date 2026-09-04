# 08: Grow the promises this map needs, from non-goals and gaps

**Type:** task
**Status:** resolved
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

## Added 2026-08-31 — two more promises, from ticket 02's grilling

Ticket 02 surfaced a Bundle lifecycle nobody had written down, and the owner chose to build it. Two
of its parts are **new promises** with no `FR-` behind them, so they belong in this same growth:

3. **Discard originals.** A Reviewer can destroy the source Findings behind a Bundle they consider
   final, reclaiming the disk their originals hold. The Bundle keeps its own burned copies and stays
   readable; it simply can no longer be disassembled. This is **not** forbidden by `BR-59`, which
   governs *composing* ("Composing does not remove the Findings it used from the Library") and stays
   true — a separate, later, explicit act removes them. But destroying captures from the Library is
   a capability the product does not currently promise at all.

4. **Reclaim space.** The surface that makes (3) usable in bulk: a list of Bundles still holding
   original captures, each with its size and a running total, reachable from the Library's header
   and from Settings' Vault section. Needs a screen-registry entry as well as its `FR-`.

**Open for the run to settle, not to guess here:** which `CAP-` (3) and (4) serve. `CAP-4` is
Bundles and is the nearest, but the act destroys Findings, so it may belong with `finding` or want a
capability of its own. `wdi-blueprint` territory if the answer is "a new one".

**The determination this ticket was asked for: yes, `wdi-product` intent `update` is required, for
all four.** Two of them (Export PDF, rename) are *explicit* non-goals and cannot be specced while
that stands. The other two are simply absent — and a capability that is absent still needs an `FR-`
before `/to-spec` can rest on it, since `/to-spec` must cite every requirement it uses by id. There
is no lighter door: `wdi-decision` is the wrong skill (its own routing table sends scope growth to
`wdi-product`), and `AGENTS.md` forbids stepping over a stated boundary on the grounds that the
corpus is "an input, not a gate" — that rule governs documents trailing *existing* code, and here
there is no code at all.

**One wave or several is the run's own call.** A PRD update normally lands several `FR-`s, and these
four are separable: Export PDF is a greenfield capability with its own effort, rename is one field,
and (3)+(4) are one storage-lifecycle story. Landing them as separate requirements is expected;
coupling their *fates* is not — if Export PDF stalls in review, rename and Discard originals must
still be able to proceed.

## Afterwards

- `DEC-` for the PDF exporter's packaging becomes writable — but see ticket 07: that decision is
  deliberately deferred to the Export PDF effort, because the research reversed itself twice on it.
- The editable title in [Prototype the Review & Update window](05-prototype-the-review-and-update-window.md)
  stops being out of bounds.

Do not run `wdi-product` without the owner present — repo rules require a go-ahead per skill, and
this one rewrites the PRD.

## Added 2026-08-31, second session — the growth list was incomplete, and it is not all `wdi-product`

Re-verified every claim above before asking for a go-ahead. All four hold verbatim, and `grep -rin
PDF .what/ .how/ .control/` still returns **0**. But the same read found a **fifth** boundary that
this map depends on far more heavily than the four listed, and it is a different *kind* of boundary:

**`BR-11` — "A Bundle is never edited in place. A change means composing a new Bundle."**
`.what/business-rules.md:32`, status **active**, touching `bundle` · `agent-access` · `sharing`,
sourced to `AD-9 · OQ-12`. Restated as a Non-Goal at `.what/bundle/SRS-bundle.md:76`: *"**Editing a
composed Bundle.** BR-11. A change means composing a new one."*

This is not an MVP scope line like the other four. It is an **active business rule derived from an
`AD-`**, and the entire Review & Update window —
[Prototype the Review & Update window](05-prototype-the-review-and-update-window.md), editable
Bundle title, Bundle notes, Finding notes, Marker notes, persisted via `update_bundle_markdown` — is
editing a composed Bundle in place. Ticket 08 as originally written would have grown the rename
field and left the window it sits in still forbidden.

### `AD-9`'s letter, read carefully, does not forbid the window — `BR-11` over-derives from it

`AD-9` (quoted at `.how/agent-access/SDD-agent-access.md:81`): *"A Bundle's Markdown MUST be composed
once, by the core, and stored. Every handoff path MUST serve those exact bytes. No surface may
re-render, re-order, decorate, or summarise a Bundle on the way out; a surface that needs a different
shape is asking for a change to the composer."*

Every clause governs the **way out**. Re-running the composer over edited inputs and re-storing the
result is not a surface re-rendering on the way out — and `AD-9`'s own last clause points at exactly
that remedy: *change the composer*. So the window is compatible with `AD-9` as written, and `BR-11`'s
absolute form ("never edited in place") is broader than the `AD-N` it cites. `BR-10` is untouched
either way and in fact **supports** this map's design: it says editing a *Finding* changes nothing in
a Bundle that holds it, which is the same invariant as "Update never touches a Finding".

That makes this a business-rule amendment, and the mandatory-`DEC-` test in `AGENTS.md` (a `DEC-` is
owed when an `AD-N` is contradicted) is arguably **not** tripped. Arguably is not good enough for a
rule this load-bearing, so the run must settle it rather than assume it — the safe reading is that
narrowing `BR-11` needs its own recorded decision even if `AD-9` survives intact.

`OQ-12`, `BR-11`'s other source, is an **assumption**, not a decision:
`.control/questions/assumptions.md:26` — *"Recomposing a bundle is acceptable in place of editing its
written Markdown"*, risk *"Bundles get edited outside Snapdown and drift from the library that
produced them"*, filed by `agent, G2`. This map's design is that assumption turning out false. It
should be **closed in place** by `wdi-question` as part of this growth, not left standing.

### Blast radius beyond `bundle`

`BR-11` names `agent-access` and `sharing` too, and `BR-65` (`.what/bundle/02-rules/rules-bundle.md:29`)
depends on it: *"Opening a Bundle shows what was composed, not a live view of the Findings as they
are now."* `BR-65` stays **true** under this map's design and must be preserved, not amended — the
window edits the Bundle's own stored copy, never re-reads the Findings. Say so explicitly so the next
reader does not amend it by reflex.

### `update_bundle_markdown` is dead code and does not license anything

Verified: its only callers are the trait declaration
(`crates/snapdown-core/src/ports/bundle_store.rs:11`), the impl
(`crates/snapdown-store/src/sqlite/bundle_store.rs:257`) and one store test
(`crates/snapdown-store/tests/test_sqlite_bundles.rs:83`). No production path reaches it. So the
"code wins over documents" rule does not apply here either: unreachable code is not behaviour, and
`BR-11` is a boundary still genuinely in force.

### Consequence for the route

`wdi-product` intent `update` remains right for the four promises. It is **not** sufficient for
`BR-11`: amending an active business rule and closing `OQ-12` are different doors. The run therefore
has three parts, and the owner must be present for all of them:

1. `wdi-product` intent `update` — the four promises (Export PDF, rename, Discard originals, Reclaim
   space), plus the `BR-11` amendment that makes the rename and the window legal.
2. `wdi-decision` — a `DEC-` recording that `BR-11` is narrowed to the handoff path, with `AD-9` left
   intact and the reasoning above as its body. Ordered **after** (1), because a `DEC-`'s `serves:`
   must name a `CAP-`.
3. `wdi-question` — close `OQ-12` in place, naming what replaced it.

**Still unresolved and not for this ticket to guess:** whether narrowing `BR-11` needs a `DEC-` at
all if `AD-9` survives unamended. Settle it in the run with the owner, in front of `AD-9`'s text.

## Progress 2026-08-31 — `wdi-product` intent `update` has run. Two of three doors remain.

Owner go-ahead given in session. The PRD growth landed; the ticket stays **claimed** rather than
resolved, because its own route named three skills and only the first has run.

### What landed

| Id | Promise | Capability | Component |
|---|---|---|---|
| `FR-39` | Export a Bundle as a PDF | **`CAP-12`** (new) | `bundle` |
| `FR-40` | Edit a composed Bundle's title and notes | `CAP-4` | `bundle` |
| `FR-41` | Discard the source Findings behind a Bundle, keeping the Bundle | `CAP-5` | **`finding`** |
| `FR-42` | See which Bundles still hold original captures, and reclaim their disk in bulk | `CAP-5` | **`finding`** |
| `NFR-19` | An exported PDF carries a real text layer, and no image in it is split across a page break | — | `bundle` |
| `OQ-31` | Is `BG-2` a close enough goal for `CAP-12`? | — | — |

Both § 6.2 non-goals were removed with their old wording quoted rather than dropped. § 4.10 *"Handing
a review to a person"* is a new feature section. `components.yaml` gives `bundle` its `CAP-12`.

### Three of this ticket's own guesses turned out wrong, and the corrections matter

1. **`CAP-` allocation.** This ticket left open *"which `CAP-` (3) and (4) serve"* and flagged
   `wdi-blueprint` if the answer was a new one. It is neither: **`CAP-5`** already reads *"Remove a
   Finding together with its file, and know when something is orphaned"* and fits exactly. Filing
   them under `CAP-4`/`bundle` as this ticket's text leaned toward would have been the precise V21
   collision that check exists for — `bundle` owns only `[Bundle, BundleItem]`, and the act destroys
   `Finding`. No `wdi-blueprint` escalation was needed for this.
2. **Export PDF needed a new `CAP-` after all**, and `wdi-product` may birth one — precedent is
   `CAP-9` through `CAP-11`, born at the G2 re-run of 2026-08-23. `CAP-12` was *not* folded into
   `CAP-4`, whose title names Markdown deliberately because `AD-9`'s one-set-of-bytes invariant rests
   on it.
3. **`BG-2` is a known-imperfect fit for `CAP-12`** and no existing goal is better. Its measure is
   handoff *time* and it promises *"no file management"*; a PDF is a file to manage, read by a person.
   Assigned rather than invented — birthing a `BG-` is a G1 act — and `OQ-31` holds it. `wdi-problem`
   may owe `BG-2` a measure amendment.

### Two promises already in the PRD pointed the opposite way, and neither this ticket nor the map knew

Both were corrected on the owner's decision in the same session, with the old wording quoted in place:

- **`FR-12` vs ticket 03.** Its consequence read *"The image references in the copied text are the
  same relative paths as in the file"*, and its proof ended *"the Bundle's complete Markdown,
  **unchanged**"*. That fixed the clipboard to folder-relative links, so ticket 03's absolute-path
  answer contradicted an existing promise — a harder objection than the `AD-9` one already recorded
  there. `FR-12` now **permits** an absolute rendering and deliberately settles none of ticket 03's
  three open items.
- **`FR-14` vs ticket 02.** Its last consequence read *"The Reviewer can choose, in the same
  confirmation, to delete the Bundle's source Findings too"* — exactly the one-click combined
  destroy ticket 02 ruled out. Withdrawn; `FR-14` and `FR-41` are now two deliberate acts. The
  withdrawn line also claimed a `Finding` write `FR-14`'s own registry row never authorised, so it
  had been promising past its component boundary since G2.

§ 8's open question 4 (*"Should composing a Bundle offer to delete the Findings it consumed…"*) is
answered and struck through: its premise was `FR-14`'s withdrawn line.

### Still owed, and each needs its own go-ahead

1. **`wdi-blueprint`** — two jobs, and this is the blocker. **(a)** Narrow `BR-11` (*"A Bundle is
   never edited in place"*, status active) to the handoff path; it is a cross-component rule, so it is
   not `wdi-product`'s to touch, and **G2 MUST NOT open on `FR-40` until it lands**. `AD-9` stays
   intact; `BR-10`, `BR-59` and `BR-65` all stay true and must not be amended by reflex. **(b)** Four
   use cases, one per new `FR-`. `validate.py` reports exactly four new `V2` findings for this and
   they are correct — all four are genuine use cases with an actor and an initiating step, so
   `no_uc:` would be a lie and was not written.
2. **`wdi-decision`** — the `DEC-` for the `BR-11` narrowing. Now writable: `serves:` can name
   `CAP-4`. It should also carry the `FR-12` reading, because that amendment rests on the same
   `AD-9` question from the opposite direction.
3. **`wdi-question`** — close `OQ-12` in place. § 4.4 and § 9 both record it as withdrawn by `FR-40`;
   the registry row itself is still open.
4. **`wdi-component`** intent `behaviour` on `bundle` — `SRS-bundle.md`'s Non-Goals at lines 76, 77
   and 84 still state all three boundaries. Note its rename entry adds *"and rewriting it contradicts
   `BR-10`"*, which is **wrong on its own terms**: `BR-10` governs a Finding's edits not propagating
   *into* a Bundle, not a Bundle rewriting its own heading. Correct it as a stated correction.
5. **`wdi-ux`** — a screen-registry entry for `FR-42`'s reclaim surface.

### One thing to fix outside this map

`DEC-005` (applied) forbids new `FR-` for `sharing` and `agent-access`. All four new `FR-`s are
`bundle`/`finding`, which is the direction `DEC-005` *sends* work, so there is no conflict. But
`BR-11` names `sharing` and `agent-access` in its `touches:`, so `wdi-blueprint` must narrow it in a
way that leaves both components' behaviour verbatim — they still serve the exact stored bytes. That
is what the handoff-path narrowing does, and it must be said explicitly in the amendment.

## Progress 2026-08-31 — `wdi-blueprint` intent `catalog` has run. `FR-40` is legal now.

**`BR-11` is narrowed.** It used to read *"A Bundle is never edited in place. A change means composing
a new Bundle"*; it now reads *"A Bundle's stored document is changed only by the composer writing it
again over the Bundle's own copy. No surface edits a Bundle's document directly, and no change to a
Bundle ever reads or writes a Finding."* Source moves from `AD-9 · OQ-12` to `AD-9 · FR-40`. **`AD-9`
was not amended and did not need to be.** `.what/business-rules.md` gains an `## Amended` section
holding the old wording and the reasoning, on the same principle as its `## Retired` section.

The blocker on `FR-40` is therefore lifted. `BR-10`, `BR-59` and `BR-65` were each checked and are
each still true; the amendment says so explicitly, because the next reader's instinct is to tidy them
into line. `sharing` and `agent-access` keep their obligation verbatim, so `DEC-005` is not breached —
the narrowing removes a prohibition that only ever bit `bundle`, the one component that runs the
composer.

**Four use cases exist.** `validate.py`'s four `V2` findings are gone.

| `UC-` | Use case | Component | `critical` |
|---|---|---|---|
| `UC-28` | I turn a review into something I can send to a person who does not have Snapdown | `bundle` | no |
| `UC-29` | I fix a typo in a review I have already put together | `bundle` | no |
| `UC-30` | I get rid of the screenshots behind a review I am finished with, but keep the review | `finding` | **yes** |
| `UC-31` | I find out which reviews are still holding my screenshots, and get that disk back | `finding` | **yes** |

`UC-28` and `UC-29` are deliberately not `critical` and the derivation is written into both the
registry and the catalogue, because both read like candidates. Marking either one would have put half
of `bundle`'s use cases under the label, which `delivery-flow-guide.md` treats as a signal to derive
again.

Also landed: `BR-122` for the sealed state, read from whether the Findings exist rather than from a
flag; `bundle`'s `domain-model.md` State Lifecycle and invariants 4 and 10; both SRS Actor Registers;
the glossary's `Bundle` entry and a new `Sealed` entry; four fossil `V3` lines added to
`.github/validate-baseline.txt` as `AGENTS.md` permits; `.how-rendered/blueprint.md` regenerated.

### One thing was deliberately NOT fixed, and it is the same `AD-9` question from ticket 03's side

`SRS-bundle.md`'s Decision Summary still claims *"the clipboard, the Local API, and a published page
all serve identical bytes — three handoff paths, one document."* `FR-12`'s amendment permits the
clipboard to render absolute image links, so that sentence is now false for one of the three.

Correcting it means deciding whether `FR-12` contradicts `AD-9`, and `wdi-blueprint` must not edit an
`AD-N` or decide one. **`BR-11`'s new wording was written without restating `AD-9`'s byte-identity
clause on purpose**, so that this question is not frozen by accident — the old rule was doing two jobs
and only the first has been rewritten.

This is now the sharpest open item on the map, and it sits across two tickets: ticket 03 owns the
clipboard decision, and `FR-12` has already been amended on the assumption that going through the
composer satisfies `AD-9`. If that reading fails, `FR-12` needs re-amending, not just a `DEC-`.

### Still owed

1. **`wdi-decision`** — one `DEC-` covering both halves of the `AD-9` reading: the `BR-11` narrowing
   (settled, needs recording) and the `FR-12` clipboard question (**not** settled, and it is the one
   that could reverse a promise already written).
2. **`wdi-question`** — close `OQ-12` in place.
3. **`wdi-review`** — five corpus files changed and `wdi-review` was not dispatched, because repo
   rules forbid invoking a skill without a go-ahead. `SRS-bundle.md`, `SRS-finding.md`,
   `business-rules.md`, `domain-model.md` and `product-glossary.md` all now sit ahead of their
   `reviewed:` trace. The G3 delta should not be treated as gated until this runs.
4. **`wdi-component`** intent `behaviour` on `bundle` — `SRS-bundle.md`'s Non-Goals at lines 76, 77
   and 84, plus its wrong `BR-10` citation on the rename entry.
5. **`wdi-ux`** — the screen-registry entry for `FR-42`'s reclaim surface.

## Answer

**All four promises exist, and the whole route ran.** Five skills, in one chain on 2026-08-31:
`wdi-product` intent `update` → `wdi-blueprint` intent `catalog` → `wdi-decision` → `wdi-component`
intents `behaviour` and `design` → `wdi-blueprint` intent `platform` → `wdi-question` → `wdi-review`.

`CAP-12` · `FR-39` Export PDF · `FR-40` edit a composed Bundle's title and notes · `FR-41` discard the
source Findings · `FR-42` reclaim space in bulk · `NFR-19` · `UC-28`–`UC-31` · `BR-122` · `OQ-31`.
`BR-11` narrowed, `OQ-12` closed, `DEC-012` applied. Committed as `57cbf96`, both CI workflows green.

**Three of this ticket's own guesses were wrong, and the corrections are the part worth keeping.**

1. `FR-41` and `FR-42` belong to **`CAP-5` / `component: finding`**, not `CAP-4` / `bundle` as this
   ticket leaned. The act destroys a `Finding` and `bundle` owns only `[Bundle, BundleItem]` — filing
   them under `bundle` would have been the exact collision `V21` exists to catch. No `wdi-blueprint`
   escalation was needed for the capability question after all.
2. **Export PDF did need a new `CAP-`**, and `wdi-product` may birth one — `CAP-9`–`CAP-11` set the
   precedent at the G2 re-run of 2026-08-23. `CAP-12` was not folded into `CAP-4`, whose title names
   Markdown deliberately.
3. **A fifth boundary existed that this ticket never listed, and it was the load-bearing one.**
   `BR-11` — *"A Bundle is never edited in place"*, status active, sourced to `AD-9` — forbade the
   **whole** Review & Update window, not just its title. Growing the rename while leaving `BR-11`
   standing would have produced a promise nobody could legally build.

**Two promises already in the PRD pointed the opposite way, and neither this ticket nor the map knew.**
`FR-12` fixed the clipboard to folder-relative image links, contradicting ticket 03's answer more
directly than the `AD-9` objection did; and `FR-14` offered to destroy a Bundle's source Findings in
the same confirmation, which ticket 02 had ruled out. Both corrected on the owner's decision, with
their old wording quoted in place.

**What the code said, and it reshaped the `AD-9` question.** Of the three handoff paths `AD-9` governs,
exactly one exists: `FR-12` has no implementation at all, the Local API does not exist (`BUG-59`), and
only the published page runs. `DEC-012` records that, and records plainly that the code **prices** the
decision rather than answering it — `AD-9`'s own Prevents is what answered it.

**Still owed, and none of it blocks `/to-spec` on the Library itself:**

- `NFR-19` forbids splitting an image across a page break while ticket 07 solves tall screenshots by
  slicing across pages. **A contradiction introduced by this chain.** → `wdi-product`.
- Editing a **published** Bundle is undefined anywhere in the corpus. → `wdi-question`.
- Discarding one Bundle's originals silently seals another that shares a Finding (`BR-12` + `BR-122`),
  and `FR-41`'s confirmation does not say so. → `wdi-product`.
- `FR-40` writes both the `bundle.markdown` column and the `bundle.md` file with no atomicity stated;
  `BR-5` covers create and remove, not update. → `wdi-blueprint` intent `catalog`.
- `FR-39` needs a real design pass at `guarded`: it adds a boundary — a file written outside the Vault
  — that § Failure Behaviour does not cover. → `wdi-component` intent `design`.
- `DEC-013` is `draft` and awaits the owner. Nothing is stamped: `wdi-review` reported open findings
  and its own rule forbids a trace over them.
