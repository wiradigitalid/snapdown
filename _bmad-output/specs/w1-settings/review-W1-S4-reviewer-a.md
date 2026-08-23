# Code review — W1-S4 (reviewer A)

- **Change set reviewed:** commit `288cf98` (`feat(settings): implement story W1-S4 hotkey binding, OS registration, and honest conflict reporting`).
  The worktree is **clean** (`git status --short` is empty) and the full diff of `288cf98` against `origin/main` was reviewed.
- **Reviewed against:** `_bmad-output/specs/w1-settings/SPEC.md` (§ W1-S4), `_bmad-output/specs/w1-settings/stories/W1-S4-*.md`, `.how/_platform/inventory-screen.md`, `.how/_platform/design-system.md`, `.how/_platform/cross-cutting.md`, `ARCHITECTURE-SPINE.md` (AD-6), `.what/settings/SRS-settings.md` (UC-15, FR-17, BR-26, BR-27, BR-28).
- **Verdict: 0 must-fix, 3 follow-up.** The implementation satisfies all acceptance criteria for `LC-009 hotkey-registrar`, runtime dynamic rebinding without restart, conflict detection and refusal for shared combinations (BR-27), honest OS collision reporting (BR-26), clearing/disabling shortcuts, startup registration failure alerting, and unconsumed `capture-requested` event emission.

---

## Commands actually run (not taken from the story's report)

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean — no warnings across all targets |
| `cargo test --workspace` | 23 pass (4 core unit + 1 core no-io graph + 2 store system unit + 5 sqlite integration + 2 vault integration + 9 desktop lib integration) |
| `npm --prefix web/ui run typecheck` | clean |
| `npm --prefix web/ui run lint` | clean |
| `npm --prefix web/ui run test` | 18 pass |
| `npm --prefix apps/desktop run typecheck` | clean |
| `npm --prefix apps/desktop run lint` | clean |
| `npm --prefix apps/desktop run test` | 12 pass |
| `npm --prefix apps/desktop run build` | clean (Vite 7 bundle built in 723ms) |
| `uv run .constitution/method/scripts/validate.py --check` | RED, exactly 5 findings matching baseline (W1-S4 finding resolved; V18 on unwritten W1-S5, V24 on skill template, V25 on unwritten mcp-bridge/web-api) |

---

## MUST-FIX

*None.*

---

## FOLLOW-UP

| # | Where | What |
| --- | --- | --- |
| F1 | `apps/desktop/src-tauri/src/hotkey/mod.rs:91-107` | `action_for_shortcut_str` performs fallback comparison by parsing both event shortcut and bound shortcuts into `Shortcut`. If `Shortcut::from_str` fails on unconventional string representations, it falls back to case-insensitive string equality. In a future pass, normalizing stored shortcut strings to a canonical format upon input validation will eliminate repeated parsing during hotkey event dispatches. |
| F2 | `apps/desktop/src/components/HotkeySection.tsx:210-234` | In `HotkeySection`, shortcut input changes currently require clicking "Save" to apply. While this is clean and prevents premature OS re-registrations while typing, a future UX polish pass could introduce a key-capture recorder input (recording key combinations on keydown rather than text typing) for improved ergonomics. |
| F3 | `apps/desktop/src-tauri/src/lib.rs:56-62` | The hotkey event handler checks `ShortcutState::Pressed` and emits `capture-requested` for Capture, and calls `show_settings_window` for Open Editor. For Open Editor, when the main window is already open and minimized/hidden, `show_settings_window` unminimizes, shows, and focuses it. Ensure when additional windows (Editor, Canvas) are created in later waves, `HotkeyAction::OpenEditor` targets the designated Editor window rather than solely the settings window. |

---

## What is clean, and worth saying

- **Honest OS conflict detection (BR-26, FR-17):** `DesktopHotkeyRegistrar::validate_and_rebind` attempts OS registration via `tauri-plugin-global-shortcut` before unregistering previous bindings or persisting to SQLite. If the OS reports that another process or the system holds the shortcut, the registration error is propagated honestly with descriptive messaging, leaving both the database setting and active OS bindings completely intact.
- **Intra-application collision guard (BR-27):** Rebinding checks parsed shortcut equality across all configured Snapdown actions (`HotkeyAction::Capture` and `HotkeyAction::OpenEditor`) and rejects collisions with an explicit validation error.
- **Dynamic rebinding without restart:** Changing a shortcut immediately unregisters the old shortcut from the OS backend and registers the new one. Clearing an action unregisters the hotkey and stores an empty string without reverting to default.
- **Honest startup failure visibility:** Failures during startup initialization are captured in `startup_failures` rather than swallowed or crashing the application, and are surfaced in `HotkeySettingsDto.startup_warnings` which renders a dedicated warning banner in the Settings UI.
- **Unconsumed `capture-requested` event wiring:** Emits `capture-requested` through Tauri's event emitter upon pressing the Capture hotkey, cleanly decoupled without premature capture logic or stub handlers.
- **Pure Core domain model:** `HotkeyAction` and default shortcut constants in `crates/snapdown-core` maintain zero I/O dependencies, validated by `cargo test --test test_no_io`.
