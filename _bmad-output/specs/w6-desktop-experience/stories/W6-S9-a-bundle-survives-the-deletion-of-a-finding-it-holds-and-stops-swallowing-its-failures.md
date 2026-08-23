---
id: W6-S9
title: 'W6-S9: A Bundle survives the deletion of a Finding it holds, and stops swallowing its failures'
type: 'bug'
wave: W6
status: ready-for-dev
created: '2026-08-23'
review_loop_iteration: 0
followup_review_recommended: false
dependencies:
  - W6-S11
files:
  - crates/snapdown-store/src/sqlite/migrations.rs
  - crates/snapdown-store/src/sqlite/bundle_store.rs
  - apps/desktop/src-tauri/src/commands/bundle.rs
  - crates/snapdown-store/tests/test_sqlite_bundles.rs
  - crates/snapdown-store/tests/test_bundle_deletion.rs
  - apps/desktop/src-tauri/tests/test_bundle_failures.rs
context:
  - _bmad-output/specs/w6-desktop-experience/SPEC.md
  - _bmad-output/specs/w6-desktop-experience/stories.yaml
  - _bmad-output/specs/w6-desktop-experience/dispatch-briefs/W6-S9-step1-plan.md
  - .what/business-rules.md
  - .what/bundle/SRS-bundle.md
  - .how/bundle/SDD-bundle.md
  - .what/bundle/05-scenarios/SCN-05-a-finding-deleted-out-from-under-a-bundle.md
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/cross-cutting.md
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:**
The bundle component currently suffers from two distinct integrity defects rooted in faulty schema constraints and swallowed failure results:
1. **`BUG-1` (High - `bundle_item` Cascade Guts Bundles):**
   In `crates/snapdown-store/src/sqlite/migrations.rs` (migration v3), the `bundle_item` table defines `FOREIGN KEY(finding_id) REFERENCES finding(id) ON DELETE CASCADE`. Because `finding_store.rs:29` enables `PRAGMA foreign_keys = ON`, deleting a Finding causes SQLite to cascade-delete all `bundle_item` rows referencing that Finding.
   This directly violates `FR-13`'s third consequence (*"A Finding that belongs to a Bundle can still be deleted; the Bundle keeps its own copy of the image and stays readable"*) and contradicts `AD-9` (*one Bundle, one Markdown, every path*). While `bundle.markdown` remains intact in the database column, the Bundle's item list silently loses rows, creating a discrepancy between the composed Markdown document and the persisted item records. Rows previously lost cannot be recovered, but future cascade deletions must be prevented by dropping the `finding_id` foreign key constraint in schema migration v6.
2. **`BUG-9` (High - `bundle.rs` Swallows Failure Invariants):**
   `apps/desktop/src-tauri/src/commands/bundle.rs` contains three paths where critical `Result`s are ignored via `let _ =` or swallowed in `if let Ok`:
   - **Lines 72–74 (Composition Markdown write & Vault open swallowed):** In `create_bundle`, `if let Ok(vault_store) = VaultBlobStore::new(&vault_path)` and `let _ = vault_store.write_blob(...)` swallow both store opening errors and filesystem write errors. The Bundle record is committed to SQLite anyway, with `markdown_path` naming a nonexistent file on disk. This violates `FR-10` (*all-or-nothing composition*) and `AD-2` (*a record and its files live or die together; never commit a record before its files exist*).
   - **Line 116 (Unpublish on Bundle delete swallowed):** In `delete_bundle`, `if pub_record.is_live() { let _ = unpublish_bundle(id.clone(), state.clone()); }` swallows unpublish failures. If the remote service is unreachable or errors, the local database record and files are deleted while the published copy remains live on the public internet with nothing pointing to it. This violates `BR-20` (*an unpublish that fails leaves the Bundle marked published; never tell the Reviewer something is private when it may not be*), `BR-23` (*deleting a published bundle unpublishes it as part of the same action*), and `AD-6`.
   - **Lines 135 & 137 (Markdown and image copy file deletions swallowed):** In `delete_bundle`, `let _ = vault_store.delete_blob(&detail.bundle.markdown_path)` and `let _ = vault_store.delete_blob(&item.image_path)` ignore filesystem deletion errors, leaving orphaned files in the Vault without reporting failure, violating `FR-14` and `NFR-5`.

**Approach:**
1. **Apply Migration v6 (Drop `finding_id` Foreign Key):**
   - Add migration `v6` in `crates/snapdown-store/src/sqlite/migrations.rs`.
   - Recreate the `bundle_item` table without any `FOREIGN KEY(finding_id)` constraint (a `BundleItem` legitimately refers to a historical Finding ID that may no longer exist in the `finding` table).
   - Retain `FOREIGN KEY(bundle_id) REFERENCES bundle(id) ON DELETE CASCADE` so deleting a Bundle continues to cascade-delete its items (`FR-14`).
   - Retain `UNIQUE(bundle_id, finding_id)` constraint.
   - Use standard SQLite table recreation migration steps: create `bundle_item_v6`, copy existing rows from `bundle_item`, drop `bundle_item`, rename `bundle_item_v6` to `bundle_item`.
2. **Fix `create_bundle` Atomic File & DB Guarantees:**
   - In `apps/desktop/src-tauri/src/commands/bundle.rs`, open `VaultBlobStore::new(&vault_path)` fallibly, returning `Err(String)` if the vault cannot be opened.
   - Write the Markdown file to the vault (`vault_store.write_blob(&md_filename, markdown_content.as_bytes())`) and handle errors explicitly; if the file write fails, abort before creating the database row.
   - If `bundle_store.create_bundle` fails after writing the blob, clean up the written Markdown file to maintain all-or-nothing consistency (`AD-2`).
3. **Fix `delete_bundle` Unpublish & File Deletion Honesty:**
   - Check if the bundle has an active publication. If `pub_record.is_live()`, call `unpublish_bundle(id.clone(), state.clone())`. If `unpublish_bundle` fails, abort the deletion immediately and return the error string without deleting local database records or disk files (`BR-20`, `BR-23`).
   - Open `VaultBlobStore` fallibly; delete the bundle markdown blob and each burned item image blob. If any file deletion fails, record or report the failure rather than silently ignoring it.
   - Perform database deletion `state.bundle_store.delete_bundle(&id)` only after remote unpublish and file cleanup are successfully initiated/handled.
4. **Implement Missing Failure & Cascade Test Suites:**
   - Test that deleting a Finding does NOT cascade-delete `bundle_item` rows in `SqliteBundleStore` and `SqliteFindingStore`.
   - Test that deleting a Finding leaves the Bundle's stored Markdown byte-identical and leaves its own burned image copy intact in the Vault.
   - Test that deleting a Bundle still cascade-deletes its `bundle_item` records.
   - Test that composition failing to open the Vault or failing to write Markdown writes no bundle row in SQLite (`AD-2`, `FR-10`).
   - Test that `delete_bundle` fails and aborts if `unpublish_bundle` fails, leaving the Bundle and its publication state intact (`BR-20`).
   - Test that `delete_bundle` reports errors when disk files cannot be removed.

## Boundaries & Constraints

**Always:**
- Dropping `bundle_item.finding_id` foreign key MUST be done via numbered forward-only schema migration `v6` in `crates/snapdown-store/src/sqlite/migrations.rs`.
- `bundle_id` cascade MUST be retained: `FOREIGN KEY(bundle_id) REFERENCES bundle(id) ON DELETE CASCADE` (`FR-14`).
- All existing migrations (`v1` through `v5`) MUST remain immutable; migration `v6` runs when database schema version is `< 6`.
- `create_bundle` MUST be strictly all-or-nothing: if `VaultBlobStore::new` fails or `write_blob` fails, no row is written to `bundle` or `bundle_item` (`AD-2`, `FR-10`).
- `delete_bundle` MUST abort and return `Err` if `unpublish_bundle` fails, leaving the Bundle in SQLite and on disk marked as published (`BR-20`, `BR-23`).
- When a Finding is deleted, any Bundle containing it MUST continue to open, return all its `BundleItem` rows, preserve its stored Markdown bytes, and keep its image copy in the Vault (`FR-13`, `AD-9`, `SCN-05`).
- Keep changes scoped to `snapdown-store` and `apps/desktop/src-tauri/src/commands/bundle.rs`; do not widen edits into `apps/web-service` or modify the publish client (`DEC-005`).

**Block If:**
- Upstream requirements in `.what/`, `.how/`, or `.control/` conflict or demand modifying read-only corpus artifacts.
- Migration v6 breaks compatibility with existing databases containing orphaned `bundle_item` references.

**Never:**
- Do not attempt lossy recovery or fictional reconstruction of `bundle_item` rows already lost prior to migration v6.
- Do not silently swallow `Result` values using `let _ =` or unconditional `if let Ok` in `create_bundle` or `delete_bundle`.
- Do not delete local Bundle database records if the remote unpublish network request fails (`BR-20`).
- Do not modify corpus documents in `.what/`, `.how/`, or `.constitution/`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Migration v6 on existing DB | Existing database at schema version 5 | Upgrades schema to version 6; `bundle_item` table recreated without `finding_id` FK; `bundle_id` FK preserved | Fails transaction on SQL error |
| Finding deleted with Bundle present | Finding `F1` belongs to Bundle `B1`; Reviewer deletes `F1` | `finding` row removed; `bundle` row and all `bundle_item` rows for `B1` remain intact; `B1` Markdown unchanged; `B1` image copy on disk untouched (`SCN-05`) | Database transaction completes cleanly |
| Bundle deleted | Bundle `B1` with 3 items deleted | `bundle` row deleted; all 3 `bundle_item` rows deleted via `bundle_id` cascade; Markdown and image files deleted from Vault | Errors returned if delete fails |
| Composition: Vault open fails | `create_bundle` called with invalid/unusable vault path | Returns `Err("Failed to open vault...")`; zero rows inserted into `bundle` or `bundle_item` table | All-or-nothing rollback |
| Composition: Markdown write fails | `create_bundle` called but vault disk is read-only / write fails | Returns `Err("Failed to write bundle markdown file...")`; zero rows inserted into `bundle` or `bundle_item` table | Aborts before DB insertion |
| Delete published bundle: Unpublish succeeds | `delete_bundle` on bundle with active publication; remote unpublish returns `Ok(())` | Remote publication unpublished; local vault files deleted; SQLite `bundle` and `bundle_item` rows removed | Returns `Ok(())` |
| Delete published bundle: Unpublish fails | `delete_bundle` on bundle with active publication; remote service down or returns 500 | Unpublish returns `Err`; `delete_bundle` aborts immediately; SQLite `bundle`, `bundle_item`, and `publication` records remain intact; files untouched | Returns `Err("Unpublish failed: ...")` |
| Delete bundle: File delete fails | `delete_bundle` on bundle where Markdown file or image cannot be removed from disk | Deletion reports error or refuses completion so Reviewer is notified of lingering disk file | Returns `Err` detailing unremovable file |
| Golden Markdown stability | Bundle created with Finding `F1`; `F1` subsequently deleted from Library | `bundle_store.get_bundle()` returns full detail; `copy_bundle_to_clipboard` returns exact same Markdown bytes as original composition | Markdown column is authoritative |

</intent-contract>

## Code Map

- `crates/snapdown-store/src/sqlite/migrations.rs` -- Schema migration definitions; add migration v6 dropping `finding_id` foreign key on `bundle_item` table while preserving `bundle_id` cascade and `UNIQUE(bundle_id, finding_id)`.
- `crates/snapdown-store/src/sqlite/bundle_store.rs` -- `SqliteBundleStore` implementation; ensure bundle creation and cascade deletion logic adhere strictly to schema v6 invariants.
- `apps/desktop/src-tauri/src/commands/bundle.rs` -- Desktop Tauri bundle IPC command handlers:
  - `create_bundle`: Make Vault store creation and Markdown blob write fallible and atomic (abort DB write on FS failure, clean up on DB error).
  - `delete_bundle`: Enforce synchronous unpublish verification (abort delete and return error on unpublish failure per `BR-20`), report file deletion failures, and perform database cascade.
- `crates/snapdown-store/tests/test_sqlite_bundles.rs` -- Store-level integration tests asserting migration v6 application, absence of cascade on Finding deletion, and preservation of `bundle_item` rows when Finding is removed.
- `crates/snapdown-store/tests/test_bundle_deletion.rs` -- Store and vault integration tests verifying file deletion synchronization and `bundle_id` cascade behavior.
- `apps/desktop/src-tauri/tests/test_bundle_failures.rs` -- Command-level integration tests covering failure paths: failed Markdown write aborts DB insert, failing unpublish aborts bundle deletion, and file removal errors are surfaced.

## Tasks & Acceptance

**Execution:**
- `crates/snapdown-store/src/sqlite/migrations.rs` -- Add migration v6 -- Recreate `bundle_item` table without foreign key on `finding_id`, keeping `bundle_id` foreign key with `ON DELETE CASCADE` and unique constraint `UNIQUE(bundle_id, finding_id)`.
- `crates/snapdown-store/src/sqlite/bundle_store.rs` -- Verify bundle store compatibility -- Ensure `SqliteBundleStore` correctly handles bundle creation, query, and cascade delete under schema version 6.
- `apps/desktop/src-tauri/src/commands/bundle.rs` -- Fix `create_bundle` error handling -- Propagate errors from `VaultBlobStore::new` and `write_blob`; refuse bundle row creation if markdown file write fails (`AD-2`, `FR-10`).
- `apps/desktop/src-tauri/src/commands/bundle.rs` -- Fix `delete_bundle` unpublish and file cleanup honesty -- Abort deletion and report error if `unpublish_bundle` fails (`BR-20`, `BR-23`); report errors if vault file deletions fail (`FR-14`, `NFR-5`).
- `crates/snapdown-store/tests/test_sqlite_bundles.rs` -- Add tests for schema v6 and cascade absence -- Assert migration v6 applies cleanly, deleting a Finding leaves `bundle_item` intact, and deleting a Bundle deletes `bundle_item` rows.
- `apps/desktop/src-tauri/tests/test_bundle_failures.rs` -- Implement failure scenario test suite -- Add tests for failed vault open/write during composition, failed unpublish during bundle deletion, and disk file cleanup reporting.

**Acceptance Criteria:**
- Given a SQLite database with schema version 5, running migrations upgrades to schema version 6 and recreates `bundle_item` without a foreign key on `finding_id`.
- Given a Bundle holding Finding `F1`, when Finding `F1` is deleted from `finding_store`, the `bundle_item` record for `F1` remains present in `bundle_store.get_bundle()`.
- Given a Bundle holding Finding `F1`, when Finding `F1` is deleted from `finding_store`, the Bundle's Markdown column is unchanged and its image copy remains in the Vault.
- Given a Bundle holding Finding `F1` where `F1` has been deleted, `copy_bundle_to_clipboard` returns the exact same Markdown bytes as at composition time.
- Given `create_bundle`, if writing the Markdown file to the Vault fails (e.g. unwritable folder or I/O error), no `bundle` or `bundle_item` row is inserted into SQLite and an error is returned.
- Given `create_bundle`, if `VaultBlobStore::new` fails, the command returns an error and creates no database records.
- Given a published Bundle, when `delete_bundle` is called and `unpublish_bundle` fails (e.g. remote service unreachable), `delete_bundle` aborts, returns `Err`, and leaves the Bundle and its publication record intact in the database and on disk.
- Given `delete_bundle`, when the Bundle is deleted successfully, `bundle_item` records are removed via `bundle_id` cascade and the Vault Markdown file and burned images are deleted.
- Given the full test suite (`cargo test --workspace`), all 9 named tests from `waves.yaml` pass cleanly.

## Spec Change Log

<!-- Append-only. Populated during review loops. -->

## Design Notes

**Why drop the `finding_id` foreign key entirely instead of `ON DELETE NO ACTION`:**
In SQLite, with `PRAGMA foreign_keys = ON`, `ON DELETE NO ACTION` or `RESTRICT` would block the deletion of a Finding whenever that Finding is referenced by a `bundle_item`. However, `FR-13` explicitly requires: *"A Finding that belongs to a Bundle can still be deleted; the Bundle keeps its own copy of the image and stays readable."* A `BundleItem` legitimately refers to a historical Finding that may no longer exist in the `finding` table. Therefore, dropping the foreign key constraint entirely on `finding_id` is the mathematically correct model for historical snapshot references. The `bundle_id` foreign key with `ON DELETE CASCADE` is retained because BundleItems have no meaning without their parent Bundle.

## Verification

**Commands:**
- `cargo test -p snapdown-store --test test_sqlite_bundles` -- expected: Tests pass asserting migration v6 and that deleting a finding does not cascade to bundle_item.
- `cargo test -p snapdown-store --test test_bundle_deletion` -- expected: Tests pass asserting bundle deletion with file synchronization and cascade.
- `cargo test -p snapdown --test test_bundle_failures` -- expected: Tests pass asserting failed markdown writes write no bundle rows, and failing unpublish aborts bundle deletion.
- `cargo test --workspace` -- expected: All unit and integration tests across the workspace pass.
- `cargo clippy --workspace --all-targets -- -D warnings` -- expected: Zero warnings.
- `npm --prefix apps/desktop run typecheck` -- expected: Zero TypeScript errors.
- `npm --prefix apps/desktop run test` -- expected: All desktop frontend tests pass.
