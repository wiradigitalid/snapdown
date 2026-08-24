---
id: SPEC-w10-loupe-magnifier
title: Loupe Magnifier (Pixel Zoom Lens & Color Inspector)
status: draft
companions:
  - .what/finding/SRS-finding.md
  - .how/finding/SDD-finding.md
  - crates/snapdown-capture/src/capturer.rs
  - apps/desktop/src/components/CaptureOverlay.tsx
sources:
  - .control/registry/requirements.yaml
---

# SPEC-w10: Loupe Magnifier (Pixel Zoom Lens & Color Inspector)

## 1. Intent & Context
Pixel-exact cropping requires visual inspection of UI edges (borders, shadows, icons). A magnifying loupe floating beside the cursor renders an enlarged pixel grid of the region beneath the cursor and displays the exact RGB/HEX color value.

## 2. Functional Requirements & Acceptance Criteria

### FR-LOUPE-1: Initial Screen Snapshot Buffer
- When `trigger_overlay` opens the overlay, Rust `snapdown-capture` takes a single full-monitor snapshot and caches it in memory during overlay liveness.
- Tauri Command `get_screen_preview(monitor: Option<String>) -> Result<Vec<u8>, String>` serves the initial unreduced PNG/raw image buffer to the frontend.

### FR-LOUPE-2: Magnifier Viewport & Pixel Grid
- The `<CaptureLoupe />` component renders a 128×128 px circular or rounded-box viewport.
- Displays a 21×21 pixel matrix around the cursor with `6x` magnification (`image-rendering: pixelated`).
- Overlays a subtle 1-px pixel grid (`rgba(255, 255, 255, 0.15)`) between adjacent magnified pixels.
- Features a central 1-px crosshair reticle targeting the exact pixel coordinate under the cursor.

### FR-LOUPE-3: Live Color Inspector Readout
- Below the magnified viewport, display:
  - Hex code: `#RRGGBB`
  - RGB code: `rgb(r, g, b)`
  - Color swatch swatch pill showing the sampled color.
- Contrast protection: Text color automatically switches between light/dark token based on relative luminance of the sampled pixel.

### FR-LOUPE-4: Keyboard Micro-Nudge
- When overlay is active, keyboard arrow keys (`ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`) move the sampling point by exactly 1 physical pixel.
- `Shift + Arrow` moves the sampling point by 10 pixels.

### FR-LOUPE-5: Collision Avoidance (Quadrant Flipping)
- The loupe sits at an offset of `+24px` horizontal and `+24px` vertical from the cursor.
- If cursor enters bottom-right screen quadrant or is inside an active drag box, the loupe flips to the opposite quadrant so it never obscures the drag rectangle.

## 3. Invariants & Non-Functional Constraints
- **NFR-PERF-2**: Sampling from local ImageBitmap/Canvas in JS must take `<2ms` per mousemove event.
- **NFR-MEM-1**: Snapshot buffer in memory must be freed immediately when overlay is dismissed or capture completes.

## 4. Test Obligations
- `cargo::screen_snapshot_buffer_returns_valid_decoded_image`
- `vitest::loupe_magnifies_pixel_grid_centered_at_cursor`
- `vitest::loupe_extracts_accurate_hex_color_at_coordinates`
- `vitest::arrow_keys_nudge_cursor_coordinates_by_one_pixel`
- `vitest::loupe_repositions_to_avoid_covering_selection_box`
