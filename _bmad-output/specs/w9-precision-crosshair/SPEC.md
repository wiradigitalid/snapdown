---
id: SPEC-w9-precision-crosshair
title: Precision Crosshair Guides & Live Coordinate HUD
status: draft
companions:
  - .what/finding/SRS-finding.md
  - .how/finding/SDD-finding.md
  - .how/finding/01-ux/DESIGN.md
  - apps/desktop/src/components/CaptureOverlay.tsx
  - web/ui/src/styles/tokens.css
sources:
  - .control/registry/requirements.yaml
---

# SPEC-w9: Precision Crosshair Guides & Live Coordinate HUD

## 1. Intent & Context
Standard region drag overlays provide only a bounding box without alignment cues. Reviewers capturing specific UI components often struggle to align regions to browser edges or grid boundaries across multi-monitor setups.
This spec introduces full-screen crosshair guide axes, dynamic coordinate indicators, and aspect ratio awareness directly into `CaptureOverlay.tsx`.

## 2. Functional Requirements & Acceptance Criteria

### FR-GUIDE-1: Full-Screen Axis Guides
- When `CaptureOverlay` is in `armed` or `dragging` phase:
  - Render a vertical axis line passing through the current cursor `clientX` from `top: 0` to `bottom: 100vh`.
  - Render a horizontal axis line passing through the current cursor `clientY` from `left: 0` to `right: 100vw`.
  - Style: `1px solid var(--color-overlay-ring)` with 40% opacity, `pointer-events: none`, `z-index: var(--z-overlay)`.
- When entering `narrating` or `saving` phase, axis lines are immediately unmounted to prevent visual clutter around the note prompt.

### FR-GUIDE-2: Floating Coordinate HUD Badge
- When in `armed` phase, render a floating HUD badge near the cursor:
  - Label text: `X: {clientX}  Y: {clientY}`.
  - Position: Offset `+16px` right and `+16px` down from the cursor point.
  - Boundary guard: If `clientX + badgeWidth > window.innerWidth`, flip horizontal offset to `-badgeWidth - 16px`. If `clientY + badgeHeight > window.innerHeight`, flip vertical offset to `-badgeHeight - 16px`.
  - Typography: `var(--font-mono)`, `font-size: var(--text-xs)`, `background-color: var(--color-surface-sunken)`.

### FR-GUIDE-3: Aspect Ratio Readout during Drag
- During `dragging` phase:
  - The dimension readout tag displays `width × height px`.
  - If the calculated ratio matches common media standards within ±2% tolerance, append the ratio tag:
    - 16:9 (`1.77`): `(16:9)`
    - 4:3 (`1.33`): `(4:3)`
    - 1:1 (`1.00`): `(1:1)`
    - 21:9 (`2.33`): `(21:9)`

## 3. Invariants & Non-Functional Constraints
- **NFR-PERF-1**: Mouse movement updates must execute at 60 FPS without layout thrashing. State updates must not trigger re-render of unrelated DOM subtrees.
- **AD-10**: All guide colors must use CSS tokens (`var(--color-overlay-ring)`, `var(--color-overlay-scrim)`).

## 4. Test Obligations
- `vitest::overlay_renders_crosshair_axes_in_armed_phase`
- `vitest::overlay_positions_coordinate_badge_with_edge_flipping`
- `vitest::readout_displays_aspect_ratio_tag_on_square_and_widescreen_drags`
- `vitest::axes_and_hud_unmount_when_narrating_note`
