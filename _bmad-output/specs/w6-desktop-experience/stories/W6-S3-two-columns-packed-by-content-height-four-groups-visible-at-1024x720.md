---
id: W6-S3
title: 'W6-S3: Two columns packed by content height, four groups visible at 1024x720'
type: 'feature'
wave: W6
status: done
created: '2026-08-24'
dependencies:
  - W6-S1
  - W6-S2
  - W6-S10
files:
  - web/ui/src/styles/tokens.css
  - apps/desktop/src/components/SettingsView.tsx
  - apps/desktop/src/components/GeneralSection.tsx
  - apps/desktop/src/components/VaultSection.tsx
  - apps/desktop/src/components/QualityBudgetSection.tsx
  - apps/desktop/src/components/HotkeySection.tsx
  - apps/desktop/src/App.tsx
  - apps/desktop/src/test/settings_layout.test.tsx
  - apps/desktop/src/test/shell.test.tsx
  - web/ui/src/test/tokens.test.ts
context:
  - _bmad-output/specs/w6-desktop-experience/SPEC.md
  - _bmad-output/specs/w6-desktop-experience/stories.yaml
  - _bmad-output/specs/w6-desktop-experience/dispatch-briefs/W6-S3-step1-plan.md
  - .how/settings/01-ux/DESIGN.md
  - .how/settings/04-components/LC-028-editor-shell.md
  - .what/settings/SRS-settings.md
  - .what/settings/04-usecases/EXPERIENCE.md
  - .control/registry/defects.yaml
  - .how/_platform/design-system.md
  - .control/registry/waves.yaml
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:**
AUDIT-4 / defect photographic evidence from 2026-08-24 (`shot-4-settings.png`):
1. **Settings exceeds viewport and forces vertical scrolling at minimum window size (violating `FR-29`):** At 1024×720, a vertical scrollbar appears down the right edge and the `Hotkeys` group is cut off at the bottom of the window. `FR-29` requires all controls on primary surfaces to fit at 1024×720 without scrolling.
2. **Empty space wasted by stretched height grid layout:** The `General` (Startup) group is more than half empty while the cut-off occurs. The legacy layout pairs groups in equal-height CSS grid rows (`display: grid; grid-template-columns: 1fr 1fr;` with stretched row height), followed by `VaultSection` and `HotkeySection` stacked vertically below row 1 across the full width. Space is wasted where there is nothing to show and withheld where there is.
3. **Missing countable layout tokens:** The layout lacks the three design tokens specified in `.how/settings/01-ux/DESIGN.md` (`--settings-group-gap`, `--settings-column-min`, `--settings-row-height`), preventing deterministic, countable assertions of layout fit.
4. **Risk of surface boundary leakage:** When restructuring Settings into four dense groups, there is a risk of folding `Agent Access` in as a fifth configuration group. `Agent Access` is a primary surface of its own reachable from the navigation rail (`LC-028`, `FR-28`, `BR-120`) and must not live inside Settings.

**Approach:**
1. **Define Settings Layout Tokens in `web/ui/src/styles/tokens.css`:**
   Add the three required tokens in `:root` and dark theme blocks:
   - `--settings-group-gap: var(--space-4);` (16px / 1rem, between groups in a column)
   - `--settings-column-min: 380px;` (column breakpoint below which two columns become one)
   - `--settings-row-height: 32px;` (standard single control row height, making group heights countable in advance)
2. **Rebuild Settings (`LC-015`) as Two Independent Column Stacks:**
   Structure Settings as two independent flex column stacks (`display: flex; flex-direction: row; align-items: flex-start; gap: var(--settings-group-gap);`), where each group is packed strictly to its own content height (`height: auto; align-self: flex-start;` or packed column):
   - **Column A:**
     - `Startup` (General group): Run at Windows startup toggle + description (~80px).
     - `Vault folder` (`VaultSection`): Path input, Browse, Apply, Open in Explorer (~180px).
     - Total Column A height: ~276px.
   - **Column B:**
     - `Quality Budget` (`QualityBudgetSection`): Presets/inputs, latest finding size readout, action (~220px).
     - `Hotkeys` (`HotkeySection`): Shortcut rows for Capture Region and Open Editor with chips and badges (~260px).
     - Total Column B height: ~496px.
   - Available height at 1024×720 (with 200px rail leaving 824×720 viewport): taller column is Column B (~496px + padding < 550px), fitting comfortably within 720px without vertical scrolling.
   - When window width shrinks below `2 * var(--settings-column-min) + var(--settings-group-gap) + 200px` (< 980px), columns may wrap into a single column and scroll per `FR-29`.
3. **Maintain Strict Surface Boundary:**
   Ensure `Agent Access` is NOT present anywhere on the Settings surface. It remains a primary surface accessible exclusively via `EditorShell`'s left navigation rail (`FR-28`, `BR-120`).
4. **Implement Three Automated Tests in Vitest with Derivation from Tokens:**
   - `vitest::all_four_settings_groups_are_visible_at_the_minimum_window_size`:
     - Parses `--settings-column-min`, `--settings-row-height`, and `--settings-group-gap` from `tokens.css`.
     - Computes the geometry for Column A (Startup + Vault) and Column B (Quality Budget + Hotkeys) at 1024×720 window dimensions (200px rail width).
     - Asserts that all four groups are mounted, visible, and the taller column + outer padding does not exceed the 720px window height (no vertical scrollbar).
     - *How it fails on regression:* If the layout regresses to the 3-row layout (stacking Vault and Hotkeys below row 1), the combined vertical height (>760px) exceeds 720px, failing the assertion.
   - `vitest::no_group_is_stretched_to_match_a_neighbours_height`:
     - Asserts that Column A and Column B are separate container elements with `align-items: flex-start`, and that Startup group container height is packed by its own content rather than stretched to match Quality Budget's height.
     - *How it fails on regression:* If someone replaces the independent columns with `display: grid` with stretched rows, the Startup group container height expands to match Quality Budget, failing the assertion.
   - `vitest::agent_access_is_a_primary_surface_and_not_a_settings_group`:
     - Asserts that the Settings surface renders exactly the four configuration groups and contains zero Agent Access elements/inputs.
     - Asserts that Agent Access is rendered only when navigation tab is `'agent-access'` via `EditorShell`.
     - *How it fails on regression:* If Agent Access is placed into Settings, this test fails.

## Boundaries & Constraints

**Always:**
- Settings MUST be arranged in two independent column stacks packed by content height, never stretched: Column A = [Startup, Vault folder], Column B = [Quality Budget, Hotkeys] (`.how/settings/01-ux/DESIGN.md`).
- Group heights MUST be packed to their content; a short group MUST NOT stretch to match its neighbour's height.
- All four groups MUST be visible at the minimum supported window size of 1024×720 without scrolling (`FR-29`).
- Layout tokens `--settings-group-gap`, `--settings-column-min`, and `--settings-row-height` MUST be defined in `web/ui/src/styles/tokens.css` for both light and dark themes.
- `Agent Access` MUST remain a primary surface reachable from the navigation rail (`LC-028`, `FR-28`, `BR-120`) and MUST NOT be placed inside Settings.
- All styling MUST use design tokens from `tokens.css` without colour literals (`AD-10`, `W6-S1`).
- The three required Vitest tests MUST derive expectations from tokens and fail on mutation/regression.

**Block If:**
- Upstream design changes introduce sub-navigation within Settings that hides groups behind clicks (explicit Non-goal).
- The 1024×720 viewport cannot accommodate both columns at `--settings-column-min: 380px` alongside the 200px rail.

**Never:**
- Do not add sub-navigation (tabs or pills) inside Settings to hide groups (Non-goal; violates `FR-29` intent).
- Do not fold `Agent Access` into Settings (`FR-28`, `BR-120`).
- Do not modify the internals/business logic of `Quality Budget` (owned by `W6-S4`), `Run at Startup` (owned by `W6-S5`), or `Hotkeys` (owned by `W6-S6`).
- Do not use hardcoded pixel values or colour literals in TSX/CSS components.
- Do not edit files in `.what/`, `.how/`, or `.constitution/`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Window size at 1024×720 (minimum supported) | Width 1024px, Height 720px, Left rail 200px | Settings renders two columns side-by-side (Col A: Startup, Vault; Col B: Quality Budget, Hotkeys). Total height < 550px, fitting within 720px without vertical scrollbar. | Verified by `all_four_settings_groups_are_visible_at_the_minimum_window_size` |
| Group height independence | Short group (Startup ~80px) next to taller group (Quality Budget ~220px) | Startup group ends where its content ends (~80px); does not stretch to 220px. Column A height is ~276px; Column B height is ~496px. | Verified by `no_group_is_stretched_to_match_a_neighbours_height` |
| Narrow viewport (< 980px) | Window width resized below 2 columns min (`< 2 * 380px + 16px + 200px`) | Columns wrap into a single vertical stack, enabling vertical scrollbar smoothly. | Graceful responsive degradation below minimum supported width |
| Large viewport (1920×1080) | Window width 1920px, Height 1080px | Settings columns remain side-by-side with max content width container, packed by height. | Standard responsive scaling |
| Primary surface separation | Settings tab active | Settings renders 4 groups: Startup, Vault, Quality Budget, Hotkeys. Agent Access is absent. | Verified by `agent_access_is_a_primary_surface_and_not_a_settings_group` |
| Surface navigation to Agent Access | User clicks "Agent Access" in rail | Main view renders `AgentAccessView`; Settings view is unmounted. | Managed cleanly by `EditorShell` |

</intent-contract>

## Code Map

- `web/ui/src/styles/tokens.css` -- Add `--settings-group-gap` (`var(--space-4)`), `--settings-column-min` (`380px`), and `--settings-row-height` (`32px`) in `:root` and dark theme blocks.
- `apps/desktop/src/components/SettingsView.tsx` -- Dedicated `LC-015` Settings surface component implementing the 2-column packed flex layout with Column A (`GeneralSection`/Startup + `VaultSection`) and Column B (`QualityBudgetSection` + `HotkeySection`).
- `apps/desktop/src/components/GeneralSection.tsx` -- Updated Startup section with explicit content-height packing and alignment.
- `apps/desktop/src/components/VaultSection.tsx` -- Vault section packed to content height in Column A.
- `apps/desktop/src/components/QualityBudgetSection.tsx` -- Quality budget section packed to content height in Column B.
- `apps/desktop/src/components/HotkeySection.tsx` -- Hotkeys section packed to content height in Column B.
- `apps/desktop/src/App.tsx` -- Clean integration of `SettingsView` (`LC-015`) within `EditorShell`.
- `apps/desktop/src/test/settings_layout.test.tsx` -- Vitest suite implementing `all_four_settings_groups_are_visible_at_the_minimum_window_size`, `no_group_is_stretched_to_match_a_neighbours_height`, and `agent_access_is_a_primary_surface_and_not_a_settings_group`.
- `apps/desktop/src/test/shell.test.tsx` -- Existing desktop tests updated to match the restructured Settings surface layout.
- `web/ui/src/test/tokens.test.ts` -- Token parity test verifying `--settings-*` tokens in light and dark schemes.

## Tasks & Acceptance

**Execution:**
- `web/ui/src/styles/tokens.css` -- Add `--settings-group-gap: var(--space-4);`, `--settings-column-min: 380px;`, and `--settings-row-height: 32px;` to both `:root` and `@media (prefers-color-scheme: dark)`.
- `apps/desktop/src/components/SettingsView.tsx` -- Implement `LC-015` `SettingsView` containing two independent flex column stacks (Column A: Startup + Vault; Column B: Quality Budget + Hotkeys) with `align-items: flex-start;` and gap `--settings-group-gap`.
- `apps/desktop/src/components/GeneralSection.tsx` -- Ensure Startup group adheres to `--settings-row-height` control row sizing and compact padding.
- `apps/desktop/src/components/VaultSection.tsx` -- Verify content-height packing and token usage.
- `apps/desktop/src/components/QualityBudgetSection.tsx` -- Verify content-height packing in Column B.
- `apps/desktop/src/components/HotkeySection.tsx` -- Verify content-height packing in Column B.
- `apps/desktop/src/App.tsx` -- Mount `SettingsView` on `activeTab === 'settings'`.
- `apps/desktop/src/test/settings_layout.test.tsx` -- Implement the 3 required Vitest tests:
  - `vitest::all_four_settings_groups_are_visible_at_the_minimum_window_size` (calculates geometry from tokens at 1024×720, asserts fit without scroll).
  - `vitest::no_group_is_stretched_to_match_a_neighbours_height` (asserts independent column stacks and content-height packing for Startup group vs Quality Budget).
  - `vitest::agent_access_is_a_primary_surface_and_not_a_settings_group` (asserts exactly 4 groups in Settings, 0 Agent Access elements in Settings).
- `apps/desktop/src/test/shell.test.tsx` -- Verify all existing settings interactions (save budget, move vault, set hotkey, toggle startup) pass under new layout.
- `web/ui/src/test/tokens.test.ts` -- Assert `--settings-group-gap`, `--settings-column-min`, `--settings-row-height` present in token suite.

**Acceptance Criteria:**
- Given `web/ui/src/styles/tokens.css`, `--settings-group-gap`, `--settings-column-min`, and `--settings-row-height` are defined in `:root` and dark theme blocks.
- Given the Settings surface at 1024×720 viewport, Column A (Startup, Vault) and Column B (Quality Budget, Hotkeys) render side-by-side with maximum column height < 550px, fitting completely within 720px without vertical scrollbar.
- Given the Startup group and Quality Budget group, the Startup group container height is packed to its content (~80px) and does NOT stretch to match Quality Budget's height (~220px).
- Given the Settings surface, exactly four groups (`Startup`, `Vault Folder`, `Quality Budget`, `Hotkeys`) are present, and zero Agent Access controls/sections exist within Settings.
- Given the Vitest suites, `all_four_settings_groups_are_visible_at_the_minimum_window_size`, `no_group_is_stretched_to_match_a_neighbours_height`, and `agent_access_is_a_primary_surface_and_not_a_settings_group` pass.
- Given all verification commands (`npm run typecheck`, `npm run lint`, `npm run test`, `npm run build`), all pass cleanly with zero errors.

## Design Notes

**Countable Layout Math:**
- Available viewport at 1024×720 minimum window:
  - Width: 1024px - 200px (Navigation Rail) = 824px.
  - Two columns at `--settings-column-min: 380px` = 760px + `--settings-group-gap: 16px` = 776px <= 824px available width. Both columns fit comfortably side-by-side.
  - Height: 720px available.
  - Column A: Startup (~80px) + Gap (16px) + Vault (~180px) = ~276px.
  - Column B: Quality Budget (~220px) + Gap (16px) + Hotkeys (~260px) = ~496px.
  - Max column height = ~496px. Container padding (24px top + 24px bottom) = ~544px total height <= 720px viewport.
  - Margin to viewport limit: ~176px buffer. Zero vertical scrollbar required.

**Test Regression / Mutation Mechanics:**
1. `all_four_settings_groups_are_visible_at_the_minimum_window_size`:
   - Reads tokens from `tokens.css`. Computes max column height from token dimensions and item row counts.
   - Asserts total calculated layout height <= 720px.
   - If layout reverts to 3-row vertical stack (Row 1: Startup+Quality Budget ~240px, Row 2: Vault ~180px, Row 3: Hotkeys ~260px + gaps + padding = ~760px), test fails with `expected 760 <= 720`.
2. `no_group_is_stretched_to_match_a_neighbours_height`:
   - Asserts Column A and Column B are separate flex containers with `align-items: flex-start`.
   - Asserts Startup container height != Quality Budget container height (packed height ratio < 0.6).
   - If layout is changed to `grid-template-rows: 1fr` / `align-items: stretch`, Startup height equals Quality Budget height, failing the test.
3. `agent_access_is_a_primary_surface_and_not_a_settings_group`:
   - Queries Settings DOM container for Agent Access headings, access key textfields, or token badges.
   - Asserts count === 0 and asserts `screen.getAllByRole('region')` / groups matches exactly 4.
   - If Agent Access is added to Settings, test fails.

## Verification

**Commands:**
- `npm --prefix web/ui run typecheck` -- expected: TypeScript compiles with zero errors
- `npm --prefix web/ui run lint` -- expected: ESLint passes with zero warnings/errors
- `npm --prefix web/ui run test` -- expected: All web/ui Vitest suites pass
- `npm --prefix apps/desktop run typecheck` -- expected: Zero TypeScript diagnostic errors
- `npm --prefix apps/desktop run lint` -- expected: ESLint passes with zero warnings/errors
- `npm --prefix apps/desktop run test` -- expected: All desktop Vitest suites pass (including settings_layout.test.tsx and shell.test.tsx)
- `npm --prefix apps/desktop run build` -- expected: Production Vite build succeeds
- `cargo test --workspace` -- expected: Rust crates remain green

## Spec Change Log

<!-- Append-only. Populated during review loops. -->
