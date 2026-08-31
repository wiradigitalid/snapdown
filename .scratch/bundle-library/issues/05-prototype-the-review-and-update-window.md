# 05: Prototype the Review & Update window

**Type:** prototype
**Status:** open
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
