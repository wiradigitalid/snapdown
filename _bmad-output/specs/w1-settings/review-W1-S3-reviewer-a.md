# Code review — W1-S3 (reviewer A)

- **Change set reviewed:** commit `1997509` (`feat(settings): implement W1-S3 settings screen, vault migration, and quality budget`).
  The worktree is **clean** (`git status --short` is empty) and the full diff of `1997509` against `origin/main` was reviewed.
- **Reviewed against:** `_bmad-output/specs/w1-settings/SPEC.md` (§ W1-S3), `_bmad-output/specs/w1-settings/stories/W1-S3-*.md`, `.how/_platform/inventory-screen.md`, `.how/_platform/design-system.md`, `.how/_platform/cross-cutting.md`, `ARCHITECTURE-SPINE.md` (AD-2, AD-6), `.what/settings/SRS-settings.md` (UC-13, UC-14, FR-5, FR-16, BR-9, BR-28, BR-29).
- **Verdict: 0 must-fix, 3 follow-up.** The implementation fulfills all acceptance criteria for Screen 12 (/settings), atomic vault directory migration with rollback, shipped Quality Budget defaults and boundary checks, accessible form controls, and latest finding size reporting.

---

## Commands actually run (not taken from the story's report)

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean — no warnings across all targets |
| `cargo test --workspace` | 19 pass (3 core unit + 1 core no-io graph + 2 store system unit + 5 sqlite integration + 2 vault integration + 6 desktop lib integration) |
| `npm --prefix web/ui run typecheck` | clean |
| `npm --prefix web/ui run lint` | clean |
| `npm --prefix web/ui run test` | 18 pass |
| `npm --prefix apps/desktop run typecheck` | clean |
| `npm --prefix apps/desktop run lint` | clean |
| `npm --prefix apps/desktop run test` | 8 pass |
| `npm --prefix apps/desktop run build` | clean (Vite 7 bundle built in 737ms) |
| `uv run .constitution/method/scripts/validate.py --check` | RED, exactly 6 findings matching baseline (W1-S3 finding resolved; V18 on unwritten W1-S4..S5, V24 on skill template, V25 on unwritten mcp-bridge/web-api) |

---

## MUST-FIX

*None.*

---

## FOLLOW-UP

| # | Where | What |
| --- | --- | --- |
| F1 | `apps/desktop/src/components/VaultSection.tsx:127-135` | `ConfirmDialog` is utilized for the Vault migration prompt ("Move Existing Files?"). `ConfirmDialog` renders its confirm action with `variant="danger"`, which is styled for destructive operations (e.g., Delete), whereas file migration is non-destructive. In a future UX pass, consider adding a `variant` prop (`primary` vs `danger`) to `ConfirmDialog` or using `Modal` with a primary button directly. |
| F2 | `apps/desktop/src-tauri/src/commands/settings.rs:243-247` | `get_latest_finding_size_internal` inspects files directly inside `vault_path`. If future waves (W2/W3) organize findings in subdirectories (e.g. `findings/<id>/image.png`), this scanner will need to recurse or query `library.db` once finding metadata tables are introduced. |
| F3 | `apps/desktop/src-tauri/src/vault_migration.rs:31-37` | `validate_directory_writable` creates a test marker file `.snapdown_write_test_<pid>` in the destination directory to verify write permissions. If the process is terminated abruptly before line 37, the temporary probe file remains until manual cleanup. Consider adding a check on startup to clean stale test marker files if any exist. |

---

## What is clean, and worth saying

- **Atomic file migration with all-or-nothing rollback (BR-29, AD-2):** `VaultMigrator::migrate_vault` performs directory writability verification, recursively collects relative files, tracks copied files in destination, validates byte size parity, and upon any copy or directory creation error, invokes `rollback` to remove copied artifacts from destination while leaving the source completely intact.
- **Strict Quality Budget boundaries (UC-13, FR-5):** Enforces long edge `[320..=7680]` and encoder quality `[10..=100]` with shipped defaults of `1600` px and `75` quality referencing OQ-3. Validated on entry in both UI and Tauri IPC command handlers.
- **Accessible form controls:** `TextField` links `<label htmlFor="...">` with input `id` using `useId` fallback, ensuring screen readers properly announce field labels.
- **Empty state handling:** Settings correctly displays "No captures yet" when `latest_finding_size` is `None` rather than displaying `0 B` or `0 KB`.
- **Zero token style overrides:** React components adhere strictly to design system tokens defined in `tokens.css` without literal colors or spacing overrides.
