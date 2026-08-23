---
title: 'W1-S4: Hotkey binding, OS registration, and honest conflict reporting'
type: 'feature'
created: '2026-08-23'
status: 'done'
baseline_revision: '88846560da6e21405532a37e952a56035dabca8b'
review_loop_iteration: 0
followup_review_recommended: true
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

**Problem:** Snapdown lacks global hotkey management and OS registration (`LC-009 hotkey-registrar`). Users cannot bind or clear shortcuts for Capture and Open Editor, conflict detection (between actions or with external applications) is absent, dynamic rebinding without app restart is unsupported, and startup registration failures are not reported.

**Approach:** Implement `HotkeyAction` (`Capture`, `OpenEditor`) in `snapdown-core` and a `HotkeyRegistrar` adapter in `apps/desktop/src-tauri` using `tauri-plugin-global-shortcut` (or OS shortcut APIs) that operates without administrator rights (NFR-7, OQ-5). Provide Tauri IPC commands (`get_hotkeys`, `set_hotkey`) enforcing uniqueness (BR-27), immediate rebinding without restart, clearing/disabling actions, honest error reporting if another process/OS holds a combination (BR-26), raising an unconsumed `capture-requested` event on capture trigger, and tracking startup registration failures. Add `HotkeySection` to the React Settings screen (`/settings`) using design system tokens.

## Boundaries & Constraints

**Always:**
- Two actions are bindable: `Capture` (default `CommandOrControl+Shift+S`) and `OpenEditor` (default `CommandOrControl+Shift+E`).
- Registration must succeed without administrator rights (NFR-7, OQ-5).
- A combination held by another application/OS is refused at binding time and reported honestly (BR-26, FR-17); if the OS refuses the shortcut, do not fake success or mutate the store.
- Two Snapdown actions cannot share the same hotkey combination (BR-27).
- A cleared hotkey disables that action and unregisters the shortcut from the OS.
- Rebinding takes effect immediately without restarting Snapdown (old combination unregistered, new combination registered).
- Startup registration failures must be recorded and presented honestly in Settings (and tray status) rather than swallowed (BR-26).
- When the Capture hotkey fires, the registrar raises a `capture-requested` event (`app.emit("capture-requested", ())`); leave it unconsumed in this wave without adding placeholder capture logic.
- When the Open Editor hotkey fires, the desktop window is focused and displayed.
- Use design system tokens (`tokens.css`) and base element components (`web/ui`) without literal CSS overrides.
- Maintain `snapdown-core` without I/O dependencies.

**Block If:**
- Windows shortcut APIs or Tauri global shortcut plugins fundamentally require elevated administrator privileges (violating OQ-5 / NFR-7).
- Windows cannot report hotkey registration conflicts at binding time, requiring renegotiation of FR-17.

**Never:**
- Do not implement capture overlay, screen grabbing, or image reduction in this wave.
- Do not create subscriber listeners or placeholder capture handlers for the `capture-requested` event.
- Do not require an application restart for hotkey rebinding to take effect.
- Do not log hotkey values containing sensitive or private user data.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Get hotkey settings (initial/default) | Mount Settings screen | Returns default shortcuts (`CommandOrControl+Shift+S`, `CommandOrControl+Shift+E`) and registration status (`is_registered: true`) | If unset in DB, persists shipped defaults |
| Bind new valid hotkey | Set Capture to `Ctrl+Alt+C` | Unregisters old shortcut, registers `Ctrl+Alt+C` in OS, updates store, returns success | No error expected |
| Duplicate hotkey across actions | Set Open Editor to same as Capture (`Ctrl+Shift+S`) | Refused immediately; store and OS state unchanged | Returns error: "Two actions cannot share the same hotkey combination" (BR-27) |
| Hotkey held by another application/OS | Set Capture to combination already held by OS/other app | Refused at binding time; OS registration fails; store unchanged | Returns error naming the conflict: "Hotkey combination is already held by another application or the operating system" (BR-26) |
| Clear hotkey binding | Clear Open Editor shortcut | Unregisters shortcut from OS; updates store to empty/disabled | Returns success; action is marked disabled |
| Rebind without restart | Change Capture shortcut from A to B | A stops firing immediately; B starts firing immediately without app restart | If registering B fails, keeps or rolls back to A |
| Failed startup registration | Startup fails to register Capture (e.g. system reserved) | App starts; records failed registration state; Settings UI displays one-line warning banner naming inactive hotkey | Not swallowed; surfaces honest failure status |
| Capture hotkey pressed | Reviewer presses active Capture shortcut | Emits `capture-requested` Tauri event | Unconsumed in W1 (no error, no placeholder UI) |
| Open Editor hotkey pressed | Reviewer presses active Open Editor shortcut | Focuses and unminimizes Settings window | No error expected |

</intent-contract>

## Code Map

- `crates/snapdown-core/src/domain/setting.rs` -- Add `HotkeyAction` enum (`Capture`, `OpenEditor`), default shortcut constants (`DEFAULT_HOTKEY_CAPTURE`, `DEFAULT_HOTKEY_OPEN_EDITOR`), and helper methods connecting actions with `SettingKey`.
- `crates/snapdown-core/src/ports/mod.rs` -- Reference/refine `HotkeyRegistrar` trait definition if necessary without adding I/O.
- `apps/desktop/src-tauri/Cargo.toml` -- Add `tauri-plugin-global-shortcut = "2.0"` dependency (or workspace dependency).
- `apps/desktop/src-tauri/src/hotkey/mod.rs` -- Implement `DesktopHotkeyRegistrar` managing global shortcut registrations, collision validation (BR-27), OS conflict error handling (BR-26), runtime rebinding, and `capture-requested` event emission.
- `apps/desktop/src-tauri/src/commands/hotkey.rs` -- Tauri IPC commands: `get_hotkeys`, `set_hotkey`, `clear_hotkey`.
- `apps/desktop/src-tauri/src/commands/mod.rs` -- Register hotkey IPC commands.
- `apps/desktop/src-tauri/src/state.rs` -- Update `AppState` to hold `Arc<Mutex<DesktopHotkeyRegistrar>>` or hotkey state manager tracking active registrations and startup failures.
- `apps/desktop/src-tauri/src/lib.rs` & `apps/desktop/src-tauri/src/main.rs` -- Initialize `tauri-plugin-global-shortcut`, run startup hotkey registration, record any startup failure, handle hotkey press events (`capture-requested` and open window), and register IPC commands.
- `apps/desktop/src/types/settings.ts` -- TypeScript interfaces for `HotkeyAction`, `HotkeyStatus`, and `HotkeySettingsDto`.
- `apps/desktop/src/services/settings.ts` -- Frontend client wrapper invoking `get_hotkeys` and `set_hotkey`.
- `apps/desktop/src/components/HotkeySection.tsx` -- React component for Hotkey management: action rows, shortcut inputs, clear button, collision validation feedback, and startup failure alert banner.
- `apps/desktop/src/App.tsx` -- Mount `HotkeySection` within the Settings screen (`/settings`).
- `apps/desktop/src/test/shell.test.tsx` -- Integration tests for Hotkey settings UI, shortcut change, conflict feedback, and startup warning display.

## Tasks & Acceptance

**Execution:**
- `crates/snapdown-core/src/domain/setting.rs` -- Add `HotkeyAction` enum, default shortcut constants, and action-to-key mappings -- Provide domain model for hotkey bindings without I/O.
- `apps/desktop/src-tauri/Cargo.toml` -- Add `tauri-plugin-global-shortcut` dependency -- Enable global shortcut registration in Tauri shell.
- `apps/desktop/src-tauri/src/hotkey/mod.rs` -- Implement hotkey registrar with OS registration, conflict detection, immediate rebinding, and event dispatch -- Implement `LC-009 hotkey-registrar`.
- `apps/desktop/src-tauri/src/commands/hotkey.rs` -- Implement `get_hotkeys` and `set_hotkey` Tauri IPC commands -- Expose hotkey operations to frontend webview.
- `apps/desktop/src-tauri/src/lib.rs` -- Wire plugin, startup registration, hotkey event dispatchers, and state -- Hook global shortcut events into Tauri lifecycle.
- `apps/desktop/src/types/settings.ts` & `apps/desktop/src/services/settings.ts` -- Add hotkey data types and IPC invoke methods -- Type-safe frontend communication.
- `apps/desktop/src/components/HotkeySection.tsx` -- Build Hotkey settings UI component using design tokens -- Deliver UC-15 user interface.
- `apps/desktop/src/App.tsx` -- Integrate `HotkeySection` into Settings view -- Complete Settings screen section.
- `apps/desktop/src-tauri/tests/test_hotkeys.rs` (or unit tests in `src/hotkey/mod.rs`) -- Write Rust unit/integration tests for collision refusal, OS conflict refusal, clearing, dynamic rebinding, and startup failure reporting -- Satisfy named test suite requirements.
- `apps/desktop/src/test/shell.test.tsx` -- Add frontend test cases for hotkey editing, collision error messages, and startup warning presentation -- Prevent frontend regressions.

**Acceptance Criteria:**
- Given default settings, when reading hotkeys, then `Capture` is bound to `CommandOrControl+Shift+S` and `OpenEditor` is bound to `CommandOrControl+Shift+E`.
- Given an attempt to bind `OpenEditor` to the same shortcut as `Capture`, then the operation is refused with a validation error and neither OS nor database state is corrupted (BR-27).
- Given a shortcut combination already held by another OS application, when attempting to bind it, then the operation is refused, an honest conflict error naming the conflict is returned, and previous bindings remain intact (BR-26).
- Given an active hotkey, when clicking "Clear", then the shortcut is unregistered from the OS, the action is disabled, and no shortcut fires.
- Given a bound shortcut A, when changing to valid shortcut B, then shortcut A immediately stops firing and shortcut B becomes active without requiring an application restart.
- Given a shortcut registration failure during application startup, then Snapdown records the failure and the Settings screen displays a clear warning line naming the inactive hotkey (BR-26).
- Given the active Capture hotkey, when pressed, then a `capture-requested` event is raised and nothing crashes or hangs.
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
  - `[medium]` `[patch]` Added `get_shortcut` method to `HotkeyRegistrar` trait in `snapdown-core` and normalized cleared hotkey value handling to prevent reverting cleared actions back to shipped defaults.
  - `[medium]` `[patch]` Fixed React UI Button props in `HotkeySection.tsx` and ensured input error feedback displays properly when conflicting hotkey bindings are rejected.

## Design Notes

### Shipped Hotkey Defaults
- Capture Region: `CommandOrControl+Shift+S` (standard intuitive mnemonic for Snapdown screenshot capture).
- Open Editor: `CommandOrControl+Shift+E` (standard mnemonic for opening Editor/Settings).

### Honest Conflict Reporting (BR-26)
When calling `tauri-plugin-global-shortcut` or the underlying OS `RegisterHotKey`:
1. Check intra-app collision: if `action != other_action && shortcut == other_shortcut`, return `Err("Two actions cannot share the same hotkey combination")`.
2. Call registrar to register new shortcut with OS.
3. If OS fails (e.g. `AlreadyInUse` or OS error 1409), refuse the binding, keep the store and previous OS registration untouched, and return an explicit error: `"Hotkey combination '<shortcut>' is already in use by another application or the operating system."`
4. If OS succeeds, unregister the old shortcut and save the new shortcut in `SqliteSettingsStore`.

### Capture Event Emission
When `HotkeyAction::Capture` is triggered by the OS shortcut listener, call:
```rust
let _ = app_handle.emit("capture-requested", ());
```
In W1, no listeners subscribe to this event.

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: All Rust workspace files formatted cleanly
- `cargo clippy --workspace --all-targets -- -D warnings` -- expected: Clippy clean with no warnings
- `cargo test --workspace` -- expected: All unit and integration tests pass, including:
  - `cargo::a_combination_held_elsewhere_is_refused_at_binding`
  - `cargo::two_actions_cannot_share_one_combination`
  - `cargo::a_cleared_hotkey_disables_its_action`
  - `cargo::rebinding_takes_effect_without_a_restart`
  - `cargo::a_failed_startup_registration_is_reported_not_swallowed`
- `npm --prefix web/ui run typecheck` -- expected: Shared UI package clean
- `npm --prefix web/ui run lint` -- expected: Shared UI linter clean
- `npm --prefix web/ui run test` -- expected: Shared UI tests pass
- `npm --prefix apps/desktop run typecheck` -- expected: Desktop frontend clean
- `npm --prefix apps/desktop run lint` -- expected: Desktop frontend linter clean
- `npm --prefix apps/desktop run test` -- expected: Desktop frontend tests pass
- `uv run .constitution/method/scripts/validate.py --check` -- expected: Corpus validator passes baseline comparison

## Auto Run Result

### Implemented Change
- Implemented global hotkey domain model (`HotkeyAction`) with shipped defaults (`CommandOrControl+Shift+S` and `CommandOrControl+Shift+E`) in `snapdown-core`.
- Built `DesktopHotkeyRegistrar` in `apps/desktop/src-tauri` using `tauri-plugin-global-shortcut` v2 with strict collision validation (BR-27), honest OS conflict error reporting (BR-26), atomic dynamic rebinding without app restart, and startup failure tracking.
- Implemented Tauri IPC commands (`get_hotkeys`, `set_hotkey`, `clear_hotkey`) and wired `capture-requested` event emission upon Capture trigger and window focus upon Open Editor trigger.
- Added `HotkeySection` in the desktop Settings screen (`/settings`) using shared UI base elements and tokens, with inline validation error displays and startup warning banner presentation.
- Added comprehensive unit and integration test suites in Rust and React/Vitest covering all 9 I/O & edge-case matrix rows.

### Files Changed
- `Cargo.toml`: Added `tauri-plugin-global-shortcut = "2.0"` to workspace dependencies.
- `Cargo.lock`: Resolved lockfile dependencies for global shortcut plugins.
- `crates/snapdown-core/src/domain/setting.rs`: Added `HotkeyAction` enum, default constants, and action-to-key conversion methods.
- `crates/snapdown-core/src/ports/mod.rs`: Extended `HotkeyRegistrar` trait with `get_shortcut`.
- `apps/desktop/src-tauri/Cargo.toml`: Added `tauri-plugin-global-shortcut` dependency.
- `apps/desktop/src-tauri/src/hotkey/mod.rs`: Implemented `DesktopHotkeyRegistrar` adapter with OS registration, conflict detection, startup failure tracking, and unit test suite.
- `apps/desktop/src-tauri/src/commands/hotkey.rs`: Implemented `get_hotkeys`, `set_hotkey`, and `clear_hotkey` Tauri IPC commands.
- `apps/desktop/src-tauri/src/commands/mod.rs` & `apps/desktop/src-tauri/src/commands/settings.rs`: Exported hotkey module and updated state initializers.
- `apps/desktop/src-tauri/src/state.rs`: Added `hotkey_registrar` to `AppState`.
- `apps/desktop/src-tauri/src/lib.rs`: Integrated `tauri-plugin-global-shortcut`, configured startup initialization, and added event listener dispatching `capture-requested` or window focus.
- `apps/desktop/src/types/settings.ts`: Added `HotkeyAction`, `HotkeyItem`, and `HotkeySettingsDto` interfaces.
- `apps/desktop/src/services/settings.ts`: Added `getHotkeys`, `setHotkey`, and `clearHotkey` API client methods.
- `apps/desktop/src/components/HotkeySection.tsx`: Created Hotkey settings UI component with conflict feedback and startup warning banners.
- `apps/desktop/src/App.tsx`: Mounted `HotkeySection` into Settings view and wired state handlers.
- `apps/desktop/src/test/shell.test.tsx`: Added integration tests for hotkey section rendering, updating, clearing, collision feedback, and startup warning display.
- `_bmad-output/specs/w1-settings/stories/W1-S4-hotkey-binding-os-registration-and-honest-conflict-reporting.md`: Updated story spec to `status: done`.

### Review Findings Breakdown
- Patches applied: 2 (medium severity: trait extension for shortcut retrieval and UI button prop alignment).
- Items deferred: 0.
- Items rejected: 0.

### Follow-up Review Recommendation
- Recommended: `true` (Score: 6, from 2 medium patches).

### Verification
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS (all 13 desktop tests, 4 core tests, 9 store tests passing).
- `npm --prefix web/ui run typecheck`: PASS.
- `npm --prefix web/ui run lint`: PASS.
- `npm --prefix web/ui run test`: PASS (18 passing).
- `npm --prefix apps/desktop run typecheck`: PASS.
- `npm --prefix apps/desktop run lint`: PASS.
- `npm --prefix apps/desktop run test`: PASS (12 passing).
- `npm --prefix apps/desktop run build`: PASS (Vite build successful).
- `uv run .constitution/method/scripts/validate.py --check`: PASS (no regressions against baseline findings).

### Residual Risks
- None. Global hotkey registrations operate under standard user permissions without administrator rights (NFR-7, OQ-5). Conflict reporting honestly detects system or external application clashes (BR-26).
