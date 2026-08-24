---
id: SPEC-STUDIO-UI-UX-INDEX
title: Snapdown Studio UI/UX Specification Suite (Complete 6-State Machine)
status: ready-for-dev
source_prototype: .how/_platform/assets/ui-ux-complete-flow.html
created: 2026-08-24
companions:
  - .how/_platform/assets/design-system.html
  - .how/_platform/assets/ui-ux-complete-flow.html
  - web/ui/src/styles/tokens.css
  - apps/desktop/src/styles/tokens.css
  - .what/finding/SRS-finding.md
  - .what/bundle/SRS-bundle.md
  - .how/finding/SDD-finding.md
  - .how/bundle/SDD-bundle.md
---

# Snapdown Studio UI/UX Specification Suite

This specification suite defines the complete technical and UI/UX blueprint to implement **Snapdown Studio**, matching the exact visual, structural, and behavioral model permanently preserved in `.how/_platform/assets/ui-ux-complete-flow.html`.

## Architectural Principles & Invariants
1. **Snagit-Inspired Studio Mental Model**: Replacing generic 3-pane IDE layout with an ergonomic top ribbon, edge-to-edge canvas, bottom filmstrip tray, and full-height 440px right properties panel.
2. **AI Agent Primitives Only**: Elimination of human drawing clutter (arrows, callouts, freehand). Sole focus on **Numbered Step Markers** (`1`, `2`, `3` Amber `#f59e0b`), **Crop Area**, and **Atomic Bundle Assembly**.
3. **Atomic Attachment (`AD-1`, `BR-1`)**: Screenshot image, observation summary, and numbered marker notes are born and stored as one immutable unit.
4. **Fractional Coordinates (`AD-3`, `BR-2`)**: Marker locations are stored as normalized percentages/fractions `(x: 0.0..1.0, y: 0.0..1.0)` of the image, immune to DPI/rescaling.
5. **Theme-Invariant Tokens (`AD-10`)**: Amber marker badge (`--color-marker: #f59e0b`), Selection ring (`--color-overlay-ring: #0284c7`), Scrim mask (`--color-overlay-scrim: rgba(15, 23, 42, 0.7)`).

---

## Modular Specification & HTML Asset Index

| Document | Feature Scope | Target State | Dedicated HTML Asset (.how Layer) | Key Components |
|---|---|---|---|---|
| [`SPEC-00-DESIGN-SYSTEM.md`](../../.how/_platform/design-system.md) | Universal Desktop Tokens & Component System | Platform | [`.how/_platform/assets/design-system.html`](../../.how/_platform/assets/design-system.html) | `tokens.css`, `Buttons`, `Badges`, `StepMarkers` |
| [`SPEC-01-STUDIO-WORKSPACE.md`](./SPEC-01-STUDIO-WORKSPACE.md) | Studio Shell, 3-Zone Ribbon & Filmstrip Tray | State 1 | [`.how/finding/01-ux/assets/01-studio-workspace.html`](../../.how/finding/01-ux/assets/01-studio-workspace.html) | `EditorShell`, `StudioRibbon`, `FilmstripTray` |
| [`SPEC-02-CANVAS-MARKERS-CROP.md`](./SPEC-02-CANVAS-MARKERS-CROP.md) | Interactive Canvas, Step Markers & Crop Mode | States 1, 3 | [`.how/finding/01-ux/assets/03-crop-mode.html`](../../.how/finding/01-ux/assets/03-crop-mode.html) | `StudioCanvas`, `StepMarkerLayer`, `CropMaskOverlay` |
| [`SPEC-03-PROPERTIES-TOKEN-PANEL.md`](./SPEC-03-PROPERTIES-TOKEN-PANEL.md) | Right Properties Panel & Token Breakdown | State 1 | [`.how/finding/01-ux/assets/01-studio-workspace.html`](../../.how/finding/01-ux/assets/01-studio-workspace.html) | `PropertiesPanel`, `MarkerNotesList`, `TokenEstimator` |
| [`SPEC-04-CAPTURE-OVERLAY-PROMPT.md`](./SPEC-04-CAPTURE-OVERLAY-PROMPT.md) | Capture Scrim Overlay & Note Prompt | State 2 | [`.how/finding/01-ux/assets/02-capture-overlay.html`](../../.how/finding/01-ux/assets/02-capture-overlay.html) | `CaptureOverlay`, `RegionSelector`, `CaptureNoteHUD` |
| [`SPEC-05-BUNDLE-ASSEMBLY-MODAL.md`](./SPEC-05-BUNDLE-ASSEMBLY-MODAL.md) | 3-Column Bundle Review & Assembly Modal | State 4 | [`.how/bundle/01-ux/assets/04-bundle-assembly-modal.html`](../../.how/bundle/01-ux/assets/04-bundle-assembly-modal.html) | `BundleModal3Col`, `MarkdownPreview`, `HandoffPanel` |
| [`SPEC-06-SAVED-BUNDLES-DRAWER.md`](./SPEC-06-SAVED-BUNDLES-DRAWER.md) | Saved Bundles History Drawer | State 5 | [`.how/bundle/01-ux/assets/05-saved-bundles-drawer.html`](../../.how/bundle/01-ux/assets/05-saved-bundles-drawer.html) | `BundlesDrawer`, `BundleHistoryCard` |
| [`SPEC-07-SETTINGS-AGENT-PREFERENCES.md`](./SPEC-07-SETTINGS-AGENT-PREFERENCES.md) | Settings: General, Quality, Hotkeys, MCP, About | State 6 (A-D) | [`.how/settings/01-ux/assets/06a-settings-general.html`](../../.how/settings/01-ux/assets/06a-settings-general.html)<br>[`.how/settings/01-ux/assets/06b-settings-hotkeys.html`](../../.how/settings/01-ux/assets/06b-settings-hotkeys.html)<br>[`.how/settings/01-ux/assets/06c-settings-agent-bridge.html`](../../.how/settings/01-ux/assets/06c-settings-agent-bridge.html)<br>[`.how/settings/01-ux/assets/06d-settings-about.html`](../../.how/settings/01-ux/assets/06d-settings-about.html) | `SettingsDialog`, `QualityBudgetSelector`, `HotkeyRecorder`, `AgentBridgeConfig` |

---

## Global Keyboard Shortcuts Map

| Shortcut | Context | Action |
|---|---|---|
| `Ctrl + Shift + S` / `PrintScreen` | Global OS | Open Capture Overlay (`State 2`) |
| `M` / `1..9` | Studio Canvas | Activate Insert Marker tool / select marker |
| `C` | Studio Canvas | Activate Crop Mode (`State 3`) |
| `Delete` / `Backspace` | Studio Canvas | Delete currently focused Step Marker |
| `Enter` | Crop Mode / Capture | Apply Crop / Confirm Capture Note |
| `Esc` | Modal / Crop / Capture | Close Modal / Cancel Crop / Cancel Capture |
| `Ctrl + B` | Studio Workspace | Open Bundle Assembly Modal (`State 4`) |
| `Ctrl + H` | Studio Workspace | Toggle Saved Bundles History Drawer (`State 5`) |
| `Ctrl + ,` | Studio Workspace | Open Settings & Preferences Modal (`State 6`) |
