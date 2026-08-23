---
id: W3-S6
title: Bundle deletion with file synchronization
wave: W3
status: done
created: 2026-08-23
dependencies:
  - W3-S1
  - W3-S4
files:
  - apps/desktop/src-tauri/src/commands/bundle.rs
  - crates/snapdown-store/src/sqlite/bundle_store.rs
---

# W3-S6: Bundle deletion with file synchronization

## User Story
As a user managing vault storage, I want deleting a bundle to cascade-remove its SQLite records and associated Markdown file from the vault filesystem atomically, so that no orphaned bundle files persist on disk.

## Acceptance Criteria
- [ ] Enhance `delete_bundle` command to remove the vault Markdown file synchronously before/alongside database cascade deletion.
- [ ] Ensure `BundleStore::delete_bundle` cascades to `bundle_item` rows within a single transaction (`INV-BUNDLE-001`).
- [ ] Add integration test verifying combined DB+file deletion and verifying no file remains on disk post-deletion.
- [ ] Full automated test coverage (`cargo test`, `npm run test`).
