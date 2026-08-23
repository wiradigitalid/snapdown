---
id: W3-S4
title: Bundle Composer UI and bundle listing view
wave: W3
status: done
created: 2026-08-23
dependencies:
  - W3-S1
  - W3-S2
files:
  - packages/shared-ui/src/components/BundleComposer.tsx
  - apps/desktop/src/components/BundleView.tsx
  - apps/desktop/src-tauri/src/commands/bundle.rs
  - apps/desktop/src/services/bundle.ts
---

# W3-S4: Bundle Composer UI and bundle listing view

## User Story
As a user creating finding reports, I want a visual bundle composer to select multiple findings, organize their ordering, preview generated Markdown, and manage existing bundles from a list view.

## Acceptance Criteria
- [ ] Implement `BundleComposer.tsx` in `packages/shared-ui` for multi-finding selection, reordering, and metadata entry (title).
- [ ] Implement `BundleView.tsx` in `apps/desktop` integrating composer and bundle listing view.
- [ ] Expose Tauri IPC commands (`create_bundle`, `list_bundles`, `get_bundle_detail`, `delete_bundle`) in `apps/desktop/src-tauri/src/commands/bundle.rs`.
- [ ] Full unit and component test coverage (`npm run test`, `cargo test`).
