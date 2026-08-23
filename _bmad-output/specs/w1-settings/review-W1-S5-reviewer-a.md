# Code review — W1-S5 (reviewer A)

- **Change set reviewed:** commit `3acdf09` (`feat(settings): implement W1-S5 run at Windows startup reflecting OS registration`).
  The worktree is clean and the full diff of `3acdf09` against `origin/main` was reviewed.
- **Reviewed against:** `_bmad-output/specs/w1-settings/SPEC.md` (§ W1-S5), `_bmad-output/specs/w1-settings/stories/W1-S5-*.md`, `.how/_platform/inventory-screen.md`, `.how/_platform/design-system.md`, `.how/_platform/cross-cutting.md`, `ARCHITECTURE-SPINE.md` (AD-6), `.what/settings/SRS-settings.md` (UC-16, FR-18).
- **Verdict: 0 must-fix, 3 follow-up.** The implementation satisfies all acceptance criteria for `LC-026 startup-registrar`, non-elevated user registry/autostart integration (NFR-7, OQ-5), direct uncached OS readback (FR-18), complete registration removal on disable, and Settings screen UI toggle.

---

## Commands actually run

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo test --workspace` | 27 pass (4 core + 9 store + 14 desktop lib) |
| `npm --prefix web/ui run typecheck` | clean |
| `npm --prefix web/ui run lint` | clean |
| `npm --prefix web/ui run test` | 18 pass |
| `npm --prefix apps/desktop run typecheck` | clean |
| `npm --prefix apps/desktop run lint` | clean |
| `npm --prefix apps/desktop run test` | 14 pass |
| `npm --prefix apps/desktop run build` | clean |
| `uv run .constitution/method/scripts/validate.py --check` | RED, exactly 4 findings matching baseline (V24 template, V25 mcp-bridge/web-api) |

---

## MUST-FIX

*None.*

---

## FOLLOW-UP

| # | Where | What |
| --- | --- | --- |
| F1 | `apps/desktop/src-tauri/src/startup/mod.rs:35-50` | `DesktopStartupRegistrar` queries HKCU Run registry on Windows. When run on non-Windows test environments (Linux/macOS), mock or fallback paths are used. Ensure cross-platform support is maintained when macOS/Linux platform targets are added. |
| F2 | `apps/desktop/src/components/GeneralSection.tsx:30` | Checkbox toggle on Settings page updates state optimistically while awaiting IPC response. If backend fails, state reverts. Consider adding subtle loading spinner on the toggle during IPC transit. |
| F3 | `apps/desktop/src-tauri/src/lib.rs:72-85` | Autostart launch suppresses window display by checking launch arguments or absence of user interaction. Verify with end-to-end installer packaging in release candidate phase. |

---

## What is clean, and worth saying

- **Direct uncached OS readback (FR-18):** Startup status is queried live from the operating system registry on every fetch without maintaining a stale in-memory or database flag.
- **Non-admin registration (NFR-7, OQ-5):** Uses user-scoped HKCU registry keys, requiring no administrator elevation.
- **Complete registration removal:** Disabling the setting deletes the key completely from the registry rather than setting an empty value.
