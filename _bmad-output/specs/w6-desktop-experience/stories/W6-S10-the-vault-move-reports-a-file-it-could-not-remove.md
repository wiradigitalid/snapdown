---
id: W6-S10
title: 'W6-S10: The Vault move reports a file it could not remove'
type: 'bug'
wave: W6
status: ready-for-dev
created: '2026-08-24'
review_loop_iteration: 0
followup_review_recommended: false
dependencies:
  - W6-S9
files:
  - apps/desktop/src-tauri/src/vault_migration.rs
  - apps/desktop/src-tauri/src/commands/settings.rs
  - apps/desktop/src-tauri/tests/test_vault_migration.rs
context:
  - _bmad-output/specs/w6-desktop-experience/SPEC.md
  - _bmad-output/specs/w6-desktop-experience/stories.yaml
  - _bmad-output/specs/w6-desktop-experience/dispatch-briefs/W6-S10-step1-plan.md
  - .what/business-rules.md
  - .what/settings/SRS-settings.md
  - .what/settings/02-rules/rules-settings.md
  - .what/settings/04-usecases/EXPERIENCE.md
  - .what/settings/04-usecases/UC-14-decide-where-my-screenshots-are-kept.md
  - .what/settings/05-scenarios/SCN-01-the-vault-move-that-fails.md
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/settings/SDD-settings.md
  - .how/settings/02-contracts/contract-inventory.md
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:**
When moving a Vault to a new directory, `apps/desktop/src-tauri/src/vault_migration.rs` currently swallows file deletion errors using `let _ = fs::remove_file(...)` in two critical paths:
1. **Line 141 (Success Path Source Cleanup):** After all files are copied and byte/size verified in the destination, source file deletions ignore failures (`let _ = fs::remove_file(&src_file)`). A source file that cannot be deleted (e.g. locked by an image viewer or permission-restricted) leaves an unreferenced, duplicate copy of an image containing potentially private Reviewer data in the old vault. Because `set_vault_path` updates the Vault location setting to the new folder, `FR-15`'s orphan sweeper (which only scans the active Vault) will never discover or clean this leftover copy, yet the product reports the migration as completely successful (`SCN-01`).
2. **Line 180 (Rollback Path Destination Cleanup):** When a migration fails mid-copy (e.g. destination disk full or file `n` locked) and invokes `rollback`, destination file deletions ignore failures (`let _ = fs::remove_file(file)`). While the source files remain completely untouched and the Setting is unchanged (`AD-2`, `BR-29`), stray destination copies that failed to delete are left behind without being reported to the caller or Reviewer.

**Evaluation of the Two Unscheduled `let _ =` in `vault_migration.rs`:**
- **Line 37 (`let _ = fs::remove_file(&test_file_path)` in `validate_directory_writable`):** This removes the ephemeral 16-byte writability probe file (`.snapdown_write_test_<pid>`). If removing the probe fails, the folder is nonetheless writable (the file was written successfully), and the artifact is an inert, tiny hidden file. Swallowing this error is acceptable, but it should be explicitly handled or accompanied by a clarifying comment.
- **Line 191 (`let _ = fs::remove_dir(&path)` in `remove_empty_dirs_recursive`):** In standard directory tree pruning, `fs::remove_dir` returns an error if a directory still contains files. Swallowing this error is the intended guard preventing removal of non-empty folders (such as directories containing unremoved source files or external files). This is deliberate and must be retained with an explicit comment explaining the guard invariant per `AGENTS.md`.

**The Reporting Trap and Architectural Shape:**
- When source cleanup fails on line 141, turning the failed deletion into an `Err` would be a major defect: all files have been safely copied and verified at the destination, and the move itself **succeeded**. Returning `Err` would falsely inform the Reviewer that nothing moved and that their files are only at the source, violating `BR-29` and `SCN-01`.
- Therefore, `migrate_vault` must succeed **and** return a structured report (`VaultMigrationReport { unremoved_sources: Vec<PathBuf> }`) that details any source files that could not be removed.
- On the failure path (`rollback`), `migrate_vault` returns an error detailing both the cause of the failure and any destination copies that could not be cleaned up (`uncleaned_destinations: Vec<PathBuf>`), so honesty is preserved in both directions.

**Approach:**
1. **Define Structured Migration Report & Error in `vault_migration.rs`:**
   - Introduce `VaultMigrationReport`:
     ```rust
     #[derive(Debug, Clone, PartialEq, Eq, Default)]
     pub struct VaultMigrationReport {
         pub unremoved_sources: Vec<PathBuf>,
     }
     ```
   - Introduce `VaultMigrationError`:
     ```rust
     #[derive(Debug, Clone, PartialEq, Eq)]
     pub struct VaultMigrationError {
         pub reason: String,
         pub uncleaned_destinations: Vec<PathBuf>,
     }
     ```
     Implement `std::fmt::Display` and convertibility to `CoreError` or string IPC errors.
   - Update `migrate_vault` signature:
     ```rust
     pub fn migrate_vault<P: AsRef<Path>, Q: AsRef<Path>>(
         src_dir: P,
         dest_dir: Q,
     ) -> Result<VaultMigrationReport, VaultMigrationError>
     ```
2. **Collect and Surface Source Cleanup Failures:**
   - In `migrate_vault`, after all destination files are copied and verified, iterate over `files_to_migrate` in reverse to remove source files.
   - For each source file, invoke deletion; if `fs::remove_file` fails, collect the relative (or canonical) path into `unremoved_sources`.
   - Prune empty directories in source.
   - Return `Ok(VaultMigrationReport { unremoved_sources })`.
3. **Collect and Surface Rollback Destination Cleanup Failures:**
   - In `rollback`, track any destination file for which deletion fails, collecting them into `uncleaned_destinations: Vec<PathBuf>`.
   - In `migrate_vault`, when an error occurs during directory creation, copying, or verification, invoke `rollback` and return `Err(VaultMigrationError { reason, uncleaned_destinations })`.
4. **Integrate with `set_vault_path` Command in `commands/settings.rs`:**
   - Receive the `VaultMigrationReport` from `migrate_vault`.
   - If `unremoved_sources` is non-empty, log/surface the warning detailing unremoved duplicate files in the old Vault while updating the `VaultPath` setting to `canonical_dest_str`.
   - If migration fails with `VaultMigrationError`, format the error string to include both the root failure cause and any leftover uncleaned destination files, returning `Err(formatted_error)`.
5. **Implement Test Seam & Required Test Suite:**
   - Create an internal filesystem abstraction (`FsOperations` or injected delete function) for `VaultMigrator` so that unit tests can simulate delete refusal deterministically on any platform without flaky environment dependencies.
   - Implement the three named tests from `waves.yaml`:
     - `cargo::vault_move_reports_a_source_file_it_could_not_remove`
     - `cargo::vault_move_reports_a_destination_copy_it_could_not_clean_up`
     - `cargo::vault_move_failing_at_file_n_leaves_every_source_file_in_place`

## Boundaries & Constraints

**Always:**
- The ordering **copy all files -> verify all copies -> only then remove sources** MUST NOT be changed (`AD-2`, `SCN-01`).
- When all copies and verifications succeed, `migrate_vault` MUST return `Ok(report)` even if some source files could not be deleted; it MUST NOT return `Err` (`BR-29`, `SCN-01`).
- Source deletion failures MUST be recorded in `report.unremoved_sources` and never silently discarded (`BUG-10` prevention, `AGENTS.md` pitfall).
- If copy or verification fails, `rollback` MUST delete destination copies and preserve every source file untouched (`BR-29`).
- Destination cleanup failures during rollback MUST be reported in `VaultMigrationError.uncleaned_destinations`.
- The deliberate `fs::remove_dir` in `remove_empty_dirs_recursive` MUST retain a comment explaining that failure to remove non-empty folders is the intended pruning guard.
- The 3 named tests from `waves.yaml` MUST be implemented and pass in `cargo test --workspace`.

**Block If:**
- Upstream requirements in `.what/` or `.how/` conflict with reporting unremoved files.

**Never:**
- Never return `Err` from `migrate_vault` if the destination files were successfully copied and verified.
- Never use `let _ =` on `fs::remove_file` when cleaning up sources or rollback destinations.
- Never delete or modify source files before all destination copies have been verified.
- Never modify files in `.what/`, `.how/`, or `.constitution/`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Clean Migration | Source has 3 files; all copy, verify, and delete cleanly | `Ok(VaultMigrationReport { unremoved_sources: [] })`; all files in dest; no files in src; `VaultPath` updated | No errors |
| Source file locked on delete | Source has 3 files; all copy and verify; `file2.png` locked/refuses delete | `Ok(VaultMigrationReport { unremoved_sources: ["file2.png"] })`; all 3 files in dest; `file2.png` in src; `VaultPath` updated | Warning surfaced with unremoved duplicate path |
| Multiple source files locked | 5 files copied and verified; 2 source files refuse delete | `Ok(VaultMigrationReport { unremoved_sources: ["file2.png", "file4.png"] })`; `VaultPath` updated | All unremoved files listed |
| Copy fails at file n | Copy fails at file 2 of 3 (e.g. disk full, permission denied) | `Err(VaultMigrationError { reason: "Failed to copy...", uncleaned_destinations: [] })`; dest `file1.png` deleted by rollback; all 3 src files untouched | `VaultPath` remains old path |
| Copy fails and dest cleanup fails | Copy fails at file 2; rollback fails to delete dest `file1.png` | `Err(VaultMigrationError { reason: "Failed to copy...", uncleaned_destinations: ["file1.png"] })`; all 3 src files untouched | `VaultPath` remains old path; error reports uncleaned dest |
| Verification size mismatch | Dest `file2.png` corrupted/size mismatch | `Err(VaultMigrationError { reason: "File size mismatch...", uncleaned_destinations: [] })`; dest files deleted; src files untouched | `VaultPath` remains old path |
| Source does not exist | `src_dir` does not exist | `Ok(VaultMigrationReport::default())`; no-op | Handled gracefully |
| Source same as destination | `canonical_src == canonical_dest` | `Ok(VaultMigrationReport::default())`; no-op | Handled gracefully |

</intent-contract>

## Code Map

- `apps/desktop/src-tauri/src/vault_migration.rs` -- Implement `VaultMigrationReport`, `VaultMigrationError`, and `VaultMigrator` with testable filesystem deletion seam; eliminate `let _ = fs::remove_file(...)` on lines 141 and 180; document deliberate `let _ =` on lines 37 and 191.
- `apps/desktop/src-tauri/src/commands/settings.rs` -- Update `set_vault_path` command to consume `VaultMigrationReport` and `VaultMigrationError`, surfacing warnings for leftover duplicate files and detailed error messages on rollback failures.
- `apps/desktop/src-tauri/tests/test_vault_migration.rs` (or unit test module in `vault_migration.rs`) -- Implement test suite for the three named tests from `waves.yaml` (`vault_move_reports_a_source_file_it_could_not_remove`, `vault_move_reports_a_destination_copy_it_could_not_clean_up`, `vault_move_failing_at_file_n_leaves_every_source_file_in_place`).

## Tasks & Acceptance

**Execution:**
- `apps/desktop/src-tauri/src/vault_migration.rs` -- Define `VaultMigrationReport` and `VaultMigrationError` structs with fields for unremoved source paths and uncleaned destination paths.
- `apps/desktop/src-tauri/src/vault_migration.rs` -- Update `migrate_vault` to track and return `unremoved_sources` on success path instead of swallowing `fs::remove_file` errors.
- `apps/desktop/src-tauri/src/vault_migration.rs` -- Update `rollback` to track and return `uncleaned_destinations` on failure path instead of swallowing `fs::remove_file` errors.
- `apps/desktop/src-tauri/src/vault_migration.rs` -- Add explicit explanatory comments to line 37 (probe cleanup) and line 191 (`remove_dir` pruning guard).
- `apps/desktop/src-tauri/src/commands/settings.rs` -- Update `set_vault_path` to handle `VaultMigrationReport` and format `VaultMigrationError` with uncleaned destination paths.
- `apps/desktop/src-tauri/tests/test_vault_migration.rs` -- Implement `vault_move_reports_a_source_file_it_could_not_remove` proving source delete failures are collected in `unremoved_sources` while migration succeeds.
- `apps/desktop/src-tauri/tests/test_vault_migration.rs` -- Implement `vault_move_reports_a_destination_copy_it_could_not_clean_up` proving destination delete failures during rollback are surfaced in `uncleaned_destinations`.
- `apps/desktop/src-tauri/tests/test_vault_migration.rs` -- Implement `vault_move_failing_at_file_n_leaves_every_source_file_in_place` proving rollback leaves all source files intact.

**Acceptance Criteria:**
- Given a Vault with 3 files, when `migrate_vault` runs and one source file refuses deletion after successful copy/verification, `migrate_vault` returns `Ok(report)` where `report.unremoved_sources` contains the path of the unremoved source file.
- Given a Vault migration that fails mid-copy (e.g. at file 2 of 3), when rollback runs and a destination copy refuses deletion, `migrate_vault` returns `Err(err)` where `err.uncleaned_destinations` contains the path of the leftover destination file.
- Given a Vault migration that fails mid-copy at file `n`, all source files remain byte-identical and present in the source folder.
- Given `cargo test --workspace`, all tests including the 3 named tests pass cleanly with 0 failures and clippy reports 0 warnings.

## Spec Change Log

<!-- Append-only. Populated during review loops. -->

## Design Notes

**Why `migrate_vault` returns `Ok(VaultMigrationReport)` instead of `Err` on source deletion failure:**
As established in `SCN-01` and `AD-2`, copy-then-delete ensures that files are never in a state where they exist in neither location. Once every destination file is copied and stat/size verified, the move itself is complete and the destination is authoritative. Returning `Err` when a source file refuses deletion would falsely inform the Reviewer that the move failed and that nothing was moved, leading to potential data confusion. Returning `Ok` with `unremoved_sources` provides exact honesty: the new Vault location is active and valid, while the Reviewer is notified of residual duplicate files in the old location.

## Verification

**Commands:**
- `cargo test -p snapdown --lib vault_migration` -- expected: All vault migration unit tests pass.
- `cargo test -p snapdown --test test_vault_migration` -- expected: Integration tests for the 3 named failure/reporting cases pass.
- `cargo test --workspace` -- expected: Full workspace test suite passes.
- `cargo clippy --workspace --all-targets -- -D warnings` -- expected: Zero warnings.
