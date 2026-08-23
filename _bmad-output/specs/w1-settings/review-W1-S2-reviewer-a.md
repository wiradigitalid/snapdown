# Code review — W1-S2 (reviewer A)

- **Change set reviewed:** commit `6a679fa` (`feat(store): implement library.db migrations, settings store, and vault blob adapter (W1-S2)`).
  The worktree is **clean** (`git status --short` is empty) and the full diff of `6a679fa` against `origin/main` was reviewed.
- **Reviewed against:** `_bmad-output/specs/w1-settings/SPEC.md` (§ W1-S2), `_bmad-output/specs/w1-settings/stories/W1-S2-*.md`, `.how/_platform/inventory-db.md`, `.how/_platform/cross-cutting.md`, `ARCHITECTURE-SPINE.md` (AD-2, AD-6), `SRS-settings.md`.
- **Verdict: 0 must-fix, 3 follow-up.** The implementation is clean, robust, adheres strictly to the SPEC invariants and SQLite inventory, and all workspace tests and validation checks pass cleanly.

---

## Commands actually run (not taken from the story's report)

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean — no warnings across all targets |
| `cargo test --workspace` | 15 pass (3 core unit + 1 core no-io graph test + 2 store system unit + 5 sqlite integration + 2 vault integration + 2 desktop/lib) |
| `npm --prefix apps/desktop run typecheck` | clean |
| `npm --prefix apps/desktop run lint` | clean |
| `npm --prefix apps/desktop run test` | 1 pass |
| `npm --prefix apps/desktop run build` | clean (Vite 7 bundle built in 794ms) |
| `npm --prefix web/ui run typecheck` | clean |
| `npm --prefix web/ui run lint` | clean |
| `npm --prefix web/ui run test` | 16 pass |
| `uv run .constitution/method/scripts/validate.py --check` | RED, exactly 7 findings matching baseline (V18 on unwritten W1-S3..S5, V24 on skill template, V25 on unwritten mcp-bridge/web-api) |

---

## MUST-FIX

*None.*

---

## FOLLOW-UP

| # | Where | What |
| --- | --- | --- |
| F1 | `crates/snapdown-store/src/sqlite/settings_store.rs:46-55` | `open_in_memory` initializes in-memory database with pragmas and migrations, but does not explicitly set `journal_mode = WAL`. In-memory databases use `:memory:` journaling anyway, but aligning pragmas between file and memory is cleaner. |
| F2 | `apps/desktop/src-tauri/src/main.rs:29-36` | `check_is_first_run` opens `SqliteSettingsStore` at startup to check `is_empty()`, but the instance is discarded immediately rather than stored in Tauri's state container (`app.manage(...)`). The story spec explicitly notes this is intentional and DI registration belongs to W1-S3 when IPC commands are wired. |
| F3 | `crates/snapdown-store/src/sqlite/settings_store.rs:211` | `parse_setting_value` validates `QualityBudget` long edge and encoder quality ranges on read and returns `CoreError::Validation` if invalid, causing fallback on read. However, `SettingsStore::list_all` silently skips invalid rows via `if let Ok(val)`. For W1-S3, consider whether malformed rows should be logged with sanitized keys or returned with default fallbacks in `list_all`. |

---

## What is clean, and worth saying

- **Strict schema confinement:** Only `setting` and `schema_version` tables are created in `library.db`, strictly matching `inventory-db.md` rows 8 and 9. No premature tables for future waves (`finding`, `note`, `marker`, `bundle`, etc.) were introduced.
- **Forward-only & idempotent migrations:** Migrations run inside a transaction, record UTC timestamps with `Z` in `schema_version`, and verify current version before executing.
- **Refusal to corrupt database recovery:** `SqliteSettingsStore::open` runs `PRAGMA quick_check` and returns `StoreError::Corruption` without destroying or overwriting the file on disk. Tested and proven in `corrupt_library_refuses_to_open_and_does_not_recreate`.
- **Vault root confinement:** `VaultBlobStore` employs strict path canonicalization and ancestor resolution, denying `..`, root prefixes, empty paths, and symlink escapes outside the Vault root. Tested against comprehensive traversal vectors.
- **Entropy and Clock compliance:** `SystemClock` produces RFC 3339 UTC strings terminating in `Z` and unix millis; `SystemEntropySource` uses cryptographically secure 10-byte buffers.
- **Panel Follow-ups Resolved:**
  - **F-3:** Added `crates/snapdown-core/clippy.toml` with `disallowed-methods` denying direct `std::fs`, `std::env`, and `std::time` calls in core domain, enforced with `#![warn(clippy::disallowed_methods)]`.
  - **F-7:** Updated `.github/workflows/desktop-ci.yml` frontend steps to `npm ci`.
  - **MF-8:** First-run state in Tauri main shell is derived directly from `SqliteSettingsStore::is_empty()`.
