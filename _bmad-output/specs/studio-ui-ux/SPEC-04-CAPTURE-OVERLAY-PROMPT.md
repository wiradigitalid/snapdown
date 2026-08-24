---
id: SPEC-04-CAPTURE-OVERLAY-PROMPT
title: Capture Scrim Overlay, Region Selection & Note Prompt
status: ready-for-dev
source_prototype: .how/_platform/assets/ui-ux-complete-flow.html (State 2)
dedicated_html_asset: .how/finding/01-ux/assets/02-capture-overlay.html
companions:
  - apps/desktop/src/components/CaptureOverlay.tsx
  - web/ui/src/styles/tokens.css
  - .what/finding/SRS-finding.md
  - .how/finding/SDD-finding.md
---

# SPEC-04: Capture Scrim Overlay, Region Selection & Note Prompt

## 1. Scope & Objective
Defines the interaction and visual flow for **State 2 (Capture Scrim Overlay)** when triggered via global hotkey (`Ctrl+Shift+S` / `PrintScreen`) or the Ribbon `🔴 Capture` button:
1. **Full-Monitor Scrim Dimming**: Transparent window covering all attached monitors with dark scrim `rgba(15, 23, 42, 0.7)`.
2. **Interactive Drag Box**: Selection cutout with accent border `2px solid var(--color-overlay-ring)` and dimension readout (`width x height px`).
3. **Capture Note Prompt HUD (`FR-2`, `UC-1`)**: Floating dialog appearing immediately upon mouse release, allowing the reviewer to type an initial observation note before saving to queue.

---

## 2. Visual & Interaction Blueprint (State 2)

```
+-----------------------------------------------------------------------------------------------+
|                                                                                               |
|  [====================== FULL MONITOR DARK SCRIM (rgba(15, 23, 42, 0.7)) ==================]  |
|                                                                                               |
|                +-----------------------------------------------+                              |
|                |                                               |                              |
|                |              SELECTED REGION                  |                              |
|                |         (Clear Transparent Cutout)            |                              |
|                |                                               |                              |
|                +-----------------------------------------------+                              |
|                | [1280 x 720 px]                                                              |
|                |                                                                              |
|                | +--------------------------------------------------------------------------+ |
|                | | 📝 Observation Note at Capture Time:                                     | |
|                | | [Input: "Checkout dialog button fails to trigger POST /api/charge..."]   | |
|                | |                                                                          | |
|                | |                      [ ✓ Capture & Open in Studio (Enter) ] [ ✕ (Esc) ]  | |
|                | +--------------------------------------------------------------------------+ |
|                                                                                               |
|  [==========================================================================================]  |
|                                                                                               |
+-----------------------------------------------------------------------------------------------+
```

---

## 3. Functional Requirements & State Transitions

### FR-CAP-1: Scrim Mount & Drag Selection
- Mounts `<CaptureOverlay />` across the active screen viewport.
- Mouse down starts drag selection; mouse move updates selection bounds in real time.
- Dimension tag renders below selection rectangle: `"{width} x {height} px"`.

### FR-CAP-2: Floating Note HUD & Immediate Capture (`FR-2`)
- Upon mouse release (`mouseup`), the region boundary locks and the floating Note HUD appears docked directly beneath the selected rectangle:
  - Autofocuses single-line/multi-line note input field.
  - Primary Action **`✓ Capture & Open in Studio (Enter)`**:
    1. Invokes Rust `snapdown-capture` to grab pixels from the selected region.
    2. Runs async Quality Budget reduction (`reduce_image`).
    3. Writes new Finding entity to SQLite database with the captured Note text.
    4. Dismisses overlay and opens Snapdown Studio (`State 1`) with the newly created Finding active on canvas.
  - Secondary Action **`✕ (Esc)`**: Cancels capture without saving any file or database record.

---

## 4. Test Obligations
- `vitest::overlay_renders_scrim_and_handles_mouse_drag_selection`
- `vitest::releasing_mouse_mounts_floating_capture_note_hud`
- `vitest::pressing_enter_saves_finding_with_typed_note_and_opens_studio`
- `vitest::pressing_esc_aborts_capture_without_persisting_data`
