# 05: Prototype the Review & Update window

**Type:** prototype
**Status:** claimed
**Blocked by:** 01

## Question

Editing a Bundle from the Library reuses the Assemble & Review modal's layout, but not its rules. The
copy is settled — title **Review & Update**, primary button **Save**, secondary Cancel — and so is
the behaviour. What is *not* settled is how the window makes its narrower rules legible.

Make a rough version and decide:

- **How frozen images read as frozen.** Images cannot be added, removed, reordered or replaced here.
  The compose-time window shows them the same way but in a context where the set is still being
  chosen. What visually says "this set is settled" without looking broken or disabled-by-accident?
- **What the header carries.** Compose mode shows a live "N Findings · ~N tk" readout
  (`appwindow.slint:3420`) and an Edit/Preview toggle (`:3438`). Does update mode keep both? The
  Finding count cannot change here, so a live count may be noise.
- **Which affordances disappear.** Anything implying the Finding set is still open should not render
  in this mode.
- **Save state.** Whether Save is always enabled, or only once something has changed, and what the
  Reviewer sees after saving — stay in the window, or return to the Library?
- **Discard.** What Cancel does when there are unsaved edits, and whether it confirms.

**Hard constraints, already settled on the map — do not reopen:**

- Editable: Bundle title, Bundle notes, Finding notes, Marker notes. Nothing else. **The title
  depends on ticket 08** — renaming a Bundle is an explicit MVP non-goal
  (`.what/_prd/capture-to-markdown/prd.md:717`) until that growth lands. Prototype everything else
  regardless; treat the title field as provisional until then.
- Every edit stays in a buffer and is persisted only on Save, via `update_bundle_markdown` (already
  implemented, currently dead code). This mode must **never** call `FindingStore` — the Bundle is
  independent of the Findings that produced it.
- The compose flow's existing live write-through to Findings and Markers
  (`main.rs:3966-4012`) is deliberately left alone. Do not "fix" it here.

## Design source to match

The Library's artboards are in `.scratch/bundle-library/design/` and set the vocabulary this window
must not diverge from — 52px header with a 15px accent icon, 3px radii, 13px body, 11px/800
uppercase section labels, 10px IBM Plex Mono meta, 32px buttons at 11px/700, and the confirmation
dialogs' 20px/14px/30px anatomy. Read its `README.md` before drawing anything new; every value there
was lifted from the running app rather than chosen on the canvas.

Canvas: https://claude.ai/code/artifact/6f798d70-77a0-491e-973b-92e7a2641a2f

## Added 2026-08-31, from ticket 08's session — the whole window, not just the title, waits on 08

The constraint above says *"The title depends on ticket 08"* and treats everything else as
prototype-able now. That understated it. `BR-11` (`.what/business-rules.md:32`, status **active**) —
*"A Bundle is never edited in place. A change means composing a new Bundle"* — forbids this window
**entirely**, not just its title field, and `.what/bundle/SRS-bundle.md:76` restates it as a Non-Goal.
Bundle notes, Finding notes and Marker notes are all editing a composed Bundle in place.

This does **not** block prototyping: a prototype exists to raise the fidelity of the discussion, and
`AD-9`'s letter appears to permit the window (see ticket 08). It does mean the prototype cannot be
handed to `/to-spec` until 08's `BR-11` amendment lands, so treat **every** editable field as
provisional, not only the title.

One constraint the prototype must actively preserve: `BR-65`
(`.what/bundle/02-rules/rules-bundle.md:29`) — *"Opening a Bundle shows what was composed, not a live
view of the Findings as they are now."* It stays true under this design and must keep staying true.
Anything in the window that re-reads a Finding to show its current state breaks it.

## Prototype, 2026-08-31 — three variants, and one of them disputes the question

**Asset:** `.scratch/bundle-library/design/ReviewUpdate.prototype.html` — self-contained, interactive,
listed in that folder's `README.md`. Readable at
https://claude.ai/code/artifact/37e75769-c23c-47f6-9bf7-3c9781278525 · `?variant=A|B|C`, or the
floating bar, or the arrow keys.

The fields are real: each variant tracks its own dirty set and prints it in the bar, Cancel opens that
variant's own discard treatment, and Save does what that variant claims. The bar also flips the app's
theme, because a rule made legible by colour has to survive both.

| | Variant | Its answer to "how do the narrower rules read?" |
|---|---|---|
| `A` | **Nothing added** | They do not need to. Nothing is added and two things are removed |
| `B` | **Provenance rail** | Said once in prose, in a second column that also names every unsaved change |
| `C` | **Read first, unlock to edit** | The window opens read-only; the primary button is `Edit`, not `Save` |

### Two facts established while building it, both verified in the code

**1. There is no affordance on an image in the compose window either.** `appwindow.slint:3470-3630` —
the whole document body of the existing modal — contains **zero** `TouchArea` and **zero**
`IconButton`. The Finding set is chosen in the filmstrip (`:2403`), never in this modal. So the
ticket's first question contains an assumption worth naming: there is nothing here to disable, and a
lock chip would have to **invent** a control in order to grey it out. Variant `A` is built on that;
`B` states the rule in words instead; `C` shows a chip only once the window is unlocked, when
everything around the image genuinely is editable.

**2. The modal is 521px wide, and that kills the obvious two-column answer.**
`appwindow.slint:3388-3391` sizes it `height = window - 64`, `width = height / 1.414` — so in an
800px-tall window it is `521 × 736`, A4 portrait. Variant `B`'s rail is drawn at `760px` and
**breaks that shared rule**. Either the two modals stop sharing a geometry, or the compose window gets
wider for a reason that has nothing to do with composing. That is a cost of `B`, not a bug in it, and
it is invisible on any canvas that does not draw the real width.

### A copy discrepancy the map should know about

The map's *Window copy* says create mode is titled **Review & Assemble**. The code says
**`"Assemble & Review"`** (`appwindow.slint:3416`). So the map's settled copy renames the *existing*
window too, not only the new one — small, but it belongs in whatever spec picks this up rather than
being discovered by a builder.

### What the prototype deliberately does not decide

`BR-11`'s narrowing landed on 2026-08-31 (`DEC-012`, `FR-40`), so the note above about this window
being forbidden entirely is **discharged** — every editable field here is now backed by a promise, and
the provisionality that note imposed is lifted. What is still open is the owner's, and an agent must
not answer it: which variant, and the four behaviours the three variants deliberately disagree about —
the header's contents, Save's enablement, where the Reviewer lands after saving, and whether the
window is read-only until unlocked.
