# Spec: Post-Testing Polish (Canvas, Marker, Bundle Handoff, Reclaim Space, About)

**Status:** ready-for-agent (sized into tickets 2026-09-05 under `DEC-023`; see `issues/`)

**Ticket breakdown, decided under this mandate (the `to-tickets` quiz is the mandate's to answer, not
the owner's, per `wdi-autopilot`):** six tickets, one per independent area named in Implementation
Decisions, none blocking another — `01-canvas-zoom-ctrl-scroll`, `02-marker-note-focus-and-tooltip`,
`03-second-assemble-button-and-filmstrip-alignment`, `04-copy-markdown-on-save`,
`05-reclaim-space-bulk-actions`, `06-about-tab-icon`. Ticket 01 also records a diagnosis finding: the
already-shipped canvas zoom (`396550c`) already satisfies User Story 4 (placement accuracy under zoom)
by construction — `appwindow.slint:2313`'s marker-placement math divides by the same `parent.width` that
`canvas-zoom` scales, so the ratio is zoom-invariant regardless of level. Only Ctrl+Scroll wiring is a
real gap. Ticket 03's open question (button (a) vs (b) behaviour) is resolved to (a), per the spec's own
"Build (a) unless the owner asks for (b)" default, since no owner is present to ask under this mandate.
**Source:** an 11-item backlog the owner gave in one message, on 2026-09-03, right after confirming
`bundle-library` (tickets 10-19, plus `BUG-90` through `BUG-104`) was done. Two of the eleven — Paste
not working, Crop not working — were recorded separately as `BUG-105` and `BUG-106` in
`.control/registry/defects.yaml` and are **not** in this spec. Of the remaining nine, two are excluded
below (Out of Scope) rather than specced: removing Agent bridge/MCP contradicts an applied `DEC-`
and needs its own decision first, and Export PDF is deliberately a separate effort per
`bundle-library/spec.md`'s own Implementation Decisions.
**Corpus ids this rests on:** `FR-8` `FR-10` `FR-12` `FR-27` `FR-34` `FR-40` `FR-41` `FR-42` · `UC-5`
· `AD-2` `AD-9` · `BR-11` `BR-49` `BR-121` `BR-122` · `DEC-002`. The corpus is an input, never a gate:
where a document and the code disagree, the code wins and the document is corrected afterwards.

---

## Problem Statement

Seven small frictions survived `bundle-library` shipping, each reported by the owner while using the
finished product rather than while reviewing a screen:

- **The canvas has a promised zoom (`FR-34`) that was never built.** Findings can only be inspected at
  natural size, so a small detail on a large capture cannot be looked at more closely without
  Windows' own screen magnifier.
- **Placing or opening a Marker does not hand the Reviewer the keyboard.** `UC-5`'s own main flow says
  *"the Reviewer types the sub-comment for line 1"* right after placing badge 1 — today that requires
  a separate click into the Note field first, on both the just-placed Marker and any existing one
  clicked open.
- **A Marker gives no preview on hover.** The only way to read what a Marker's note says is to open
  it; a capture with several Markers has no fast way to scan them.
- **Assemble's only entry point sits far from the canvas.** It is reachable from the filmstrip footer
  and the context menu, both near the bottom, and the owner wants a second, closer door — ticket 19
  deliberately kept Assemble out of the ribbon on the reasoning that *"it acts on the filmstrip's
  ticked selection, not on the canvas beside it"*; the owner is asking for that reasoning to bend for
  reachability, not for it to have been wrong.
- **The filmstrip's Assemble button area does not align with the filmstrip's own frame.** A visual
  defect, not a behavioural one.
- **Handing a Bundle to an agent is two actions where the owner wants one.** Composing (`FR-10`) or
  saving an edit (`FR-40`) does not also copy the Markdown (`FR-12`) — the Reviewer has to remember to
  come back and press Copy Markdown separately every time.
- **Reclaim space (`FR-42`) can only reclaim originals, one Bundle at a time to select, and cannot
  touch the Bundles themselves.** Selecting many rows means clicking each one, and there is no bulk
  door to the `FR-41`/"Delete both" pattern `ticket 17` and `BUG-104` already built for a single
  Bundle.
- **Settings' About tab (`FR-27`'s home) names the product but does not show it.**

## Solution

Build the zoom `FR-34` already promises, with Ctrl+Scroll and a button pair beside the
resolution/size readout. Auto-focus a Marker's Note field the moment a Marker is placed or clicked,
matching what `UC-5` already describes. Add a hover tooltip that previews a Marker's note text.
Add a second Assemble entry point at the canvas's top-right, beside the existing filmstrip-footer and
context-menu doors, and fix the filmstrip's own misaligned Assemble area while in that code. Make a
successful Assemble & Save and a successful Review & Update Save also copy the Bundle's Markdown to
the clipboard, reusing Copy Markdown's own toast wording. Add a select-all checkbox and a bulk
"Delete both" action to Reclaim space, built on the same live-read, deliberate-confirmation pattern
`FR-41`/`BUG-104` already established for one Bundle at a time. Add the product's icon to Settings'
About tab.

## User Stories

**Canvas zoom (`FR-34`)**

1. As a Reviewer, I want the canvas to open at 100% by default, so that what I see is the capture's
   true size unless I ask for something else.
2. As a Reviewer, I want Ctrl+Scroll over the canvas to zoom in and out, so that inspecting a detail
   does not need a menu.
3. As a Reviewer, I want zoom buttons in the same panel area that already shows the canvas's
   resolution and size, so that zoom controls live where I am already looking for size information.
4. As a Reviewer, I want a Marker I place while zoomed in to land exactly where my pointer is, not
   where it would have landed at 100%, so that zooming in to place a precise Marker actually helps.
5. As a Reviewer, I want zoom to reset to natural size the next time I open a Finding, so that zoom is
   a way of looking, never a stored property of the Finding.

**Marker notes focus and preview (`FR-8`, `UC-5`)**

6. As a Reviewer, I want the Note field to be focused and ready to type the instant I place a new
   Marker, so that I can describe what I just pointed at without an extra click.
7. As a Reviewer, I want the Note field to be focused and ready to type the instant I click an
   existing Marker, so that correcting or extending a note is one gesture, not two.
8. As a Reviewer, I want a readable tooltip when I hover a Marker, so that I can scan what several
   Markers say without opening each one.

**A second Assemble door, and a visual fix beside it**

9. As a Reviewer, I want an Assemble button near the top of the canvas, so that I do not have to
   travel to the filmstrip footer every time I am ready to compose.
10. As a Reviewer, I want the new Assemble button to behave exactly like the filmstrip footer's own —
    same ticked-selection rule, same refusal message when nothing is ticked — so that having two doors
    never means two different behaviours to learn.
11. As a Reviewer, I want the filmstrip's Assemble button area to actually line up with the
    filmstrip's own frame, so that the strip looks finished rather than assembled from mismatched
    pieces.

**Copy on save (`FR-10`, `FR-12`, `FR-40`)**

12. As a Reviewer, I want a successful Assemble & Save to also copy the Bundle's Markdown to my
    clipboard, so that composing and handing it to an agent is one action instead of two.
13. As a Reviewer, I want a successful Save inside Review & Update (editing mode) to also copy the
    updated Markdown to my clipboard, so that fixing a typo and handing over the corrected version is
    one action instead of two.
14. As a Reviewer, I want the toast after either kind of save to say the same thing Copy Markdown's
    own toast says — including that the copied links are absolute paths on my disk — so that I am
    told the same fact regardless of which door produced the copy.

**Reclaim space, in bulk (`FR-41`, `FR-42`, `BR-122`)**

15. As a Reviewer, I want a "select all" checkbox in Reclaim space, so that clearing every listed
    Bundle's originals does not mean ticking each row by hand.
16. As a Reviewer, I want a "Delete both" action in Reclaim space, so that I can remove a batch of
    Bundles and their originals together, not just the originals.
17. As a Reviewer, I want Reclaim space's bulk actions to ask one confirmation naming the whole
    selected set and what is destroyed, the same discipline `FR-41`'s and `BUG-104`'s single-Bundle
    confirmations already use, so that a bulk act is never a surprise.
18. As a Reviewer, I want a Bundle that another Bundle also shares a Finding with to still be named in
    the confirmation, per `BR-122`'s live read, so that a bulk act does not silently seal a Bundle I
    never selected.

**Settings' About tab**

19. As a Reviewer, I want to see Snapdown's own icon in Settings' About tab, so that the tab that names
    the product also shows it.

## Implementation Decisions

**Scope of this spec.** Canvas zoom, Marker note auto-focus, a Marker hover tooltip, a second Assemble
entry point plus the filmstrip alignment fix beside it, copy-on-save for both save paths, Reclaim
space's select-all and bulk Delete both, and the About tab's icon. Nine of the owner's eleven items;
two are Out of Scope below.

**Canvas zoom (`LC-007` marker-canvas).** `FR-34` already states the invariants this must hold: zoom
is a view state only (never written to the Finding's stored record), Marker/annotation coordinates
stay normalised to the image so nothing drifts as the zoom level changes, and a Marker placed while
zoomed lands at the pointer's true image-relative position. Ctrl+Scroll changes the zoom level;
plain Scroll is unaffected (it already scrolls the canvas viewport, and must keep doing so). The zoom
buttons sit beside the existing resolution/size readout — the exact panel is a builder decision, not
a new one; reuse whatever component already renders that readout rather than adding a second info
panel. No persistence: reopening a Finding always shows it at natural size, per `FR-34`'s first
bullet.

**Marker note auto-focus and hover tooltip (`LC-007`, `LC-006` findings-editor).** Auto-focus applies
to both the newly-placed-Marker case and the click-an-existing-Marker case; `UC-5`'s own step 3
("the Reviewer types the sub-comment for line 1") already describes typing immediately after placing,
which auto-focus is what actually delivers rather than leaving as an unstated assumption. Dragging a
Marker (`UC-5` step 5) is a different gesture and does not focus the Note field — only a click/place
does. The hover tooltip previews the Marker's own Note line text; an empty note (`BR-3` — the Note
line may be empty) shows no tooltip or an explicitly empty-state one, a builder decision, but it must
never show stale text left over from a different Marker.

**A second Assemble entry point (`LC-028` editor-shell) and the filmstrip alignment fix.** The new
button fires the exact same `assemble-bundle-clicked` callback and reads the exact same ticked
selection `prepare_bundle` already reads (`main.rs:3423`) — it is a second door to the identical act,
not a new act. **Open question the ticket must resolve with the owner before building, not assume**:
the owner's own framing ("Canvas aktif bisa langsung diassemble") reads as though the active canvas
Finding can already be assembled alone — as built today it cannot; `prepare_bundle` refuses with
"Tick at least one Finding in the strip first" when nothing is ticked, active canvas or not. Two
readings are consistent with the owner's words and only one is in scope here: (a) the new button is
purely a closer door to the same selection-gated act (cheapest, matches every existing Assemble
door's behaviour, User Story 10 above), or (b) clicking it should also tick the active Finding first
if nothing is ticked, which is new behaviour beyond `FR-10` as currently written. Build (a) unless the
owner asks for (b) when the ticket opens. This spec deliberately reverses part of ticket 19's stated
reasoning (Assemble kept out of the ribbon because "it acts on the filmstrip's ticked selection, not
on the canvas") on the owner's explicit request for reachability — record that reversal in the ticket
the way `BUG-104`'s entry records its own reversal of `ticket 17`, with the old reasoning quoted
rather than silently dropped. The filmstrip alignment fix is a plain visual defect with no behaviour
change and no corpus citation; fix it in the same ticket since it touches the same area, but keep it
a separate, clearly-labelled acceptance line so a reviewer can tell "moved" from "fixed" apart.

**Copy on save (`bundle-composer` `LC-010`, `bundle` component).** Reuses the exact clipboard write
and toast wording `copy-markdown-clicked`'s handler already produces (`FR-12`'s absolute-link
rendering, `FR-12`'s "the Reviewer is told the copy succeeded and that what was copied carries
absolute image paths" requirement) — this is not a second implementation of Copy Markdown, it is the
same one called from two more places: `on_bundle_preview_confirmed`'s `Ok` arm (Assemble & Save) and
`on_review_update_save_clicked`'s `Saved` arm (`FR-40`'s save path). Only a *successful* save copies;
a failed save (either path) copies nothing and shows only its existing failure toast — copying stale
or unsaved content would misinform whatever the Reviewer pastes it into next.

**Reclaim space bulk actions (`finding` component, `FR-41`/`FR-42`/`BR-122`).** Select-all ticks every
listed row (Reclaim space already excludes a Bundle whose originals are already gone, per `FR-42`'s
second bullet — select-all operates over exactly the rows shown, nothing hidden). The bulk "Delete
both" action extends `FR-42`'s stated promise ("reclaim their disk in bulk", originals only) to also
remove the Bundles themselves for the selected set — **this widens `FR-42`'s current wording and
should get a quick `wdi-product` pass to fold the widened promise back into the PRD once built**,
the same "document follows the code" discipline `AGENTS.md` already states, applied going forward
rather than backward. The confirmation follows `FR-41`'s and `BUG-104`'s own discipline: one dialog,
naming the whole selected set and what is destroyed, "this cannot be undone." Reads `BR-122` live at
confirmation time the same way the single-Bundle Discard-originals confirmation already does
(`bundles_sharing_findings` in `main.rs`) — a Bundle outside the selected set that shares a Finding
with one inside it must still be named as something that will be sealed by the act. Write ordering
inside the bulk act follows `AD-2` per Bundle (row before its own folder, for every Bundle in the
set) — a partial failure partway through the batch is a builder decision to resolve against `AD-2`'s
"prior state intact if any part fails" reading, the same ordering trap `bundle-library/issues/02`
already worked through for the single-Bundle case; do not re-derive it from scratch.

**About tab icon (`LC-015` settings-screen).** `apps/desktop/assets/app-icon.ico` is the product's
existing icon and the nearest asset to reuse; confirm at build time whether Slint's `@image-url`
loads `.ico` directly or whether a `.svg`/`.png` export of the same mark needs to be added to
`assets/icons/` first — a builder decision, not a spec one. Placed in the existing "SNAPDOWN" `SdCard`
(`settings.slint:901`), beside or above the version line — exact layout is a builder decision. `FR-27`
and `BR-121` govern the product's *name* (the executable, the tray tooltip, the window title all read
from one source) and are the reason this lives on the About tab at all, not a new requirement about
the icon itself, which the corpus does not otherwise mention.

## Testing Decisions

**A good test here asserts what a Reviewer can observe or what a handler actually calls — never a
copy of the code it is checking.** This codebase's own two standing failure modes govern every test
below: a component built and mounted nowhere (`AGENTS.md`'s "REACHABLE, not merely built" rule), and a
test asserting a hardcoded literal that cannot go red when the behaviour breaks (`AGENTS.md`'s
"never asserts a copy of its own input" rule, from the image-decode and token-contrast lessons).

**Seam 1 — the desktop (source-reading tests, the house style, matching every ticket in
`bundle-library`).**
- Callback reachability: any new callback (zoom in/out, select-all, bulk delete-both) is bound in Rust
  and appears in neither `DELIBERATELY_UNHANDLED` nor `KNOWN_STUBS` — the existing
  `test_ui_callbacks_reach_rust.rs` ratchet, extended, not reinvented.
- Component/wiring: the new zoom buttons, the second Assemble button, the Reclaim-space select-all
  checkbox and bulk Delete-both button are each instantiated somewhere and their callbacks bound —
  `test_annotation_wiring.rs`'s shape.
- Focus-on-place / focus-on-click: asserted the way this codebase already proves a `FocusScope`
  claims focus on creation (`library-keys := FocusScope { init => { self.focus(); } }` in
  `library.slint` is the existing pattern to copy) — a structural check that the Note field's
  `FocusScope`/text-input claims focus in both the place-Marker and click-Marker handlers, not a
  literal string match that would pass by coincidence.
- Copy-on-save: the `Saved` arm of `on_review_update_save_clicked` and the `Ok` arm of
  `on_bundle_preview_confirmed` each call the same clipboard-write function `copy-markdown-clicked`'s
  own handler calls — proven by asserting the SAME function name appears in all three call sites,
  not by re-describing the clipboard write a third time.

**Seam 2 — the core/store, where zoom's coordinate math and the bulk delete-both's write ordering
live.**
- Coordinate normalisation: a Marker placed at a known pixel position under a known zoom level
  resolves to the same normalised `[0,1]` coordinate `FR-8`'s existing placement math already
  produces at 100% — asserted by computing both and comparing, not by trusting the zoom transform by
  inspection.
- Bulk delete-both: extends the existing single-Bundle `remove_bundle_row_and_folder` +
  `delete_finding_everywhere` test pattern (`crates/snapdown-store/tests/test_bundle_deletion.rs`) to
  a set of two-or-more Bundles, including the case where two selected Bundles share one Finding — the
  shared Finding must not be deleted twice or reported twice.
- Prior art: `test_bundle_deletion.rs`, the existing Reclaim-space single-Bundle discard tests in
  `test_reclaim_space_wiring.rs`.

**Seen red before trusted.** Every guard above — focus claimed, coordinate normalisation, bulk write
ordering, the copy-on-save call-site match — is broken deliberately, watched fail, then restored,
matching every fix landed in `bundle-library`'s own `BUG-9x`/`BUG-10x` entries. A verification run
uses `--no-fail-fast`, and its real exit code is read from the command itself, never from a trailing
`echo`.

**What no test here can vouch for, and the owner is told to look at**: whether the zoom feels right at
the extremes (how far in, how far out); whether the tooltip's timing and position feel natural rather
than intrusive; whether copying on every successful save becomes noise the Reviewer wants to turn
off (not asked for here, worth watching); whether the second Assemble button's placement actually
reads as closer once it exists next to real content, not a mock. Each implementation ticket should
name what to look at, the way every `bundle-library` ticket did.

## Out of Scope

- **Removing Agent bridge / MCP from Settings and from the feature set** (the owner's item 9).
  `DEC-002` (status `applied`) is the architecture decision that put the two-process MCP bridge there,
  for four stated reasons (no client-side secret on disk, exactly one valid key, must work with a
  stdio-only MCP client, no second copy of the Library). An applied `DEC-` cannot be edited except to
  record its supersession (`AGENTS.md`), and a change of mind produces a new `DEC-`, not a ticket. This
  needs `wdi-decision` — a new `DEC-` explicitly superseding `DEC-002`, with the owner present — before
  any removal ticket can be written. Recorded here so the request is not lost, not because it is
  refused.
- **Export PDF** (the owner's item 11). `bundle-library/spec.md`'s own Implementation Decisions already
  say this is a separate effort with its own spec. It is not a blank slate: `FR-39` (Export a Bundle as
  a PDF), `NFR-19` (a real text layer, no image split across a page break) and `CAP-12` already exist in
  the PRD, and `bundle-library/issues/07` already researched the render engine, licences, image
  handling and escaping. `bundle-library/issues/08`'s own "Still owed" list flags that `FR-39` needs a
  design pass at `guarded` before a spec can rest on it, and that `DEC-013` (review intensity) is still
  `draft`. The next `/to-spec` for this should start from `issues/07` and `issues/08`, not from zero.
- **Paste from clipboard, and the Crop tool** — recorded as `BUG-105` and `BUG-106`, not specced here.
- **Anything about the marker canvas beyond zoom, focus and the tooltip** — resize handles, undo/redo
  history (`FR-33`), front-to-back ordering (`FR-38`) are already promised elsewhere and untouched by
  this spec.
- **A settings toggle to turn copy-on-save off.** Not asked for; Testing Decisions flags it as worth
  watching, not as scope.
- **Changing what Reclaim space's single-row actions do.** This spec only adds a bulk door beside them;
  the existing per-row Discard-originals flow is untouched.

## Further Notes

- **`FR-42`'s wording will trail the code once bulk Delete both ships**, the same accepted lag
  `AGENTS.md` names for every document in this repo — "reclaim their disk in bulk" currently means
  originals only, and this spec widens the behaviour before the promise's own wording is updated to
  match. Flagged explicitly in Implementation Decisions above so it is not lost as silent drift.
- **The owner's item 6 was written twice, once under each of two headings, in the original message** —
  it is one item ("copy on save"), not two, and appears once in this spec's User Stories.
  Cross-referenced against the raw session record if a future reader wants to verify.
- **This spec's own Assemble-button reversal is the same shape `BUG-104` already went through inside
  `bundle-library`**: a design choice recorded with its reasoning, later reconsidered by the owner on
  direct experience of using the product, corrected in place with the old reasoning quoted rather than
  deleted. That pattern, not a fresh design discussion, is the template for how the ticket should
  record it.
