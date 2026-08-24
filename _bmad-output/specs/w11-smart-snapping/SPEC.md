---
id: SPEC-w11-smart-snapping
title: Smart Window & UI Element Auto-Detection (Magnetic Snapping)
status: draft
companions:
  - .what/finding/SRS-finding.md
  - .how/finding/SDD-finding.md
  - crates/snapdown-capture/src/capturer.rs
  - apps/desktop/src/components/CaptureOverlay.tsx
sources:
  - .control/registry/requirements.yaml
---

# SPEC-w11: Smart Window & UI Element Auto-Detection (Magnetic Snapping)

## 1. Intent & Context
Manual drag selection of windows and sub-elements (dialogs, buttons, sidebars) is tedious and prone to including unwanted 1-px margins or window shadows.
This spec introduces Win32 DWM and UI Automation (UIA) window/control boundary detection, enabling 1-click capture of whole windows/controls and magnetic edge snapping during manual drag.

## 2. Functional Requirements & Acceptance Criteria

### FR-SNAP-1: Win32 Extended Frame Bounds Extraction
- Rust `snapdown-capture` queries top-level window at cursor `(x, y)` using `WindowFromPoint` and `DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, ...)`.
- Correctly strips Windows 10/11 invisible DWM drop-shadow margins to yield true visible client boundaries.
- Filters out the Snapdown capture overlay window itself (`overlay` and `main` handles).

### FR-SNAP-2: UI Element Tree Enumeration via UIA
- Underneath the cursor, recursively queries child elements (buttons, panes, edit fields, list items) via `IUIAutomation::ElementFromPoint`.
- Returns bounding rectangle `(x, y, width, height)` and control type identifier (`window`, `pane`, `button`, `toolbar`).

### FR-SNAP-3: Hover Highlight & 1-Click Capture
- When mouse moves in `armed` phase without dragging:
  - If an element boundary is detected under cursor, render a highlighted bounding box with `2px solid var(--color-overlay-ring)` and `background: var(--color-overlay-selection-bg)`.
  - A subtle tag at the top-left of the box displays the element name (e.g. `Window: Visual Studio Code` or `Button: Submit`).
- Clicking left mouse button on a highlighted element locks its boundary immediately and transitions to `narrating` phase without requiring drag.

### FR-SNAP-4: Magnetic Snapping during Manual Drag
- When user manually drags a region:
  - If any edge of the selection box comes within a **threshold distance of 8 logical pixels** from a nearby window or element boundary, the edge snaps to that boundary.
  - Snapping triggers a subtle haptic/visual cue (snapped edge highlights with solid accent color).
  - Holding `Alt` key temporarily disables magnetic snapping for unrestricted freeform selection.

## 3. Invariants & Non-Functional Constraints
- **NFR-PERF-3**: Point-to-element boundary resolution must respond within `<10ms` on Windows.
- **NFR-SEC-1**: Querying window titles and bounds must not require administrator (UAC) elevation.

## 4. Test Obligations
- `cargo::win32_element_detector_resolves_valid_frame_bounds`
- `cargo::extended_frame_bounds_strips_dwm_shadow_padding`
- `vitest::hovering_element_displays_auto_detect_highlight_box`
- `vitest::clicking_auto_detect_box_selects_exact_element_dimensions`
- `vitest::drag_snaps_to_nearest_edge_within_eight_pixel_threshold`
- `vitest::holding_alt_disables_magnetic_snapping`
