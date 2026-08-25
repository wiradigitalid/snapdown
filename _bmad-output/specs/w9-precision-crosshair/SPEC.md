---
id: SPEC-w9-precision-crosshair
title: Precision Crosshair Guides, Pixel Loupe Magnifier & Smart Window/Panel Auto-Detection
status: ready-for-dev
companions:
  - .what/finding/SRS-finding.md
  - .how/finding/SDD-finding.md
  - .how/finding/01-ux/DESIGN.md
  - .how/finding/06-flows/flow-capture.md
  - apps/desktop/src/components/CaptureOverlay.tsx
  - web/ui/src/styles/tokens.css
sources:
  - .control/registry/requirements.yaml
  - .control/registry/usecases.yaml
---

# SPEC-w9: Precision Crosshair Guides, Pixel Loupe Magnifier & Smart Window/Panel Auto-Detection

## 1. Intent & Context
Standard region drag overlays provide only an unassisted bounding box. Reviewers capturing UI defects require pixel-exact alignment tools and fast 1-click selection of windows, sidebars, and sub-panels (Snagit-style experience).

This spec integrates:
1. Full-screen crosshair axis guides.
2. Floating circular loupe magnifier (6x-8x) with pixel grid and live dimensions/aspect ratio tag.
3. Top-center Fullscreen shortcut button.
4. Smart auto-detection of windows & sub-panels with dynamic cutout preview (highlighted target is sharp/normal, outer canvas is dimmed).
5. Interactive 1-click selection vs manual freeform drag and seamless re-selection before saving.

## 2. Functional Requirements & Acceptance Criteria

### FR-GUIDE-1: Full-Screen Crosshair Axis Guides
- When in `armed` or `dragging` phase:
  - Render a vertical axis line passing through `clientX` (`top: 0` to `bottom: 100vh`).
  - Render a horizontal axis line passing through `clientY` (`left: 0` to `right: 100vw`).
  - Style: `1px dashed var(--color-overlay-ring)` or solid with 50% opacity, `pointer-events: none`, `z-index: 1000`.
- When in `narrating` (note prompt) or `saving` phase, crosshair lines unmount automatically.

### FR-GUIDE-2: Circular Pixel Loupe Magnifier
- In `armed` and `dragging` phase, render a floating circular loupe (~100–120px diameter):
  - Magnifies pixels under the cursor at 6x-8x magnification (`image-rendering: pixelated`).
  - Overlays a subtle 1px pixel grid and a central crosshair reticle target.
  - Attached bottom dark badge displaying live dimensions `W × H px` or `X: clientX  Y: clientY`.
  - Boundary guard: Flips position quadrant if near screen edges or inside active selection box so it never obscures the target.

### FR-GUIDE-3: Top-Center Fullscreen Button & 1-Click Monitor Capture
- Render floating button `[ 🖥️ Fullscreen ]` at `top: 12px, left: 50%, transform: translateX(-50%)`.
- Hovering illuminates the entire screen; clicking selects 100% monitor viewport `(0, 0, window.innerWidth, window.innerHeight)` and transitions immediately to `narrating` phase.

### FR-GUIDE-4: Smart Window / Panel Auto-Detection & Cutout Preview
- In `armed` phase when hovering:
  - Automatically identifies rectangular container boundaries under the mouse (via Win32 DWM bounds or DOM element rects).
  - Highlights the suggested container with an un-dimmed cutout preview (sharp target, dimmed background) and dashed outline.
  - 1-Click on a highlighted container locks its boundary and transitions to `narrating` without requiring drag.

### FR-GUIDE-5: Re-Selection & Aspect Ratio Tags
- During drag, calculate aspect ratio tags within ±2% tolerance:
  - 16:9 (`1.77`), 4:3 (`1.33`), 1:1 (`1.00`), 21:9 (`2.33`).
- In `narrating` phase, clicking or dragging outside the committed region allows re-selecting another region immediately.

## 3. Test Obligations
- `vitest::overlay_renders_crosshair_axes_in_armed_phase`
- `vitest::overlay_renders_fullscreen_button_and_selects_viewport_on_click`
- `vitest::overlay_renders_loupe_magnifier_and_tracks_pointer`
- `vitest::hovering_auto_detects_container_and_1_click_selects_bounds`
- `vitest::dragging_displays_live_aspect_ratio_tags`
- `vitest::clicking_outside_reselects_new_region`
