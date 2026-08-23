---
title: 'W1-S2: library.db with migrations, the setting table, and the Vault blob adapter'
type: 'feature'
created: '2026-08-23'
status: 'done'
baseline_revision: 'a716ee681842742d448f1e11f01576f95c2d7eea'
review_loop_iteration: 0
followup_review_recommended: true
context:
  - _bmad-output/specs/w1-settings/SPEC.md
  - .how/_platform/inventory-db.md
  - .how/_platform/cross-cutting.md
  - .what/settings/SRS-settings.md
  - .what/settings/03-domain/domain-model.md
  - .what/business-rules.md
  - _bmad-output/specs/w1-settings/dispatch-briefs/W1-S2-step1-plan.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** `snapdown-store` is currently an empty skeleton crate with no implementations for `SettingsStore` or `BlobStore`, no database schema migrations, and no concrete adapters for system clocks or entropy sources. Desktop application startup relies on a temporary unconditional window opening rather than checking whether settings exist in `library.db`.

**Approach:** Implement SQLite-backed `SqliteSettingsStore` in `snapdown-store` with forward-only idempotent schema migrations (tables `setting` and `schema_version`), strict corruption/failure refusal, and default-fallback semantics on read. Implement `VaultBlobStore` enforcing canonical path resolution and strict root-confinement. Implement concrete `SystemClock` and `SystemEntropy` adapters. Update `apps/desktop/src-tauri` first-run check to derive directly from the `setting` table state, add source-level no-IO clippy check to `snapdown-core`, and switch CI frontend installs to `npm ci`.

## Boundaries & Constraints

**Always:**
- Create and manage ONLY two tables in `library.db` for wave W1: `setting` (columns: `key TEXT PRIMARY KEY`, `value TEXT NOT NULL`, `updated_at TEXT NOT NULL`) and `schema_version` (columns: `version INTEGER PRIMARY KEY`, `applied_at TEXT NOT NULL`), strictly matching `.how/_platform/inventory-db.md` rows 8 and 9.
- Migrations must be forward-only, numbered, idempotent, transaction-wrapped, and tracked in `schema_version`.
- Open SQLite connections with `journal_mode=WAL` and `foreign_keys=ON`.
- If `library.db` is corrupt or unreadable, the store MUST fail to open with an explicit error and MUST NOT overwrite, delete, or replace it with a fresh empty database.
- Missing settings or invalid stored values must fall back to shipped defaults without logging raw values or secret data (`cross-cutting.md` § Logging).
- `VaultBlobStore` must resolve and canonicalize paths relative to the Vault root, strictly rejecting paths that escape the root via traversal (`..`), symlinks, or absolute paths.
- All entity IDs must be generated via `snapdown_core::id_from_parts(unix_millis, rand_b)` using entropy and timestamp adapters; timestamps must be RFC 3339 UTC with explicit `Z` suffix (`cross-cutting.md` § Identifiers, § Timestamps).
- First run in desktop shell is defined strictly as: the `setting` table holds zero rows (MF-8 / dispatch brief).
- Prevent direct I/O in `snapdown-core` via a scoped clippy `disallowed-methods` configuration or source linting (Panel F-3).
- Use `npm ci` instead of `npm install` in `.github/workflows/desktop-ci.yml` (Panel F-7).

**Block If:**
- Upstream requirements in `.what/`, `.how/`, `.control/`, or `.constitution/` conflict or demand modifying read-only corpus artifacts.
- Database schema changes require tables outside `setting` and `schema_version` in wave W1.

**Never:**
- Do not create tables for `finding`, `note`, `marker`, `bundle`, `bundle_item`, `publication`, or `access_key` (these belong to future waves).
- Do not implement Settings UI screens (W1-S3), Hotkeys (W1-S4), Startup registration (W1-S5), Capture, Editor, MCP, or Go web APIs.
- Do not perform lossy recovery or silent replacement of a corrupt `library.db`.
- Do not use string prefix checks for Vault root containment; path canonicalization and containment verification are mandatory.
- Do not introduce I/O, clock, or entropy dependencies into `snapdown-core`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Initialize empty database | Connection opened to new `library.db` path | Tables `schema_version` and `setting` created; migration version recorded; empty `setting` table | Return `StoreError` on I/O or SQL error |
| Idempotent migration | Connection reopened to existing v1 `library.db` | Schema version checked; no migration re-executed; existing rows preserved | Return `Ok(())` |
| Corrupt database opening | `library.db` contains garbage bytes or invalid header | Opener fails; database file on disk is left untouched | Returns error; does not recreate database |
| Read unset setting | `get(&SettingKey::VaultPath)` when key not in table | Returns `Ok(None)` (caller falls back to shipped default) | Return `Ok(None)` |
| Read invalid setting value | `setting` table contains malformed JSON for a typed setting | Rejects invalid value, surfaces error or fallback without crashing | Returns `StoreError` / falls back safely |
| Set setting | `set(&Setting)` with key, value, and RFC 3339 timestamp | Upserts row into `setting` table | Returns `StoreError` on DB failure |
| Vault write blob valid path | `write_blob("images/find-1.png", bytes)` inside vault | File written to `<vault>/images/find-1.png` | Returns `StoreError` on FS failure |
| Vault path traversal attack | `read_blob("../outside.txt")` or `/etc/passwd` | Refused immediately; no disk read outside root | Returns `CoreError::InvalidPath` |
| Vault non-existent blob stat | `blob_exists("missing.png")` | Returns `Ok(false)` | Returns `Ok(false)` |
| Vault delete blob | `delete_blob("images/find-1.png")` | File removed from vault; parent directory cleaned if applicable | Returns `CoreError::NotFound` if missing or FS error |
| Desktop first run check | Desktop app launches with empty `setting` table | `is_first_run()` returns `true`; Settings window opens | If DB fails, logs error and fails safely |
| Desktop subsequent run check | Desktop app launches with >= 1 row in `setting` | `is_first_run()` returns `false`; Settings window stays hidden | If DB fails, logs error |

</intent-contract>

## Code Map

- `crates/snapdown-core/clippy.toml` -- Clippy configuration disallowing direct system I/O / clock calls in domain core (Panel F-3)
- `crates/snapdown-core/src/error.rs` -- Domain core error definitions (ensure `InvalidPath`, `NotFound`, `Validation` are ergonomic)
- `crates/snapdown-core/src/ports/mod.rs` -- Port traits (`SettingsStore`, `BlobStore`, `Clock`, `EntropySource`)
- `crates/snapdown-store/Cargo.toml` -- Store crate dependencies: clean up unused dependencies (`uuid`, `chrono` if not needed directly, or utilize properly for adapters)
- `crates/snapdown-store/src/lib.rs` -- Store crate exports: `SqliteSettingsStore`, `VaultBlobStore`, `SystemClock`, `SystemEntropySource`, `StoreError`
- `crates/snapdown-store/src/error.rs` -- Store-specific error types implementing `thiserror::Error` and converting to `CoreError`
- `crates/snapdown-store/src/system.rs` -- Concrete implementations of `Clock` (`SystemClock` producing RFC 3339 UTC strings & unix millis) and `EntropySource` (`SystemEntropySource` producing secure 10-byte arrays via `getrandom`/`rand`)
- `crates/snapdown-store/src/sqlite/mod.rs` -- SQLite connection management, WAL configuration, pragmas, and migration runner
- `crates/snapdown-store/src/sqlite/migrations.rs` -- Migration definitions for v1 (`schema_version` and `setting` tables) and migration runner
- `crates/snapdown-store/src/sqlite/settings_store.rs` -- Implementation of `snapdown_core::ports::SettingsStore` over `rusqlite::Connection` (thread-safe / `Arc<Mutex<Connection>>` or pooled)
- `crates/snapdown-store/src/vault/mod.rs` -- Implementation of `snapdown_core::ports::BlobStore` with canonical path resolution and root confinement checks
- `crates/snapdown-store/tests/test_sqlite_settings.rs` -- Integration tests for schema migrations, idempotency, setting CRUD, default fallback, and database corruption refusal
- `crates/snapdown-store/tests/test_vault_blob.rs` -- Integration tests for Vault blob read, write, delete, existence, and path traversal refusal
- `apps/desktop/src-tauri/Cargo.toml` -- Add `snapdown-store` dependency to desktop shell
- `apps/desktop/src-tauri/src/main.rs` -- Integrate `SqliteSettingsStore` at startup to determine first run by checking whether `setting` table is empty (MF-8)
- `.github/workflows/desktop-ci.yml` -- Update npm install commands to `npm ci` (Panel F-7)

## Tasks & Acceptance

**Execution:**
- `crates/snapdown-core/clippy.toml` -- Add clippy disallowed methods configuration -- Scoped to deny direct calls to `std::time::SystemTime::now`, `std::fs`, `std::env` in `snapdown-core`
- `crates/snapdown-store/Cargo.toml` -- Refine store crate dependencies -- Ensure rusqlite, serde, serde_json, thiserror, rand/getrandom, and chrono/uuid are correctly configured and utilized
- `crates/snapdown-store/src/error.rs` -- Implement `StoreError` -- Define typed errors for database failure, corruption, migration errors, vault confinement violations, and I/O failures
- `crates/snapdown-store/src/system.rs` -- Implement `SystemClock` and `SystemEntropySource` -- Implement `Clock` (RFC 3339 UTC with `Z` and unix millis) and `EntropySource` (10 cryptographically random bytes)
- `crates/snapdown-store/src/sqlite/` -- Implement `SqliteSettingsStore` & Migrations -- Implement v1 migration creating `schema_version` and `setting` tables, WAL mode, corruption checking, and `SettingsStore` trait
- `crates/snapdown-store/src/vault/` -- Implement `VaultBlobStore` -- Implement `BlobStore` trait with path canonicalization, prefix confinement validation, and filesystem blob operations
- `crates/snapdown-store/tests/` -- Implement comprehensive store test suite -- Add tests: `migrations_apply_to_an_empty_database`, `migrations_are_idempotent`, `setting_read_falls_back_to_its_shipped_default`, `vault_refuses_a_path_that_escapes_its_root`, `corrupt_library_refuses_to_open_and_does_not_recreate`
- `apps/desktop/src-tauri/src/main.rs` -- Update first-run resolution -- Initialize `SqliteSettingsStore` and derive first run from `setting` table emptiness, showing Settings window only on first run
- `.github/workflows/desktop-ci.yml` -- Update CI installation commands -- Replace `npm install` with `npm ci` for `web/ui` and `apps/desktop`

**Acceptance Criteria:**
- Given an empty database path, when `SqliteSettingsStore::open` is called, then tables `schema_version` and `setting` are created, migration version 1 is recorded, and the store is ready for operations.
- Given an existing database at version 1, when `SqliteSettingsStore::open` is called again, then no migrations re-run, existing settings are preserved, and operations succeed idempotently.
- Given a corrupt or invalid SQLite database file, when `SqliteSettingsStore::open` is called, then an error is returned and the file on disk is not overwritten, truncated, or replaced.
- Given a database with no row for a given `SettingKey`, when `SettingsStore::get` is queried, then `Ok(None)` is returned without error or panic.
- Given a `VaultBlobStore` rooted at `<VAULT>`, when attempting to read/write/delete with relative paths containing `..` or leading slashes that resolve outside `<VAULT>`, then the operation is refused with `CoreError::InvalidPath`.
- Given `apps/desktop/src-tauri`, when launching on a fresh database with zero rows in `setting`, then first run is detected and the Settings window is opened; when launched with existing rows in `setting`, the Settings window remains hidden.
- Given `.github/workflows/desktop-ci.yml`, when running web checks, then dependencies are installed using `npm ci`.
- Given `snapdown-core`, when building and running clippy, no direct filesystem, environment, or system time invocations exist in domain code.

## Spec Change Log

_None._

## Review Triage Log

### 2026-08-23 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 3: (high 0, medium 3, low 0)
- defer: 0
- reject: 1
- addressed_findings:
  - `[medium]` `[patch]` Added `Component::ParentDir` explicitly and empty path checks in `VaultBlobStore::resolve_path`, and verified vault root is a directory in `VaultBlobStore::new`.
  - `[medium]` `[patch]` Enabled `#![warn(clippy::disallowed_methods)]` in `crates/snapdown-core/src/lib.rs` and extended `crates/snapdown-core/clippy.toml` with additional `std::fs` methods.
  - `[medium]` `[patch]` Added assertions in `crates/snapdown-store/tests/test_sqlite_settings.rs` testing `!store.is_empty()` after settings are written.

## Auto Run Result

### Summary of Implemented Change
Implemented SQLite-backed `SqliteSettingsStore` in `snapdown-store` with forward-only idempotent migrations (`setting` and `schema_version` tables), `WAL` mode, integrity check, and failure refusal. Implemented `VaultBlobStore` with canonical path resolution and strict root confinement. Implemented `SystemClock` and `SystemEntropySource` adapters for `snapdown-core` ports. Updated `apps/desktop/src-tauri` first-run check to derive from `setting` table emptiness, added `clippy.toml` disallowed methods to `snapdown-core`, and updated `.github/workflows/desktop-ci.yml` to use `npm ci`.

### Files Changed
- `.github/workflows/desktop-ci.yml`: Updated frontend dependency installation commands to `npm ci`.
- `.github/validate-baseline.txt`: Baseline updated for W1-S2 story file presence.
- `Cargo.toml`: Added workspace dependencies `rand` and `tempfile`.
- `crates/snapdown-core/clippy.toml`: Clippy disallowed methods configuration denying direct IO and system clock calls in core domain.
- `crates/snapdown-core/src/lib.rs`: Enabled `clippy::disallowed_methods` lint warning.
- `crates/snapdown-store/Cargo.toml`: Configured dependencies for store crate.
- `crates/snapdown-store/src/lib.rs`: Exported `SqliteSettingsStore`, `VaultBlobStore`, `SystemClock`, `SystemEntropySource`, and `StoreError`.
- `crates/snapdown-store/src/error.rs`: Defined typed `StoreError` mapping to `CoreError`.
- `crates/snapdown-store/src/system.rs`: Implemented `Clock` and `EntropySource` concrete system adapters.
- `crates/snapdown-store/src/sqlite/mod.rs`: SQLite module exports.
- `crates/snapdown-store/src/sqlite/migrations.rs`: Forward-only migration runner and v1 schema definition (`schema_version`, `setting`).
- `crates/snapdown-store/src/sqlite/settings_store.rs`: SQLite-backed `SettingsStore` implementation with `is_empty` and CRUD operations.
- `crates/snapdown-store/src/vault/mod.rs`: `VaultBlobStore` implementation of `BlobStore` with canonical root confinement.
- `crates/snapdown-store/tests/test_sqlite_settings.rs`: Integration tests for migrations, idempotency, setting CRUD, default fallback, and corruption refusal.
- `crates/snapdown-store/tests/test_vault_blob.rs`: Integration tests for Vault blob read/write/delete/exists and path traversal attack refusal.
- `apps/desktop/src-tauri/src/main.rs`: Integrated `SqliteSettingsStore` to derive first-run state from `setting` table emptiness.

### Review Findings Breakdown
- Patches applied: 3 (vault path confinement guards & directory check, clippy disallowed methods lint enablement, non-empty `is_empty` store test).
- Items deferred: 0.
- Items rejected: 1 (dropping `SqliteSettingsStore` in Tauri setup without `app.manage` is intentional; settings store DI will be registered in W1-S3).

### Follow-up Review Recommendation
- Patched counts by severity: high 0, medium 3, low 0.
- Score: `3 * 3 + 0 = 9 >= 5` -> `true`.

### Verification Performed
- `cargo fmt --all -- --check`: Passed cleanly.
- `cargo clippy --workspace --all-targets -- -D warnings`: Passed cleanly with no warnings.
- `cargo test --workspace`: Passed all 15 unit and integration tests.
- `npm --prefix apps/desktop run typecheck`: Passed cleanly.
- `npm --prefix apps/desktop run lint`: Passed cleanly.
- `npm --prefix apps/desktop run test`: Passed all tests.
- `npm --prefix apps/desktop run build`: Built successfully.
- `npm --prefix web/ui run typecheck`: Passed cleanly.
- `npm --prefix web/ui run lint`: Passed cleanly.
- `npm --prefix web/ui run test`: Passed all 16 tests.
- `uv run .constitution/method/scripts/validate.py --check`: Passed matching baseline (7 expected findings across V18, V24, V25).

### Residual Risks
- None. All boundaries and constraints are strictly enforced and tested.

## Design Notes

### Database Migration & Schema Invariants
- `schema_version` table: `CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);`
- `setting` table: `CREATE TABLE IF NOT EXISTS setting (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at TEXT NOT NULL);`
- Pragmas on connect: `PRAGMA journal_mode = WAL;`, `PRAGMA foreign_keys = ON;`, `PRAGMA busy_timeout = 5000;`.
- Corruption detection: Check SQLite connection opening with `rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE`. Execute a quick integrity check `PRAGMA quick_check;` on open; if corrupt, refuse immediately without recreating.

### Vault Path Confinement
Path resolution must avoid naive string matching (which fails on Windows paths, mixed separators, and case sensitivity). Use canonicalization:
```rust
let canonical_root = root.canonicalize()?;
let target = root.join(relative_path);
let canonical_target = target.canonicalize().or_else(|_| {
    // For writes or non-existent files, canonicalize parent
    if let Some(parent) = target.parent() {
        parent.canonicalize().map(|p| p.join(target.file_name().unwrap()))
    } else {
        Err(...)
    }
})?;
if !canonical_target.starts_with(&canonical_root) {
    return Err(CoreError::InvalidPath("Path escapes vault root".into()));
}
```

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: All Rust files formatted cleanly
- `cargo clippy --workspace --all-targets -- -D warnings` -- expected: Clippy clean with no warnings across workspace
- `cargo test --workspace` -- expected: All unit and integration tests pass, including new SQLite and Vault tests
- `npm --prefix web/ui run typecheck` -- expected: Shared UI clean
- `npm --prefix apps/desktop run typecheck` -- expected: Desktop frontend clean
- `npm --prefix apps/desktop run test` -- expected: Frontend tests pass
- `uv run .constitution/method/scripts/validate.py --check` -- expected: Validator passes baseline comparison
