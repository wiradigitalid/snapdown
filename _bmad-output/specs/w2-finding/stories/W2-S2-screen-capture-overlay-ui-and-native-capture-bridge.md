---
id: W2-S2
title: Screen capture overlay UI and native capture bridge
wave: W2
status: done
created: 2026-08-23
dependencies:
  - W2-S1
files:
  - apps/desktop/src/components/CaptureOverlay.tsx
  - apps/desktop/src-tauri/src/commands/capture.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/overlay.rs
---

# W2-S2: Screen capture overlay UI and native capture bridge

## User Story
As a tester performing exploratory testing, I want to trigger a region screenshot capture overlay using a global hotkey or shortcut, select or snap to an area, and save the captured image into the vault with normalized coordinates, so that I can document visual findings accurately.

## Acceptance Criteria
- [ ] Implement `CaptureOverlay.tsx` with region selection (drag bounding box), keyboard dismissal (Escape), and capture confirmation.
- [ ] Expose Tauri IPC commands for trigger capture (`capture_screen_region` / `trigger_overlay`).
- [ ] Save captured image buffer into the configured vault blob storage and return valid relative file path and dimensions.
- [ ] Ensure normalized `[0.0, 1.0]` coordinates for selected region bounds.
- [ ] Full automated test coverage in Rust (`cargo test`) and TypeScript (`npm run test`).
