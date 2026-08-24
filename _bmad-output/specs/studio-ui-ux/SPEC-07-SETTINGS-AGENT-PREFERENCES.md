---
id: SPEC-07-SETTINGS-AGENT-PREFERENCES
title: Settings & Agent Access Preferences Modal (4 Tabs Complete Specification)
status: ready-for-dev
source_prototype: .how/_platform/assets/ui-ux-complete-flow.html (State 6)
dedicated_html_assets:
  tab_1_general: .how/settings/01-ux/assets/06a-settings-general.html
  tab_2_hotkeys: .how/settings/01-ux/assets/06b-settings-hotkeys.html
  tab_3_agent_bridge: .how/settings/01-ux/assets/06c-settings-agent-bridge.html
  tab_4_about: .how/settings/01-ux/assets/06d-settings-about.html
companions:
  - web/ui/src/styles/tokens.css
  - web/ui/src/components/SettingsDialog.tsx
  - .what/settings/SRS-settings.md
  - .how/settings/SDD-settings.md
  - .control/decisions/DEC-004-quality-budget-presets.md
---

# SPEC-07: Settings & Agent Access Preferences Modal

## 1. Scope & Objective
Implements the unified **Settings & Agent Access Preferences Modal (State 6)** triggered via Titlebar icon `⚙️ Settings`, Ribbon shortcut, or `Ctrl + ,`:
1. **Clean Navigation Tabs**: `⚙️ General & Quality`, `⌨️ Hotkeys`, `🤖 Local Agent Bridge`, and `ℹ️ About`.
2. **Height-Packed 2-Column Layout (`FR-29`, `DEC-004`)**: Two independent flex stacks that fit naturally within standard window dimensions without artificial vertical stretching or scrollbars.
3. **Quality Budget Selector**: Segmented control with 4 preset intents (`Auto`, `Sharp`, `Balanced`, `Small`) and collapsible `▸ Advanced` parameters.
4. **Hotkey Recorder with OS Conflict Guard**: Visual listening chips with clear conflict warnings.
5. **Local Agent Bridge Access**: Display token key and loopback listener status on port 3849.

---

## 2. Modal Layout & 4-Tab Blueprint (State 6)

### TAB 1: General & Quality Budget (DEC-004, FR-5, FR-18)
```
+-----------------------------------------------------------------------------------------------+
| ⚙️ Snapdown Preferences & Agent Access                                              [✕ (Esc)]  |
+-----------------------------------------------------------------------------------------------+
| [ ⚙️ General & Quality (Active) ]  [ ⌨️ Hotkeys ]  [ 🤖 Local Agent Bridge ]  [ ℹ️ About ]     |
+-----------------------------------------------------------------------------------------------+
| COLUMN 1: SYSTEM & STORAGE                  | COLUMN 2: QUALITY & COMPRESSION                 |
|                                             |                                                 |
| +-----------------------------------------+ | +---------------------------------------------+ |
| | SYSTEM STARTUP (FR-18)                  | | | MULTIMODAL QUALITY BUDGET         [DEC-004] | |
| | [X] Run Snapdown at Windows Startup     | | | [ Auto | Sharp | Balanced | Small ]         | |
| | Starts in system tray on login.         | | | Auto sizes each capture (~120 KB).          | |
| +-----------------------------------------+ | | Latest: 184 KB · 1408 px · Auto             | |
|                                             | | ▸ Advanced (Dimensions & WebP quality)      | |
| +-----------------------------------------+ | +---------------------------------------------+ |
| | VAULT STORAGE FOLDER                    | |                                                 |
| | [ C:\Users\User\SnapdownVault         ] | |                                                 |
| | [ Browse Folder ]  [ Open Explorer ]    | |                                                 |
| +-----------------------------------------+ |                                                 |
+-----------------------------------------------------------------------------------------------+
| Settings are automatically synced live to SQLite database.              [ Done & Close (Esc) ]|
+-----------------------------------------------------------------------------------------------+
```

### TAB 2: Shortcuts & Global Hotkey Recorder (FR-16, UC-15)
```
+-----------------------------------------------------------------------------------------------+
| ⚙️ Snapdown Preferences & Agent Access                                              [✕ (Esc)]  |
+-----------------------------------------------------------------------------------------------+
| [ ⚙️ General & Quality ]  [ ⌨️ Hotkeys (Active) ]  [ 🤖 Local Agent Bridge ]  [ ℹ️ About ]     |
+-----------------------------------------------------------------------------------------------+
| Klik pada chip shortcut untuk merekam kombinasi tombol keyboard secara langsung.              |
|                                                                                               |
| +-------------------------------------------------------------------------------------------+ |
| | Global Capture Region Scrim                          [Active]              [Ctrl+Shift+S] | |
| | Membuka overlay capture transparan pada layar aktif.                                      | |
| +-------------------------------------------------------------------------------------------+ |
| | Toggle Snapdown Studio Window                        [Listening...] [Press keys… (Esc)]   | |
| | Menampilkan atau meminimalkan window utama.                                               | |
| +-------------------------------------------------------------------------------------------+ |
| | Quick Fullscreen Screenshot                          [Conflict ⚠️]          [PrintScreen] | |
| | ⚠️ Konflik OS: Digunakan oleh Snipping Tool Windows.                                      | |
| +-------------------------------------------------------------------------------------------+ |
+-----------------------------------------------------------------------------------------------+
```

### TAB 3: Local Agent Bridge & MCP Integration (FR-19, FR-20, FR-21)
```
+-----------------------------------------------------------------------------------------------+
| ⚙️ Snapdown Preferences & Agent Access                                              [✕ (Esc)]  |
+-----------------------------------------------------------------------------------------------+
| [ ⚙️ General & Quality ]  [ ⌨️ Hotkeys ]  [ 🤖 Local Agent Bridge (Active) ]  [ ℹ️ About ]     |
+-----------------------------------------------------------------------------------------------+
| +-------------------------------------------------------------------------------------------+ |
| | LOCAL LOOPBACK BRIDGE STATUS                                       🟢 Active on 127.0.0.1:3849|
| | Menyediakan akses instan bagi Claude Code, OpenCode, dan Codex agent via stdio/HTTP.     | |
| +-------------------------------------------------------------------------------------------+ |
| | AUTHENTICATION ACCESS KEY (AD-7)                                                          | |
| | [ ************************ ]  [ 📋 Copy Key ]  [ 🔄 Regenerate Key ]                       | |
| +-------------------------------------------------------------------------------------------+ |
| | CLAUDE DESKTOP & CLI MCP CONFIGURATION                                                    | |
| | { "mcpServers": { "snapdown": { "command": "snapdown-bridge", "args": ["--port", "3849"]}}| |
| +-------------------------------------------------------------------------------------------+ |
+-----------------------------------------------------------------------------------------------+
```

### TAB 4: About & System Information (FR-27, NFR-16)
```
+-----------------------------------------------------------------------------------------------+
| ⚙️ Snapdown Preferences & Agent Access                                              [✕ (Esc)]  |
+-----------------------------------------------------------------------------------------------+
| [ ⚙️ General & Quality ]  [ ⌨️ Hotkeys ]  [ 🤖 Local Agent Bridge ]  [ ℹ️ About (Active) ]     |
+-----------------------------------------------------------------------------------------------+
|                                      [ ⚡ Snapdown Studio ]                                   |
|                               Version 1.4.0 (Tauri v2 · x64 Windows 11)                       |
|                                                                                               |
| Visual UI/UX Observation & Multimodal Handoff Tool built for AI Coding Agents.                |
| Developed under WDI Method by Wira Digital Indonesia.                                         |
|                                                                                               |
|     [ VAULT STATS: 48.2 MB · 128 Findings ]    [ ENGINE: Rust Core + WebP Codec ]             |
|                                                                                               |
|              [ Check for Updates ]    [ Documentation ]    [ GitHub Repo ]                    |
+-----------------------------------------------------------------------------------------------+
```

---

## 3. Detailed Functional Requirements

### FR-SET-1: Quality Budget Selection (`FR-5`, `DEC-004`)
- Renders segmented control with 4 options: `Auto` (default), `Sharp`, `Balanced`, `Small`.
- Selecting an option updates the active preset and adjusts target WebP compression thresholds.
- Expanding `▸ Advanced` reveals max long-edge and encoder quality inputs. Editing either automatically switches the selector to a `Custom` segment.

### FR-SET-2: System Startup Registration (`FR-18`)
- Reading state queries the Windows Run registry key (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`).
- Toggling the checkbox updates the OS registry immediately without requiring administrative UAC elevation.

### FR-SET-3: Global Hotkey Recorder (`FR-16`, `UC-15`)
- Clicking a hotkey chip enters `listening` state (`"Press keys… Esc to cancel"`).
- Pressing a valid key combination registers the new shortcut with Windows API (`RegisterHotKey`).
- If another application holds the shortcut, displays an inline OS conflict warning.

### FR-SET-4: Local Agent Bridge Token & Port (`FR-19`, `FR-20`)
- Under `🤖 Local Agent Bridge` tab:
  - Displays loopback server status (default `127.0.0.1:3849`).
  - Displays masked Access Key with `[📋 Copy Key]` and `[🔄 Regenerate Key]` actions.

---

## 4. Test Obligations
- `vitest::settings_modal_renders_two_column_packed_layout_in_general_tab`
- `vitest::changing_quality_budget_preset_updates_store_value`
- `vitest::hotkey_tab_records_keystrokes_and_renders_conflict_warnings`
- `vitest::local_agent_tab_copies_mcp_config_and_regenerates_key`
- `vitest::about_tab_displays_accurate_version_and_vault_stats`
- `vitest::startup_checkbox_reflects_os_registration_truth`
