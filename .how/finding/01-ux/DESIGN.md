---
type: ux
component: finding
document: design
created: "2026-08-23"
updated: "2026-08-23"
---

# DESIGN — Finding

Tokens, elements, iconography, and both themes are in [`.how/_platform/design-system.md`](../../_platform/design-system.md) and demonstrated live in [`.how/_platform/assets/design-system.html`](../../_platform/assets/design-system.html).

### Living Design Assets (.how Layer)
- **Finding Studio Workspace**: [`.how/finding/01-ux/assets/01-studio-workspace.html`](assets/01-studio-workspace.html)
- **Capture Scrim & Note HUD**: [`.how/finding/01-ux/assets/02-capture-overlay.html`](assets/02-capture-overlay.html)
- **Crop Mode Overlay**: [`.how/finding/01-ux/assets/03-crop-mode.html`](assets/03-crop-mode.html)

## Tokens

Component-specific only.

| Token | For |
|---|---|
| `--rail-thumb-width` | `176px` — a capture thumbnail in the rail |

**`--overlay-dim`, `--overlay-region-ring` and `--canvas-checker` are NOT here.** An earlier draft of
this document defined them locally and argued they should "stay here rather than being promoted",
because they are drawn over the Reviewer's screen content rather than over a Snapdown surface.

That reasoning is right about *why they do not follow the theme* and wrong about *where they live*.
`AD-10` requires every colour to be defined once in the token file, and says explicitly that a
deliberately theme-invariant token must still be defined there and must say why. All three are in
`.how/_platform/design-system.md` § Colour — theme-invariant, alongside `--color-marker*`, which was
already handled that way. Corrected 2026-08-23; the spine wins over a component document.

## Screens

| Screen | LC | Purpose |
|---|---|---|
| Capture Overlay | `LC-001` `capture-overlay` | Region selection. Serves `FR-1` |
| Capture note field | `LC-029` `capture-note-field` | The narration step. Serves `FR-2` |
| Findings | `LC-006` `findings-editor` | Rail, canvas, note pane. Serves `FR-6`, `FR-7`, `FR-9` |
| Marker canvas | `LC-007` `marker-canvas` | Markers on one image. Serves `FR-8` |
| Orphan report | `LC-030` `orphan-report` | Serves `FR-15` |

`LC-029` and `LC-030` are new and registered as part of landing this document. Both are screens the
inventory already named (rows 2 and 7) that had no build unit behind them.

## Layout and states

### Capture Overlay (`LC-001`) and note field (`LC-029`)

One transparent, always-on-top window per monitor. No chrome, no title, no toolbar. Features full-screen crosshair axes, floating pixel loupe with live pixel grid/dimensions, smart container/panel auto-detection with dynamic un-dimmed cutout preview, and top-center Fullscreen shortcut.

```
                    [ 🖥️ Fullscreen ] (Top-Center)
────────────────────────────────────────────────────────────
              │ (Vertical Full-Screen Crosshair Guide)
              │
    ┌─────────┼─────────────────────────────────┐
    │ PANEL / WINDOW DETECTED                   │
    │ (Un-dimmed Sharp Preview)                 │
    │                                           │
────┼─────────● (Cursor) ───────────────────────┼───────────
    │         │ (Circular Loupe Magnifier 6x)   │ (Horizontal Guide)
    │        [417 × 1192 px]                    │
    └───────────────────────────────────────────┘
              (Rest of screen remains dimmed)

        ╔═══════════════════════════════════════╗
  dim   ║   selected region (1-click or drag)   ║   dim
        ╚═══════════════════════════════════════╝
              1408 × 620 px (16:9)  ← readout, --font-mono
        ┌───────────────────────────────────────┐
        │ What is wrong here?                   │  ← LC-029, anchored beneath
        └───────────────────────────────────────┘
              Enter to save · Esc to cancel
```

- The readout sits **outside** the region, never over the content being captured. During drag, standard aspect ratios (16:9, 4:3, 1:1, 21:9) are automatically tagged.
- Top-center button `[ 🖥️ Fullscreen ]` enables 1-click capture of the active monitor.
- Smart auto-detection highlights top-level windows and sub-panels as un-dimmed cutouts; 1-click on a highlighted container selects it immediately.
- Re-selection is supported: clicking or dragging elsewhere before saving instantly selects a new region without requiring dismiss.
- The note field anchors beneath the region, and flips above it when the region is near the screen foot. It never covers the thing being described.
- The hint line is `--text-xs`, `--color-text-muted`, and is the only instruction anywhere in the capture path.

| State | Rendering |
|---|---|
| Armed | Full-screen crosshair axes, pixel loupe magnifier near pointer, auto-detect container cutout highlights, top-center Fullscreen button |
| Dragging | Region undimmed with `--overlay-region-ring`; full-screen crosshairs track pointer; readout tracks region with aspect ratio tag |
| Narrating | Region stays lit, note field focused, crosshairs/loupe unmounted, re-selection enabled by clicking/dragging outside |
| Saving | Overlay already dismissed. Reduction runs behind it (`NFR-2`) |
| Error | Overlay gone; a `Toast` with `--color-danger` names the Vault folder and offers Settings |

### Findings (`LC-006`)

**Three columns, and the middle one is the one that grows.** This is the Snagit reading of the
problem — a recents rail, a canvas, a properties pane — with Cobalt's note-beside-the-image in the
third column instead of a tool inspector.

```
┌ rail 200px ─┐┌─ canvas (flex) ────────────┐┌ note 320px ─┐
│ ☐ ▣ 12:04   ││                            ││ Note        │
│ ☐ ▣ 12:01   ││        ①                   ││ ┌─────────┐ │
│ ☑ ▣ 11:58   ││              ②             ││ │1. the … │ │
│ ☐ ▣ 11:52   ││                            ││ │2. this …│ │
│             ││        ③                   ││ └─────────┘ │
│             ││                            ││ Markers     │
│             ││   1408 × 620 · 184 KB      ││  ① ② ③      │
├─────────────┤└────────────────────────────┘├─────────────┤
│ 1 selected  │                              │ [Delete]    │
│ [Compose →] │                              │             │
└─────────────┘                              └─────────────┘
```

- **The rail is the capture rail, not a sidebar.** Thumbnails, newest first, timestamp beneath.
  A checkbox appears on hover or keyboard focus; it does not occupy space permanently.
- **The rail's foot is the multi-select action** (`FR-9`): a count and one button, Compose. It appears
  only when something is selected, and it is the only bridge from Finding to Bundle.
- **The canvas fits the image to the pane** and shows its real dimensions and stored size beneath —
  which is where `NFR-18`'s recorded budget surfaces to a human.
- **The note pane lists the Markers under the Note**, each row focusable, so Markers are reachable
  without the canvas. That list is what makes the keyboard path in the accessibility floor real.

The panels take the height available and the columns are independent. The shipped build renders both
list surfaces at a fixed height and leaves a third of the window dark beneath them.

| State | Rendering |
|---|---|
| Empty | The three columns collapse to one centred `EmptyState`: "No findings yet", then the bound capture combination rendered as a `HotkeyChip`, not a button |
| Loading | Rail shows four skeleton thumbs at `--rail-thumb-width`; canvas and note pane hold their shape. No layout shift |
| Nothing selected | Rail populated; canvas shows a muted "Select a finding" and the note pane is empty and inert. Visually distinct from the empty state |
| Populated | Normal |
| Image missing | Canvas shows a `--color-warning-bg` panel naming the missing file, with one action: open the orphan report |
| Error | Centred `ErrorState`: the Library could not be read, one Retry |

**Every panel here draws from `--color-surface` and `--color-text`.** `FindingsView` used to paint
`#ffffff` and `#f8fafc` panels regardless of theme, so under the Windows dark theme the shell's white
`--color-text` landed on them — the white-on-white the Reviewer reported. **Resolved by `W6-S1` at
`420ecce`**, and by the only fix that holds: no literal in the component at all (`NFR-17`).

### Marker canvas (`LC-007`)

A Marker is a `28px` disc: `--color-marker` fill, `--color-marker-text` numeral, `2px`
`--color-marker-ring`. The ring is what keeps it legible over a dark screenshot and a light one, which
is why the Marker group does not follow the app theme.

| State | Rendering |
|---|---|
| Idle | Disc at its point |
| Hover / focus | Ring thickens to `3px`; the matching note row highlights simultaneously |
| Dragging | Disc follows the pointer at 80% opacity; the point beneath stays marked |
| Unbound | The disc is unchanged; the **note pane** shows the mismatch. The image is the artifact that gets exported, so it never carries an app-only state |

### Orphan report (`LC-030`)

A plain table: file, size, last seen. One action, Delete, confirmed once. Empty state — "Nothing
orphaned" — is the state this screen should almost always be in, and it says so rather than looking
broken.

## Do's and don'ts for this surface

**Do** keep the capture path to a crosshair, a readout, and one field.
**Do** let the note pane be the keyboard path to every Marker.
**Do** show dimensions and stored size where the Reviewer can see them.

**Don't** add a tool palette, an arrow, a callout, a highlight, or a blur. Non-Goal in the PRD, and the
same conclusion Cobalt Capture reached for the same audience.
**Don't** put chrome on the overlay. Every pixel of it is over the thing being described.
**Don't** require the Editor for a capture.
**Don't** write a literal colour in a component — this surface is where that habit did its damage.
