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

### Canvas Annotation & Marker Layer (`LC-007`)

The canvas supports structured numbered Markers (which bind to Markdown note lines) alongside rich visual markup elements (Shape, Callout, Blur, Arrow, Text) that render onto the image overlay without creating Markdown lines.

```
┌─────────────────────────────────────────────────────────────┐
│ [ ● Marker ] [ ▢ Shape ] [ 💬 Callout ] [ ░ Blur ] [ ↗ Arrow ] [ T Text ]  [ ↶ Undo ] [ ↷ Redo ]
└─────────────────────────────────────────────────────────────┘
┌─ canvas area ───────────────────────────────────────────────┐
│                                                             │
│       ┌─────────┐ (Shape: red outline, transparent fill)    │
│       │ ▢       │                                           │
│       └─────────┘                                           │
│                 \                                           │
│                  \ (Arrow: start handle & end arrowhead)    │
│                   ↘                                         │
│       ① (Numbered Marker: solid red circle, white num)      │
│                                                             │
│       [ ░░░░░░░░░ ] (Blur Redaction Box)                    │
│                                                             │
│       ┌──────────────────┐                                  │
│       │ Custom Text      │╲                                 │
│       └──────────────────┘ ╲ (Callout: text bubble + tail)  │
│                             ●                               │
│                                                             │
│       Floating Text [Font: Inter / 16px]                    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

#### Canvas Interaction Model & Control Points

1. **Toolbar Controls**:
   - Primary Tools: `Marker` (1), `Shape` (2), `Callout` (3), `Blur` (4), `Arrow` (5), `Text` (6).
   - Global Actions: `Undo` (Ctrl+Z), `Redo` (Ctrl+Y), `Delete` (Del/Backspace).
2. **Marker**:
   - 28px solid red circle with bold white numeral in center (no border outline).
   - Direct click places next sequential marker (e.g. 1, 2, 3) and appends line to Markdown note.
   - Draggable central handle to reposition.
3. **Shape (Rectangle Outline)**:
   - Transparent fill with solid stroke (`--color-marker` default / red).
   - Active state displays 8 bounding control handles (4 corners + 4 edge midpoints) for free proportional or edge resizing.
   - Clicking and dragging interior moves/translates the shape.
4. **Callout**:
   - Text bubble container with custom background/border + adjustable pointer tail.
   - Double-click inside bubble enters inline text editing mode.
   - Property bar allows customizing font family and font size.
   - Tail anchor features a dedicated circular control handle that can be dragged in 360 degrees to point at any target spot.
   - 8-point handles on the bubble box for resizing the text area.
5. **Blur (Redaction Box)**:
   - Rectangular box that applies real-time Gaussian / pixelation shader over underlying screenshot pixels.
   - 8-point resize handles for boundary adjustments.
   - Burnt permanently into export PNGs to prevent leaking confidential tokens/PII.
6. **Arrow / Line**:
   - Directional vector line with distinct arrowhead at destination point.
   - 2 primary control handles: Start point handle and End (head) point handle.
   - Dragging handles repositions/rotates the arrow; dragging the line body translates the entire arrow.
7. **Floating Text**:
   - Borderless text block. Double-click enters inline typing.
   - Editable font family, font size, and text color.
   - Draggable bounding box to reposition anywhere on the screenshot.

| Element | Control Handles | Markdown Binding? | Deletion / Manipulation |
|---|---|:---:|---|
| **Marker** | 1 Center drag point | **YES** | Delete removes note line & renumbers; drag updates coordinate |
| **Shape** | 8 Box handles (Corners + Midpoints) | **NO** | Delete removes shape; handles resize; body drags |
| **Callout** | 8 Box handles + 1 Tail point | **NO** | Delete removes callout; tail points; double-click edits text |
| **Blur** | 8 Box handles | **NO** | Delete removes blur; handles resize; body drags |
| **Arrow** | 2 Endpoints (Start + End head) | **NO** | Delete removes arrow; endpoints change direction/length; shaft drags |
| **Text** | 4 Corner handles | **NO** | Delete removes text; double-click edits text; handles scale/reflow |

## Do's and don'ts for this surface

**Do** keep the capture path to a crosshair, a readout, and one field.
**Do** let the note pane be the keyboard path to every Marker.
**Do** show dimensions and stored size where the Reviewer can see them.
**Do** keep visual canvas annotations (Shape, Arrow, Callout, Text, Blur) strictly separate from Markdown finding notes.

**Don't** mix visual annotations into the structured Markdown note lines — Markers remain the only structured finding bridges.
**Don't** put chrome on the overlay. Every pixel of it is over the thing being described.
**Don't** require the Editor for a capture.
**Don't** write a literal colour in a component — this surface is where that habit did its damage.
