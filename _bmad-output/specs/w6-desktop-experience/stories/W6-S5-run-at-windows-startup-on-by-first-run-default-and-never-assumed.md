---
id: W6-S5
title: 'W6-S5: Run at Windows startup — on by first-run default, and never assumed'
type: 'feature'
wave: W6
status: done
created: '2026-08-24'
dependencies:
  - W6-S4
files:
  - crates/snapdown-core/src/domain/setting.rs
  - crates/snapdown-store/src/sqlite/settings_store.rs
  - apps/desktop/src-tauri/src/commands/startup.rs
  - apps/desktop/src-tauri/src/startup/mod.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tests/test_startup.rs
  - apps/desktop/src/types/settings.ts
  - apps/desktop/src/services/settings.ts
  - apps/desktop/src/components/GeneralSection.tsx
  - apps/desktop/src/components/SettingsView.tsx
  - apps/desktop/src/App.tsx
  - apps/desktop/src/test/shell.test.tsx
  - apps/desktop/src/test/startup.test.tsx
context:
  - _bmad-output/specs/w6-desktop-experience/SPEC.md
  - _bmad-output/specs/w6-desktop-experience/stories.yaml
  - _bmad-output/specs/w6-desktop-experience/dispatch-briefs/W6-S5-step1-plan.md
  - .what/settings/05-scenarios/SCN-02-the-first-run-and-the-startup-default.md
  - .how/settings/06-flows/flow-startup-reconciliation.md
  - .what/settings/03-domain/state-machines.md
  - .how/settings/SDD-settings.md
  - .how/settings/02-contracts/contract-inventory.md
  - .what/business-rules.md
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .control/decisions/DEC-003-one-process-two-windows.md
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:**
1. **First-Run Startup Default and the Silent Run-3 Regression (`SCN-02`, `BR-112`, `AUDIT-4`):**
   - The shipped build leaves `Run at Windows startup` unchecked by default (`AUDIT-4`). The owner requested that Snapdown run at Windows sign-in by default on a fresh installation (`BR-112`).
   - The naive implementation (*"if not registered in Windows, register it"*) passes a fresh install (Run 1) and passes the moment a Reviewer unchecks the toggle (Run 2), but **fails Run 3 silently**: on the next sign-in, Snapdown detects it is not registered, assumes it must register, and silently re-registers itself in Windows autostart against the Reviewer's explicit instruction.
   - **The Run 4 Trap (`FR-18`, `BR-114`):** If startup registration is removed outside Snapdown (e.g. via Task Manager or group policy), the stored state may say `on` while the OS says not registered. Snapdown must reflect the actual OS registration (`Off`) and must **not** silently re-register.
   - The store must track whether the Reviewer's preference has ever been expressed (`startup.registered`), distinguishing *unset* (first run) from *decided*.
2. **Honest Control State & The Unknown State (`SCN-02`, `FR-18`, `BR-108`, `state-machines.md` § 1):**
   - Reading the autostart registration from Windows is asynchronous.
   - The shipped build initialized React state with `useState(true)` (guessing `On`) and repainted to `Off` once the read resolved, making the product visibly change its mind.
   - `StartupStatusDto`'s `enabled: bool` cannot convey a three-state domain (`On`, `Off`, `Unreadable`).
   - The startup control must render `Unknown` (`aria-checked="mixed"`, `data-state="indeterminate"`, inert) and never render a definite `On` or `Off` before Windows answers (`state-machines.md` § 1). If the read fails, it must show `Unreadable` with a `Retry` action.
3. **BUG-12 (High Severity): Silent Application Absence on Corrupt Database (`BR-118`, `AD-11`, `SDD-settings.md`):**
   - `apps/desktop/src-tauri/src/lib.rs:109-119` opens all five SQLite stores (`SqliteSettingsStore`, `SqliteFindingStore`, `SqliteBundleStore`, `SqliteAccessKeyStore`, `SqlitePublicationStore`) with `.expect(...)`.
   - On Windows release binaries, there is no attached console. An unreadable or corrupt `library.db` panics inside the Tauri setup hook, immediately terminating the process without creating a window, tray icon, message box, or log entry. The Reviewer double-clicks `Snapdown.exe` and **nothing happens at all** — the application appears absent rather than broken.
   - `SDD-settings.md` Failure Behaviour and `BR-118` require that a corrupt store is **reported with the file's path, and nothing is created over it** (preventing silent data loss).
   - The setup hook must open stores fallibly. On error, it must notify the Reviewer via a native system dialog naming the exact database path and failure reason, write the failure to a log file, and exit cleanly without panicking.

**Approach:**
1. **Domain Model & Settings Store (`crates/snapdown-core`, `crates/snapdown-store`):**
   - In `crates/snapdown-core/src/domain/setting.rs`:
     - Add `SettingKey::StartupRegistered` mapping to `"startup.registered"`.
   - In `crates/snapdown-store/src/sqlite/settings_store.rs`:
     - Support reading and writing `SettingKey::StartupRegistered` as a string (`"expressed"`) or boolean.
2. **Startup Reconciliation Logic (`apps/desktop/src-tauri/src/startup/`):**
   - Implement `reconcile_startup_on_boot(store: &dyn SettingsStore, registrar: &mut dyn StartupRegistrar, clock: &dyn Clock) -> Result<(), CoreError>`:
     - Read `startup.registered` from `store`.
     - If `None` (Run 1: fresh install with nothing configured):
       - Apply first-run default (`BR-112`): call `registrar.enable()`.
       - Write `startup.registered = "expressed"` to `store` (even if OS refused, recording that default was applied to prevent an infinite fight at every sign-in).
     - If `Some(_)` (Run 2, 3, 4: preference already expressed):
       - Do **not** touch the registrar during boot. The OS state remains whatever the Reviewer or system left it (`SCN-02`).
3. **Tauri IPC Command & DTO Update (`apps/desktop/src-tauri/src/commands/startup.rs`):**
   - Update `StartupStatusDto`:
     ```rust
     #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
     #[serde(rename_all = "snake_case")]
     pub enum StartupState {
         On,
         Off,
         Unreadable,
     }

     #[derive(Debug, Clone, Serialize, Deserialize)]
     pub struct StartupStatusDto {
         pub enabled: bool,
         pub state: StartupState,
     }
     ```
   - `get_startup_status_impl(state: &AppState) -> Result<StartupStatusDto, String>`:
     - Query `registrar.is_enabled()`.
     - `Ok(true)` -> `StartupStatusDto { enabled: true, state: StartupState::On }`.
     - `Ok(false)` -> `StartupStatusDto { enabled: false, state: StartupState::Off }`.
     - `Err(e)` -> `StartupStatusDto { enabled: false, state: StartupState::Unreadable }` (or return error string if IPC error).
   - `set_startup_status_impl(enabled: bool, state: &AppState) -> Result<StartupStatusDto, String>`:
     - If `enabled`: `registrar.enable()`.
     - If `!enabled`: `registrar.disable()`.
     - Re-read `registrar.is_enabled()` to return the actual resulting state (`FR-18`, `BR-114`).
     - Write `startup.registered = "expressed"` to ensure future boots recognize the preference was set.
4. **BUG-12 Fallible Store Setup & Native Error Reporting (`apps/desktop/src-tauri/src/lib.rs`):**
   - Implement `init_app_stores(db_path: &Path) -> Result<StoresBundle, StartupError>`:
     - Open all 5 stores fallibly (`SqliteSettingsStore::open`, `SqliteFindingStore::open`, `SqliteBundleStore::open`, `SqliteAccessKeyStore::open`, `SqlitePublicationStore::open`).
     - If any store fails, return `Err(StartupError::DatabaseOpen { path: db_path.to_path_buf(), source: err })`.
     - Never delete, overwrite, or re-initialize a corrupt store (`BR-118`).
   - In `lib.rs` setup hook:
     - Call `init_app_stores(&db_path)`.
     - If `Err(err)`:
       - Format error message detailing the file path: *"Snapdown could not open its library database at <path>.\n\nError: <details>\n\nSnapdown will not recreate or overwrite this file to prevent data loss."*
       - Display native system message box to the Reviewer (Win32 `MessageBoxW` on Windows, system alert on macOS/Linux).
       - Write error detail to `app_data_dir/startup-error.log`.
       - Exit the process cleanly (`std::process::exit(1)` or return setup `Err`) without an unhandled panic.
     - If `Ok(stores)`: proceed to `reconcile_startup_on_boot` and initialize app state.
5. **Frontend 3-State Toggle & Error Recovery (`apps/desktop/src/`):**
   - Update `types/settings.ts` and `services/settings.ts`:
     - Add `StartupState = 'unknown' | 'on' | 'off' | 'unreadable'`.
     - `StartupSettingsDto { enabled: boolean; state?: 'on' | 'off' | 'unreadable'; }`.
   - Update `GeneralSection.tsx`:
     - Replace boolean `Checkbox` with `@snapdown/ui` `Toggle` component.
     - When `status === 'unknown'`: `Toggle` renders with `indeterminate={true}`, `checked={false}`, `aria-checked="mixed"`, `disabled={true}`.
     - When `status === 'on'`: `Toggle` renders with `indeterminate={false}`, `checked={true}`, `aria-checked="true"`.
     - When `status === 'off'`: `Toggle` renders with `indeterminate={false}`, `checked={false}`, `aria-checked="false"`.
     - When `status === 'unreadable'`: Display an inline warning badge/message *"Could not read Windows startup status"* and a *"Retry"* button (`state-machines.md` § 1).
   - Update `App.tsx`:
     - Initialize `startupStatus` as `'unknown'` (never defaulting to `true` or `false`).
     - Query `getStartupStatus()` on mount; update state on resolution.
     - On toggle click, call `setStartupStatus(newEnabled)` and update state from the returned DTO.
6. **Automated Verification Suite:**
   - Rust unit & integration tests (`snapdown-store`, `apps/desktop/src-tauri/tests/test_startup.rs`):
     - `cargo::an_unreadable_library_db_is_reported_with_its_path_and_not_recreated` (`BUG-12`)
     - `cargo::a_corrupt_library_db_does_not_panic_the_setup_hook` (`BUG-12`)
     - `cargo::a_first_run_with_nothing_configured_registers_at_startup` (`SCN-02`, Run 1)
     - `cargo::a_run_after_the_reviewer_disabled_it_does_not_re_register` (`SCN-02`, Run 3)
     - `cargo::a_registration_removed_outside_snapdown_shows_off_and_is_not_restored` (`SCN-02`, Run 4)
   - Vitest component & shell tests (`apps/desktop/src/test/`):
     - `vitest::the_startup_toggle_renders_unknown_until_the_os_has_answered`
     - `vitest::the_startup_toggle_never_renders_a_definite_state_before_the_read_resolves`
     - `vitest::startup_toggle_shows_on_when_registered`
     - `vitest::startup_toggle_shows_off_when_not_registered`
     - `vitest::startup_toggle_shows_unreadable_with_retry_on_read_failure`

## Boundaries & Constraints

**Always:**
- Startup registration state MUST ALWAYS be read directly from the operating system, never assumed or read from a stored cache (`FR-18`, `BR-114`).
- The startup toggle MUST render `Unknown` (`indeterminate`, `aria-checked="mixed"`) until the OS read resolves, and MUST NOT guess `On` or `Off` (`BR-108`, `state-machines.md` § 1).
- First-run registration default MUST only execute if `startup.registered` is absent in SQLite (`BR-112`, `SCN-02`).
- The store write `startup.registered = expressed` MUST happen after the registration attempt on first run, whether the attempt succeeded or was refused (`SCN-02`, `flow-startup-reconciliation.md`).
- When startup is disabled by the Reviewer, Snapdown MUST NOT re-register on subsequent boots (`SCN-02`, Run 3).
- When startup registration is removed outside Snapdown, the toggle MUST reflect `Off` and Snapdown MUST NOT re-register (`SCN-02`, Run 4).
- A corrupt or unreadable `library.db` MUST NOT panic the process and MUST NOT be overwritten or recreated (`BR-118`, `BUG-12`).
- Database open failure MUST be reported to the Reviewer naming the exact database path and failure reason via native message dialog and log file (`SDD-settings.md`, `BUG-12`).
- Use `@snapdown/ui` `Toggle` for the 3-state control.
- Use `MockAutoStartBackend` / `NoopAutoStartBackend` in unit/integration tests without touching real OS registry keys.
- Extract testable logic into plain `_impl` functions taking `&AppState` rather than using `tauri::test`.

**Block If:**
- Upstream requirements demand hardcoding auto-registration on every boot without checking `startup.registered`.
- Native dialog calls block headless CI execution (dialog calls must only run when setup fails in real binary launch or be abstracted behind reporting seam).

**Never:**
- Never call `.expect()` / `.unwrap()` on database opening in production code.
- Never delete or recreate a corrupt SQLite database file on disk (`BR-118`).
- Never default the frontend toggle to `checked={true}` or `checked={false}` before the backend query resolves.
- Never modify corpus documents in `.what/`, `.how/`, or `.constitution/`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Run 1 — Fresh install | `startup.registered` is absent (unset); OS has no registration | Snapdown calls `registrar.enable()`; writes `startup.registered = "expressed"`; toggle renders `On` once resolved | Default applied cleanly |
| Run 2 — Reviewer turns off | Reviewer toggles switch to Off | Calls `registrar.disable()`; OS confirms; toggle updates to `Off`; `startup.registered` remains `"expressed"` | State updated in OS |
| Run 3 — Next sign-in | `startup.registered` is present (`"expressed"`); OS has no registration | Boot does NOT call `registrar.enable()`; toggle queries OS, receives `false`, renders `Off` | Preserves Reviewer's choice |
| Run 4 — External removal | `startup.registered` is present; registration deleted in Task Manager | Boot does NOT call `registrar.enable()`; toggle queries OS, receives `false`, renders `Off` (`FR-18`, `BR-114`) | Reflects OS ground truth |
| Run 5 — Lost store | `library.db` lost/deleted; OS registration remains | First-run branch fires: calls `registrar.enable()`, writes `expressed` (`SCN-02` accepted behavior) | Handled per SCN-02 |
| Asynchronous read pending | Settings opens; `get_startup_status` is pending | Toggle renders indeterminate / `Unknown` (`aria-checked="mixed"`), inert; no `On` or `Off` painted | No false state displayed |
| OS read error | `get_startup_status` fails / registrar returns error | Toggle renders `Unreadable` state; shows error text and a `Retry` button (`state-machines.md` § 1) | User can click Retry |
| First run OS refusal | Fresh install; OS forbids autostart (group policy) | `registrar.enable()` returns error; store still writes `startup.registered = "expressed"`; toggle renders `Off` | No infinite boot loop |
| Corrupt `library.db` at boot | `library.db` contains corrupted bytes | Process does NOT panic; native dialog shows naming `library.db` path and corruption details; log written; exits cleanly | Zero data overwritten (`BR-118`) |
| Unreadable `library.db` permissions | `library.db` is locked or read-only/inaccessible | Process does NOT panic; reports path and permission error; exits cleanly without recreating | File preserved on disk |

</intent-contract>

## Code Map

- `crates/snapdown-core/src/domain/setting.rs`
  - Add `SettingKey::StartupRegistered` (`"startup.registered"`).
  - Update `SettingKey::as_str` and `SettingKey::from_key_str`.
- `crates/snapdown-store/src/sqlite/settings_store.rs`
  - Handle `SettingKey::StartupRegistered` in `parse_setting_value` and `serialize_setting_value`.
- `apps/desktop/src-tauri/src/startup/mod.rs`
  - Define `reconcile_startup_on_boot` logic taking `SettingsStore`, `StartupRegistrar`, and `Clock`.
  - Provide testable `reconcile_startup_impl` function.
- `apps/desktop/src-tauri/src/commands/startup.rs`
  - Update `StartupStatusDto` and `StartupState` enum (`On`, `Off`, `Unreadable`).
  - Implement `get_startup_status_impl(&AppState)` and `set_startup_status_impl(bool, &AppState)`.
  - Update `#[tauri::command] get_startup_status` and `set_startup_status` shims.
- `apps/desktop/src-tauri/src/lib.rs`
  - Implement `init_app_stores(db_path: &Path)` fallibly without `.expect()`.
  - Implement `report_startup_error(err: &StartupError, db_path: &Path)` displaying native message box (Win32 `MessageBoxW`) and logging to `startup-error.log`.
  - In `setup` hook: call `init_app_stores`, handle failure cleanly, and run `reconcile_startup_on_boot`.
- `apps/desktop/src-tauri/tests/test_startup.rs`
  - Implement `cargo::an_unreadable_library_db_is_reported_with_its_path_and_not_recreated`.
  - Implement `cargo::a_corrupt_library_db_does_not_panic_the_setup_hook`.
  - Implement `cargo::a_first_run_with_nothing_configured_registers_at_startup`.
  - Implement `cargo::a_run_after_the_reviewer_disabled_it_does_not_re_register`.
  - Implement `cargo::a_registration_removed_outside_snapdown_shows_off_and_is_not_restored`.
- `apps/desktop/src/types/settings.ts`
  - Add `StartupState = 'unknown' | 'on' | 'off' | 'unreadable'`.
  - Update `StartupSettingsDto { enabled: boolean; state?: 'on' | 'off' | 'unreadable'; }`.
- `apps/desktop/src/services/settings.ts`
  - Update `getStartupStatus` and `setStartupStatus` return types.
- `apps/desktop/src/components/GeneralSection.tsx`
  - Rebuild with `@snapdown/ui` `Toggle` component.
  - Support `status: StartupState` (`'unknown'`, `'on'`, `'off'`, `'unreadable'`).
  - Render indeterminate state with `aria-checked="mixed"` when `unknown`.
  - Render `Unreadable` error state with `Retry` button.
- `apps/desktop/src/components/SettingsView.tsx`
  - Update props to pass `startupStatus: StartupState` and `onRetryStartup: () => void`.
- `apps/desktop/src/App.tsx`
  - Manage `startupStatus: StartupState` initialized to `'unknown'`.
  - Load startup status on mount; handle toggle and retry interactions.
- `apps/desktop/src/test/startup.test.tsx` (and `shell.test.tsx`)
  - Implement `vitest::the_startup_toggle_renders_unknown_until_the_os_has_answered`.
  - Implement `vitest::the_startup_toggle_never_renders_a_definite_state_before_the_read_resolves`.
  - Implement `vitest::startup_toggle_shows_on_when_registered`.
  - Implement `vitest::startup_toggle_shows_off_when_not_registered`.
  - Implement `vitest::startup_toggle_shows_unreadable_with_retry_on_read_failure`.

## Tasks & Acceptance

**Execution:**
- `crates/snapdown-core/src/domain/setting.rs` -- Add `SettingKey::StartupRegistered` with key string `"startup.registered"`.
- `crates/snapdown-store/src/sqlite/settings_store.rs` -- Support `SettingKey::StartupRegistered` serialization and retrieval.
- `apps/desktop/src-tauri/src/startup/mod.rs` -- Implement `reconcile_startup_on_boot` implementing the first-run default logic of `BR-112` and `SCN-02`.
- `apps/desktop/src-tauri/src/commands/startup.rs` -- Update `StartupStatusDto` to support 3-state reporting (`on`, `off`, `unreadable`), and extract `_impl` functions.
- `apps/desktop/src-tauri/src/lib.rs` -- Replace `.expect()` on store opens with fallible `init_app_stores`, native dialog error reporting (`BUG-12`, `BR-118`), and startup reconciliation execution.
- `apps/desktop/src-tauri/tests/test_startup.rs` -- Write the 5 required Rust unit/integration tests for SCN-02 and BUG-12.
- `apps/desktop/src/types/settings.ts` & `services/settings.ts` -- Update startup TypeScript types to support `'unknown' | 'on' | 'off' | 'unreadable'`.
- `apps/desktop/src/components/GeneralSection.tsx` -- Update UI to use `@snapdown/ui` `Toggle` supporting `indeterminate` (`Unknown`) and `Unreadable` error with `Retry`.
- `apps/desktop/src/App.tsx` & `SettingsView.tsx` -- Wire 3-state startup management without guessing boolean values before API returns.
- `apps/desktop/src/test/startup.test.tsx` & `shell.test.tsx` -- Add Vitest assertions proving `Unknown` state rendering and state transitions.

**Acceptance Criteria:**
- Given a fresh installation with no `startup.registered` key in SQLite, when Snapdown starts, `reconcile_startup_on_boot` registers startup with the OS and persists `startup.registered = "expressed"`.
- Given a system where the Reviewer has turned startup off (`startup.registered` exists), when Snapdown restarts, it does not re-register with the OS.
- Given a system where startup registration was removed externally, Settings queries Windows and displays `Off`, and Snapdown does not re-register.
- Given Settings opens, the startup toggle renders in an `indeterminate` (`Unknown`) state with `aria-checked="mixed"` until the backend read completes, and never flashes `On` or `Off` beforehand.
- Given a corrupt or unreadable `library.db`, `init_app_stores` returns an error naming the file path without panicking, leaves the corrupt database untouched on disk (`BR-118`), and reports the error via a system dialog and log file.
- Given `cargo test --workspace` and `npm --prefix apps/desktop run test`, all tests pass with 0 failures and 0 clippy warnings.

## Spec Change Log

<!-- Append-only. Populated during review loops. -->

## Design Notes

**Why `startup.registered` writes `expressed` even on first-run OS refusal:**
If the operating system or user policy refuses autostart registration (e.g. non-admin environment or disabled by administrator), recording `expressed` in the store ensures that Snapdown acknowledges that the first-run attempt took place. Without this record, every subsequent process start would detect an absent key and repeatedly attempt registration, resulting in continuous failed operations and potential error dialogs at every boot.

**Why `Unknown` is a rendered state rather than a loading spinner:**
As defined in `state-machines.md` § 1 and `flow-startup-reconciliation.md`, reading registration from the Windows registry is asynchronous. If the control rendered a loading spinner, the settings layout would shift when the toggle loaded. If it rendered a boolean default, it would flash a false state. Rendering `Unknown` via the three-state `Toggle` (`aria-checked="mixed"`, `indeterminate`) allows the control to occupy its final layout position while honestly communicating that the OS status is being queried.

**Why native dialog is used for `BUG-12`:**
Because Snapdown runs as a single process owning the tray, hotkeys, overlay, and editor (`AD-11`, `DEC-003`), when the database cannot be opened at boot, no webview window exists yet to render an HTML error page. On Windows, a release binary has no terminal or standard output attached. Therefore, a native Win32 `MessageBoxW` dialog is the only reliable way to present an unignorable, clear error message to the human Reviewer specifying the file path before exiting.

## Verification

**Commands:**
- `cargo test -p snapdown --test test_startup` -- expected: Integration tests for SCN-02 runs and BUG-12 pass.
- `cargo test --workspace` -- expected: Full workspace test suite passes.
- `cargo clippy --workspace --all-targets -- -D warnings` -- expected: Zero warnings.
- `npm --prefix apps/desktop run test` -- expected: All frontend tests including startup toggle unknown and state tests pass.
- `npm --prefix apps/desktop run typecheck && npm --prefix apps/desktop run lint` -- expected: Clean typecheck and lint.