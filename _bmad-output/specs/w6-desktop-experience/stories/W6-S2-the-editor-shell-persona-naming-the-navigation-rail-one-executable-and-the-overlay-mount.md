---
id: W6-S2
title: 'W6-S2: The editor shell — persona naming, the navigation rail, one executable, and the overlay mount'
type: 'feature'
wave: W6
status: done
created: '2026-08-23'
dependencies:
  - W6-S1
files:
  - apps/desktop/src/main.tsx
  - apps/desktop/src/App.tsx
  - apps/desktop/src/components/EditorShell.tsx
  - apps/desktop/src/test/mount.test.tsx
  - apps/desktop/src/test/editor_shell.test.tsx
  - apps/desktop/src/test/shell.test.tsx
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src-tauri/tests/test_executable_identity.rs
context:
  - _bmad-output/specs/w6-desktop-experience/SPEC.md
  - _bmad-output/specs/w6-desktop-experience/stories.yaml
  - _bmad-output/specs/w6-desktop-experience/dispatch-briefs/W6-S2-step1-plan.md
  - .how/settings/04-components/LC-028-editor-shell.md
  - .how/settings/01-ux/DESIGN.md
  - .what/settings/04-usecases/EXPERIENCE.md
  - .control/registry/defects.yaml
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/design-system.md
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** 
The application currently has two critical defects and architectural gaps:
1. **`BUG-4` (Critical):** `capture.rs:106` creates the overlay window pointing to `index.html?overlay=true`, but `apps/desktop/src/main.tsx` ignores `window.location.search` and unconditionally renders `<App />` (the Editor shell on Settings) instead of `<CaptureOverlay />`. Pressing the capture hotkey shows Settings rather than the capture scrim/crosshair/drag region, leaving `FR-1`, `FR-2`, `UC-1`, and `UC-2` entirely broken in the shipped desktop app.
2. **Missing `LC-028 editor-shell` Frame Architecture:** The Editor window frame is currently inline JSX inside `App.tsx` with a top tab bar that wastes ~64px of vertical height (causing Settings to exceed 1024×720 viewport, violating `FR-29`), distinguishes active tabs by color fill alone (violating `NFR-16`), lacks a pinned Capture action in chrome (`FR-28`), sets window title to `Snapdown` rather than `Snapdown Editor` (`DEC-003`, `FR-27`), and lacks build assertions proving exactly one desktop binary is produced (`AD-11`, `BR-121`).

**Approach:**
1. **Fix BUG-4 Root Routing:** In `apps/desktop/src/main.tsx` (or a dedicated Root component / router entry), parse `window.location.search` (checking `overlay=true`) and mount `<CaptureOverlay />` when active, or `<App />` (housing `<EditorShell />`) for standard window launches. Add automated unit tests verifying the root mount decision based on URL search query parameters.
2. **Implement `LC-028 EditorShell`:** Extract the window frame from `App.tsx` into a dedicated `<EditorShell />` component that:
   - Renders a 200px left navigation rail (`--color-surface` on `--color-bg`, border-right `1px solid var(--color-border)`).
   - Lists all four primary surfaces (`Findings`, `Bundles`, `Agent Access`, `Settings`) on every surface (`FR-28`, `BR-120`), keeping frozen surfaces (`agent-access`, `sharing`) reachable.
   - Highlights the active navigation item using multiple visual cues: background fill (`--color-accent`) **and** a prominent left edge bar (e.g. 4px solid indicator), fulfilling `NFR-16`.
   - Pins the Capture action button (`var(--color-accent)` with shortcut cue or icon) to the foot of the rail separated by a divider rule, triggering `triggerOverlay()` from anywhere.
   - Maintains zero inbound dependencies: does not read business/store state so navigation never fails to render (`FR-28`).
3. **Set Window Persona Title & Assert Single Binary:**
   - Update `tauri.conf.json` main window title to `Snapdown Editor` (`FR-27`, `BR-121`, `DEC-003`).
   - Add integration/crate test asserting Cargo workspace configuration produces exactly one desktop binary named `Snapdown` (`AD-11`, `BR-121`).

## Boundaries & Constraints

**Always:**
- Root mount decision MUST read `window.location.search` and route `overlay=true` to `<CaptureOverlay />` and default/empty to the Editor `<App />` shell.
- Automated tests MUST explicitly verify the URL-based root mount decision (`mount.test.tsx`) to guard against regressions.
- `LC-028` EditorShell navigation rail MUST be 200px wide, left-aligned, and span full viewport height.
- All four primary surfaces (`Findings`, `Bundles`, `Agent Access`, `Settings`) MUST be listed and clickable on every screen (`FR-28`, `BR-120`).
- The active navigation tab MUST use both fill color AND a left edge indicator bar (`NFR-16`).
- The Capture action button MUST be pinned to the bottom of the navigation rail separated by a border/rule.
- Main window title MUST be `Snapdown Editor` (`FR-27`, `DEC-003`, `BR-121`) and remain constant during navigation.
- Build assertions MUST verify `snapdown` package outputs exactly one desktop binary `Snapdown` (`AD-11`, `BR-121`).
- All styling MUST use design tokens from `tokens.css` without color literals (`AD-10`).

**Block If:**
- Routing requires multi-page HTML bundles that violate single-executable/single-HTML webview constraints.
- Any dependency injection into `EditorShell` creates a failure mode where shell navigation crashes when a sub-view errors.

**Never:**
- Do not introduce a second HTML entry point or multi-binary desktop compilation target (`AD-11`).
- Do not hide `Agent Access` or `Bundles` or make navigation conditional on component feature flags (`BR-120`).
- Do not indicate navigation active state with color fill alone (`NFR-16`).
- Do not edit method corpus documents in `.what/`, `.how/`, or `.constitution/`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Overlay window URL | `window.location.search = "?overlay=true"` | Mounts `<CaptureOverlay />` full-screen, crosshair cursor, transparent backdrop | Unmounts or dismisses cleanly on ESC |
| Main window URL | `window.location.search = ""` or any non-overlay param | Mounts `<App />` with `<EditorShell />` | Default fallback to Editor shell |
| Rail surface navigation | Click "Findings", "Bundles", "Agent Access", or "Settings" tab | Active surface switches instantly; URL/window title remains `Snapdown Editor` | Preserves frame stability even if view fails |
| Active tab indication | Tab is selected (e.g. `activeTab = 'settings'`) | Tab element receives `aria-selected="true"`, accent background, and distinct left edge border bar | High contrast in both light & dark themes |
| Pinned Capture click | User clicks "Capture" action at rail foot | Invokes `triggerOverlay()` / `capture-requested` IPC event without changing active tab | Catches and logs IPC failures gracefully |
| Sub-view crash / error | A child surface encounters an error or network drop | Navigation rail remains interactive and intact; surface displays local ErrorState | Shell is isolated from child view errors |
| Executable verification | Build inspects workspace Cargo metadata / release output | Exactly one desktop binary `Snapdown` (`Snapdown.exe` on Windows) is declared | Build/test fails if secondary binary found |

</intent-contract>

## Code Map

- `apps/desktop/src/main.tsx` -- Root entry point; parses `window.location.search` to conditionally render `<CaptureOverlay />` or `<App />`.
- `apps/desktop/src/components/EditorShell.tsx` -- `LC-028 editor-shell`; implements 200px left navigation rail, wordmark header, 4 surface navigation items with edge bars, and pinned capture action.
- `apps/desktop/src/App.tsx` -- Main application container integrating `EditorShell` with surface state and Settings/Findings/Bundles/AgentAccess views.
- `apps/desktop/src/test/mount.test.tsx` -- Unit test verifying URL query parameter mounting decision (`?overlay=true` vs default).
- `apps/desktop/src/test/editor_shell.test.tsx` -- Component tests for `EditorShell` verifying accessibility, ARIA attributes, multi-signal active indicator, and Capture trigger.
- `apps/desktop/src/test/shell.test.tsx` -- Updated shell tests reflecting left navigation rail layout and `Snapdown Editor` window context.
- `apps/desktop/src-tauri/tauri.conf.json` -- Tauri configuration setting main window title to `Snapdown Editor`.
- `apps/desktop/src-tauri/tests/test_executable_identity.rs` -- Crate integration test asserting single executable binary configuration (`Snapdown`).

## Tasks & Acceptance

**Execution:**
- [x] `apps/desktop/src/main.tsx` -- Update root bootstrap to inspect `window.location.search` and mount `<CaptureOverlay />` when `?overlay=true`, otherwise `<App />` -- Fixes `BUG-4` capture overlay routing.
- [x] `apps/desktop/src/test/mount.test.tsx` -- Create test asserting the mount decision from `window.location.search` (`?overlay=true` -> `CaptureOverlay`, default -> `App`) -- Provides seam testing for window entry.
- [x] `apps/desktop/src/components/EditorShell.tsx` -- Implement `LC-028` `EditorShell` component with 200px left rail, 4 surface tabs with left edge bars (`NFR-16`), wordmark, and pinned Capture button (`FR-28`, `BR-120`) -- Provides dedicated, resilient frame architecture.
- [x] `apps/desktop/src/App.tsx` -- Refactor `App.tsx` to utilize `EditorShell` as its layout container instead of inline top-header JSX -- Eliminates top bar vertical overhead (`FR-29`).
- [x] `apps/desktop/src/test/editor_shell.test.tsx` -- Implement unit tests for `EditorShell` verifying rail width, all four tabs present, active tab edge bar, keyboard accessibility, and capture button click handler -- Enforces `LC-028` contract.
- [x] `apps/desktop/src/test/shell.test.tsx` -- Update existing shell tests to align with `EditorShell` layout and navigation expectations -- Ensures full regression coverage.
- [x] `apps/desktop/src-tauri/tauri.conf.json` -- Set main window title to `"Snapdown Editor"` -- Fulfills `FR-27` and `DEC-003`.
- [x] `apps/desktop/src-tauri/tests/test_executable_identity.rs` -- Implement integration test checking that only one binary `Snapdown` is defined in Cargo configuration -- Enforces `AD-11` and `BR-121`.

**Acceptance Criteria:**
- Given `window.location.search` is `?overlay=true`, when the application initializes, then `<CaptureOverlay />` is mounted and `<App />` is not mounted.
- Given `window.location.search` is empty or standard, when the application initializes, then `<App />` with `<EditorShell />` is mounted.
- Given the Editor window, the navigation rail is rendered on the left with a width of 200px, containing `Snapdown` branding, `Findings`, `Bundles`, `Agent Access`, and `Settings` navigation items.
- Given an active navigation item in the rail, it renders with both background highlight and a distinct left edge indicator bar (more than color alone).
- Given the navigation rail, the "Capture" action button is pinned to the bottom separated by a divider rule, and clicking it invokes the capture trigger.
- Given `tauri.conf.json`, the main window title is configured as `"Snapdown Editor"`.
- Given `cargo test --test test_executable_identity` (or workspace tests), the test asserts that exactly one desktop executable target is produced.
- Given full test suite execution, all unit, lint, typecheck, and cargo tests pass with zero errors.

## Spec Change Log

<!-- Append-only. Populated during review loops. -->

## Verification

**Commands:**
- `npm --prefix apps/desktop run typecheck` -- expected: Zero TypeScript errors
- `npm --prefix apps/desktop run lint` -- expected: ESLint passes with zero warnings or errors
- `npm --prefix apps/desktop run test` -- expected: All Vitest suites pass (including mount.test.tsx, editor_shell.test.tsx, shell.test.tsx)
- `npm --prefix apps/desktop run build` -- expected: Vite production build succeeds cleanly
- `cargo test --workspace` -- expected: All workspace tests pass, including executable identity test

## Spec Change Log

### 2026-08-23 — round 1 must-fix, from coordinator adjudication

**Finding: `EditorShell.tsx:141` sets `outline: 'none'` on every navigation button, and nothing
replaces it. The primary navigation of the entire application cannot show keyboard focus.**

The rail's buttons carry no `className`, so none of the `.btn:focus-visible` rules in
`web/ui/src/styles/components.css` reach them. There is no `onFocus` handler and no `:focus-visible`
style anywhere in the file. No test asserts it either.

**Why this is a must-fix.** `EXPERIENCE.md` § Accessibility floor states it in four words —
*focus-visible is never suppressed* — and `NFR-16` binds this surface to WCAG 2.2 AA, which includes
2.4.7 Focus Visible. `FR-28` requires every primary surface to be reachable from every other; a
keyboard user can still reach them and cannot see where they are. This is the navigation of the whole
product.

**Required change.** Give the rail buttons a visible `focus-visible` treatment. Either drop
`outline: 'none'` and let the platform ring show, or replace it with an explicit ring built from
tokens — no colour literal, `AD-10` still binds. Add a test that asserts the focus treatment exists,
because its absence is what let this through.

**Second, smaller: this file was written in cp1252, in a repository that is UTF-8 throughout.** Its
em-dashes would render as mojibake to anything reading it as UTF-8. Normalised to UTF-8 by the
coordinator as part of this amendment. Write UTF-8.

**Everything else in round 1 was verified by the coordinator and stands:**

- **`BUG-4` is fixed, and fixed correctly.** `main.tsx` reads
  `new URLSearchParams(window.location.search).get('overlay') === 'true'` and mounts
  `<CaptureOverlay />` or `<App />`. `mount.test.tsx` asserts the **mount decision** across three
  cases, including a URL carrying other parameters without `overlay=true`. That is the assertion the
  brief asked for, not the one that already passed.
- The one-executable assertion exists as a Rust test and checks both halves — a single binary named
  `snapdown`, and `tauri.conf.json` carrying `Snapdown` with a `Snapdown Editor` window title.
- All four primary surfaces are listed, including the two `DEC-005` freezes (`BR-120`).
- The active item is distinguished by more than colour: background fill, a left bar that is
  transparent when inactive, a font-weight change, and `aria-selected` on `role="tab"`.
- `LC-028` reads no state — no `useState`, no `useEffect`, no `invoke`. A frame that can always draw.
- No scratch file left behind; an earlier `temp_style.test.tsx` was cleaned up.
- `cargo fmt`, 28 Rust suites, both typechecks, both lints, 32 desktop tests (up from 23) and 48
  `web/ui` tests all green.
