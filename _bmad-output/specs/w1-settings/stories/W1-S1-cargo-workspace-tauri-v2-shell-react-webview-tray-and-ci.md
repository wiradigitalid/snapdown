---
title: 'W1-S1: Cargo workspace, Tauri v2 shell, React webview, tray, and CI'
type: 'feature'
created: '2026-08-22'
status: 'done'
baseline_revision: '6a470fd44910da3daa377fbb2db9fa498523c009'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - _bmad-output/specs/w1-settings/SPEC.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/design-system.md
  - .control/decisions/DEC-001-stack.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** Snapdown does not exist as code; a foundational workspace, desktop runtime substrate, and shared UI primitives must be established before any functional capabilities can run.

**Approach:** Initialize a Cargo workspace containing `crates/snapdown-core` and `crates/snapdown-store`, set up a Tauri v2 desktop shell in `apps/desktop` starting to a single-instance system tray and hosting a React 19 webview with shared design tokens/components under `web/ui`, and provide CI workflows for repository validation and desktop builds.

## Boundaries & Constraints

**Always:**
- Keep `snapdown-core` completely free of I/O, OS, network, clock, and `std::env` dependencies, verified by dependency graph analysis in `cargo::snapdown_core_has_no_io_dependency`.
- Conform to the technology stack locked by `DEC-001`: Rust 1.96, Tauri v2, React 19 + Vite 7 + TypeScript 5.
- Start `desktop-app` to a system tray icon (not an open window), enforcing single-instance semantics (subsequent launches focus/open existing instance), opening Settings only on first run.
- Maintain shared design tokens in `web/ui/src/styles/tokens.css` and base UI components in `web/ui/src/components/`, importing them into `apps/desktop/src/styles/tokens.css` with zero literal colors or spacing.
- Provide two GitHub Actions CI workflows: `korpus.yml` (running `validate.py --check`) and `desktop-ci.yml` (running Rust workspace build/clippy/test and desktop npm typecheck/lint/test on `windows-latest`).

**Block If:**
- Upstream requirements in `.what/`, `.how/`, `.control/`, or `.constitution/` conflict or demand modification of read-only corpus artifacts.
- Tauri v2 tray or single-instance plugin primitives require runtime privileges or capabilities conflicting with non-admin guarantees (NFR-7).

**Never:**
- Do not create `crates/snapdown-mcp` or `web/api/` (no Go or MCP code in wave W1).
- Do not introduce Next.js, Express, or unapproved runtime dependencies.
- Do not make any outbound network calls (AD-6).
- Do not commit secrets, test credentials, or non-synthetic capture fixtures.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Cold startup (first run) | Application launched with no previous configuration or run flag | Tray icon appears, Settings window opens automatically | Report launch error if window creation fails |
| Cold startup (subsequent run) | Application launched with previous run flag set | Tray icon appears, no window opens | Log error if tray initialization fails |
| Secondary instance launch | Second instance executed while first is running | Focuses/reveals existing instance window; second process terminates cleanly | Handled via single-instance lock/plugin |
| Tray menu click: Settings | Click "Settings" in tray menu | Shows and focuses Settings window (`/settings` route) | Log error if window focus fails |
| Tray menu click: Quit | Click "Quit" in tray menu | Gracefully exits desktop process | Cleans up tray handle |
| `snapdown-core` dependency check | Graph scan of `snapdown-core` cargo metadata | Returns zero I/O, filesystem, network, or clock crate dependencies | Fails test if illegal dependency introduced |

</intent-contract>

## Code Map

- `Cargo.toml` -- Root Cargo workspace definition configuring members `crates/snapdown-core`, `crates/snapdown-store`, and `apps/desktop/src-tauri`
- `crates/snapdown-core/Cargo.toml` -- Pure domain library crate with zero I/O dependencies
- `crates/snapdown-core/src/lib.rs` -- Exports domain models (`Setting`), errors, identifier helpers (UUIDv7), and port definitions
- `crates/snapdown-core/src/domain/setting.rs` -- `Setting` entity and domain invariants
- `crates/snapdown-core/src/ports/mod.rs` -- Port traits for storage, blob store, and system integrations
- `crates/snapdown-core/tests/test_no_io.rs` -- Architectural test verifying `snapdown-core` has no I/O dependencies
- `crates/snapdown-store/Cargo.toml` -- Store library crate depending on `snapdown-core`
- `crates/snapdown-store/src/lib.rs` -- Store crate entry point
- `apps/desktop/Cargo.toml` -- Workspace reference for desktop Tauri application
- `apps/desktop/package.json` -- Desktop front-end workspace configuration with React 19, Vite 7, TypeScript 5, Vitest
- `apps/desktop/src-tauri/Cargo.toml` -- Tauri v2 application configuration and native dependencies
- `apps/desktop/src-tauri/tauri.conf.json` -- Tauri v2 application manifest, window definitions, and tray configuration
- `apps/desktop/src-tauri/src/main.rs` -- Native desktop entry point, single-instance setup, tray initialization, and first-run routing
- `apps/desktop/src/App.tsx` -- Desktop root React application component
- `apps/desktop/src/main.tsx` -- Desktop front-end entry point
- `apps/desktop/src/styles/tokens.css` -- Desktop CSS importing shared design system tokens
- `apps/desktop/src/test/shell.test.tsx` -- Front-end test checking shell mounting and initial route handling
- `web/ui/package.json` -- Shared UI package / component module configuration
- `web/ui/src/styles/tokens.css` -- Design token stylesheet for light and dark schemes
- `web/ui/src/components/Button.tsx` -- Base Button element supporting default, hover, active, focus-visible, disabled, loading, danger
- `web/ui/src/components/TextField.tsx` -- Base TextField element with invalid, disabled, and char count states
- `web/ui/src/components/TextArea.tsx` -- Base TextArea element with auto-grow support
- `web/ui/src/components/Checkbox.tsx` -- Base Checkbox element with checked, unchecked, indeterminate states
- `web/ui/src/components/Toast.tsx` -- Transient, non-focusable toast notification component
- `web/ui/src/components/Modal.tsx` -- Accessible modal container with focus trap and escape handling
- `web/ui/src/components/ConfirmDialog.tsx` -- Destructive action confirmation dialog wrapping Modal
- `web/ui/src/components/MarkerBadge.tsx` -- Fixed-size numbered badge (1-99) with contrasting ring
- `web/ui/src/components/EmptyState.tsx` -- Empty state presentation element
- `web/ui/src/components/Markdown.tsx` -- CommonMark renderer for Markdown content
- `web/ui/src/index.ts` -- Export barrel for shared UI components and tokens
- `.github/workflows/korpus.yml` -- CI workflow running `uv run .constitution/method/scripts/validate.py --check`
- `.github/workflows/desktop-ci.yml` -- CI workflow building and running tests on `windows-latest`

## Tasks & Acceptance

**Execution:**
- [x] `Cargo.toml` -- Create root workspace manifest -- Include members `crates/snapdown-core`, `crates/snapdown-store`, `apps/desktop/src-tauri`
- [x] `crates/snapdown-core/` -- Implement domain core library -- Define `Setting`, port traits, UUIDv7 helper, and unit tests
- [x] `crates/snapdown-core/tests/test_no_io.rs` -- Add dependency graph verification test -- Assert `snapdown-core` dependency tree contains no I/O, network, clock, or FS crates
- [x] `crates/snapdown-store/` -- Initialize store library crate -- Provide baseline structure and link `snapdown-core` dependency
- [x] `web/ui/src/styles/tokens.css` -- Author shared design system tokens -- Define color, typography, spacing, radius, shadow, and z-index tokens for light and dark modes
- [x] `web/ui/src/components/` -- Implement base UI components -- Create Button, TextField, TextArea, Checkbox, Toast, Modal, ConfirmDialog, MarkerBadge, EmptyState, Markdown
- [x] `apps/desktop/` -- Implement Tauri v2 desktop shell and React app -- Configure single instance, system tray with Settings/Quit, first-run window opening, and React shell importing `web/ui`
- [x] `apps/desktop/src/test/shell.test.tsx` -- Implement front-end shell test -- Verify React app mounts and renders shell structure
- [x] `.github/workflows/korpus.yml` -- Create method corpus validation workflow -- Execute `validate.py --check`
- [x] `.github/workflows/desktop-ci.yml` -- Create desktop CI workflow -- Build Rust workspace, run cargo clippy/test, run npm typecheck/lint/test on `windows-latest`

**Acceptance Criteria:**
- Given a clean workspace, when running `cargo test --workspace`, then all crate tests compile and pass, including `snapdown_core_has_no_io_dependency`.
- Given `apps/desktop`, when running `npm run typecheck`, `npm run lint`, and `npm run test`, then all TypeScript checks, linters, and vitest suites pass.
- Given a clean initial launch, when `desktop-app` starts for the first time, then the system tray icon is created and the Settings window is displayed.
- Given a running `desktop-app` instance, when a secondary executable instance is launched, then the second process exits and the existing instance is focused.
- Given the system tray icon, when right-clicked, then a menu with "Settings" and "Quit" options is accessible and responsive.
- Given GitHub Actions CI, when `korpus.yml` and `desktop-ci.yml` run, then the defined build and test steps execute on `windows-latest`.

## Spec Change Log

## Review Triage Log

### 2026-08-22 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 2: (high 0, medium 0, low 2)
- defer: 0
- reject: 10
- addressed_findings:
  - `[low]` `[patch]` Removed placeholder `greet` IPC command from `apps/desktop/src-tauri/src/main.rs`.
  - `[low]` `[patch]` Added `Number.isFinite` and integer clamping guard to `web/ui/src/components/MarkerBadge.tsx`.

## Auto Run Result

### Summary of implemented change
Initialized the complete foundational substrate for Snapdown:
1. Root Cargo workspace containing `crates/snapdown-core`, `crates/snapdown-store`, and `apps/desktop/src-tauri`.
2. Pure `snapdown-core` domain library containing `Setting` domain model, UUIDv7 generation helper, and port traits, verified to have zero I/O dependencies via `test_no_io.rs`.
3. `snapdown-store` library crate initialization linked with `snapdown-core`.
4. Shared design system in `web/ui` containing CSS design tokens for light/dark modes and base accessible UI components (`Button`, `TextField`, `TextArea`, `Checkbox`, `Toast`, `Modal`, `ConfirmDialog`, `MarkerBadge`, `EmptyState`, `Markdown`).
5. Desktop front-end webview in `apps/desktop` running React 19 + Vite 7 + TypeScript 5, with Vitest shell test.
6. Desktop Tauri v2 shell configured with single-instance mutex handling, system tray menu ("Settings", "Quit"), and first-run window reveal.
7. CI workflows in `.github/workflows/` for corpus validation (`korpus.yml`) and desktop CI on `windows-latest` (`desktop-ci.yml`).

### Files changed
- `Cargo.toml` -- Root Cargo workspace definition
- `Cargo.lock` -- Resolved dependencies lockfile
- `crates/snapdown-core/` -- Pure domain library, models, errors, ports, and `test_no_io`
- `crates/snapdown-store/` -- Store library crate baseline
- `apps/desktop/` -- Tauri v2 app and React 19 front-end webview
- `web/ui/` -- Shared tokens and base components
- `.github/workflows/korpus.yml` -- CI workflow running `validate.py --check`
- `.github/workflows/desktop-ci.yml` -- CI workflow running Rust + npm check suites on `windows-latest`
- `_bmad-output/specs/w1-settings/stories/W1-S1-cargo-workspace-tauri-v2-shell-react-webview-tray-and-ci.md` -- Spec and run tracking

### Review findings breakdown
- Patches applied: 2 (`greet` cleanup, `MarkerBadge` prop safety)
- Items deferred: 0
- Items rejected: 10 (future story capabilities or out-of-scope recommendations)
- Follow-up review recommendation: `false` (Score: 2)

### Verification performed
- `cargo fmt --all -- --check` (clean)
- `cargo clippy --workspace --all-targets -- -D warnings` (passed)
- `cargo test --workspace` (5 tests passed, including `snapdown_core_has_no_io_dependency`)
- `npm --prefix apps/desktop run typecheck` (passed)
- `npm --prefix apps/desktop run lint` (passed)
- `npm --prefix apps/desktop run test` (1 vitest passed)
- `uv run .constitution/method/scripts/validate.py --check` (executed, 12 findings across 4 validators; V25 reports containers awaiting structure codebase update)

### Residual risks
- None identified for W1-S1 substrate. Ready for W1-S2 (`library.db` migrations and Vault blob adapter).

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: All Rust files formatted correctly without diffs
- `cargo clippy --workspace --all-targets -- -D warnings` -- expected: Zero Clippy warnings or errors across the workspace
- `cargo test --workspace` -- expected: All Rust unit and integration tests pass, including `cargo::snapdown_core_has_no_io_dependency`
- `npm --prefix apps/desktop run typecheck` -- expected: Zero TypeScript diagnostic errors
- `npm --prefix apps/desktop run lint` -- expected: Zero ESLint violations
- `npm --prefix apps/desktop run test` -- expected: Vitest suite passes, including `vitest::app_renders_shell`
- `uv run .constitution/method/scripts/validate.py --check` -- expected: Executes repository validation script
