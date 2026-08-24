---
id: SPEC-01-STUDIO-WORKSPACE
title: Studio Shell, 3-Zone Ribbon & Filmstrip Tray
status: ready-for-dev
source_prototype: .how/_platform/assets/ui-ux-complete-flow.html (State 1)
dedicated_html_asset: .how/finding/01-ux/assets/01-studio-workspace.html
companions:
  - web/ui/src/styles/tokens.css
  - apps/desktop/src/App.tsx
  - apps/desktop/src/components/NavigationRail.tsx
---

# SPEC-01: Studio Shell, 3-Zone Ribbon & Filmstrip Tray

## 1. Scope & Objective
Replace the generic 3-pane navigation layout with the modern **Snapdown Studio Shell** layout:
1. **Titlebar**: Compact window controls, brand logo, active finding name pill, and quick access to `📚 Bundles History`.
2. **3-Zone Balanced Ribbon**: Standardized 40x40px square icon buttons with floating CSS tooltips (`[data-tooltip]`):
   - **Left Zone (Input)**: `🔴` Capture (`--color-snagit-red`), `📂` Open Image, `📥` Paste from Clipboard.
   - **Center Zone (Annotation)**: `🔢` Insert Marker (Step badge), `🗑️` Delete Marker, `✂️` Crop Image.
   - **Right Zone (Export/Assembly)**: `📦` Assemble Bundle button, `📋` Copy Image, `Share Bundle ▼` split button.
3. **Bottom Filmstrip Tray**: Horizontal thumbnail strip with `● Editing` active state indicators, item selection checkboxes, Amber marker count badges, and left-docked **Assemble Batcher Banner**.

---

## 2. Layout Structure & CSS Specs

```
+-----------------------------------------------------------------------------------------------+
| [Logo] Snapdown Studio — [Finding Name Pill]                              [📚 History] [_][o][X]|
+-----------------------------------------------------------------------------------------------+
| [🔴][📂][📥]           |             [🔢][🗑️][✂️]            |      [📦 Assemble][📋][Share ▼]|
+-----------------------------------------------------------------------------------------------+
|                                                              |                                |
|                                                              |                                |
|                                                              |      PROPERTIES PANEL          |
|                      CANVAS ARTBOARD                         |      (Full Height: 440px)      |
|                                                              |                                |
|                                                              |                                |
+--------------------------------------------------------------+                                |
| [📦 Batcher] | [Thumb 1 (● Editing)] [Thumb 2] [Thumb 3]     |                                |
+--------------------------------------------------------------+--------------------------------+
```

### Key Dimensions & Styling Tokens:
- **Titlebar Height**: `34px`, `background: var(--snagit-ribbon-bg)`, border bottom `1px solid var(--snagit-ribbon-border)`. Contains `data-tauri-drag-region` on drag area and 44px `.win-control-btn` (Minimize `🗕`, Maximize `🗖`, Close `✕` with red hover `#dc2626`).
- **Ribbon Bar Height**: `56px`, `display: flex; justify-content: space-between; align-items: center; padding: 0 16px;`.
- **Ribbon Buttons**: `width: 40px; height: 40px; border-radius: var(--radius-sm); border: 1px solid var(--color-border);`.
- **Filmstrip Tray Height**: `110px`, docked at bottom left (from left window edge to `width: calc(100% - 440px)`).

---

## 3. Interaction & Functional Requirements

### FR-SHELL-1: Titlebar Navigation & Custom Frameless Chrome
- **Drag Region**: Area brand logo dan judul finding mengusung atribut `data-tauri-drag-region` untuk dragging window di desktop OS.
- **Window Controls**: Tombol native minimize (`🗕`), maximize/restore (`🗖`), dan close (`✕`) beroperasi langsung via Tauri window webview API.
- Titlebar displays current active finding title inside a rounded pill (`background: var(--color-surface-sunken)`).
- Clicking `📚 Bundles History` (or `Ctrl+H`) opens the **Saved Bundles Drawer** (`SPEC-06`).
- Clicking `⚙️ Settings` (or `Ctrl+,`) opens the **Settings & Preferences Modal** (`SPEC-07`).

### FR-SHELL-2: 3-Zone Ribbon Button Actions
- **`🔴` Capture Button**: Invokes Tauri command `trigger_capture_overlay` to activate State 2 (`SPEC-04`).
- **`📂` Open Image**: Invokes native OS file dialog (`dialog.open({ filters: [{ name: 'Images', extensions: ['png', 'jpg', 'webp'] }] })`).
- **`📥` Paste Clipboard**: Reads image data from system clipboard. If valid image exists, creates a new Finding in queue and switches active view to it.
- **`🔢` Insert Marker**: Toggles Marker stamping mode on canvas. Button gets `.active` class (`background: var(--color-accent-subtle); color: var(--color-accent);`).
- **`🗑️` Delete Marker**: Deletes currently active/focused Step Marker on canvas. Disabled if no marker is selected.
- **`✂️` Crop Image**: Enters Crop Mode (`SPEC-02`, State 3).
- **`📦` Assemble Bundle**: Opens the 3-Column Bundle Review & Assembly Modal (`SPEC-05`, State 4).
- **`📋` Copy Image**: Copies the currently visible image (including burned markers) directly to clipboard.

### FR-SHELL-3: Bottom Filmstrip Tray & Batcher
- **Left-Docked Batcher**:
  - Displays count: `{selectedCount} selected`.
  - Primary button: `📦 Assemble ({selectedCount})`. Triggers State 4 Bundle Modal.
- **Thumbnail Cards**:
  - Displays thumbnail WebP, finding title, and timestamp.
  - Active finding card gets `border: 2px solid var(--color-accent)` and a floating `● Editing` green badge.
  - Top-left checkbox allows selecting/deselecting items for multi-finding bundling.
  - Top-right amber badge shows count of attached Step Markers (e.g. `🟡 3`).

---

## 4. Test Obligations
- `vitest::renders_studio_titlebar_with_active_finding_and_history_button`
- `vitest::ribbon_left_zone_triggers_capture_open_and_paste`
- `vitest::ribbon_center_zone_toggles_marker_and_crop_modes`
- `vitest::filmstrip_highlights_active_editing_card`
- `vitest::filmstrip_batcher_reflects_selected_findings_count`
