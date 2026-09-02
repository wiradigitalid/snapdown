# Map: Bundle Library

**Label:** wayfinder:map

## Destination

A handed-off spec for the **Bundle Library** — the screen behind the Editor's Library icon where a
Reviewer browses every Bundle they have assembled, updates one, copies its Markdown, opens its file
location, and deletes it — together with the Editor action-vocabulary rework the Library forces, the
**Export PDF** stage, and the cloud **Publish** stage that follows once `DEC-005` lifts by its own
terms.

The map ends when `/to-spec` can be run without a further decision being needed. It does not build
anything. **That condition was met on 2026-09-02**, when ticket 09 - the last open decision - closed.
`/to-spec` ran the same day and produced `spec.md`; `/to-tickets` cut it into implementation tickets
`issues/10`-`19`, numbered after the nine decision tickets. Three can start at once: 10, 11, 19.

## Notes

**Domain.** Snapdown desktop app: Slint UI in `apps/desktop`, Rust workspace behind it. A **Bundle**
is a set of assembled Findings composed into one Markdown document plus burned image copies, stored
in the Vault. The **Library** is the Reviewer's whole local collection.

**Method boundary.** This repo splits systems: WDI Method owns documents, mattpocock owns code. This
map is a planning artifact, so its output routes to `/to-spec` → `/to-tickets` → `/implement-spec`.
The corpus (`.what/`, `.how/`, `.control/`) is an **input, never a gate** — where a document and the
code disagree, the code wins and the document is what gets corrected.

**The design lives in two places, both current.** The canvas at
https://claude.ai/code/artifact/6f798d70-77a0-491e-973b-92e7a2641a2f is what to look at; the
artboard source in `.scratch/bundle-library/design/` is what to build from, because it carries the
exact values. Its `README.md` records which running-app component each value was lifted from.

**Skills every session on this map should consult.**
- `.constitution/project/design-system-guide.md` — mandatory for any UI change, in either window.
- `grilling` + `domain-modeling` for decision tickets.
- `prototype` for the two prototype tickets.
- `wdi-ux` for the G2 experience-bar ticket specifically.

**Prerequisites sitting outside this map.** Two defects were found while charting. Neither is a map
ticket; both are ordinary defect work, and the Library's hand-off actions are close to pointless
until the first one is fixed:

1. ~~**Bundle Markdown image links are broken.**~~ **FIXED 2026-08-31, `BUG-86`.** The serializer
   wrote links relative to the vault root while `bundle.md` lives inside the per-Bundle folder, so
   every link doubled a `bundles/<id>/` segment and resolved to nothing — in every reader, for every
   Bundle. `serialize_bundle` now takes `markdown_path` and derives each link from it, so the
   document's location and its links' base cannot drift apart again. `NFR-8`'s missing enforcement
   now exists as `test_nfr8_image_resolution.rs`, seen red before the fix and mutation-verified
   after. Five assertions that had enshrined the broken form were corrected.
2. **Filmstrip "Copy image" targets the wrong Finding.** `appwindow.slint:1440` reads
   `active-finding-id` instead of `menu-target`, so right-clicking card B while card A is open
   copies A. Its sibling "Open file location" (`:1441`) does it correctly.

**Two things on this map are explicit MVP non-goals in the corpus, and need scope growth first.**
Verified 2026-08-31 — both appear verbatim, and "PDF" appears **zero** times anywhere in `.what/`,
`.how/` or `.control/`:

- `.what/_prd/capture-to-markdown/prd.md:723` and `.what/bundle/SRS-bundle.md:84` —
  *"Exporting a Bundle to anything but Markdown"* / *"Exporting to anything but Markdown."*
  → **Export PDF**.
- `.what/_prd/capture-to-markdown/prd.md:717` — *"Renaming a Bundle. Same reason; and a rename that
  does not rewrite the document's heading is a lie."* → the **editable Bundle title** in the Review
  & Update window.

**The "code wins over documents" rule does not apply to either.** That rule governs a document
trailing *existing* code. There is no code here at all — these are deliberate scope boundaries still
in force, so they must be grown through the front door rather than stepped over.

**The owner decided on 2026-08-31 to grow both into MVP scope.** That growth is
[Grow the promises this map needs](issues/08-grow-pdf-export-and-bundle-rename-into-scope.md),
via `wdi-product` intent `update`. Until it lands, neither Export PDF nor an editable Bundle title
has a promise behind it.

**A fifth boundary, found 2026-08-31, and it is the load-bearing one.** `BR-11`
(`.what/business-rules.md:32`, status **active**, sourced to `AD-9 · OQ-12`) — *"A Bundle is never
edited in place. A change means composing a new Bundle"* — forbids the **whole** Review & Update
window, not just its title. `.what/bundle/SRS-bundle.md:76` restates it as a Non-Goal. It is a
different kind of boundary from the four MVP scope lines: an active business rule derived from an
`AD-`. `AD-9`'s letter governs only the **way out** (*"every handoff path MUST serve those exact
bytes"*) and its last clause points at the remedy this map already uses — *change the composer* — so
the window looks compatible with `AD-9` while `BR-11`'s absolute form over-derives from it.
`BR-10` and `BR-65` both stay true and must be preserved rather than amended. `OQ-12`, `BR-11`'s
other source, is an assumption that this map's design falsifies, and it should be closed in place.
All of this is now scoped into
[Grow the promises this map needs](issues/08-grow-pdf-export-and-bundle-rename-into-scope.md), whose
route grows from one skill to three: `wdi-product` for the promises and the `BR-11` amendment, then
`wdi-decision`, then `wdi-question`.

**`AD-9` also bites `Copy Markdown`, in the opposite direction.** The clipboard is a handoff path and
absolute-link rewriting is different bytes on the way out; `crates/snapdown-store/tests/test_golden_markdown.rs:137`
pins the serializer byte-for-byte citing `AD-9`. See
[Decide what Copy Markdown puts on the clipboard](issues/03-decide-what-copy-markdown-puts-on-the-clipboard.md).
Both questions are the same `AD-9` reading and are best answered in one sitting so they cannot drift.

Both were written as **MVP/r1** boundaries, not permanent bans — a sibling entry, *"Searching or
filtering the Library"*, even carries `[NOTE FOR PM] … revisit for r2`. Growing them is ordinary.

Note the rename reason is *satisfied* by this map's design: editing the title block in Review &
Update does rewrite the document's heading, so the objection the PRD records does not apply — only
the scope boundary does.

**Settled while charting.** These are constraints for every session on this map, not open questions:

- **Update never touches a Finding.** In the Library's update mode, edits to title, bundle notes,
  finding notes and marker notes stay in a buffer and are persisted on Save. It must never call
  `FindingStore`. **Corrected 2026-08-31 (ticket 05):** this line used to say the save happens
  *"via `update_bundle_markdown` (which exists and is currently dead code)"*, and that covers only
  three of the four fields. That function runs `UPDATE bundle SET markdown = ?1` and nothing else
  (`bundle_store.rs:257`); `bundle.name` is written by nothing anywhere in the tree, and there is no
  `update_bundle_name` or `rename_bundle`. So the **title needs a store path that does not exist**,
  while the three note fields need only the dead function. The old wording would have had a builder
  discover that mid-story. The existing *compose* flow writes note/marker edits through to the live Finding
  immediately (`main.rs:3966-4012`); that behaviour is deliberately **left alone** — out of scope.
- **Images are frozen in update mode.** No add, no remove, no reorder, no replace. Editable:
  Bundle title, Bundle notes, Finding notes, Marker notes. Nothing else.
- **No action named "Share".** Vocabulary is `Edit` · `Copy Markdown` · `Open file location` ·
  `Delete`, with `Export PDF` and `Publish` arriving later from their own efforts/stages. A payload
  verb (Copy/Export/Publish) always beats the umbrella word, and an unbacked button repeats the
  mistake of the toolbar's fake 1-6 shortcut badges.
- **A Bundle has two states, and the verb on its row says which.** While its source Findings still
  exist: `Disassemble…` (Bundle goes, Findings become assemblable again) and `Discard originals…`
  (Findings go, Bundle stays and seals). Once the originals are gone: `Delete…` only. There is
  deliberately **no single button that destroys both** — it is reachable in two steps, and one click
  is the wrong price for the most destructive act in the product. Migration v6 dropped
  `bundle_item`'s foreign key to `finding` precisely so the sealed state is legal.
- **Assembling copies, it never moves.** A Finding's image survives assembly untouched; the Bundle
  gets a burned copy. What looks like the Finding vanishing is the filmstrip filtering out anything
  a Bundle holds. This is why deleting a Bundle returns its Findings, and why `Disassemble` is the
  honest name for that.
- **A button lives next to the object it acts on.** Ribbon → the active Finding on canvas. Filmstrip
  footer → the ticked selection. Library row → that Bundle.
- **No local Markdown export.** The Markdown and its images already sit together on disk in the
  Vault; once prerequisite 1 is fixed, that copy is portable as-is. Copying it to a second local
  folder produces nothing new. Hand-off is `Copy Markdown` (clipboard) and `Open file location`.
- **MCP / `agent-access` is not part of this.** The bridge exists but its Local API server was never
  rebuilt after the move to Slint (`BUG-59`, critical), and by design it is loopback-only
  (`DEC-002`, `NFR-9`). Real work, different effort.
- **Window copy.** Create mode: title **Review & Assemble**, primary **Assemble**. Update mode:
  title **Review & Update**, primary **Save**. Secondary is Cancel in both.
- **Ribbon rework.** "Share" is deleted (zero Rust behind it today). "Copy" becomes **Copy Image**.
  Ribbon "Assemble" is deleted because it acts on the filmstrip selection, and the filmstrip footer
  already carries an Assemble button in the right place (`appwindow.slint:2447`).

## Decisions so far

- [Reconcile the open ribbon-sizing ticket](issues/04-reconcile-the-open-ribbon-sizing-ticket.md):
  the ribbon-sizing ticket was rewritten to cover only the button that survives the vocabulary
  rework, so the two efforts no longer collide.
- [Prototype the Library screen](issues/01-prototype-the-library-screen.md): a full-window overlay
  like Review & Assemble; rows carry a thumbnail (always `BundleItem` at `position 1`), name and a
  mono meta line, and **two actions on hover** (Copy Markdown, Open file location) with the rest in a
  menu the overflow button and right-click both open — matching the filmstrip's existing gesture, and
  leaving room for Export PDF and Publish to join without touching the row. Menu order: Edit ·
  Copy Markdown · Open file location · Export PDF — Publish — Delete. No search or filter.
- [Decide what deleting a Bundle destroys](issues/02-decide-what-deleting-a-bundle-destroys.md):
  deleting turned out to be a two-state lifecycle. Assembling **copies**, never moves, so a Bundle's
  Findings survive it and reappear once the Bundle goes — which makes `Disassemble` the honest name
  for that act. Three named outcomes while a Bundle still holds its Findings, one per thing worth
  keeping: `Disassemble` (captures stay), `Discard originals` (Bundle stays), `Delete` (neither).
  Once sealed, only `Delete` remains. The whole `bundles/<id>/` folder goes,
  **the database row before its files** — *corrected 2026-08-31, see below* — and bulk lives in a
  `Reclaim space` screen reached from the Library header and from Settings.
- **Correction, 2026-08-31, found by `wdi-review`.** Ticket 02's line above used to read *"files
  before the database row, so a failure stays visible and retryable"*. That contradicts `AD-2`, which
  ticket 02 never checked itself against: *"MUST leave the prior state intact if any part of it fails.
  A record MUST NOT be committed before its files exist, and files MUST NOT be removed before the
  record is."* Both halves bite — the order is reversed, and `AD-2` forbids the partial state the
  ticket was deliberately buying. The visibility it wanted is real, but it pays for it with a `bundle`
  row whose Markdown points at images that are gone, which is the **second of the two harms `AD-2`'s
  own Prevents names**. Row first, then files: a crash then leaves orphan files that nothing points at,
  which `AD-2` also dislikes but which is recoverable by inspection, and that is why its Rule picks
  this order rather than the other. `BR-5` was challenged over this and survived — its all-or-nothing
  is `AD-2` restated. If the owner still wants files-first, that narrows `AD-2` and a `DEC-` is
  **mandatory**, not optional.
- [Decide what Copy Markdown puts on the clipboard](issues/03-decide-what-copy-markdown-puts-on-the-clipboard.md):
  the whole stored document, image links rewritten to absolute paths, encoded as **forward slashes
  wrapped in `<>`** — settled by running six candidate forms through a CommonMark reference
  implementation over three real Vault paths, which killed `file:///` (blocked by readers' `file:`
  security blocklist, not by syntax) and showed `<>` hands the consumer a path needing no decode. The
  toast names the **paths**, not the images: images can never travel on a text clipboard, while an
  absolute path carries the operator's user name and the toast is the only place they learn it.
  `Open file location` is **not** redundant and the question was a fossil — ticket 01 had already placed
  it. Two consequences landed: `FR-43`, because that action had no promise at all, and a glossary entry
  fixing **export = PDF only**. `AD-9` cleared by `DEC-012`.
- [Grow the promises this map needs](issues/08-grow-pdf-export-and-bundle-rename-into-scope.md):
  all four promises exist — `CAP-12`/`FR-39` Export PDF, `FR-40` edit a composed Bundle's title and
  notes, `FR-41` discard the source Findings, `FR-42` reclaim space — plus `NFR-19`, `UC-28`–`UC-31`
  and `BR-122`. Three of the ticket's own guesses were wrong: `FR-41`/`FR-42` belong to `CAP-5` and
  component **`finding`**, not `bundle`, because they destroy a `Finding`; Export PDF did need a new
  `CAP-`; and a **fifth** boundary nobody had listed, `BR-11`, forbade the whole Review & Update
  window rather than just its title. `BR-11` is now narrowed to the handoff path with `AD-9` left
  intact — `DEC-012`, whose reasoning came from `AD-9`'s own Prevents. Two existing promises pointed
  the other way and were corrected: `FR-12`'s relative-link clause and `FR-14`'s combined destroy.
  Committed as `57cbf96`.
- [Prototype the Review & Update window](issues/05-prototype-the-review-and-update-window.md):
  the window **opens locked**, showing the Bundle as composed; the footer's primary is `Edit`, and
  only once unlocked are the four fields editable. `Save` is then **always clickable**, even with
  nothing changed. Two controls became one - the Edit/Preview pair is gone, because locked already
  *is* the preview - and the images carry a `Fixed at compose` chip only in edit mode, where the
  exception needs saying. The prototype killed a premise and a shape: there is **no affordance on an
  image in the compose window either** (`appwindow.slint:3470-3630` holds zero `TouchArea` and zero
  `IconButton`), so a lock chip in locked mode would have had to invent a control in order to disable
  it; and a provenance rail was ruled out outright, which also retired the question of whether the
  modal could stop sharing its `height / 1.414` width. Always-clickable `Save` was verified free
  before it was accepted: a Bundle has no `updated_at` and the Library orders by `composed_at`, so a
  no-op save disturbs nothing visible. That verification is what opened ticket 09.
- [Research the PDF render engine](issues/07-research-the-pdf-render-engine.md): `typst` via
  `typst-as-lib`, **in-process** — a sidecar was proposed for panic isolation then withdrawn, since
  the repo sets no `panic = "abort"` (verified) so `catch_unwind` gives the same protection, and
  typst returns clean errors at 100,000 nesting levels rather than overflowing the stack. Staying
  in-process contradicts no `AD-`, so no `DEC-` is owed. Permissive licences; text layer and
  12-image embedding verified by decoding the output. Costs +35.5 MiB on disk but only **+0.2 MiB
  RAM at idle** — only download size remains at issue. `printpdf` was a third the size and silently
  dropped every image across three input forms. Markdown leaf text is inserted via typst's
  `#"..."` string literals, whose escape set is closed by definition. **Packaging (in-process vs a
  separate exporter crate) is deferred to the Export PDF effort** — the research reversed itself
  twice on it, which marks it as an architectural judgement rather than a measurable fact; the
  ticket records everything established so it is not redone. Tall-image handling is solved and
  measured from the PDF's own placement matrices: clamp to 85% text height above aspect 1.25, slice
  across pages above aspect 3 with the image embedded once — both thresholds still need calibrating
  against real screenshots.
- [Write and verify the G2 experience bar](issues/06-write-and-verify-the-g2-experience-bar.md):
  **the bar is not met, so `DEC-005` does not lift and Publish stays unspecified.** The ticket's own
  first bullet was stale — `wdi-ux` ran on 2026-08-23 and G2 sharpened `BG-7` into `FR-27`, `FR-28`,
  `FR-29`, `NFR-16` and `NFR-17` the same day `DEC-005` said the bar did not exist. What was missing
  was the verdict, now at `.control/reports/ASSESS-EXPERIENCE-BAR-2026-09-01.md`: four of six
  checkable items fail, and a seventh has never been observed. The decisive one is `BUG-54` — six
  token pairings below WCAG AA, including every primary button label in dark mode — and the trap
  worth carrying is that **`test_theme_contrast.rs` is green while `NFR-16` is unmet**, because it is
  a ratchet over known failures rather than a pass mark. **The dependency runs opposite to what the
  ticket assumed:** the Library is not blocked by the bar, the bar is blocked by the Library, since
  `FR-28`'s Bundles surface is one of the four items that fail. Landed: `BUG-89` (the Editor window
  titles itself `Snapdown`, not `Snapdown Editor` — a regression `DEC-007`'s rewrite dropped in
  silence), plus `BUG-57` and `BUG-61` re-counted against the tree after going stale.

- **An edited Bundle says so (ticket 09, 2026-09-02).** Option B: a Bundle gains `updated_at`,
  backfilled with `composed_at`, moved only when stored content actually changes, shown in the Library
  row only when it differs from the composed time, and the sort stays newest-composed-first. The
  deciding point was asymmetry - the column is free to add now and hide later, but edits made before it
  exists are lost for good, and Publish's own future scope already asks a question only this answers.
  **This was the last open decision on the map.** Also landed the same day, outside this map: `BUG-89`
  (the Editor window now titles itself `Snapdown Editor`) and `BUG-54` (every token pairing clears
  WCAG AA in both themes), which turns two of the experience bar's four failing items into passes.

## Not yet specified

- **Publish / cloud.** In scope for this map's destination, and **still frozen.** `DEC-005` forbids
  new FR/UC/UX for the `sharing` component, it lifts *by its own terms* once the G2 experience bar is
  met and verified, and ticket 06 assessed that bar on 2026-09-01 and found it **not met** — so the
  freeze stands and nothing here graduates yet. Do not re-litigate the freeze; the route out is the
  failing items in `.control/reports/ASSESS-EXPERIENCE-BAR-2026-09-01.md`. Two of its four were fixed
  on 2026-09-02 (`BUG-89` for B1, `BUG-54` for B4); of the two left, this map owns one (B2 - the
  Library, which is `FR-28`'s missing Bundles surface) and the other (B3) is the owner's call on
  whether Settings has tabs. B7 still needs a first-encounter session nobody has run. When it does lift, this patch
  graduates into its own tickets — publish/unpublish lifecycle, slug
  handling, credential storage, the `apps/web-service` (Go) side, and what a Reviewer sees when a
  published Bundle is edited afterwards.
- **Export PDF.** In scope. The engine question is sharp and now sits in
  [Research the PDF render engine](issues/07-research-the-pdf-render-engine.md). Settled already:
  PDF is a **human** artifact (the agent path is Copy Markdown), **A4 only**, single column, title
  block rather than a cover page, one section per Finding, page numbers, no image ever split across
  a page break, and a **real text layer** so a machine is not obstructed. Still fog: where the
  action lives in the Library row — that graduates once the Library screen's shape is known.
- **What the Library does at scale.** Sort order beyond newest-first, search, filtering, and whether
  a Reviewer with hundreds of Bundles needs pagination or virtualisation. Likely absorbed by the
  Library prototype ticket; if it survives that, it graduates here.

## Out of scope

- **The Bundles Drawer.** `bundles-drawer-clicked` (`appwindow.slint:1483`) is a second, separate
  stubbed callback for a toggleable drawer. Nothing asked for it; conflating it with the Library
  screen would import an undescribed second UI pattern.
- **Rebuilding the Local API server / MCP bridge** (`BUG-59`). Tracked, critical, and specced under
  `agent-access` with its own SRS. Not this effort. **A `SNAPDOWN_VAULT_PATH` environment variable
  belongs to this patch, not to the Library**: as a way for an agent to *find* the Vault without
  being told it is a reasonable idea worth taking up there; as the contents of a Markdown image link
  it does not work, because no CommonMark renderer expands variables (see ticket 03).
- **Changing the compose-time flow's write-through behaviour.** See "Settled while charting".
