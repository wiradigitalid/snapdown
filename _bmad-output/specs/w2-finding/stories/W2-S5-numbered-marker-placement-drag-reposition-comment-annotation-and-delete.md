---
id: W2-S5
title: Numbered Marker placement, drag reposition, comment annotation, and delete
wave: W2
status: done
created: 2026-08-23
dependencies:
  - W2-S1
  - W2-S2
  - W2-S3
  - W2-S4
files:
  - packages/shared-ui/src/components/MarkerLayer.tsx
  - packages/shared-ui/src/components/FindingsEditor.tsx
  - apps/desktop/src-tauri/src/commands/finding.rs
  - apps/desktop/src/services/finding.ts
---

# W2-S5: Numbered Marker placement, drag reposition, comment annotation, and delete

## User Story
As a tester identifying specific bug regions on a screenshot, I want to click to drop numbered markers (1, 2, 3...), drag them to reposition, add a comment to each marker, and delete markers with automatic single-sequence renumbering (no gaps), so that my annotations are crystal clear.

## Acceptance Criteria
- [ ] Implement `MarkerLayer.tsx` canvas overlay for image viewing with marker click-to-place and drag-to-reposition.
- [ ] Maintain relative `[0.0, 1.0]` coordinates for markers regardless of render zoom/dimensions.
- [ ] Support adding/editing marker comments.
- [ ] Implement marker deletion with single-sequence contiguous renumbering.
- [ ] Expose Tauri IPC commands (`add_marker`, `update_marker`, `delete_marker`).
- [ ] Full test coverage in Rust (`cargo test`) and TypeScript (`npm run test`).
