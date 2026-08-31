# Library design source

The artboards behind the Library design canvas, kept here rather than only in the published
artifact so an implementer can read **exact values** — hex, px, weights, spacing — instead of
measuring them off a picture.

Canvas: https://claude.ai/code/artifact/6f798d70-77a0-491e-973b-92e7a2641a2f

| File | Shows |
|---|---|
| `Main.dc.html` | The Library, populated, dark, a row hovered with its menu open |
| `LibraryLight.dc.html` | The same in light theme — generated from `Main.dc.html` by token substitution, so the two cannot drift |
| `States.dc.html` | Empty · Loading · Cannot be read |
| `RowStates.dc.html` | A Bundle that still holds its Findings vs one whose originals are discarded, with both menus |
| `Dialogs.dc.html` | The four confirmations: Disassemble · Discard originals · Delete both · Delete (sealed) |
| `ReclaimSpace.dc.html` | The bulk screen, reached from the Library header and from Settings |
| `ReclaimEmpty.dc.html` | Reclaim space with nothing to reclaim |
| `canvas.json` | Artboard layout and the sticky notes carrying the reasoning |
| `ReviewUpdate.prototype.html` | **Not an artboard.** Ticket 05's prototype: three variants of the Review & Update window, standalone and interactive |

**Every value in these files was lifted from the running app, not invented on the canvas**:
`apps/desktop/ui/theme.slint` for colour, `SdModalHeader` (52px, 15px accent icon, 26px close),
`SdActionButton` (32px, radius 3px, 12px padding-x, 11px/700), `SdContextMenu` (210px wide, 28px
rows, 7px separators, 12px labels, 11px mono hints), `SdCheckbox` (15px, radius 3px,
`accent-primary` when checked), and the scale table in
`.constitution/project/design-system-guide.md`.

Two conventions worth not re-deriving:

- **Thumbnails are theme-invariant.** They sit on `canvas-ground`, so their bars keep the dark
  values in both themes — the same reasoning that makes the capture overlay theme-invariant.
- **`LibraryLight.dc.html` is derived, not hand-written.** It was produced from `Main.dc.html` by
  mapping dark tokens to light ones while skipping the thumbnail internals. Regenerate it that way
  rather than editing it, or the two will drift the way they did once already.

These are `.dc.html` Design Component files. They render as artboards on the canvas; opening one
directly in a browser will not show much, since the canvas supplies the runtime.

`ReviewUpdate.prototype.html` is the exception and is **self-contained** — double-click it, or read it
at https://claude.ai/code/artifact/37e75769-c23c-47f6-9bf7-3c9781278525. It is a throwaway prototype,
not a source of truth, and it is marked as one in its own first line. It differs from the artboards in
one way worth copying: its tokens are CSS custom properties, so the light theme is **derived** from the
same block rather than kept as a second hand-written copy — which is the drift `LibraryLight.dc.html`
warns about, solved instead of documented.
