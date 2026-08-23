---
id: W7-S1
title: 'W7-S1: Open the stores fallibly, and report the file that could not be opened'
type: 'bug'
wave: W7
status: ready-for-dev
created: '2026-08-24'
review_loop_iteration: 0
followup_review_recommended: false
dependencies: []
files:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tests/test_startup.rs
  - crates/snapdown-store/src/sqlite/settings_store.rs
  - crates/snapdown-store/src/sqlite/finding_store.rs
  - crates/snapdown-store/src/sqlite/bundle_store.rs
  - crates/snapdown-store/src/sqlite/access_key_store.rs
  - crates/snapdown-store/src/sqlite/publication_store.rs
context:
  - _bmad-output/specs/w7-failure-paths/SPEC.md
  - _bmad-output/specs/w7-failure-paths/stories.yaml
  - _bmad-output/specs/w7-failure-paths/dispatch-briefs/W7-S1-step1-plan.md
  - .what/business-rules.md
  - .what/settings/02-rules/rules-settings.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/cross-cutting.md
  - .how/settings/SDD-settings.md
  - .control/decisions/DEC-003-one-process-two-windows.md
  - .control/decisions/DEC-005-desktop-first-ordering.md
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:**
When `library.db` is corrupt, locked, or unreadable, the desktop application terminates without presenting any message, window, or trace (`BUG-12`):
1. **Silent Termination via `.expect()` Panics:**
   In `apps/desktop/src-tauri/src/lib.rs:109-119`, the five SQLite stores (`SqliteSettingsStore`, `SqliteFindingStore`, `SqliteBundleStore`, `SqliteAccessKeyStore`, `SqlitePublicationStore`) are opened with `.expect(...)`. Because a Tauri release binary on Windows runs in the GUI subsystem without an attached console, a panic inside Tauri's setup hook unwinds and exits the process immediately. The Reviewer double-clicks `Snapdown.exe` and **nothing happens at all** — no window, no system tray icon, no dialog, and no file path identified.
2. **Missing Surface Under `AD-11` & `DEC-003`:**
   Per `AD-11`, exactly one desktop process owns the Library, the system tray, global hotkeys, the capture overlay, and the Editor window. When store initialization fails during the setup hook, **no webview window exists yet** to render an HTML/React error view. Furthermore, writing to `stderr` reaches nobody in a release Windows binary. As acknowledged in `DEC-003`'s Cost section (*"a panic in the editor's Tauri commands kills the tray, the hotkeys, and the overlay with it... this raises the bar on every unwrap in the command layer"*), the startup failure must be reported explicitly to the human operator using a native system dialog before process exit.
3. **Preservation of Corrupt Database Without Silent Overwrite (`BR-118`, `SDD-settings.md`):**
   `BR-118` and `.how/settings/SDD-settings.md` § Failure Behaviour (row `LC-025 -> library.db`) mandate: *"Reported with the file's path, and nothing is created over it (`BR-118`). A store recreated beside a corrupt one is silent data loss."* While panicking prevented overwriting the file, it completely failed to report the problem. The tempting shortcut — starting a fresh in-memory or replacement database so the application keeps running — trades an uncommunicative crash for silent data loss and is strictly forbidden. The corrupt file must remain untouched on disk, no fresh database may be started beside it, and the exact path of the unreadable database file must be presented to the Reviewer.

**Deliberately Out of Scope (Pre-Swept Groups from `BUG-12` Register):**
- `apps/desktop/src-tauri/src/server/handlers.rs`: 26 `Header::from_bytes` unwraps over compile-time byte constants (e.g. `b"Content-Type"`), which are infallible.
- `apps/desktop/src-tauri/src/lib.rs:226`: `.expect("error while running tauri application")` on the final Tauri `run` call.
- `crates/snapdown-bridge/src/mcp.rs:55,81`: `serde_json::to_string` on well-formed response structs.

**Approach:**
1. **Fallible Store Initialization (`init_app_stores`):**
   - In `apps/desktop/src-tauri/src/lib.rs`, provide `init_app_stores(db_path: &Path) -> Result<StoresBundle, StartupError>`.
   - Open all five SQLite stores (`SqliteSettingsStore::open`, `SqliteFindingStore::open`, `SqliteBundleStore::open`, `SqliteAccessKeyStore::open`, `SqlitePublicationStore::open`).
   - If any store fails to open (e.g. invalid SQLite header, failed `PRAGMA quick_check`, migration error, or OS permissions issue), map the failure into `StartupError::DatabaseOpen { path: db_path.to_path_buf(), source: err }` and return `Err`.
2. **Path-Carrying Error Formatting & Reporting (`format_startup_error_message` & `report_startup_error`):**
   - Implement `format_startup_error_message(err: &StartupError) -> String` that formats an unambiguous notification containing:
     - The exact absolute path of the database file (`path.display()`).
     - The underlying SQLite / I/O error description.
     - Explicit reassurance that Snapdown will not overwrite or delete the existing file to prevent data loss (`BR-118`).
   - Implement `report_startup_error(err: &StartupError, app_data_dir: &Path)`:
     - On Windows (`#[cfg(windows)]`), display a native modal error dialog via Win32 `MessageBoxW` with `MB_OK | MB_ICONERROR` titled `"Snapdown - Database Error"`.
     - On non-Windows platforms (`#[cfg(not(windows))]`), log the formatted error message to `eprintln!`.
     - Write the error text and an RFC 3339 timestamp to `app_data_dir/startup-error.log` as a persistent audit trace.
3. **Setup Hook Error Handling:**
   - In `tauri::Builder::setup`, call `init_app_stores(&db_path)`.
   - On `Err(err)`: call `report_startup_error(&err, &app_data_dir)` and return `Err(Box::new(std::io::Error::other(err.to_string())))` so the Tauri runtime terminates gracefully without panicking or creating any windows.
   - On `Ok(stores)`: proceed with normal startup and state attachment.
4. **Implement Verification Test Suite (`apps/desktop/src-tauri/tests/test_startup.rs`):**
   - Write/bind the four named tests from `waves.yaml`:
     - `cargo::a_store_that_cannot_be_opened_yields_an_error_not_a_panic`: Asserts that `init_app_stores` over a corrupt file returns `Err(StartupError)` instead of panicking.
     - `cargo::the_startup_error_names_the_path_of_the_file_that_failed`: Asserts that the returned `StartupError::DatabaseOpen` and formatted error string explicitly include the exact database `PathBuf`.
     - `cargo::a_corrupt_store_is_never_recreated_beside_itself`: Asserts that following an open failure, the file on disk remains byte-identical to the original corrupted fixture and no replacement store is created (`BR-118`).
     - `cargo::a_readable_store_still_starts_normally`: Asserts that a valid/readable database opens successfully, returning `Ok(StoresBundle)` with all 5 stores initialized.

## Boundaries & Constraints

**Always:**
- A corrupt or unreadable database MUST return `Err(StartupError)` and MUST NOT panic or call `.expect()` / `.unwrap()` during opening (`BUG-12`, `DEC-003`).
- Startup errors MUST explicitly name the exact file path of the database that failed to open (`CAP-6`, `SDD-settings.md`).
- The unreadable database file MUST be preserved unmodified on disk with byte-for-byte fidelity (`BR-118`).
- No fallback, temporary, or replacement database may be created over or beside an unreadable database file (`BR-118`).
- In the desktop application, when startup store initialization fails, a native modal dialog (`MessageBoxW` on Windows) MUST be displayed before process exit because no webview window is available (`AD-11`).
- All 4 named tests from `waves.yaml` MUST be implemented and pass cleanly in `cargo test --workspace`.

**Block If:**
- Upstream requirements in `.what/` or `.how/` contradict `BR-118` or demand automatic destructive recreation of corrupt databases.

**Never:**
- Never panic on store initialization in `lib.rs` or `snapdown-store`.
- Never create or overwrite a database file when `open` fails.
- Never use `tauri::test::mock_app` in test fixtures (`STATUS_ENTRYPOINT_NOT_FOUND` hazard noted in brief).
- Never modify files in `.what/`, `.how/`, or `.constitution/`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Corrupt SQLite Header | `library.db` contains garbage bytes `CORRUPT_SQLITE_DATA` | `init_app_stores` returns `Err(StartupError::DatabaseOpen { path, .. })`; file on disk unchanged | Native dialog shown; logged to `startup-error.log`; setup returns `Err` |
| Invalid SQLite Pages / Quick Check Failure | `library.db` has valid SQLite header but corrupt B-Tree pages | `PRAGMA quick_check` returns corruption message; `open` fails with `StoreError::Corruption` | `StartupError::DatabaseOpen` names file path and corruption error |
| Non-existent Database (Fresh Install) | `library.db` does not exist on disk | `init_app_stores` creates new database, applies migrations v1-v7; returns `Ok(StoresBundle)` | Normal startup proceeds |
| Valid Database | `library.db` is valid and migrated | `init_app_stores` opens database, runs idempotent migrations; returns `Ok(StoresBundle)` | Normal startup proceeds |
| Unreadable File Permissions | `library.db` has read/write permissions revoked (or locked by another process) | `init_app_stores` returns `Err(StartupError::DatabaseOpen)` with I/O or SQLite permission error | Native dialog displays exact path and permission error; zero data modified |
| Error Message Content | `StartupError::DatabaseOpen { path: "C:/app/library.db", .. }` | Message string contains exact path `"C:/app/library.db"` and data preservation notice | Reassures operator that data is not overwritten |

</intent-contract>

## Code Map

- `apps/desktop/src-tauri/src/lib.rs`
  - Define `StartupError` enum with `DatabaseOpen { path: PathBuf, source: StoreError }`.
  - Define `StoresBundle` holding all five initialized SQLite store instances.
  - Implement `init_app_stores(db_path: &Path) -> Result<StoresBundle, StartupError>`.
  - Implement `format_startup_error_message(err: &StartupError) -> String`.
  - Implement `show_native_message_dialog(title: &str, message: &str)` (`MessageBoxW` on Windows).
  - Implement `report_startup_error(err: &StartupError, app_data_dir: &Path)` writing to `startup-error.log` and displaying dialog.
  - In `run()` setup hook, open stores fallibly using `init_app_stores` and handle errors cleanly via `report_startup_error`.
- `apps/desktop/src-tauri/tests/test_startup.rs`
  - Implement `a_store_that_cannot_be_opened_yields_an_error_not_a_panic`.
  - Implement `the_startup_error_names_the_path_of_the_file_that_failed`.
  - Implement `a_corrupt_store_is_never_recreated_beside_itself`.
  - Implement `a_readable_store_still_starts_normally`.
- `crates/snapdown-store/src/sqlite/settings_store.rs`, `finding_store.rs`, `bundle_store.rs`, `access_key_store.rs`, `publication_store.rs`
  - Ensure all store `open` implementations enforce `PRAGMA quick_check` and return `Result<Self, StoreError>`.

## Tasks & Acceptance

**Execution:**
- `apps/desktop/src-tauri/src/lib.rs` -- Ensure `init_app_stores` opens all 5 SQLite stores fallibly and returns `StartupError::DatabaseOpen` carrying the `PathBuf` on any failure.
- `apps/desktop/src-tauri/src/lib.rs` -- Verify `format_startup_error_message` formats an unambiguous message containing the file path and data preservation notice (`BR-118`).
- `apps/desktop/src-tauri/src/lib.rs` -- Verify `report_startup_error` logs to `startup-error.log` and invokes `show_native_message_dialog` before setup returns `Err`.
- `apps/desktop/src-tauri/tests/test_startup.rs` -- Implement `cargo::a_store_that_cannot_be_opened_yields_an_error_not_a_panic` asserting `init_app_stores` returns `Err` on corrupt file.
- `apps/desktop/src-tauri/tests/test_startup.rs` -- Implement `cargo::the_startup_error_names_the_path_of_the_file_that_failed` asserting the error and formatted message contain the path of the failed file.
- `apps/desktop/src-tauri/tests/test_startup.rs` -- Implement `cargo::a_corrupt_store_is_never_recreated_beside_itself` asserting corrupt file bytes remain unmodified on disk after failed open.
- `apps/desktop/src-tauri/tests/test_startup.rs` -- Implement `cargo::a_readable_store_still_starts_normally` asserting valid database opens and initializes all 5 stores.

**Acceptance Criteria:**
- Given a corrupted `library.db` file, `init_app_stores(&db_path)` returns `Err(StartupError::DatabaseOpen { path, .. })` without panicking, where `path == db_path`.
- Given a `StartupError::DatabaseOpen`, `format_startup_error_message(&err)` produces a string containing the exact database path and the text `"Snapdown will not recreate or overwrite this file to prevent data loss."`
- Given a corrupted `library.db` file, after `init_app_stores` fails, the byte content of `library.db` on disk is identical to its pre-launch state, and no secondary or replacement database file is created.
- Given a valid or freshly initialized `library.db` file, `init_app_stores(&db_path)` returns `Ok(StoresBundle)` with all five stores accessible.
- Given `cargo test -p snapdown --test test_startup` and `cargo test --workspace`, all four named tests and the full workspace test suite pass cleanly with 0 warnings.

## Spec Change Log

<!-- Append-only. Populated during review loops. -->

## Design Notes

**Why Win32 `MessageBoxW` is used for reporting:**
As noted in `AD-11` and `DEC-003`, Snapdown runs as a single process combining system tray, global hotkeys, capture overlay, and editor webview. When `library.db` fails to open during the Tauri setup hook, no webview window exists, and any attempt to create one without valid stores would violate store ownership invariants. In release builds on Windows (`windows_subsystem = "windows"`), console output (`stdout`/`stderr`) is detached and discarded by the operating system. Therefore, invoking Win32 `MessageBoxW` via FFI is the only standard, dependable mechanism to display a modal error message to the user before terminating the process.

**Why corrupt stores are never overwritten or recreated (`BR-118`):**
A corrupt SQLite database may still contain recoverable user data (Findings, Notes, Marker coordinates, and Access Keys). Silently deleting or recreating the database file destroys the only existing record of the user's Library. Halting startup while clearly identifying the file path gives the Reviewer full transparency and allows manual backup or recovery.

## Verification

**Commands:**
- `cargo test -p snapdown --test test_startup` -- expected: All startup unit/integration tests including the 4 named tests pass.
- `cargo test --workspace` -- expected: Entire workspace test suite passes.
- `cargo clippy --workspace --all-targets -- -D warnings` -- expected: Zero warnings.
- `cargo fmt --all -- --check` -- expected: Clean formatting.
- `uv run .constitution/method/scripts/validate.py` -- expected: Validator confirms story status and contract.