---
type: ux
component: finding
document: design
created: "2026-08-23"
updated: "2026-08-23"
---

# DESIGN — Finding

Tokens, elements, and both themes are in `.how/_platform/design-system.md`.

## Tokens

| Token | For |
|---|---|
| `--overlay-dim` | `rgba(0,0,0,0.45)` — the scrim over everything outside the selected region |
| `--overlay-region-ring` | `1px solid #ffffff` plus a `1px` black outer line, so the edge is visible over any content |
| `--rail-thumb-width` | `176px` — a capture thumbnail in the rail |
| `--canvas-checker` | The transparency checkerboard behind an image that does not fill the pane |

`--overlay-*` and `--canvas-checker` are **theme-invariant on purpose** and stay here rather than being
promoted. They are drawn over the Reviewer's screen content or over an image, not over Snapdown's own
surfaces, so the app's theme is the wrong reference for them. This is the same reasoning that keeps
`--color-marker*` theme-invariant in the design system.

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

One transparent, always-on-top window per monitor. No chrome, no title, no toolbar.

```
        ╔═════════════════════════════════╗
  dim   ║   the selected region, sharp    ║   dim
        ╚═════════════════════════════════╝
              1408 × 620          ← readout, --font-mono
        ┌─────────────────────────────────┐
        │ What is wrong here?             │  ← LC-029, anchored beneath
        └─────────────────────────────────┘
              Enter to save · Esc to cancel
```

- The readout sits **outside** the region, never over the content being captured.
- The note field anchors beneath the region, and flips above it when the region is near the screen
  foot. It never covers the thing being described.
- The hint line is `--text-xs`, `--color-text-muted`, and is the only instruction anywhere in the
  capture path.

| State | Rendering |
|---|---|
| Armed | Full dim, crosshair cursor, readout follows the pointer |
| Dragging | Region undimmed with `--overlay-region-ring`; readout tracks the region |
| Narrating | Region stays lit, note field focused |
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

**Every panel here draws from `--color-surface` and `--color-text`.** The shipped `FindingsView` paints
`#ffffff` and `#f8fafc` panels regardless of theme, and under the Windows dark theme the shell's white
`--color-text` lands on them. That is the white-on-white the Reviewer reported, and it is fixed by
having no literal in the component at all (`NFR-17`).

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
