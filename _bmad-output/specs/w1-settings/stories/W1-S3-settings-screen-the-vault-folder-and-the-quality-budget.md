---
title: 'W1-S3: Settings screen — the Vault folder and the Quality Budget'
type: 'feature'
created: '2026-08-23'
status: 'done'
baseline_revision: 'f99989a91c3986cc717576c2039627b08b7fd9a5'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - _bmad-output/specs/w1-settings/SPEC.md
  - .how/_platform/inventory-screen.md
  - .how/_platform/design-system.md
  - .how/_platform/cross-cutting.md
  - .what/settings/SRS-settings.md
  - .what/settings/03-domain/domain-model.md
  - .what/business-rules.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** The React front end only has a placeholder UI with an unassociated `TextField`, and Tauri lacks IPC commands to read and write settings, validate Vault folder permissions and perform atomic file migration/rollback on Vault path changes, or validate and persist Quality Budget configurations with last finding size reporting.

**Approach:** Implement Tauri backend IPC commands (`get_settings`, `set_vault_path`, `set_quality_budget`, `get_latest_finding_size`, `open_vault_folder`) with atomic directory migration rollback on error and range validation. Fix `TextField` accessible labelling (linking label `htmlFor` with input `id`). Build the React Settings screen at route `/settings` (Screen 12) utilizing design system tokens and base elements (`TextField`, `Button`, `Toast`, `ConfirmDialog`, `EmptyState`) to manage the Vault folder (with folder picker / change confirmation, Open in Explorer) and Quality Budget (1600 px default, 75 quality default, range validation, and latest finding size display).

## Boundaries & Constraints

**Always:**
- Implement Screen 12 as one screen with structured sections at route `/settings` per `inventory-screen.md`.
- Quality Budget shipped defaults must be `1600` px long edge and quality `75` as named constants referencing OQ-3.
- Range bounds for Quality Budget: max long edge `320..=7680` px, encoder quality `10..=100`. Values outside this range must be rejected at point of entry (`CoreError::Validation`).
- Vault folder writability must be validated at the moment of choosing (refused immediately if unwritable, not at next capture per FR-16).
- Vault folder changes must migrate existing files with all-or-nothing guarantee (BR-29, AD-2): if copying or moving any file fails (e.g. file lock or permissions), rollback completely to the original Vault path, restore any moved files, and report the refusal.
- Provide action to open the current Vault folder in Windows Explorer (`tauri-plugin-opener` or `std::process::Command` / platform explorer invocation).
- Report stored size of the most recent Finding so the effect of Quality Budget is visible; if no Findings exist, display a clear message rather than `0 B` or `0 KB`.
- Use design system tokens (`tokens.css`) and base element components (`web/ui`) without any literal colour, spacing, radius, or font size overrides.
- Ensure `TextField` provides accessible name linkage via `id` / `htmlFor` so screen readers announce "Vault Path" rather than relying on placeholder.
- Maintain `snapdown-core` without I/O; place migration, filesystem checks, and IPC commands in `desktop` / `snapdown-store`.

**Block If:**
- Upstream requirements in `.what/`, `.how/`, `.control/`, or `.constitution/` demand schema changes to `library.db` beyond the `setting` and `schema_version` tables.
- Changes require implementing the Editor canvas, Capture overlay, Hotkeys (W1-S4), or Startup registration (W1-S5).

**Never:**
- Do not re-encode existing stored images when Quality Budget changes (BR-9).
- Do not introduce telemetry, crash reporters, or network calls (AD-6).
- Do not hardcode literal CSS styles for colors, typography, or spacing; tokens only.
- Do not log Vault path values or personal directory names (`cross-cutting.md` § Logging).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Read initial settings | Settings screen mounts | Invokes `get_settings`; loads current Vault path, Quality Budget (1600px / 75), and latest finding size | If unset, falls back to shipped defaults without error |
| Set valid Quality Budget | Slider/Input: edge=1920, quality=80 | Invokes `set_quality_budget`; updates `quality_budget` row in `setting` table | No error expected |
| Set out-of-range Quality Budget | Input: edge=100 or quality=105 | Rejected immediately in UI and IPC command | Returns validation error; UI displays error message |
| Set unwritable Vault path | Path to read-only directory | Refused at moment of choosing | Returns error: directory is not writable; Vault path is not changed |
| Change Vault path (empty source) | New valid writable directory | Updates `vault_path` in `setting` table | No error expected |
| Change Vault path with files (success) | Source has 3 files, dest writable | All 3 files moved to new destination; `vault_path` updated in DB | No error expected |
| Change Vault path (file locked / error) | Source file held open / copy fails | Migration aborted; all copied files cleaned up; source intact; DB unchanged | Returns error naming failure; rollback guarantees no partial state |
| Latest finding size (empty library) | DB has 0 findings | Returns `None`; UI displays "No captures yet" | No error expected |
| Open Vault in Explorer | Click "Open Folder" | Opens current Vault directory in OS file manager | If directory missing, creates it or reports error |

</intent-contract>

## Code Map

- `web/ui/src/components/TextField.tsx` -- Update to generate/accept `id` and connect `label` via `htmlFor` for screen-reader accessibility.
- `web/ui/src/test/components.test.tsx` -- Test accessibility and `htmlFor`/`id` association in `TextField`.
- `apps/desktop/src-tauri/src/commands/mod.rs` -- Module declarations for Tauri IPC commands.
- `apps/desktop/src-tauri/src/commands/settings.rs` -- Tauri IPC command handlers: `get_settings`, `set_vault_path`, `set_quality_budget`, `get_latest_finding_size`, `open_vault_folder`.
- `apps/desktop/src-tauri/src/vault_migration.rs` -- Atomic Vault directory file migration and rollback logic (all-or-nothing move per BR-29).
- `apps/desktop/src-tauri/src/state.rs` -- Tauri managed application state holding `Arc<SqliteSettingsStore>`.
- `apps/desktop/src-tauri/src/main.rs` -- Register Tauri state and invoke handler commands.
- `apps/desktop/src/types/settings.ts` -- TypeScript interfaces for `Settings`, `QualityBudget`, and IPC responses.
- `apps/desktop/src/services/settings.ts` -- Frontend client wrapper invoking Tauri IPC commands.
- `apps/desktop/src/components/VaultSection.tsx` -- Vault folder selector, change confirmation dialog, open in Explorer button, and validation feedback.
- `apps/desktop/src/components/QualityBudgetSection.tsx` -- Quality Budget inputs/sliders (max long edge px and quality percentage) and latest finding size indicator.
- `apps/desktop/src/App.tsx` -- Settings screen (Screen 12, `/settings`) composing header, `VaultSection`, `QualityBudgetSection`, and design system tokens.
- `apps/desktop/src/test/shell.test.tsx` -- Integration tests for Settings screen rendering, form interactions, validation errors, and IPC mocks.

## Tasks & Acceptance

**Execution:**
- `web/ui/src/components/TextField.tsx` -- Add `id` prop / autogenerated fallback and attach `htmlFor` on `<label>` -- Fix accessible name announcing for screen readers.
- `web/ui/src/test/components.test.tsx` -- Add test asserting `<label htmlFor="...">` matches input `id` -- Ensure accessibility regression prevention.
- `apps/desktop/src-tauri/src/vault_migration.rs` -- Implement atomic file migration with rollback and writability check -- Ensure all-or-nothing file transfer across directories (BR-29).
- `apps/desktop/src-tauri/src/commands/settings.rs` -- Implement Tauri IPC commands for settings retrieval, quality budget update, vault path migration, and opening in explorer -- Wire frontend to backend store.
- `apps/desktop/src-tauri/src/main.rs` -- Register managed state and IPC commands in Tauri builder -- Expose commands to webview.
- `apps/desktop/src/types/settings.ts` & `apps/desktop/src/services/settings.ts` -- Create typed frontend IPC communication layer -- Type-safe Tauri IPC invokes.
- `apps/desktop/src/components/VaultSection.tsx` -- Build Vault section with path input, browse/change dialog, and open-in-explorer -- Implement UC-14 / FR-16.
- `apps/desktop/src/components/QualityBudgetSection.tsx` -- Build Quality Budget section with range-validated inputs and latest finding size display -- Implement UC-13 / FR-5.
- `apps/desktop/src/App.tsx` -- Assemble Settings screen with sections and notifications -- Deliver Screen 12.
- `apps/desktop/src/test/shell.test.tsx` -- Write comprehensive tests for Settings UI interactions, validation, and display -- Verify acceptance criteria.

**Acceptance Criteria:**
- Given a `TextField` with `label="Vault Path"`, when rendered, then the `<label>` has a `htmlFor` attribute matching the `<input>` element's `id`, ensuring screen readers announce "Vault Path".
- Given an unwritable folder path, when submitted as a new Vault location, then the backend refuses the change with an error and the existing Vault path remains unchanged.
- Given an existing Vault containing files, when changing the Vault location to a new folder and a file move fails midway, then all moved files are restored to the source Vault, no partial files remain in the target, and `vault_path` in `library.db` is not modified.
- Given a Quality Budget input with `max_long_edge` outside `320..=7680` or `encoder_quality` outside `10..=100`, when entered, then the change is refused and an error message is presented.
- Given a library with zero Findings, when the Settings screen loads, then the Quality Budget section displays that no captures exist yet rather than displaying `0 B` or `0 KB`.
- Given the Settings screen, when clicking "Open in Explorer", then the current Vault directory is opened in the operating system file manager.
- Given `cargo clippy`, `cargo test`, `npm run lint`, and `npm run test` across workspace, all checks pass with zero warnings or errors.

## Spec Change Log

_None._

## Review Triage Log

### 2026-08-23 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 2: (high 0, medium 2, low 0)
- defer: 0
- reject: 0
- addressed_findings:
  - `[medium]` `[patch]` Added comprehensive Rust test coverage for Vault migration rollback on failure and store immutability on path refusal in `apps/desktop/src-tauri/src/vault_migration.rs` and `apps/desktop/src-tauri/src/commands/settings.rs`.
  - `[medium]` `[patch]` Added frontend integration tests covering "Leave Files" migration branch and backend refusal error presentation in `apps/desktop/src/test/shell.test.tsx`.

## Auto Run Result

### Summary of Implemented Change
Implemented the Snapdown Settings screen (Screen 12, route `/settings`) satisfying UC-13, UC-14, FR-5, and FR-16. Implemented backend Tauri IPC commands for settings management (`get_settings`, `set_vault_path`, `set_quality_budget`, `get_latest_finding_size`, `open_vault_folder`), atomic vault directory migration with rollback on copy failure (BR-29), quality budget validation against shipped defaults (1600 px, 75 quality) and boundaries (`[320..=7680]`, `[10..=100]`), and latest finding size reporting. Fixed accessible labeling in `TextField` by connecting label `htmlFor` to input `id`.

### Files Changed
- `web/ui/src/components/TextField.tsx` — Added accessible `htmlFor`/`id` linking with `useId` fallback.
- `web/ui/src/test/components.test.tsx` — Added unit tests verifying `TextField` label association.
- `apps/desktop/src-tauri/Cargo.toml` — Added workspace `tempfile` dev-dependency for integration tests.
- `apps/desktop/src-tauri/src/vault_migration.rs` — Atomic file migration with rollback and writability checks.
- `apps/desktop/src-tauri/src/state.rs` — Tauri application managed state holding `SqliteSettingsStore`.
- `apps/desktop/src-tauri/src/commands/mod.rs` — Module declaration for Tauri IPC commands.
- `apps/desktop/src-tauri/src/commands/settings.rs` — Tauri IPC command handlers with boundary validation.
- `apps/desktop/src-tauri/src/lib.rs` & `apps/desktop/src-tauri/src/main.rs` — Registered IPC commands and managed state in Tauri application builder.
- `apps/desktop/src/types/settings.ts` — TypeScript interfaces for Settings and QualityBudget.
- `apps/desktop/src/services/settings.ts` — Tauri IPC client invoke bindings.
- `apps/desktop/src/components/VaultSection.tsx` — React component for Vault path management and Explorer action.
- `apps/desktop/src/components/QualityBudgetSection.tsx` — React component for Quality Budget limits and latest finding size.
- `apps/desktop/src/App.tsx` — Main Settings screen assembly using design system tokens.
- `apps/desktop/src/test/shell.test.tsx` — Integration test suite verifying Settings screen rendering, validation, and actions.

### Review Findings Breakdown
- Patches applied: 2 (medium severity test additions for edge-case coverage)
- Items deferred: 0
- Items rejected: 0

### Follow-up Review Recommendation
- Recommended: false (0 high, 2 medium, 0 low; score = 6; follow-up review recommended is false)

### Verification Performed
- `cargo fmt --all -- --check` — Passed
- `cargo clippy --workspace --all-targets -- -D warnings` — Passed cleanly
- `cargo test --workspace` — Passed (19 unit/integration tests across workspace)
- `npm --prefix web/ui run typecheck` — Passed
- `npm --prefix web/ui run lint` — Passed
- `npm --prefix web/ui run test` — Passed (18 tests)
- `npm --prefix apps/desktop run typecheck` — Passed
- `npm --prefix apps/desktop run lint` — Passed
- `npm --prefix apps/desktop run test` — Passed (8 tests)
- `npm --prefix apps/desktop run build` — Passed
- `uv run .constitution/method/scripts/validate.py --check` — Passed (baseline comparison intact)

### Residual Risks
None. All components adhere strictly to token styles and core port seams.

## Design Notes

### Shipped Quality Budget Defaults
- Default Long Edge: `1600` px (`DEFAULT_MAX_LONG_EDGE_PX`, see OQ-3 in `.control/questions/assumptions.md`).
- Default Encoder Quality: `75` (`DEFAULT_ENCODER_QUALITY`, see OQ-3 in `.control/questions/assumptions.md`).
- Bounds: Long edge `[320, 7680]`, Quality `[10, 100]`.

### Atomic Vault Migration (BR-29)
When moving files from old Vault to new Vault:
1. Validate new directory exists and is writable (attempt creating and removing a temporary test file).
2. Scan old Vault for all files/subdirectories.
3. If old Vault has files, copy each file to a staging area or directly to destination, tracking copied files.
4. If any file fails to copy (e.g., file lock, permission error, out of disk space), delete all copied files in destination, leave old Vault untouched, and return an explicit refusal error.
5. Once all files are copied and verified, delete the source files from old Vault.
6. Update `vault_path` in `SqliteSettingsStore`.

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: All Rust files formatted cleanly
- `cargo clippy --workspace --all-targets -- -D warnings` -- expected: Clippy clean with no warnings across workspace
- `cargo test --workspace` -- expected: All unit and integration tests pass, including vault migration and IPC tests
- `npm --prefix web/ui run typecheck` -- expected: Shared UI package clean
- `npm --prefix web/ui run lint` -- expected: Shared UI linter clean
- `npm --prefix web/ui run test` -- expected: Shared UI component tests pass
- `npm --prefix apps/desktop run typecheck` -- expected: Desktop frontend clean
- `npm --prefix apps/desktop run lint` -- expected: Desktop frontend linter clean
- `npm --prefix apps/desktop run test` -- expected: Desktop frontend tests pass
- `uv run .constitution/method/scripts/validate.py --check` -- expected: Validator passes baseline comparison
