---
id: SPEC-02-CANVAS-MARKERS-CROP
title: Interactive Canvas, Step Markers & Crop Mode
status: ready-for-dev
source_prototype: .how/_platform/assets/ui-ux-complete-flow.html (States 1 & 3)
dedicated_html_asset: .how/finding/01-ux/assets/03-crop-mode.html
companions:
  - web/ui/src/styles/tokens.css
  - web/ui/src/components/MarkerLayer.tsx
  - web/ui/src/components/FindingsEditor.tsx
  - .what/finding/SRS-finding.md
  - .how/finding/SDD-finding.md
---

# SPEC-02: Interactive Canvas, Step Markers & Crop Mode

## 1. Scope & Objective
Implements the core image interaction area in Snapdown Studio:
1. **Clean Canvas Artboard**: Fullscreen-ready edge-to-edge canvas with zero distracting checkerboards.
2. **Numbered Step Markers (`1`, `2`, `3`)**: Draggable circular badges (`28x28px`, `#f59e0b` Amber) placed at fractional coordinates `(x: 0..1, y: 0..1)` and linked 1:1 with observation note lines.
3. **Crop Mode Overlay (State 3)**: Interactive bounding box with 8 resize handles, dimming scrim mask around crop area, and floating confirmation HUD (`Apply Crop (Enter)` / `Cancel (Esc)`).

---

## 2. Step Marker Architecture & Interaction Rules

### FR-CANVAS-1: Step Marker Stamping & Placement
- Clicking the canvas while **`🔢 Insert Marker`** tool is active inserts a new Marker at clicked position:
  - Coordinate stored as normalized percentage: `x_pct = clientX / imageWidth`, `y_pct = clientY / imageHeight`.
  - Automatically assigns the next sequential integer ordinal (`1`, `2`, `3`, ...).
  - Automatically appends a corresponding marker note line in the right Properties Panel (`SPEC-03`).

### FR-CANVAS-2: Marker Selection, Dragging & Deletion
- **Selection**: Clicking an existing marker focuses it:
  - Renders a glowing accent ring around the badge (`box-shadow: 0 0 0 3px #0284c7`).
  - Auto-scrolls and highlights the corresponding note textarea in the Properties Panel.
- **Dragging**: Markers are directly draggable (`cursor: grab` -> `cursor: grabbing`).
  - On mouse release, updates fractional coordinates in state.
- **Deletion**:
  - Pressing `Delete` or `Backspace` while a marker is focused deletes it.
  - Clicking the `🗑️ Delete Marker` ribbon button deletes the focused marker.
  - Re-indexes remaining marker ordinals sequentially (`1`, `2`, `3`) and syncs note lines immediately (`AD-1`).

---

## 3. Crop Mode Workflow (State 3)

```
+-----------------------------------------------------------------------------------------------+
|  [Ribbon] ... [✂️ Crop (Active)] ...                                                          |
+-----------------------------------------------------------------------------------------------+
|                                                                                               |
|  [====================== DARK SCRIM MASK (rgba(15, 23, 42, 0.7)) ==========================]  |
|                                                                                               |
|                +-----------------------------------------------+                              |
|                | [NW]                [N]                  [NE] |                              |
|                |                                               |                              |
|                |            ACTIVE CROP BOUNDARY               |                              |
|                | [W]                                       [E] |                              |
|                |                                               |                              |
|                | [SW]                [S]                  [SE] |                              |
|                +-----------------------------------------------+                              |
|                         [ ✓ Apply Crop (Enter) ] [ ✕ Cancel (Esc) ]                           |
|                                                                                               |
|  [==========================================================================================]  |
|                                                                                               |
+-----------------------------------------------------------------------------------------------+
```

### FR-CROP-1: Crop Activation & Scrim Overlay
- Clicking `✂️ Crop Image` or pressing `C` activates Crop Mode:
  - Background is masked with dark scrim `rgba(15, 23, 42, 0.7)`.
  - Crop area defaults to current image bounds or previous crop rect.
  - Renders 8 grab handles on boundary corners and edge centers (`NW`, `N`, `NE`, `E`, `SE`, `S`, `SW`, `W`).

### FR-CROP-2: Interactive Resizing & HUD Controls
- Dragging any handle resizes the active crop rectangle with real-time dimension readout (`width x height px`).
- Floating confirmation HUD docks beneath crop box:
  - **`✓ Apply Crop (Enter)`**: Crops the image, downscales/re-encodes buffer via `snapdown-capture`, recalibrates existing marker fractional coordinates relative to new crop rect, and exits crop mode.
  - **`✕ Cancel (Esc)`**: Discards crop changes and restores original view.

---

## 4. Test Obligations
- `vitest::canvas_places_marker_at_accurate_fractional_coordinates`
- `vitest::dragging_marker_updates_fractional_coordinates`
- `vitest::deleting_marker_reindexes_remaining_ordinals_and_notes`
- `vitest::activating_crop_mode_renders_8_handles_and_scrim`
- `vitest::applying_crop_updates_image_and_recalibrates_markers`
- `vitest::pressing_esc_cancels_crop_mode`
