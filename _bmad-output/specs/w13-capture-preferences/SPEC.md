---
id: SPEC-w13-capture-preferences
title: Capture Preferences & User Customization
status: draft
companions:
  - .what/settings/SRS-settings.md
  - .how/settings/SDD-settings.md
  - crates/snapdown-core/src/domain/setting.rs
  - apps/desktop/src/components/SettingsView.tsx
sources:
  - .control/registry/requirements.yaml
---

# SPEC-w13: Capture Preferences & User Customization

## 1. Intent & Context
Different workflows require different assistance levels. Some users prefer minimalist fast capture without crosshairs or loupe, while others require high-precision magnifier and element snapping.
This spec adds persistent domain configuration settings and UI toggles under Settings Screen.

## 2. Functional Requirements & Acceptance Criteria

### FR-PREF-1: Domain Setting Keys & Default Values
- In `snapdown-core::domain::setting::SettingKey`, add keys:
  - `EnableCrosshairGuides`: `"enable_crosshair_guides"`, default `true`.
  - `EnableLoupeMagnifier`: `"enable_loupe_magnifier"`, default `true`.
  - `LoupeZoomFactor`: `"loupe_zoom_factor"`, default `"6x"` (allowed: `"4x"`, `"6x"`, `"8x"`).
  - `EnableSmartElementSnapping`: `"enable_smart_element_snapping"`, default `true`.
  - `ScrollSettlingDelayMs`: `"scroll_settling_delay_ms"`, default `60` (range: 30..300 ms).

### FR-PREF-2: Settings Screen - Capture Assistance Section
- In `apps/desktop/src/components/SettingsView.tsx` (under Capture / General tab):
  - Add a distinct Card/Section titled **Capture Assistance**:
    - Toggle Switch: **Precision Crosshair Guides** (`Enable/disable full-screen X/Y alignment guides`).
    - Toggle Switch: **Pixel Loupe Magnifier** (`Show magnified pixel grid and color code at cursor`).
    - Segmented Control for Zoom Factor: `[ 4x | 6x | 8x ]` (active when Loupe is enabled).
    - Toggle Switch: **Smart Window & Element Snapping** (`Auto-detect window boundaries and snap selection edges`).

### FR-PREF-3: Live Synchronization & Hot Reloading
- Changing any capture preference immediately writes to SQLite `SettingStore`.
- The `CaptureOverlay` reads updated preferences from `SettingStore` on launch or via event emission without requiring an application restart.

## 3. Invariants & Non-Functional Constraints
- **BR-SET-1**: An unknown or invalid setting value read from disk gracefully falls back to its compiled default.
- **NFR-UI-1**: Settings controls must be keyboard accessible and respect light/dark theme tokens.

## 4. Test Obligations
- `cargo::capture_setting_keys_roundtrip_with_defaults`
- `cargo::invalid_loupe_zoom_factor_falls_back_to_default_six_x`
- `vitest::settings_renders_capture_assistance_section_and_toggles`
- `vitest::toggling_loupe_setting_persists_and_updates_overlay_behavior`
