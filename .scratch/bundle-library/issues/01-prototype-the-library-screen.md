# 01: Prototype the Library screen

**Type:** prototype
**Status:** resolved
**Blocked by:** None (can start immediately)

## Question

There is no UI or UX for the Library yet — only a stubbed `library-clicked` callback
(`appwindow.slint:1753-1759`, handler is a `println!` at `main.rs:4122`). What does this screen
actually look like, and how does a Reviewer move in and out of it?

Make something cheap and concrete to react to, then decide:

- **Shape and reach.** Full screen replacing the Editor canvas, a modal over it, or a panel? How
  does the Reviewer get back to what they were doing, and what happens to unsaved canvas state?
- **The row.** What does one Bundle show — name, Finding count, composed date, a thumbnail, a
  Markdown excerpt? Newest-first is the store's existing order (`list_bundles()` sorts by
  `composed_at DESC`).
- **Where the four actions live** on a row: `Edit` · `Copy Markdown` · `Open file location` ·
  `Delete`. Always visible, on hover, or behind an overflow menu? Room must be left for `Export PDF`
  and `Publish` to join later without a redesign.
- **Empty state.** What a Reviewer who has never assembled anything sees, and what it tells them to
  do next.
- **Loading and failure.** What shows while bundles load, and what shows when the store cannot be
  read.
- **Scale.** Whether search, filter, or pagination is needed at all, or deferred until someone has
  enough Bundles to want it.

Read `.constitution/project/design-system-guide.md` first — it governs any UI in either window.

Worth reading as prior art, not as a template: `archive/desktop-tauri/src/components/BundleView.tsx`
(the pre-Slint React screen: list + read-only preview + item list, with copy/delete) and
`.what/bundle/04-usecases/EXPERIENCE.md`, which describes the intended three-region layout. Both
predate the decision that the Library is *editable*, so their read-only framing no longer holds.

The store side already exists and needs nothing new: `list_bundles()`, `get_bundle()`,
`update_bundle_markdown()`, `delete_bundle()` are all implemented in
`crates/snapdown-store/src/sqlite/bundle_store.rs`.

## Proposal (2026-08-31) — canvas published, awaiting the owner's pick

Design canvas: https://claude.ai/code/artifact/6f798d70-77a0-491e-973b-92e7a2641a2f

Six artboards: Library populated (dark), the same in light, Empty · Loading · Cannot-be-read, and
Options A/B/C for the row actions. Every value is lifted from `theme.slint`, `SdModalHeader`,
`SdActionButton` and `SdContextMenu` rather than chosen on the canvas.

**Proposed and not contentious:**

- **Shape.** The Library opens as a full-window overlay, the same pattern as Review & Assemble, so
  the Editor behind it keeps its state. Closed with the header's X.
- **Row content.** A 44×30 thumbnail of the Bundle's first burned image, the name at 13px, and a
  mono meta line (`N Findings · <relative date>`). The thumbnail is an addition — a list of names
  alone is hard to scan, and `BundleItem.image_path` already holds what it needs. It sits on
  `canvas-ground` and is therefore theme-invariant, identical in both themes.
- **Empty state** names the actual next action ("Tick Findings in the strip and press Assemble"),
  rather than describing the emptiness.
- **Loading** is skeleton rows in the list's own shape, so nothing jumps when data arrives.
- **Failure** states what failed and offers Try again beside Open file location.
- **No search or filter.** The corpus puts it in r2 and the map records it as fog.

**Still the owner's call:** which row-action option. A shows all four labelled (nothing hidden, but
it cannot absorb Export PDF and Publish). B — the leading candidate — opens the Bundle from the row,
reveals two icons on hover, and puts the rest in a menu that right-click also opens, matching the
filmstrip's existing gesture. C is right-click only: quietest, but Copy Markdown becomes
undiscoverable, and that is the action the Library exists to serve.

A second pass over the artboards caught and fixed: two invented hex values, a 28px close button that
should be 26px, `text-dim` section headings that should be `text-secondary`, a missing accent shadow
on the primary button, a sans-serif menu hint that should be mono, prose at a font size the scale has
no role for, structural drift between the dark and light artboards, and an open context menu that
painted over the artboard below it.

### Settled: the row thumbnail stays

Owner's call, 2026-08-31, on the canvas comment thread. The thumbnail is kept, and it is **always
the `BundleItem` at `position 1`** — automatic, no stored choice, no new field.

Why it needs no choice: `BundleItem.position` is the selection order, dense and never reordered after
composition, so "the first image" is already deterministic. And why a *chosen* thumbnail was rejected
rather than merely not built: it would need a new field on `Bundle`, which is the same
"second source of truth" the PRD used to reject both reordering (`prd.md:715`) and renaming
(`prd.md:717`).

The known cost, accepted: one Vault blob read plus a PNG decode per row each time the Library opens.
If that bites once a Reviewer has many Bundles, the fix is lazy-loading the visible rows — not
removing the thumbnail.

## Answer

Resolved 2026-08-31. Canvas: https://claude.ai/code/artifact/6f798d70-77a0-491e-973b-92e7a2641a2f
Three artboards — Library dark with a row right-clicked, the same in light, and
Empty · Loading · Cannot-be-read. Every value is lifted from `theme.slint`, `SdModalHeader`,
`SdActionButton` and `SdContextMenu`.

- **Shape.** A full-window overlay, the same pattern as Review & Assemble, so the Editor behind it
  keeps its state. Closed from the header's X.
- **Row actions: two on hover, the rest in the menu (Option B).** The row itself opens the Bundle.
  Copy Markdown and Open file location appear on hover; everything destructive or rare lives in the
  menu, which the overflow button opens and right-click opens too — the gesture the filmstrip already
  teaches. Export PDF and Publish join the menu later without touching the row, which is precisely
  what an all-labelled row could not have absorbed. **Accepted cost:** the two hover icons are
  invisible until the pointer is on the row.
- **Menu order:** Edit · Copy Markdown · Open file location · Export PDF — separator — Publish —
  separator — Delete Bundle…
- **Export PDF carries no "soon" marker**; it was grown into MVP scope by ticket 08. **Publish is
  marked**, being the one genuinely blocked, by `DEC-005`.
- **Row content.** Thumbnail, name at 13px, mono meta line. See the thumbnail decision above.
- **Empty state** names the next action rather than describing the emptiness. **Loading** is skeleton
  rows in the list's own shape so nothing jumps. **Failure** says what failed and offers Try again
  beside Open file location.
- **No search or filter** — r2 per the corpus, and recorded as fog on the map.

**One flag carried forward, not resolved here:** a greyed "Publish — soon" row in shipped UI is the
same shape as the fake 1-6 shortcut badges just removed from the toolbar — an affordance promising
something the app cannot do. The owner asked for the marker and it is drawn, but before this ships,
consider not rendering Publish at all until it works. Recorded as a canvas annotation too.

## Design source

The artboards this ticket produced live in `.scratch/bundle-library/design/`, beside the tickets,
and are what to build from — they carry exact hex, px and weights, and their `README.md` records
which running-app component each value came from. The canvas is the view of them:
https://claude.ai/code/artifact/6f798d70-77a0-491e-973b-92e7a2641a2f
