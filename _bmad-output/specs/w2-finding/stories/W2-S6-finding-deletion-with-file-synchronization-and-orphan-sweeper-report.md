---
id: W2-S6
title: Finding deletion with file synchronization and orphan sweeper report
wave: W2
status: done
created: 2026-08-23
dependencies:
  - W2-S1
  - W2-S2
  - W2-S3
  - W2-S4
  - W2-S5
files:
  - crates/snapdown-store/src/vault/sweeper.rs
  - crates/snapdown-store/src/vault/mod.rs
  - crates/snapdown-store/tests/test_orphan_sweeper.rs
  - apps/desktop/src-tauri/src/commands/finding.rs
---

# W2-S6: Finding deletion with file synchronization and orphan sweeper report

## User Story
As a user managing vault storage, I want deleting a finding to remove its SQLite records and image blob synchronously, and have an orphan sweeper utility report/clean unreferenced disk files so that no dead data leaks into the filesystem.

## Acceptance Criteria
- [ ] Implement synchronous file deletion on finding removal (`INV-DELETE-001`).
- [ ] Implement `OrphanSweeper` to compare database image paths against files on disk in the vault findings directory.
- [ ] Produce structured orphan reports detailing unreferenced files.
- [ ] Expose Tauri IPC command / API for running orphan scan and cleanup.
- [ ] Full unit and integration test coverage (`cargo test`).
