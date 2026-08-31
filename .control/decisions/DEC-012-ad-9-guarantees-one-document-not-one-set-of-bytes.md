---
type: decision
id: DEC-012
status: applied
serves: [CAP-4]
touches:
  # Half 1 (the BR-11 narrowing) landed in these two BEFORE this decision was written, on the same
  # day. That ordering is irregular and is recorded rather than tidied: the narrowing was carried out
  # by wdi-blueprint intent `catalog` and wdi-product intent `update` while this decision was still
  # being reasoned out, and it is what surfaced the AD-9 question that Half 2 settles.
  - .what/business-rules.md
  - .what/_prd/capture-to-markdown/prd.md
  # Applied in layer order, .what/ before .how/
  - .what/bundle/SRS-bundle.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/cross-cutting.md
  - .how/_platform/inventory-api.md
  - .how/_platform/inventory-db.md
  - .how/bundle/SDD-bundle.md
  - .how/agent-access/SDD-agent-access.md
  - .control/questions/assumptions.md
  - .control/questions/answered.md
supersedes: null
superseded_by: null
created: "2026-08-31"
---

# DEC-012 — `AD-9` guarantees one authored document, not one set of bytes

## Decision

`AD-9` guarantees that every handoff path serves the **same authored document**: a path MAY substitute
the base of a Bundle's image links so that they resolve for its own reader, and MUST NOT change
anything else — no re-ordering, no decoration, no summarising, no editing of a single character the
composer wrote. The substitution is made **by the composer**, taking the base path as a parameter;
a surface MUST NOT rewrite a document the composer has already produced.

Two consequences follow, and both were written before this decision was recorded:

1. **`FR-12` stands.** The clipboard may carry absolute image links. It is not a breach of `AD-9`.
2. **`BR-11`'s narrowing stands.** A Bundle's stored document is changed only by the composer writing
   it again over the Bundle's own copy. Editing a Bundle's title or its notes is that, not a surface
   editing a document in place.

`AD-9`'s **Rule** and its **title** are narrowed to match. Its **Prevents** is unchanged, because the
Prevents was already correct and is what settled this.

## Why

**`AD-9` contradicts itself, and its Prevents is the half that is right.** The spine entry
(`.how/_platform/ARCHITECTURE-SPINE.md:149`) reads:

> **AD-9 — One Bundle, one Markdown, byte-identical on every path**
> **Prevents:** the clipboard, MCP, and web paths drifting into three renderings of one Bundle, so
> that two agents reading the same review disagree about it and nobody can say which is right.
> **Rule:** A Bundle's Markdown MUST be composed once, by the core, and stored. Every handoff path
> MUST serve those exact bytes. No surface may re-render, re-order, decorate, or summarise a Bundle
> on the way out; a surface that needs a different shape is asking for a change to the composer.

The named harm is **two readers disagreeing about the same review**. Substituting the base of an image
link does not produce that harm: the words are identical, the order is identical, the images are the
same images, and the only thing that differs is where each reader is told to find a file — which
differs precisely so that it resolves at all. Two agents handed the two renderings agree about the
review completely.

So the Rule's phrase *"those exact bytes"* is stricter than the Prevents needs, and strictly enough to
forbid the one thing that makes a link usable. The title carries the same over-claim. This decision
corrects the Rule to the invariant the Prevents already states.

**The Rule also prescribes the remedy taken.** Its closing clause reads *"a surface that needs a
different shape is asking for a change to the composer"* — which is exactly the route both consequences
use. `FR-12` renders through the composer with a base-path parameter, not around it. `FR-40` re-runs
the composer over edited inputs. Neither is a surface rewriting a finished document, which is what the
middle clause forbids and what stays forbidden.

**What the code says, and what it does not.** `AD-9` speaks of "every handoff path", and
`SRS-bundle.md` names three — the clipboard, the Local API, and a published page. Verified 2026-08-31:
**exactly one of the three exists.**

- The clipboard-Markdown path has **no code**. Every `clipboard-win` use in the tree is
  `raw::set_bitmap_with` for images, serving `FR-36`; there is no text clipboard call anywhere, and
  `apps/desktop/ui/appwindow.slint` has no copy-markdown callback, only a `bundle-preview-markdown`
  display property at `:1342`. `FR-12` is a promise with no implementation.
- The Local API path has **no code**. `BUG-59` says so in its own title — *"The Local API does not
  exist, so the MCP Bridge cannot reach the product at all"* — and
  `crates/snapdown-bridge/src/client.rs:27` builds `http://127.0.0.1:{port}` against a server that was
  never rebuilt after the move to Slint. No bind or `TcpListener` exists in `apps/desktop/src/`.
- The published-page path **exists** and serves the stored bytes verbatim, as one column in
  `apps/web-service/internal/store/store.go`.
- The composer **already takes the parameter this needs**: `serialize_bundle` in
  `crates/snapdown-core/src/domain/markdown.rs:25` has a `markdown_path: &str` argument, put there by
  `BUG-86` on 2026-08-31 to fix links that resolved one folder too deep.

This is worth stating plainly rather than leaning on: **the code does not answer the design question,
it prices it.** One surface serves the document today, so there is no second rendering for anything to
be identical to, and no existing code has to change for this decision to hold. That makes the decision
cheap, not right. What makes it right is `AD-9`'s own Prevents.

**Why it is recorded at all.** Changing an `AD-N` is the one case the project's rules make a `DEC-`
mandatory for, and none of the reasoning above is readable from the code — the code contains no trace
of a Rule having been read two ways.

## Cost

- **A second rendering is a second thing to test, and today there is one guard.**
  `crates/snapdown-store/tests/test_golden_markdown.rs:137` pins the serializer byte-for-byte, and it
  pins the **stored** form. The absolute rendering will have no guard unless one is written with it,
  and it will be the untested path by default. This repository has been burned by exactly this shape
  before: a fabricated 17-byte PNG passed every image assertion for five waves because the assertions
  checked a signature instead of decoding the output. A test that asserts "the link starts with `C:`"
  is that mistake again.
- **An absolute path carries the operator's OS username.** Pasted into a shared or public chat, that
  goes with it. This decision does not solve that; it accepts it as the price of a link that resolves,
  and records that the alternative considered — an environment variable — fails for a harder reason
  (no CommonMark renderer expands one).
- **Every document citing `AD-9` now has to be re-read**, and one of them relies on `AD-9` for an
  unrelated promise. See Trace.
- **`AD-9` is weaker than it was.** "Byte-identical" is checkable by anyone with two files and a diff;
  "the same authored document, differing only in the base of an image link" needs a reader to know what
  counts as the base. The narrowing buys a working link and pays for it in how easy the invariant is to
  check. A future path that quietly substitutes something *else* and calls it a base path is the abuse
  this opens, and the guard against it is that the composer owns the substitution.
- **`sharing` and `agent-access` gain no work and lose none.** `DEC-005` freezes both. Neither renders
  anything today: the published page serves stored bytes and continues to, and the Local API does not
  exist. The guidance below for the Local API is guidance for whoever fixes `BUG-59`, not work now.

## Alternatives

Required: `bundle` is `risk_accepted: low`.

| Option | Why not |
| --- | --- |
| Reverse `FR-12` and keep relative links on the clipboard | Restores an `AD-9` nobody has to reinterpret, and leaves the primary handoff path promising something that does not arrive: a relative link resolves for no reader outside the Bundle's own folder, which is every reader of a clipboard |
| Leave `AD-9` alone and let `FR-12` sit in contradiction with it | The state this decision found, and the reason it exists. An `AD-N` and an `FR-` that disagree means the next reader picks one, and neither choice is recorded |
| Make the stored document carry absolute links too, so all paths match again | Breaks `NFR-8`, whose whole point is that the stored document renders in a plain CommonMark reader from its own folder, and makes the Vault unmovable — an absolute link in a stored file is stale the moment `vault_migration.rs` runs |
| Have the clipboard path rewrite the composer's output after the fact | Cheapest to build and exactly what `AD-9`'s middle clause forbids. Two renderings from two pieces of code drift; two renderings from one composer cannot |
| Export `SNAPDOWN_VAULT_PATH` and reference it from the copied Markdown | Fails on its first requirement: no CommonMark renderer expands a variable, so the link renders as literal text and is simply broken. It also inherits three problems — a running process never sees a newly set variable, the Vault can move and leave it stale, and writing to the user's environment outlives the app |
| Narrow `AD-9` by deleting the byte clause entirely | Throws away the part that is load-bearing. "Composed once, by the core, and stored" is what stops a surface authoring content, and it must survive verbatim |

## Reversal trigger

- **Two readers of one Bundle disagree about the review itself.** That is `AD-9`'s Prevents actually
  firing, and it means the narrowing let something through that the byte clause would have caught.
  Reverse to byte-identity and take the broken link instead.
- **A second substitution is proposed that is not a base path** — a different heading, a filtered set
  of Findings, a summarised note. This decision does not permit it and must not be cited as though it
  did. If one is wanted, that is a new `DEC-`, and it is the abuse named in Cost.
- **`BUG-59` is fixed and the Local API needs a rendering.** Not a reversal, but the point at which
  this decision is due a re-read. **Guidance, so it is not re-litigated then:** the Local API serves
  the stored bytes with relative links, unchanged, because its reader is an agent on this machine that
  already knows the Vault path and can be told it once, rather than being told it again inside every
  link. If that turns out wrong, it is a change to one surface and this trigger is where it starts.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | `OQ-12` is falsified by `BR-11`'s narrowing and is owed closure by `wdi-question`. `OQ-1` (*"a coding agent handed a Markdown file with relative image paths can open those images"*) is the assumption this decision acts on and it stays open, because `FR-12` has no implementation to test it against |
| Source material | `.what/business-rules.md` § Amended; `.control/memlog/bundle.md`; `.control/memlog/prd-capture-to-markdown.md` entries 21 and 24; `.scratch/bundle-library/issues/03-decide-what-copy-markdown-puts-on-the-clipboard.md`; `.scratch/bundle-library/issues/08-grow-pdf-export-and-bundle-rename-into-scope.md` |
| Code read | `crates/snapdown-core/src/domain/markdown.rs:25`; `crates/snapdown-store/tests/test_golden_markdown.rs:137`; `crates/snapdown-bridge/src/client.rs:27`; `apps/desktop/src/main.rs:1264-1339`; `apps/desktop/ui/appwindow.slint:1342`; `apps/web-service/internal/store/store.go`; `.control/registry/defects.yaml` § `BUG-59` |

### Every document citing `AD-9`, and whether it needs re-reading

| Document | Needs re-reading | Why |
| --- | --- | --- |
| `.how/_platform/ARCHITECTURE-SPINE.md:149` | **yes — the change itself** | Title and Rule narrowed. Prevents untouched |
| `.what/bundle/SRS-bundle.md` § Decision Summary | **yes** | Claims *"the clipboard, the Local API, and a published page all serve identical bytes — three handoff paths, one document"*. False twice over: two of the three do not exist, and identity is no longer what is promised |
| `.how/agent-access/SDD-agent-access.md:81` | **yes** | Quotes `AD-9`'s old Rule verbatim, and adds that the golden-file test *"covers this path"* for a route with no server. `DEC-005` freezes this component, so this is a correction to a stale citation and not new work |
| `.how/bundle/SDD-bundle.md:8, :32, :72` | **yes** | `binds: [AD-1, AD-2, AD-9, AD-10]`, and `:32` argues that composing on demand *"makes AD-9 unenforceable"* — an argument that still holds and should be checked rather than assumed |
| `.what/bundle/SRS-bundle.md` § Constraints | **yes** | *"every handoff path serves those exact bytes"* |
| `.how/bundle/02-contracts/contract-inventory.md:49` | no | Says the contract reads one column of bytes. Still true |
| `.how/bundle/04-components/LC-013-bundle-store.md:17` | no | Says the store holds the authoritative bytes. Still true |
| `.how/bundle/05-model/data-model.md:41` | no | Says the `markdown` column is `AD-9` made structural. Still true |
| `.control/registry/requirements.yaml` § NOT PROMISED | **no, and MUST NOT be changed** | Relies on `AD-9` for a different promise: *a Bundle's image copy is byte-identical to the Finding's when nothing is drawn on it*. That is about **images**, not the document, and this decision does not touch it. It is load-bearing for the refusal of crop and destructive resize, and weakening it would reopen a named non-goal |
| `.how/bundle/01-ux/assets/04-bundle-assembly-modal.html:163` | no | A label in a design asset |
