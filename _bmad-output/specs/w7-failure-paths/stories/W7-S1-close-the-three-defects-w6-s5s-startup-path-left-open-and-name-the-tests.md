---
id: W7-S1
title: "W7-S1: Close the three defects W6-S5's startup path left open, and name the tests"
type: 'bug'
wave: W7
status: ready-for-dev
created: '2026-08-24'
review_loop_iteration: 0
followup_review_recommended: false
dependencies: []
files:
  - crates/snapdown-store/src/sqlite/settings_store.rs
  - crates/snapdown-store/src/sqlite/finding_store.rs
  - crates/snapdown-store/src/sqlite/bundle_store.rs
  - crates/snapdown-store/src/sqlite/access_key_store.rs
  - crates/snapdown-store/src/sqlite/publication_store.rs
  - crates/snapdown-store/tests/test_sqlite_settings.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tests/test_startup.rs
context:
  - _bmad-output/specs/w7-failure-paths/SPEC.md
  - _bmad-output/specs/w7-failure-paths/stories.yaml
  - _bmad-output/specs/w7-failure-paths/dispatch-briefs/W7-S1-step1-plan.md
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .what/business-rules.md
  - .what/settings/02-rules/rules-settings.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/settings/SDD-settings.md
  - .control/decisions/DEC-003-one-process-two-windows.md
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Context & Re-Scoping Rationale:**
An earlier version of this story was planned against `BUG-12` (five `.expect()` store opens causing silent process crashes on a corrupt `library.db`). As discovered during the W7 SPEC review, `BUG-12` was **already fixed** at `HEAD` by `W6-S5` (commit `aa30434`), which landed:
- `StartupError::DatabaseOpen { path, source }` (`apps/desktop/src-tauri/src/lib.rs:51-59`)
- `StoresBundle` (`lib.rs:61-67`)
- `init_app_stores(&Path) -> Result<StoresBundle, StartupError>` (`lib.rs:69-102`)
- `format_startup_error_message` (`lib.rs:105-120`)
- `show_native_message_dialog` via `MessageBoxW` (`lib.rs:122-152`)
- `report_startup_error` writing `startup-error.log` (`lib.rs:154-163`)
- Fallible setup hook returning `Err` (`lib.rs:226-233`)

None of those symbols or structures should be rebuilt or redesigned. This story is strictly re-scoped to the three defects that `W6-S5` left open, along with implementing the complete 7-test suite promised in `waves.yaml`.

**The Three Defects:**

1. **`BUG-15` (HIGH) — All 5 SQLite stores mutate page 1 and create `-wal`/`-shm` before running `quick_check`:**
   In all five SQLite store adapters (`settings_store.rs:24-38`, `finding_store.rs:28-40`, `bundle_store.rs:28-40`, `access_key_store.rs:28-40`, `publication_store.rs:28-40`):
   ```rust
   let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;
   let mut conn = Connection::open_with_flags(path_ref, flags)?;
   conn.pragma_update(None, "journal_mode", "WAL")?;      // <- MUTATES page 1 and creates -wal/-shm
   conn.pragma_update(None, "foreign_keys", "ON")?;
   conn.pragma_update(None, "busy_timeout", 5000)?;
   // only now:
   let mut integrity_stmt = conn.prepare("PRAGMA quick_check;")?;
   ```
   Switching an existing database to WAL mutates page 1 of the file and creates `library.db-wal` and `library.db-shm` beside it *before* verifying integrity. For a database with a **valid SQLite header and corrupt B-tree pages** (the exact case `quick_check` exists to catch), the store *is* modified and auxiliary files *are* created on disk before corruption is detected. The Reviewer is then presented with a dialog stating *"Snapdown will not recreate or overwrite this file to prevent data loss"*, which is false. This directly violates `BR-118` (*"opened, never created over ... no fresh one is started beside it"*) and `SDD-settings.md` (*"a store recreated beside a corrupt one is silent data loss"*).
   *Why undetected:* All existing tests used random garbage bytes as the corrupt fixture. SQLite rejects garbage bytes at `Connection::open` before any pragma executes. The byte-identity assertion passed without ever exercising the WAL pragma on corrupt databases.

2. **`BUG-16` (Medium) — Routine startup failure exits by panic via `.expect()` on `tauri::Builder::run`:**
   In `apps/desktop/src-tauri/src/lib.rs:346-347`:
   ```rust
   .run(tauri::generate_context!())
   .expect("error while running tauri application");
   ```
   When the setup hook returns `Err(Box::new(...))` on store open failure, `Builder::run` returns `Err`, and `.expect(...)` turns this routine, handled exit path into a process panic. The message dialog is shown first, but the process exits via panic rather than a clean non-zero exit code.

3. **`BUG-17` (Medium) — `MessageBoxW` missing foreground flags, opening behind other windows:**
   In `apps/desktop/src-tauri/src/lib.rs:122-152`, `MessageBoxW` is invoked with `MB_OK | MB_ICONERROR` and a null owner window handle (`null_mut()`), omitting `MB_SETFOREGROUND` and `MB_TOPMOST`. When the user double-clicks the application executable from Windows Explorer and setup fails, the process lacks foreground activation, causing the dialog to flash on the taskbar and remain hidden behind the active Explorer window—recreating the "nothing happens at all" symptom.

**Approach:**
1. **Fix `BUG-15` across all 5 SQLite stores:**
   - In `SqliteSettingsStore::open`, `SqliteFindingStore::open`, `SqliteBundleStore::open`, `SqliteAccessKeyStore::open`, and `SqlitePublicationStore::open`:
     - If the target file exists on disk (`path_ref.exists()`):
       - Open the existing file with `OpenFlags::SQLITE_OPEN_READ_ONLY` first.
       - Execute `PRAGMA quick_check;` on the read-only connection.
       - If `quick_check` returns anything other than `"ok"` (or returns a query error), immediately return `Err(StoreError::Corruption(msg))` without touching the file or creating WAL/SHM artifacts.
       - Drop the read-only connection.
     - Open the database with `OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE`.
     - Apply pragmas (`journal_mode = WAL`, `foreign_keys = ON`, `busy_timeout = 5000`).
     - Run migrations.
   - Note: `SQLITE_OPEN_CREATE` MUST remain on the read-write open so that first-run initialization against a non-existent database file continues to succeed seamlessly.

2. **Fix `BUG-16` in desktop runner:**
   - In `apps/desktop/src-tauri/src/lib.rs:346-347`:
     - Replace `.expect("error while running tauri application")` with a match / `if let Err(e)` check.
     - On error, log the error and terminate the process with `std::process::exit(1)` rather than panicking.

3. **Fix `BUG-17` in native message dialog:**
   - In `apps/desktop/src-tauri/src/lib.rs:142-151`:
     - Define `const MB_SETFOREGROUND: u32 = 0x00010000;` and `const MB_TOPMOST: u32 = 0x00040000;`.
     - Pass `MB_OK | MB_ICONERROR | MB_SETFOREGROUND | MB_TOPMOST` to `MessageBoxW`.
     - Note: Visibility of the native dialog is a manual check (`OQ-24`). Automated unit tests must NOT assert flag literals.

4. **Implement the 7 tests required by `waves.yaml`:**
   - Rename existing tests in `apps/desktop/src-tauri/tests/test_startup.rs` to match the registry:
     - `a_corrupt_library_db_does_not_panic_the_setup_hook` -> `a_store_that_cannot_be_opened_yields_an_error_not_a_panic`
     - `an_unreadable_library_db_is_reported_with_its_path_and_not_recreated` -> `the_startup_error_names_the_path_of_the_file_that_failed`
   - Re-fixture `a_corrupt_store_is_never_recreated_beside_itself` using a valid SQLite header with corrupted internal B-tree pages.
   - Add `a_readable_store_still_starts_normally` proving clean startup and store availability on valid databases.
   - Add `a_valid_header_with_corrupt_pages_is_not_written_to_before_it_is_checked` (BUG-15 proof).
   - Add `a_failed_open_leaves_no_wal_or_shm_file_beside_the_database` (BUG-15 proof).
   - Add `a_store_failure_exits_without_panicking` (BUG-16 proof).

## Boundaries & Constraints

**Always:**
- `BR-118`: A corrupt store must be reported with its exact path, left completely unmodified on disk (byte-identical), and no new database, `-wal`, or `-shm` file created beside it.
- `SQLITE_OPEN_CREATE` must remain enabled for first-run absent database files.
- All 5 SQLite stores (`settings`, `finding`, `bundle`, `access_key`, `publication`) must enforce identical pre-WAL read-only integrity verification.
- `MessageBoxW` must include `MB_SETFOREGROUND | MB_TOPMOST` on Windows.
- All 7 tests declared in `waves.yaml` must pass cleanly in `cargo test --workspace`.

**Block If:**
- Upstream requirements contradict the read-only check or the non-zero exit behavior.

**Never:**
- Never execute `journal_mode = WAL` or any modifying pragma on an existing database before `PRAGMA quick_check;` passes.
- Never exit via `.expect(...)` or panic on a failed startup setup hook.
- Never write automated tests that assert `MessageBoxW` flag constant literals (violates behavioral testing rules; manual check only per `OQ-24`).
- Never rebuild or modify `StartupError`, `StoresBundle`, `init_app_stores`, `format_startup_error_message`, or `report_startup_error` signatures already verified at `HEAD`.
- Never modify files in `.what/`, `.how/`, or `.constitution/`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Behavior / Output | Invariants & Guarantees |
|---|---|---|---|
| Absent Database (First Run) | Database file does not exist at `db_path` | Read-only check skipped (`!path.exists()`); read-write create opens and initializes database, sets WAL mode, runs migrations | Normal first run succeeds; `BR-112`, `BR-118` |
| Valid Existing Database | Normal, intact `library.db` | Read-only `quick_check` passes with `"ok"`; database reopened read-write with WAL and migrations; returns `Ok(StoresBundle)` | Existing data preserved, normal startup; `a_readable_store_still_starts_normally` |
| Valid Header + Corrupt Pages | SQLite file with valid 100-byte header, but corrupted internal B-tree page bytes | Read-only connection opens, `quick_check` detects corruption and returns `Err(StoreError::Corruption)`; open fails immediately | `library.db` bytes untouched; no `-wal` or `-shm` created; `BUG-15`, `BR-118` |
| Garbage Bytes Header | File filled with random non-SQLite bytes | Read-only `open_with_flags` fails immediately with SQLite error; open fails immediately | File untouched; no auxiliary files created; `BR-118` |
| Setup Hook Failure Exit | `init_app_stores` returns `Err(StartupError::DatabaseOpen)` | `report_startup_error` logs to `startup-error.log` and displays native dialog; setup hook returns `Err`; `run()` terminates with exit code 1 without panic | Process terminates without panic; `BUG-16`, `AD-11` |
| Windows Dialog Launch | Setup fails while Explorer is focused window | `MessageBoxW` called with `MB_OK \| MB_ICONERROR \| MB_SETFOREGROUND \| MB_TOPMOST` | Dialog brought to foreground over Explorer; `BUG-17` |

</intent-contract>

## Code Map

- `crates/snapdown-store/src/sqlite/settings_store.rs` — Add read-only pre-WAL `PRAGMA quick_check;` validation for existing files in `open()`.
- `crates/snapdown-store/src/sqlite/finding_store.rs` — Add read-only pre-WAL `PRAGMA quick_check;` validation for existing files in `open()`.
- `crates/snapdown-store/src/sqlite/bundle_store.rs` — Add read-only pre-WAL `PRAGMA quick_check;` validation for existing files in `open()`.
- `crates/snapdown-store/src/sqlite/access_key_store.rs` — Add read-only pre-WAL `PRAGMA quick_check;` validation for existing files in `open()`.
- `crates/snapdown-store/src/sqlite/publication_store.rs` — Add read-only pre-WAL `PRAGMA quick_check;` validation for existing files in `open()`.
- `crates/snapdown-store/tests/test_sqlite_settings.rs` — Add unit tests validating pre-WAL corruption rejection and byte preservation.
- `apps/desktop/src-tauri/src/lib.rs` — Update `show_native_message_dialog` with `MB_SETFOREGROUND | MB_TOPMOST`, and update `run()` to handle startup `Err` without panicking via `std::process::exit(1)`.
- `apps/desktop/src-tauri/tests/test_startup.rs` — Implement the full suite of 7 named tests from `waves.yaml` with the valid-header corrupt fixture.

## Tasks & Acceptance

**Execution:**
- `crates/snapdown-store/src/sqlite/*.rs` — Update all 5 store `open()` implementations to perform read-only integrity pre-checks before executing WAL pragmas on existing database files.
- `apps/desktop/src-tauri/src/lib.rs` — Add `MB_SETFOREGROUND | MB_TOPMOST` flags to `show_native_message_dialog`.
- `apps/desktop/src-tauri/src/lib.rs` — Replace `.expect()` in `run()` with graceful non-zero process termination (`std::process::exit(1)`).
- `apps/desktop/src-tauri/tests/test_startup.rs` — Implement all 7 required tests:
  1. `cargo::a_store_that_cannot_be_opened_yields_an_error_not_a_panic`
  2. `cargo::the_startup_error_names_the_path_of_the_file_that_failed`
  3. `cargo::a_corrupt_store_is_never_recreated_beside_itself`
  4. `cargo::a_readable_store_still_starts_normally`
  5. `cargo::a_valid_header_with_corrupt_pages_is_not_written_to_before_it_is_checked`
  6. `cargo::a_failed_open_leaves_no_wal_or_shm_file_beside_the_database`
  7. `cargo::a_store_failure_exits_without_panicking`

**Acceptance Criteria:**
- Given an existing database file with a valid 100-byte SQLite header and corrupt internal B-tree pages, attempting to open any of the 5 stores fails with `StoreError::Corruption`, leaves the file byte-for-byte identical on disk, and creates no `-wal` or `-shm` files.
- Given an intact readable database file, calling `init_app_stores` succeeds with `Ok(StoresBundle)` and allows normal read/write operations.
- Given a corrupt database file, launching the desktop application reports the failure via native dialog and `startup-error.log`, and the process terminates cleanly with a non-zero exit code without triggering a Rust panic.
- All 7 named tests in `apps/desktop/src-tauri/tests/test_startup.rs` pass cleanly in `cargo test --workspace`.
- `cargo clippy --workspace --all-targets -- -D warnings` reports 0 warnings and `cargo fmt --all -- --check` reports clean formatting.

## Spec Change Log

### 2026-08-24 — Initial Plan Creation (Re-scoped after SPEC review findings H1, H2, H3, M1)

- **Re-scoping from `BUG-12`:** `BUG-12` was resolved by `W6-S5` (commit `aa30434`). The core error types (`StartupError`), store bundle structure (`StoresBundle`), formatting helper (`format_startup_error_message`), log reporting (`report_startup_error`), and fallible setup hook are already present at `HEAD`.
- **`BUG-15` (High):** Addressed the vulnerability where WAL pragma execution mutated page 1 and created `-wal`/`-shm` auxiliary files on corrupt databases before `quick_check` ran. Implemented read-only integrity pre-check across all 5 stores.
- **`BUG-16` (Medium):** Replaced `.expect()` panic on `tauri::Builder::run()` with graceful non-zero exit (`std::process::exit(1)`).
- **`BUG-17` (Medium):** Added `MB_SETFOREGROUND | MB_TOPMOST` to `MessageBoxW` parameters to ensure foreground visibility when launched without prior foreground activation.
- **Test Alignment:** Established the full suite of 7 tests specified in `waves.yaml`, re-fixturing the corruption tests with valid-header/corrupt-page fixtures.

## Design Notes

**Read-Only Integrity Pre-Check:**
By opening existing database files with `OpenFlags::SQLITE_OPEN_READ_ONLY` before any connection with `SQLITE_OPEN_READ_WRITE` or `SQLITE_OPEN_CREATE` is established, SQLite is guaranteed not to modify the file header, write WAL frames, or create `-wal` / `-shm` files. Only after `PRAGMA quick_check;` returns `"ok"` is the read-only connection closed and the standard read-write connection initialized.

**Graceful Termination vs Panic:**
A Tauri setup hook returning `Err` causes `Builder::run()` to return `Err`. Catching this `Err` in `run()` and calling `std::process::exit(1)` ensures that the process exits cleanly without a panic backtrace after `report_startup_error` has presented the error dialog and written `startup-error.log`.

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: Clean formatting with zero diffs.
- `cargo clippy --workspace --all-targets -- -D warnings` -- expected: Zero compiler or linter warnings across the workspace.
- `cargo test -p snapdown-store` -- expected: All store unit and integration tests pass.
- `cargo test --test test_startup` -- expected: All 7 startup tests pass cleanly.
- `cargo test --workspace` -- expected: Full workspace test suite passes with 0 failures.
