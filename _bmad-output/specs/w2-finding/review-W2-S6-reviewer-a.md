---
reviewer: reviewer-a
model: 9router/combo
date: 2026-08-23
story: W2-S6
verdict: ACCEPTED
---

# Code Review: W2-S6 — Finding deletion with file synchronization and orphan sweeper report

## Scope & Implementation Review
- **Synchronous Deletion**: In `apps/desktop/src-tauri/src/commands/finding.rs`, deleting a finding now looks up the finding's relative image path and removes the image blob synchronously before/in tandem with database record deletion (`INV-DELETE-001`).
- **Orphan Sweeper Utility**: Implemented `OrphanSweeper` in `crates/snapdown-store/src/vault/sweeper.rs` to scan directory files vs active database records, identifying unreferenced files and missing files, with automatic orphan cleanup capability.
- **Frontend Report**: Added `OrphanReportView.tsx` with scan/clean actions and clear metrics display.
- **Automated Tests**: Unit and integration tests in Rust (`test_orphan_sweeper.rs`, `orphan_sweeper_detects_unreferenced_and_missing_files`) and React (`orphan_report.test.tsx`) pass 100%.

## Invariant Adherence
- `INV-DELETE-001` (Synchronous Deletion): Guaranteed image deletion upon finding deletion.
- `INV-SWEEPER-001` (Non-destructive Scan): `scan_orphans` is read-only; deletions only occur when `clean_orphans` is explicitly called.

## Verdict
ACCEPTED.
