---
title: 'W1-S5: Run at Windows startup, reflecting the real registration'
type: 'feature'
created: '2026-08-23'
status: 'done'
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

**Problem:** Snapdown lacks startup registration management (`LC-026 startup-registrar`). The Reviewer cannot configure Snapdown to run at Windows sign-in or disable it, the setting state is not read directly from the OS to guarantee honesty against external registry drift (FR-18), and the Settings screen at `/settings` lacks a General/Startup toggle section.

**Approach:** Implement a `StartupRegistrar` adapter (`DesktopStartupRegistrar`) in `apps/desktop/src-tauri` using `tauri-plugin-autostart` (or Windows CurrentUser registry `Software\Microsoft\Windows\CurrentVersion\Run`) that requires no administrator privileges (NFR-7, OQ-5). Expose Tauri IPC commands (`get_startup_status`, `set_startup_status`) that query OS registration directly on every invocation without caching (FR-18) and remove registration completely upon disabling. Add a `GeneralSection` containing the startup toggle to the React Settings screen (`/settings`) using the design system's `Checkbox` element and tokens.

## Boundaries & Constraints

**Always:**
- Startup registration MUST succeed and operate without administrator rights (NFR-7, OQ-5), writing exclusively to the current user's startup scope (e.g. `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` or `tauri-plugin-autostart`).
- Starting Snapdown via Windows startup opens NO window — the system tray icon only (FR-18).
- The startup setting reflects the actual OS registration, NOT a remembered intention or cached database boolean (FR-18). Every time the Settings screen or status is requested, query the OS directly.
- Disabling startup removes the registration completely from the OS rather than leaving it in place and ignoring it (FR-18).
- Keep `snapdown-core` completely free of I/O, OS, filesystem, network, or clock dependencies.
- Build the Settings UI toggle using shared UI primitives (`Checkbox`) and design system tokens (`tokens.css`) without literal CSS overrides.

**Block If:**
- Enabling autostart on Windows requires administrator privileges or elevation prompts (violating NFR-7 / OQ-5).
- OS startup query cannot be performed reliably without administrator rights.

**Never:**
- Do not cache or assume the startup registration state in SQLite or memory; the OS is the single source of truth for `RunAtStartup`.
- Do not open the main Settings window when launched via autostart/startup args.
- Do not introduce telemetry, network calls, or crash reporting (AD-6).
- Do not write to `HKEY_LOCAL_MACHINE` or any system-wide registry keys requiring UAC elevation.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Query startup status (initially unregistered) | Settings screen mounts | Invokes `get_startup_status`; queries OS directly; returns `{ enabled: false }` | If OS query fails, returns error; UI displays error toast |
| Query startup status (registered in OS externally or previously) | Settings screen mounts; OS registry has entry | Invokes `get_startup_status`; queries OS directly; returns `{ enabled: true }` | No error expected |
| Enable startup registration | Reviewer toggles "Run at Windows startup" checkbox to checked | Invokes `set_startup_status(enabled: true)`; registers current executable in user startup registry; returns `{ enabled: true }` | If OS rejects registration, returns error; checkbox remains unchecked; store unchanged |
| Disable startup registration | Reviewer toggles "Run at Windows startup" checkbox to unchecked | Invokes `set_startup_status(enabled: false)`; completely deletes entry from user startup registry; returns `{ enabled: false }` | If deletion fails, returns error; checkbox reverts |
| Launch app via OS startup | App launched with autostart / sign-in flag | App initializes background runtime and tray icon; main window is hidden (not opened) | Handled in Tauri startup lifecycle |
| Query without admin rights | Standard user session without elevation | `is_enabled`, `enable`, and `disable` succeed in `HKCU` scope without UAC prompt | NFR-7 satisfied |

</intent-contract>

## Code Map

- `crates/snapdown-core/src/ports/mod.rs` -- `StartupRegistrar` port trait (`is_enabled`, `enable`, `disable`).
- `apps/desktop/src-tauri/Cargo.toml` -- Add `tauri-plugin-autostart = "2.0"` dependency (or Windows registry support via `windows-sys` / `winreg` / Tauri plugin).
- `apps/desktop/src-tauri/src/startup/mod.rs` -- Implement `DesktopStartupRegistrar` adapter implementing `StartupRegistrar` and Tauri backend wrapper querying and mutating OS startup state without elevation.
- `apps/desktop/src-tauri/src/commands/startup.rs` -- Tauri IPC commands: `get_startup_status`, `set_startup_status`.
- `apps/desktop/src-tauri/src/commands/mod.rs` -- Export startup IPC command module.
- `apps/desktop/src-tauri/src/state.rs` -- Add `startup_registrar: Arc<Mutex<dyn StartupRegistrar + Send + Sync>>` (or concrete `DesktopStartupRegistrar`) to `AppState`.
- `apps/desktop/src-tauri/src/lib.rs` & `apps/desktop/src-tauri/src/main.rs` -- Initialize `tauri-plugin-autostart`, configure startup flags/suppression to ensure autostart opens tray only, register IPC commands.
- `apps/desktop/src/types/settings.ts` -- TypeScript interfaces for `StartupSettingsDto`.
- `apps/desktop/src/services/settings.ts` -- Add `getStartupStatus` and `setStartupStatus` frontend API client functions.
- `apps/desktop/src/components/GeneralSection.tsx` -- React component for General/Startup section with accessible `Checkbox` toggle reflecting real OS state.
- `apps/desktop/src/App.tsx` -- Mount `GeneralSection` on the Settings screen (`/settings`) and wire state handlers.
- `apps/desktop/src/test/shell.test.tsx` -- Front-end tests for startup toggle rendering, reading OS state, and toggle interaction.

## Tasks & Acceptance

**Execution:**
- `apps/desktop/src-tauri/Cargo.toml` -- Add `tauri-plugin-autostart` dependency -- Provide non-elevated user startup registration capability.
- `apps/desktop/src-tauri/src/startup/mod.rs` -- Implement `DesktopStartupRegistrar` adapter -- Implement `LC-026 startup-registrar` reading and modifying OS registration directly.
- `apps/desktop/src-tauri/src/commands/startup.rs` -- Implement `get_startup_status` and `set_startup_status` Tauri IPC commands -- Expose direct OS queries to webview.
- `apps/desktop/src-tauri/src/lib.rs` -- Wire plugin, state, IPC handlers, and autostart silent launch behavior -- Hook startup management into Tauri lifecycle.
- `apps/desktop/src/types/settings.ts` & `apps/desktop/src/services/settings.ts` -- Add startup data contracts and IPC client invoke methods -- Type-safe frontend communication.
- `apps/desktop/src/components/GeneralSection.tsx` -- Build General / Startup settings UI component using `Checkbox` and tokens -- Implement UI for UC-16 / FR-18.
- `apps/desktop/src/App.tsx` -- Integrate `GeneralSection` into Settings view -- Complete Settings screen sections.
- `apps/desktop/src-tauri/tests/test_startup.rs` (or unit tests in `src/startup/mod.rs`) -- Write Rust unit/integration tests for non-admin registration, live OS readback, and complete removal on disable -- Satisfy named test suite requirements (`startup_registration_needs_no_administrator_rights`, `the_setting_is_read_back_from_the_os_not_remembered`, `disabling_removes_the_registration`).
- `apps/desktop/src/test/shell.test.tsx` -- Add frontend test cases for startup toggle interaction and live status sync -- Prevent frontend regressions.

**Acceptance Criteria:**
- Given a machine without administrator privileges, when enabling startup registration, then the operation succeeds in the user registry scope without prompting for UAC elevation (NFR-7, OQ-5).
- Given the Settings screen opening, when startup status is displayed, then the state is queried directly from the operating system registration rather than read from a cached DB flag (FR-18).
- Given an active startup registration, when toggling the setting to disabled, then the registration entry is completely removed from the OS startup registry.
- Given an application launch initiated by the OS startup/autostart runner, then Snapdown starts directly to the system tray icon with no window displayed.
- Given `cargo clippy`, `cargo test`, `npm run lint`, and `npm run test` across workspace, all checks pass with zero warnings or errors.

## Spec Change Log

_None._

## Review Triage Log

_None._

## Design Notes

### Non-Elevated Registration (NFR-7, OQ-5)
The startup registrar operates in user scope via `tauri-plugin-autostart` (targeting `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` on Windows). Because it targets `HKCU`, standard user accounts can register and unregister without administrative privileges or elevation prompts.

### OS Readback vs Database Cache (FR-18)
`SqliteSettingsStore` does not act as the authoritative source for `RunAtStartup`. Instead, `get_startup_status` delegates to `StartupRegistrar::is_enabled()`, ensuring that external modifications (e.g. user toggling startup items via Windows Task Manager) are immediately and accurately reflected in the Settings UI.

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: All Rust workspace files formatted cleanly
- `cargo clippy --workspace --all-targets -- -D warnings` -- expected: Clippy clean with no warnings
- `cargo test --workspace` -- expected: All unit and integration tests pass, including:
  - `cargo::startup_registration_needs_no_administrator_rights`
  - `cargo::the_setting_is_read_back_from_the_os_not_remembered`
  - `cargo::disabling_removes_the_registration`
- `npm --prefix web/ui run typecheck` -- expected: Shared UI package clean
- `npm --prefix web/ui run lint` -- expected: Shared UI linter clean
- `npm --prefix web/ui run test` -- expected: Shared UI tests pass
- `npm --prefix apps/desktop run typecheck` -- expected: Desktop frontend clean
- `npm --prefix apps/desktop run lint` -- expected: Desktop frontend linter clean
- `npm --prefix apps/desktop run test` -- expected: Desktop frontend tests pass
- `npm --prefix apps/desktop run build` -- expected: Desktop Vite build succeeds
- `uv run .constitution/method/scripts/validate.py --check` -- expected: Corpus validator passes baseline comparison
