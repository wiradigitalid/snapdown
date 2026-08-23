---
id: W6-S6
title: 'W6-S6: Hotkey rows — chip states, honest conflicts, and readable badges'
type: 'feature'
wave: W6
status: ready-for-dev
created: '2026-08-24'
dependencies:
  - W6-S5
files:
  - web/ui/src/components/HotkeyChip.tsx
  - web/ui/src/styles/components.css
  - apps/desktop/src/components/HotkeySection.tsx
  - apps/desktop/src/components/SettingsView.tsx
  - apps/desktop/src-tauri/src/hotkey/mod.rs
  - apps/desktop/src-tauri/src/commands/hotkey.rs
  - apps/desktop/src/types/settings.ts
  - apps/desktop/src/services/settings.ts
  - web/ui/src/test/components.test.tsx
  - apps/desktop/src/test/hotkey.test.tsx
  - apps/desktop/src/test/shell.test.tsx
  - apps/desktop/src-tauri/tests/test_startup.rs
context:
  - _bmad-output/specs/w6-desktop-experience/SPEC.md
  - _bmad-output/specs/w6-desktop-experience/stories.yaml
  - _bmad-output/specs/w6-desktop-experience/dispatch-briefs/W6-S6-step1-plan.md
  - .how/settings/01-ux/DESIGN.md
  - .what/settings/04-usecases/UC-15-change-the-keys-that-set-snapdown-off.md
  - .what/settings/03-domain/state-machines.md
  - .what/settings/03-domain/rules-settings.md
  - .what/settings/04-usecases/EXPERIENCE.md
  - .how/settings/SDD-settings.md
  - .how/settings/02-contracts/contract-inventory.md
  - .what/business-rules.md
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .how/_platform/design-system.md
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:**
1. **Missing Shipped Defaults (`AUDIT-4`, `FR-1`, `FR-17`, `UC-15`, `BR-28`):**
   - Photographic audit (`shot-4-settings.png` on 2026-08-24) revealed that on a clean profile, the `Capture Region` hotkey row reads `Disabled` and its chip shows `Click to record` (or unbound) with no shortcut bound.
   - Consequently, the core capture path (`FR-1`, `UC-1`) — the primary reason the application exists — is completely unreachable by keyboard until the Reviewer navigates to Settings and manually records a key combination.
   - The shipped defaults (`CommandOrControl+Shift+S` for Capture and `CommandOrControl+Shift+E` for Open Editor, defined in `crates/snapdown-core/src/domain/setting.rs`) must be present and active upon initial launch, stored reliably, and accurately reported by `get_hotkeys`.
2. **Chip States & Visual Contract (`DESIGN.md` § Hotkeys group):**
   - `.how/settings/01-ux/DESIGN.md` defines four distinct states for `HotkeyChip`:
     - **bound:** Rendered with `--color-surface-sunken`, `--font-mono`, `--radius-full`, and displaying the registered combination text.
     - **listening:** Rendered with `--color-info-bg` / `--color-info-text`, a `2px` `--color-accent` ring, reading "Press keys… Esc to cancel".
     - **unbound:** Rendered with dashed `--color-border-strong`, reading "Click to set".
     - **conflicted:** Rendered with `--color-warning-bg` / `--color-warning-text`, with the conflict description rendered on the line beneath.
   - The legacy implementation used `--radius-sm`, non-standard placeholder strings ("Press shortcut keys (ESC to cancel)..." / "Click to record"), and lacked clean CSS mapping for all 4 states.
3. **Focus Leakage During Recording (`UC-15`, `EXPERIENCE.md`):**
   - If a chip enters the `listening` state and the Reviewer switches focus away (via Tab, mouse click elsewhere, or window blur), a chip that continues listening will capture and swallow subsequent keystrokes intended for other inputs or OS tasks.
   - A listening chip MUST stop listening immediately when focus leaves it (`blur` event) and revert to its previous state.
4. **Distinguishing Conflict Failure Modes (`UC-15`, `BR-26`, `BR-27`, `BR-114`):**
   - Three distinct failure/inactive situations all superficially resemble *"the hotkey doesn't work"*, but have entirely different causes and remedies:
     - **Refused at Binding (Snapdown-Internal Conflict, `BR-27`):** When the Reviewer attempts to bind a shortcut already allocated to another Snapdown action (e.g. binding Open Editor to the same combination as Capture), it must be reported with internal wording (*"Another Snapdown action already uses this combination"*). The Reviewer can resolve this immediately in the same panel.
     - **Refused at Binding (OS / External Conflict, `BR-26`):** When the shortcut is held by Windows or another running application, it must be reported with external wording (*"This combination is already held by Windows or another application"*), prompting the Reviewer to pick a different key sequence.
     - **Startup Registration Failure (`BR-26`, `NFR-7`, `DESIGN.md`):** When a previously valid stored hotkey fails to register at startup because another process grabbed it, the row must display a warning badge and explanatory conflict message *before* the Reviewer interacts with the control. Crucially, per `BR-114`, this OS failure does NOT wipe or alter the stored Setting value.
     - **Disabled (`BR-113`):** When the Reviewer deliberately clicks `Clear`, the action is disabled. The row must display `Disabled` (on both the badge and the chip/row) rather than an empty box that looks broken or uninitialized.
5. **Readable Badges & Accessibility Floor (`EXPERIENCE.md`, `NFR-16`, `AD-10`):**
   - The status badge next to each hotkey row must carry explicit text words (**Active** or **Disabled**), never conveying status by colour alone.
   - Badges must strictly consume design tokens (`--color-success-bg` / `--color-success-text` for Active, `--color-neutral-bg` / `--color-neutral-text` for Disabled), completely avoiding hardcoded hex literals.
6. **Immediate Apply Without Restart (`BR-26`, `rules-settings.md`):**
   - Re-registration must take effect immediately without requiring an application restart.
   - Registration ordering: Register the new key combination with the OS backend *before* unregistering the existing one. If registration fails, the old combination remains active and registered.

**Approach:**
1. **Frontend `HotkeyChip` Component Refactor (`web/ui/src/components/HotkeyChip.tsx`, `web/ui/src/styles/components.css`):**
   - Update `HotkeyChip` to implement the four canonical states: `bound`, `listening`, `unbound`, `conflicted`.
   - Update strings: listening reads "Press keys… Esc to cancel"; unbound reads "Click to set".
   - Implement `onBlur` listener to automatically cancel listening and restore prior state whenever focus moves away.
   - Ensure `Escape` cancels recording without closing the parent window.
   - Update CSS in `components.css`: `.hotkey-chip` uses `--radius-full`, `--color-surface-sunken`, mono font; `[data-state="listening"]` uses `--color-info-bg`, `--color-info-text`, `2px solid var(--color-accent)`; `[data-state="unbound"]` uses dashed `1px solid var(--color-border-strong)`; `[data-state="conflicted"]` uses `--color-warning-bg`, `--color-warning-text`.
2. **Settings Hotkey Section (`apps/desktop/src/components/HotkeySection.tsx`):**
   - Render each hotkey row: Label, `HotkeyChip`, readable `Badge` (`Active` / `Disabled`), and inline conflict/warning messages.
   - Display a cleared hotkey as **Disabled** with the `Disabled` badge and unbound chip state.
   - If startup registration failed for an action (`startup_warnings` or `is_registered === false` while shortcut exists), render a warning badge and specific message under the control before any user interaction.
   - Display clear, differentiated error messages for Snapdown-internal conflicts vs. external OS conflicts.
3. **Backend Hotkey Registrar (`apps/desktop/src-tauri/src/hotkey/mod.rs`, `commands/hotkey.rs`):**
   - Guarantee first-run default initialization (`Capture -> CommandOrControl+Shift+S`, `OpenEditor -> CommandOrControl+Shift+E`) if settings are unset or uninitialized.
   - Ensure distinct error types / error strings returned from `validate_and_rebind`:
     - Internal conflict (`BR-27`): "Another Snapdown action already uses this combination"
     - External OS conflict (`BR-26`): "This combination is already held by Windows or another application"
   - Maintain strict registration ordering: call `backend.register_shortcut(new)` first; only upon success call `backend.unregister_shortcut(old)`. If new registration fails, old shortcut stays registered and active.
   - Ensure `get_hotkeys` DTO clearly surfaces `startup_warnings` and per-item `is_active` / `is_registered` flags.
4. **Comprehensive Test Suite (`waves.yaml` Named Tests):**
   - `vitest::a_listening_chip_stops_listening_when_focus_leaves_it`
   - `vitest::a_snapdown_internal_conflict_is_worded_differently_from_an_os_conflict`
   - `vitest::a_cleared_hotkey_reads_disabled_rather_than_empty`
   - `vitest::a_startup_registration_failure_carries_a_badge_before_the_reviewer_acts`
   - `vitest::the_active_and_disabled_badges_carry_a_word_not_only_a_colour`

## Boundaries & Constraints

**Always:**
- Shipped default hotkeys MUST be initialized on first run (`DEFAULT_HOTKEY_CAPTURE` = "CommandOrControl+Shift+S", `DEFAULT_HOTKEY_OPEN_EDITOR` = "CommandOrControl+Shift+E") and active upon initial start (`FR-1`, `FR-17`, `UC-15`).
- The four chip states (`bound`, `listening`, `unbound`, `conflicted`) MUST match `.how/settings/01-ux/DESIGN.md`.
- A listening chip MUST stop listening on blur / focus loss (`UC-15`, `EXPERIENCE.md`).
- Snapdown-internal conflicts MUST be worded differently from OS / external conflicts (`BR-26`, `BR-27`, `UC-15`).
- A cleared hotkey MUST render as `Disabled`, never as an empty or broken input (`BR-113`, `UC-15`).
- A startup registration failure MUST display a warning badge and error message under the control before the Reviewer acts (`DESIGN.md`, `BR-26`, `NFR-7`).
- A startup registration failure MUST NOT overwrite or delete the saved setting value in the database (`BR-114`).
- Status badges MUST carry explicit text words (**Active** / **Disabled**), not only colour (`EXPERIENCE.md`, `NFR-16`).
- All colours MUST come from `web/ui/src/styles/tokens.css` with zero colour literals (`AD-10`).
- Re-registration MUST take effect immediately without a restart (`BR-26`, `rules-settings.md`).
- Re-registration MUST register the new shortcut before unregistering the old one to avoid leaving an unregistered gap upon failure.

**Block If:**
- Do NOT add periodic background registration health check polling loops (explicit Non-goal in `SPEC.md`).
- Do NOT re-lay out the Settings surface frame (two columns packed by content height was established in `W6-S3`).
- Do NOT move `Agent Access` into the Settings panel (`Agent Access` is a primary surface on the navigation rail, `FR-28`, `BR-120`).

## User Scenarios & State Transitions

### State Machine: Hotkey Binding Lifecycle (`state-machines.md` § 3)

| Current State | Trigger / Action | Resulting State | UI & System Behaviour |
|---|---|---|---|
| Initial / Shipped Default | First application launch | `Bound` | Default shortcuts registered with OS backend; rows show combination text with `Active` badge |
| `Bound` | User clicks `HotkeyChip` | `Listening` | Chip gains `--color-info-bg`, 2px `--color-accent` ring, reads "Press keys… Esc to cancel" |
| `Listening` | User presses valid new combination | `Bound` (validating) | Registers new with OS backend; on success unregisters old, updates store, returns to `Bound` with `Active` badge |
| `Listening` | User presses Esc | `Bound` (restored) | Listening cancelled; previous combination restored; window does not close |
| `Listening` | Focus leaves chip (`blur`) | `Bound` (restored) | Listening stops immediately; previous combination restored without swallowing next keystrokes |
| `Listening` | Combination matches other Snapdown action | `Conflicted` / `Refused` | Refused; displays error: *"Another Snapdown action already uses this combination"*; old combination preserved |
| `Listening` | Combination held by another running OS app | `Conflicted` / `Refused` | Refused; displays error: *"This combination is already held by Windows or another application"*; old combination preserved |
| `Bound` | User clicks `Clear` button | `Disabled` | Unregisters shortcut from OS; stores empty string; row displays `Disabled` badge and unbound chip state |
| `Disabled` | User clicks `HotkeyChip` | `Listening` | Chip enters listening state to record a new shortcut |
| `Bound` | Startup registration fails (OS conflict) | `Unregistered` | Stored setting preserved (`BR-114`); row carries warning badge and conflict note under control before user acts |
| `Unregistered` | Subsequent startup succeeds | `Bound` | OS registers shortcut cleanly; warning badge disappears; `Active` badge restored |

</intent-contract>

## Code Map

- `web/ui/src/components/HotkeyChip.tsx` -- Hotkey recording chip component; supports 4 states (`bound`, `listening`, `unbound`, `conflicted`), focus loss (`onBlur`) handling, Escape cancellation, and standard text strings ("Press keys… Esc to cancel", "Click to set").
- `web/ui/src/styles/components.css` -- CSS styling for `.hotkey-chip` tokens (`--radius-full`, `--color-surface-sunken`, info state styling with `--color-accent` ring, dashed unbound styling, warning conflicted styling).
- `apps/desktop/src/components/HotkeySection.tsx` -- Settings Hotkey group; renders each action row (Capture Region, Open Workspace / Editor) with label, `HotkeyChip`, `Badge` (`Active` / `Disabled`), immediate Save/Clear handlers, startup failure indicators, and honest conflict error messages.
- `apps/desktop/src-tauri/src/hotkey/mod.rs` -- Backend hotkey registrar (`DesktopHotkeyRegistrar`); initializes first-run defaults (`DEFAULT_HOTKEY_CAPTURE`, `DEFAULT_HOTKEY_OPEN_EDITOR`), differentiates internal vs OS conflict errors, maintains register-before-unregister invariant, and tracks startup failures without overwriting store (`BR-114`).
- `apps/desktop/src-tauri/src/commands/hotkey.rs` -- Tauri commands (`get_hotkeys`, `set_hotkey`, `clear_hotkey`) serializing hotkey state, active/registered flags, and startup warnings.
- `apps/desktop/src/types/settings.ts` -- TypeScript interface definitions for hotkey actions, items, and settings DTO.
- `apps/desktop/src/services/settings.ts` -- Frontend service invoking Tauri hotkey IPC commands.
- `web/ui/src/test/components.test.tsx` -- Vitest unit tests for `HotkeyChip`: focus blur listening cancellation, Escape cancellation, key combo recording, and visual state attributes.
- `apps/desktop/src/test/hotkey.test.tsx` (and `apps/desktop/src/test/shell.test.tsx`) -- Vitest tests verifying internal vs OS conflict wording, cleared hotkey disabled display, startup warning badge before interaction, and text-carrying Active/Disabled badges.
- `apps/desktop/src-tauri/tests/test_startup.rs` (and `apps/desktop/src-tauri/src/hotkey/mod.rs` tests) -- Rust unit and integration tests verifying backend registration defaults, conflict rejection, and non-restart re-registration.

## Tasks & Acceptance

**Execution:**
- [ ] `web/ui/src/components/HotkeyChip.tsx` & `web/ui/src/styles/components.css` -- Update `HotkeyChip` component and CSS:
  - Add explicit state styling for `bound`, `listening`, `unbound`, `conflicted`.
  - Use `--radius-full`, `--color-surface-sunken`, mono font.
  - Update listening label to "Press keys… Esc to cancel" and unbound label to "Click to set".
  - Implement `handleBlur` to cancel listening on focus loss.
  - Ensure Escape key stops listening without bubbling to window dismiss.
- [ ] `apps/desktop/src-tauri/src/hotkey/mod.rs` -- Enhance `DesktopHotkeyRegistrar`:
  - Ensure first-run default shortcuts are seeded and registered if store value is missing.
  - Differentiate validation errors: return "Another Snapdown action already uses this combination" for `BR-27` internal conflict, and "This combination is already held by Windows or another application" for OS backend conflict (`BR-26`).
  - Maintain atomic registration order: register new shortcut before unregistering old shortcut.
  - Track startup registration failures in `startup_failures` map without mutating stored setting (`BR-114`).
- [ ] `apps/desktop/src-tauri/src/commands/hotkey.rs` -- Update `get_hotkeys` command to return complete `HotkeySettingsDto` with action items, `is_active`, `is_registered`, and `startup_warnings`.
- [ ] `apps/desktop/src/components/HotkeySection.tsx` -- Update HotkeySection component:
  - Render readable status badges with explicit text: `<Badge variant="success">Active</Badge>` when active, `<Badge variant="neutral">Disabled</Badge>` when disabled.
  - If startup failure exists for an action, display a warning badge and error message under that specific control before user interaction (`DESIGN.md`).
  - When cleared, display row state as `Disabled` and chip as unbound.
  - Display distinct conflict error messages underneath the row for internal vs external conflicts.
- [ ] `web/ui/src/test/components.test.tsx` -- Add/update Vitest unit tests:
  - `a_listening_chip_stops_listening_when_focus_leaves_it`: blur event triggers cancellation of listening state.
  - `the_active_and_disabled_badges_carry_a_word_not_only_a_colour`: badge renders text "Active" and "Disabled" with proper CSS classes.
- [ ] `apps/desktop/src/test/hotkey.test.tsx` (and `shell.test.tsx`) -- Add Vitest tests for the 5 named requirements:
  - `a_listening_chip_stops_listening_when_focus_leaves_it`
  - `a_snapdown_internal_conflict_is_worded_differently_from_an_os_conflict`
  - `a_cleared_hotkey_reads_disabled_rather_than_empty`
  - `a_startup_registration_failure_carries_a_badge_before_the_reviewer_acts`
  - `the_active_and_disabled_badges_carry_a_word_not_only_a_colour`
- [ ] `apps/desktop/src-tauri/src/hotkey/mod.rs` & `tests/test_startup.rs` -- Verify Rust backend tests for hotkey registration, startup failure retention, and non-restart re-registration.

**Acceptance Criteria:**
- Given a fresh application profile, when Settings opens, both `Capture Region` (`CommandOrControl+Shift+S`) and `Open Workspace / Editor` (`CommandOrControl+Shift+E`) are bound, active, and display `Active` badges (`FR-1`, `FR-17`, `UC-15`).
- Given a `HotkeyChip` in listening state, when focus leaves the chip (blur), it immediately ceases listening and reverts to its prior state without swallowing further keystrokes (`a_listening_chip_stops_listening_when_focus_leaves_it`, `UC-15`).
- Given the Reviewer attempting to bind a shortcut already allocated to another Snapdown action, the interface displays an internal conflict message distinct from an OS conflict message (`a_snapdown_internal_conflict_is_worded_differently_from_an_os_conflict`, `BR-27`, `UC-15`).
- Given a cleared hotkey, the row displays `Disabled` on its badge and an unbound chip ("Click to set"), clearly distinguishing a disabled action from an empty or broken field (`a_cleared_hotkey_reads_disabled_rather_than_empty`, `BR-113`).
- Given a hotkey that failed to register at startup, the row displays a warning badge and error message under the control before the Reviewer interacts with it, while preserving the stored setting in the database (`a_startup_registration_failure_carries_a_badge_before_the_reviewer_acts`, `BR-26`, `BR-114`, `NFR-7`).
- Given status badges for hotkeys, they display explicit text words ("Active" / "Disabled") and use semantic token classes (`badge-success`, `badge-neutral`), fulfilling the accessibility floor (`the_active_and_disabled_badges_carry_a_word_not_only_a_colour`, `EXPERIENCE.md`, `NFR-16`).
- Given hotkey re-registration, changes take effect immediately without requiring an application restart (`BR-26`).
- Given all verification commands, TypeScript typechecks, lint checks, and test suites pass with zero errors, and zero colour literals exist outside `tokens.css` (`AD-10`).

## Verification

**Commands:**
- `npm --prefix web/ui run typecheck` -- expected: TypeScript compiles cleanly with zero errors
- `npm --prefix web/ui run lint` -- expected: ESLint passes with zero warnings/errors (no colour literals outside tokens.css)
- `npm --prefix web/ui run test` -- expected: All Vitest suites pass in `web/ui` (including HotkeyChip focus/blur tests and Badge text tests)
- `npm --prefix apps/desktop run typecheck` -- expected: Zero TypeScript errors in desktop app
- `npm --prefix apps/desktop run lint` -- expected: ESLint passes with zero warnings/errors in desktop app
- `npm --prefix apps/desktop run test` -- expected: All Vitest suites pass in `apps/desktop` (including hotkey tests for internal/OS conflicts, cleared state, startup failure badges, and active/disabled words)
- `npm --prefix apps/desktop run build` -- expected: Production Vite build succeeds
- `cargo fmt --all -- --check` -- expected: Rust formatting is clean
- `cargo clippy --workspace --all-targets -- -D warnings` -- expected: Clippy passes with zero warnings
- `cargo test --workspace` -- expected: All Rust unit and integration tests pass

## Spec Change Log

- 2026-08-24: Created W6-S6 story specification (Step 1 PLAN). Outlined four chip states, distinguishable conflict failure modes, startup warning indicators, accessibility text badges, focus leak prevention on blur, and test plan for the 5 required Vitest tests.
