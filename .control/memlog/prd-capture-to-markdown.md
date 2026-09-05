---
topic: Capture to Markdown — the desktop review loop
artifact: .what/_prd/capture-to-markdown/prd.md
updated: 2026-09-05T23:34
---

- (event) headless run, intent create. Two initiatives split by the reader test: someone looking for MCP or web publishing would not open this document
- (decision) CAP-1..CAP-6, FR-1..FR-18, NFR-1..NFR-8, UJ-1..UJ-4 allocated from requirements.yaml; the sequence continues into the agent-handoff PRD
- (decision) Note is written at capture time, inline at the region. Wins on every criterion set — recorded in the addendum because a non-trade-off should not be revisited as one
- (decision) editor does not auto-open after a capture; setting exists, default off
- (decision) image reduction happens once on the way in; the unreduced capture is not retained
- (decision) Marker coordinates are stored, not burned into the Finding image, because FR-8 requires repositioning and renumbering
- (decision) selection order is the only ordering inside a Bundle; no reorder step in r1
- (decision) hard deletion only. NFR-5 states the no-orphan property as an invariant, and FR-15 reports violations
- (assumption) not auto-opening the editor is what the Reviewer wants — OQ-9
- (assumption) reading cost tracks pixel area, so the long-edge cap is the dominant lever — OQ-2
- (assumption) recomposing is acceptable in place of editing a Bundle's Markdown — OQ-12
- (gap) default long edge of 1600px is a working answer, unmeasured against a real agent's reading cost — OQ-3
- (change) stack answered mid-run by the owner: Tauri v2 + Rust, desktop UI React + Vite + TypeScript (Svelte was the earlier plan). Lands as AD-N at G3, not in this PRD
- (event) PRD and addendum written; review lenses structure+prose applied at write time

---

## 2026-08-23 — update: § 4.7, the surface itself

**change** — Added `CAP-9` and `FR-27`–`FR-29` (name the surface · reach every surface · fit the
window), `NFR-16`–`NFR-18`, rewrote `FR-5`, amended `FR-18`. Target release `r3`.

**decision** — `CAP-9` is administered by `settings` even though `FR-28` and `FR-29` govern surfaces
`finding` and `bundle` own. `settings` already holds the container-level Logical Components — the
startup registrar, the hotkey registrar, the settings store — which are the app's own machinery
rather than any one screen's, and the window shell is machinery of the same kind. The alternative,
three copies of the same requirement under three components, is how a shell drifts. The defence is
written into § 4.7 itself so a later reader does not have to reconstruct it.

**decision** — `FR-5` was **amended in place, not replaced**. Its promise — "defaults the Reviewer
never has to change" — was already right. What failed was its presentation: two raw numbers a
Reviewer can accept and cannot judge. Retiring `FR-5` and issuing a new number would have implied the
original promise was wrong, and it was not. `DEC-004` is cited from the requirement text.

**change** — `FR-5` now carries a consequence that is deliberately awkward to satisfy: *Auto resolves
different parameters for a small region than for a full-screen capture, and a test finding them
identical is a failing test.* Without it "Auto" can be implemented as the old constant wearing a new
label, and every other consequence would still pass.

**decision** — `FR-18`'s new default (on after first run) is written so it applies to a **first run
nobody configured**, never to a Reviewer's decision. A default that re-asserts itself over a Reviewer
who turned it off is a bug wearing a default's clothes, and the requirement says so rather than
leaving it to the implementer.

**change** — `NFR-18` (store the resolved budget with each Finding) exists only because `FR-5` forbids
re-encoding an existing Finding. That interaction is not obvious: taken together they mean two
Findings captured a month apart on "the same" Auto setting can legitimately differ, and without the
stored record nothing can explain why. It was found by reading `DEC-004`'s Cost section, not by
reading the requirement.

**event** — ID collision caught and corrected during this update. The new requirements were first
numbered `FR-19`–`FR-21`, which are already held by `CAP-7` in the `agent-handoff` PRD. FR numbering
is global to the product and does not restart per PRD; the highest allocated was `FR-26`. They are
now `FR-27`–`FR-29`. `NFR-16`–`NFR-18` were checked against the same rule and are clean.

**assumption** — `OQ-3` is restated rather than closed by `DEC-004`, and § 8 says so in the document
rather than quietly dropping the old question. `OQ-18` and `OQ-20` added to § 8.

### Not done, and why

- **No use cases written for `FR-27`–`FR-29`.** The UC catalogue belongs to `wdi-blueprint` intent
  `catalog`, and these three components now sit at `mode: deep`, so the catalogue is being re-derived
  there rather than extended here.
- **No screen specifications.** `wdi-ux` owns those. § 4.7 states what must be true of a surface and
  deliberately does not say what it should look like — `FR-29` names a condition (nothing discovered
  only by scrolling), not a layout.
- **`agent-handoff` PRD untouched.** `DEC-005` freezes `sharing` and `agent-access`; editing their
  PRD would be new work on them.
- **`NFR-3` left alone.** It still names "the shipped default" for a 4K capture under 250 KB. Under
  Auto there is no single shipped default, so the wording is now loose. It is flagged here rather
  than changed, because tightening it needs the derivation to exist first — a finding for
  `wdi-reconcile`, not a silent edit.
- (event) Intent Update opened 2026-08-31 to grow Export PDF, Bundle rename/in-place edit, Discard originals and Reclaim space into promises. Driven by .scratch/bundle-library/issues/08. Active initiative switched in _bmad/custom/bmad-prd.user.toml from agent-handoff to capture-to-markdown, both lines together as that file requires.
- (change) Step 5 owns-check overrode ticket 08's guess: Discard originals and Reclaim space destroy Finding, which bundle does not own (owns: [Bundle, BundleItem]). They go to CAP-5 / component finding, not CAP-4 / bundle. Precedent for the alternative exists in FR-14's registry row, which uses defers_to: [FR-25] for Publication rather than claiming a write it does not own.
- (event) CONFLICT A surfaced, not yet applied: FR-12's consequence in prd.md reads 'The image references in the copied text are the same relative paths as in the file.' Ticket 03's owner answer of 2026-08-31 rewrites them to absolute paths for the clipboard. Direct contradiction of an existing promise, independent of AD-9. Ticket 03 recorded the AD-9 exposure but not this one.
- (event) CONFLICT B surfaced, not yet applied: FR-14's last consequence reads 'The Reviewer can choose, in the same confirmation, to delete the Bundle's source Findings too.' Ticket 02 settled the opposite - no single button destroys both, two deliberate steps. Landing FR-41 Discard originals beside FR-14 unchanged would make the PRD promise both designs at once. Also note FR-14's prose claims a Finding write its own registry row does not authorise.
- (decision) Owner 2026-08-31 on CONFLICT B: amend FR-14 to ticket 02's two-step lifecycle. The 'delete the Bundle's source Findings too in the same confirmation' consequence is removed; FR-14 keeps only Bundle-side destruction. This also retires FR-14's unauthorised Finding write claim, which its own registry row never carried.
- (decision) Owner 2026-08-31 on CONFLICT A: amend FR-12 now rather than deferring to ticket 03. Scope held deliberately narrow - FR-12 is widened to PERMIT an absolute-path rendering of image links for the clipboard, and does NOT pick the encoding, does NOT decide whether a warning or a plain Copied toast is shown, and does NOT decide whether Open file location becomes redundant. Those three stay ticket 03's to close.
- (assumption) FR-12's amendment rests on the reading that going through the composer with a base-path parameter satisfies AD-9 rather than contradicting it. If that reading fails, AD-9 is contradicted and a DEC- becomes mandatory. Flagged as owed to wdi-decision alongside the BR-11 DEC-, not assumed silently.
- (change) prd.md, addendum.md, requirements.yaml (CAP-12, FR-39..FR-42, NFR-19, amended FR-12 and FR-14), components.yaml (bundle gains CAP-12), assumptions.md (OQ-31) all written. FR-39's mechanism kept out of prd.md and pointed at from addendum.md instead, per check 7.
- (change) bmad-review (structure+prose) fired on prd.md and addendum.md per doc_standards: 10 findings. Three applied as correctness rather than taste - (1) check 7 had been passed on a too-narrow grep and FR-12's correction block did carry solution shape ('one composer with a base-path parameter rather than a second serializer'), now moved to addendum.md under a new 'Two path conventions for one Bundle' section; (2) 4.5's Description said Bundle deletion 'belongs with Bundles, in 4.4' while FR-41/FR-42 now live in 4.5, so it misdirected the reader - amended to say what puts them there; (3) FR-12's consequence 'resolves for a reader outside the Bundle's own folder' named nothing observable under a heading that promises testable - rewritten as absolute-and-working-directory-independent. Seven left for the owner: three structure placements, four prose tightenings.
- (event) PRD update complete. status: NOT raised - the wdi-product wrapper forbids it, and the reviewed: block is wdi-review's to write. Owed and each needing its own go-ahead: wdi-blueprint (narrow BR-11, plus four UCs for FR-39..FR-42 which validate.py correctly reports as V2), wdi-decision (the BR-11 DEC-, which should also carry FR-12's AD-9 reading), wdi-question (close OQ-12 in place), wdi-component behaviour on bundle (SRS-bundle Non-Goals lines 76/77/84, and its wrong BR-10 citation), wdi-ux (screen registry entry for FR-42).
- (change) 2026-08-31, second update pass, closing ticket 03. FR-43 born - opening a Bundle's folder had NO promise anywhere (zero hits in requirements.yaml or prd.md; only 'reveal' for a Finding in defects.yaml), while ticket 01 had already put it on the Library row. Deliberately the thinnest FR in the registry: CAP-4, component bundle, writes: [], with a no_uc: stating the owner's own reason - a power user's way out to the filesystem, and WHY is deliberately not traced. FR-12's proof and consequences now carry what the Reviewer is told; the encoding was settled by test (six forms, CommonMark reference implementation, three real Vault paths) which eliminated both file:/// forms on readers' file: security blocklist rather than on syntax. NFR-19 corrected hours after being written: it forbade the page-slicing that ticket 07's own tall-image solution prescribes. One wording fix in 2.3, where a journey called the Markdown an 'export' - export means PDF in this product. Export landed as a glossary entry; the brief already agreed at G1.
- (change) wdi-product, 2026-09-03: resolved the UJ-5 id collision wdi-upgrade found. The canvas-annotation journey in 2.3 (never registered) had been calling itself UJ-5 since it was written, colliding with agent-handoff's real UJ-5. Renumbered to UJ-7 (next free id after UJ-6), registered in requirements-capture-to-markdown.yaml's journeys:, and the upgrade's flagging comment removed. One Revision History row added. No journey content or other section touched.
- (event) wdi-upgrade tooling note (from the 0.5.15->0.5.38 pass), moved out of prd.md's body on 2026-09-03 per corpus-guide.md (audit content belongs in the memlog, not the document): '§1 kept whole under a delta note rather than trimmed, because no sentence is word-for-word identical to the brief's Why - deciding which paraphrases are copies is the owner's call.' Still open: paragraph reads as a restatement of brief.md's Why rather than a delta, flagged independently by two bmad-review lenses on 2026-09-03; needs the owner's read before trimming.
- (event) wdi-upgrade tooling note (from the 0.5.15->0.5.38 pass), moved out of addendum.md's body on 2026-09-03: 'Each FR's Consequences (testable) bullets moved verbatim out of prd.md §4, which now cites the FR by id under Realizes: instead of carrying a full block. requirements-capture-to-markdown.yaml is the sole home for each FR's statement and proof of done.'
- (change) bmad-review (adversarial, edge-case-hunter, structure, prose) ran on prd.md+addendum.md, 2026-09-03, closing wdi-product check 11. Fixed: FR-38 mis-cited under 3.9 (registry says CAP-11) - moved to 3.8's Realizes line; three serves: lines corrected against the registry (CAP-12->BG-8, CAP-11 drops BG-7, 3.9 gains BG-2, NFR-18 drops BG-7); FR-40's BR-11 gate note was stale - DEC-012 narrowed BR-11 on 2026-08-31 and FR-40 is legal, matching 4.1's own scope list; two Revision History section citations (4.7, 4.10) pointed at sections that no longer exist post-renumbering, and the table's row order was fixed to match date order; two wdi-upgrade tooling comments moved here (see prior two entries); 3.4's split Realizes: line merged into one; FR-41's paragraph, duplicated verbatim in both prd.md and addendum.md, cut from the addendum copy; FR-39's addendum bullet still said no image is ever split across a page break with no exception, contradicting the corrected NFR-19 - added the exception clause; FR-12's addendum paragraph called three questions still unsettled that its own next two subsections already answer - rewritten to say so; 3.7's Logical-Components solution-shape paragraph trimmed in prd.md and moved to addendum.md; Rejected Alternatives freeform-annotation row reconciled with CAP-11, which later did exactly what it rejected, for a different reader; UJ-7 fixed an internal ordering contradiction (Arrow referenced a Callout not yet placed) and a wrong term (numbered findings -> numbered lines); addendum.md frontmatter updated: date bumped. NOT fixed, reported to the owner instead: CAP-10 has no FR row carrying capability: CAP-10 in the registry (orphaned capability, needs a judgement call on whether to retag FR-1 or split a row); section 6 Cross-Cutting NFRs restates most registry rows' statement and enforcer in full prose, the same duplicate-proof-of-done pattern the guide repealed for FRs - a 12-entry rewrite judged too invasive for this pass; nine edge-case-hunter findings about missing behavior specification (partial-composition failure, bulk-reclaim partial failure, Blur burn timing, export-then-edit staleness, etc.) are genuine gaps but are new promise content, not corrections - left for a future wdi-product update pass; a stale addendum.md:FR-5 citation to a nonexistent section 8 was already known from the wdi-upgrade pass and is still unresolved; capitalization drift on Shape/Arrow/Callout/Text/Blur and Handoff/handoff and burnt/burned was reported but not swept, as too broad and low-value for this pass.
- (change) Added FR-44 (bulk 'Delete both' in Reclaim space, component: bundle, defers_to: [FR-25, FR-42]) rather than widening FR-42 - entity-one-writer forbids one FR crossing bundle/finding ownership, and FR-14/FR-41's own split already set this precedent. Extended UC-12 to also satisfy FR-44. Corrected two stale sentences (§3.4, Safety) that BUG-104 (2026-09-03) had already made false: 'Delete both' is one dedicated act today, not two separate steps. Ticket 05 of post-testing-polish, wdi-autopilot mandate DEC-023.
- (event) Structure/prose self-review of the diff against prd-guide.md's persistent_facts: corrections labelled, no FR statement/proof duplicated in prd.md prose, exactly one Revision History row, no forbidden sections reintroduced. validate.py --check: 0 new findings, all 6 remaining match the documented .github/validate-baseline.txt fossils (CAP-7 refs, agent-access memlog).
