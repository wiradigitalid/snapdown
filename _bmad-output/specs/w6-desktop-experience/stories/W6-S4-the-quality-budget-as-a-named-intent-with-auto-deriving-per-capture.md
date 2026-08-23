---
id: W6-S4
title: 'W6-S4: The Quality Budget as a named intent, with Auto deriving per capture'
type: 'feature'
wave: W6
status: ready-for-dev
created: '2026-08-24'
dependencies:
  - W6-S1
  - W6-S2
  - W6-S3
files:
  - crates/snapdown-core/src/domain/setting.rs
  - crates/snapdown-core/src/domain/finding.rs
  - crates/snapdown-core/src/domain/image.rs
  - crates/snapdown-core/src/ports/mod.rs
  - crates/snapdown-store/src/sqlite/migrations.rs
  - crates/snapdown-store/src/sqlite/finding_store.rs
  - crates/snapdown-store/src/sqlite/settings_store.rs
  - crates/snapdown-store/src/image/pipeline.rs
  - crates/snapdown-store/tests/test_image_reduction.rs
  - crates/snapdown-store/tests/test_sqlite_settings.rs
  - crates/snapdown-store/tests/test_sqlite_findings.rs
  - apps/desktop/src-tauri/src/commands/settings.rs
  - apps/desktop/src-tauri/src/commands/capture.rs
  - apps/desktop/src-tauri/src/commands/finding.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/types/settings.ts
  - apps/desktop/src/types/finding.ts
  - apps/desktop/src/services/settings.ts
  - apps/desktop/src/components/QualityBudgetSection.tsx
  - apps/desktop/src/components/SettingsView.tsx
  - apps/desktop/src/test/quality_budget.test.tsx
  - apps/desktop/src/test/settings_layout.test.tsx
  - apps/desktop/src/test/shell.test.tsx
  - apps/desktop/src/test/findings_view.test.tsx
context:
  - _bmad-output/specs/w6-desktop-experience/SPEC.md
  - _bmad-output/specs/w6-desktop-experience/stories.yaml
  - _bmad-output/specs/w6-desktop-experience/dispatch-briefs/W6-S4-step1-plan.md
  - .control/decisions/DEC-004-quality-budget-presets.md
  - .what/finding/05-scenarios/SCN-03-the-quality-budget-that-resolves-differently.md
  - .how/finding/04-components/LC-003-image-reducer.md
  - .how/settings/01-ux/DESIGN.md
  - .how/settings/02-contracts/contract-inventory.md
  - .how/finding/02-contracts/contract-inventory.md
  - .how/settings/05-model/data-model.md
  - .how/finding/05-model/data-model.md
  - .what/settings/04-usecases/UC-13-decide-how-much-picture-quality-a-screenshot-is-worth.md
  - .what/settings/03-domain/state-machines.md
  - .what/business-rules.md
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:**
1. **Presentation Defeated the Promise (`FR-5`, `DEC-004`, `AUDIT-4`):**
   The shipped Quality Budget control exposes two raw numeric text boxes (`Max Long Edge (px)` `1600` and `Encoder Quality (10-100)` `75`). `FR-5` promised that *"both values have defaults the Reviewer never has to change to get a usable result."* The Reviewer cannot reason about raw pixel dimensions or encoder percentages without trial and error. Furthermore, 1600 px is an unmeasured constant (`OQ-3`).
2. **Fixed Constant Fails Small Tooltips and 4K Screens (`SCN-03`):**
   A single fixed long edge constant (`1600`) and quality number (`75`) cannot serve the range of captures:
   - Tooltip capture (`312 × 118`): Under any cap, so long edge downscaling does nothing. Encoder quality decides everything, and 75% quality produces noticeable lossy compression artefacts around 11 px text where no pixels can be spared.
   - 4K dashboard capture (`3840 × 2160`): Exceeds cap by over 2×. Downscaling aggressively removes fine detail, making a high quality setting redundant and wasting storage.
3. **Four Legacy Tests Hardcode the Old Constant:**
   Existing tests (`crates/snapdown-core/src/domain/image.rs:92-93`, `crates/snapdown-core/src/domain/setting.rs:171`, `crates/snapdown-store/src/image/pipeline.rs:92`, `crates/snapdown-store/tests/test_image_reduction.rs:16`) assert that images reduce to fixed 1600 px. Replacing them with arbitrary Auto outputs would create tautological assert-what-the-code-does tests. They must be replaced with `SCN-03`'s variance test (`assert resolved(A) != resolved(B)`) and explicit assertions for fixed presets (`Sharp`, `Balanced`, `Small`).
4. **Missing Stored Derivation Record on Findings (`NFR-18`, `BR-105`):**
   The `finding` table stores only `image_width` and `image_height` (the effect of reduction). It cannot record which named budget was active or what long edge and encoder quality were applied to that capture. When `Auto` derives parameters dynamically, existing Findings must not be re-encoded (`BR-9`), requiring each Finding to store its own `resolved_long_edge`, `resolved_encoder_quality`, and `budget_name`.
5. **Setting Store Shape Violates Atomic State Integrity (`BR-116`):**
   Settings currently treats `QualityBudget` as raw `{ max_long_edge, encoder_quality }`. Under `DEC-004`, the Quality Budget is a named intent (`Auto`, `Sharp`, `Balanced`, `Small`, `Custom`). The named state and its resolved/custom pair must be persisted as one atomic write (`BR-116`).
6. **UI Lacks Named Presets, Intent Prose, Attributed Readout, and Advanced Disclosure (`DESIGN.md`):**
   `QualityBudgetSection` lacks the four-option `SegmentedControl`, the explanatory prose per preset, the attributed readout (`Latest: 184 KB · 1408 px · Auto`), and the `▸ Advanced` disclosure that moves to `Custom` only when an underlying number is edited (`BR-117`).

**Approach:**
1. **Apply Schema Migration v7 (`crates/snapdown-store/src/sqlite/migrations.rs`):**
   - Add migration `v7` to alter `finding` table:
     - `ALTER TABLE finding ADD COLUMN resolved_long_edge INTEGER;`
     - `ALTER TABLE finding ADD COLUMN resolved_encoder_quality INTEGER;`
     - `ALTER TABLE finding ADD COLUMN budget_name TEXT;`
   - Existing rows default to `NULL` (or inferred legacy values `1600`, `75`, `'Balanced'`).
2. **Domain Models & Derivation Logic (`crates/snapdown-core`):**
   - Define `NamedBudget` enum: `Auto`, `Sharp`, `Balanced`, `Small`, `Custom` (default: `Auto`).
   - Define `ResolvedPair` struct: `{ max_long_edge: u32, encoder_quality: u8 }`.
   - Define `QualityBudget` / `QualityBudgetState` carrying `named: NamedBudget` and `custom_pair: Option<ResolvedPair>`.
   - Define fixed preset values:
     - `Sharp`: `(max_long_edge: 2560, encoder_quality: 90)` — *"Keeps small text crisp. Files are larger."*
     - `Balanced`: `(max_long_edge: 1600, encoder_quality: 75)` — *"A middle setting that does not change with the capture."*
     - `Small`: `(max_long_edge: 1280, encoder_quality: 50)` — *"The smallest file that is still readable."*
   - Implement `QualityBudgetResolver` in `snapdown-core` (`LC-003`):
     - For `Auto`, derive `(max_long_edge, encoder_quality)` dynamically as a function of the region dimensions:
       - Small region (long edge <= 800 px, e.g. `312 × 118` tooltip): no downscale cap needed (`max_long_edge: 1280`), encoder quality is **high** (`92`), preserving sharp 11 px text without compression artifacts.
       - Medium region (`800 < long edge <= 1920` px): `max_long_edge: 1600`, encoder quality `82`.
       - Large/4K region (long edge > 1920 px, e.g. `3840 × 2160` dashboard): `max_long_edge: 1600` (or `1920`), encoder quality is **lower** (`70`), since downscaling already removed high-frequency detail.
       - Guarantees `resolved(A: 312×118) != resolved(B: 3840×2160)` (`SCN-03`).
       - Document curve rationale in code referencing `OQ-3`.
     - For `Sharp`, `Balanced`, `Small`: return their fixed `ResolvedPair`.
     - For `Custom`: return the stored custom `ResolvedPair`.
3. **Store & Image Pipeline Updates (`crates/snapdown-store`):**
   - Update `SqliteFindingStore`:
     - Update insert queries to persist `resolved_long_edge`, `resolved_encoder_quality`, and `budget_name`.
     - Update `get_finding` and `list_findings` to return `Finding` with new fields (`NFR-18`, `BR-105`).
   - Update `SqliteSettingsStore`:
     - Support storing `SettingValue::QualityBudget` holding `QualityBudget` (named state + optional custom pair) in one atomic JSON serialization/write (`BR-116`).
   - Update `ImageReducer` (`pipeline.rs`):
     - Accept `ResolvedPair` and `budget_name`. Compute target dimensions via `original_dims.compute_reduced_dimensions_with_edge(resolved.max_long_edge)` and encode with `resolved.encoder_quality`.
4. **Tauri IPC Command Layer (`apps/desktop/src-tauri`):**
   - Add `CS-12` `get_quality_budget_presets`: returns preset definitions, prose strings, and fixed parameters.
   - Update `CS-3` `set_quality_budget`: accepts `(budget: NamedBudget, advanced: Option<ResolvedPair>) -> QualityBudgetDto`.
     - Passing `advanced` moves state to `Custom` (`BR-117`).
     - Rejects out-of-range values on entry (`MIN_LONG_EDGE_PX..=MAX_LONG_EDGE_PX`, `MIN_ENCODER_QUALITY..=MAX_ENCODER_QUALITY`). Out-of-range input does NOT transition to `Custom`.
     - Performs a single atomic write (`BR-116`).
   - Update `CS-1` `get_settings`: returns `QualityBudgetDto` containing `named`, `resolved_pair`, and prose.
   - Update `CF-1` `capture_screen_region`:
     - Resolves budget for the captured region.
     - Runs reduction, creates finding record with `resolved_long_edge`, `resolved_encoder_quality`, `budget_name`.
     - Returns `CaptureResultDto` carrying resolved pair and budget name (`NFR-18`).
   - Update `CS-4` `get_latest_finding_size` / latest finding query:
     - Returns latest finding size, dimensions, and `budget_name` for the attributed readout (`Latest: 184 KB · 1408 px · Auto`).
5. **Desktop UI (`apps/desktop/src/components/QualityBudgetSection.tsx`):**
   - Render `@snapdown/ui` `SegmentedControl` with options `Auto`, `Sharp`, `Balanced`, `Small` (and `Custom` when active).
   - Render the single line of intent prose below the control for the active preset:
     - Auto: *"Sizes each capture to what it is. Most captures land near 120 KB."*
     - Sharp: *"Keeps small text crisp. Files are larger."*
     - Balanced: *"A middle setting that does not change with the capture."*
     - Small: *"The smallest file that is still readable."*
     - Custom: *"Custom limits set in Advanced."*
   - Render the attributed readout: `Latest: 184 KB · 1408 px · Auto` (or *"No captures yet"*).
   - Provide `▸ Advanced` disclosure:
     - Collapses `TextField`s for `Max Long Edge (px)` and `Encoder Quality (10-100)`.
     - Editing either field immediately switches the `SegmentedControl` to `Custom` (`BR-117`).
     - A reviewer who never opens Advanced never sees raw numbers.
   - Selecting any named preset (`Auto`, `Sharp`, `Balanced`, `Small`) applies immediately (`UC-13`), writes atomic state (`BR-116`), updates Advanced inputs to reflect that preset's resolved values, and removes/hides `Custom` if unselected.
6. **Automated Verification Suite:**
   - Rust unit & integration tests in `snapdown-core` and `snapdown-store`:
     - `finding::auto_resolves_a_different_pair_for_a_small_region_than_for_a_full_screen` (`SCN-03`)
     - `finding::auto_resolves_a_higher_encoder_quality_when_no_downscale_applies`
     - `finding::every_stored_finding_carries_the_pair_that_was_applied_to_it` (`NFR-18`, `BR-105`)
     - `finding::a_finding_stored_before_a_derivation_change_is_not_re_encoded` (`BR-9`)
     - `finding::a_finding_can_state_which_named_budget_produced_it`
     - `settings::fixed_presets_resolve_pinned_constants` (`Sharp`, `Balanced`, `Small`)
     - `settings::quality_budget_atomic_write_preserves_consistency` (`BR-116`)
     - `settings::invalid_advanced_values_are_refused_and_do_not_transition_to_custom` (`BR-117`)
   - Vitest component and UI tests in `apps/desktop/src/test/`:
     - `vitest::a_reviewer_who_never_opens_advanced_never_sees_a_raw_number`
     - `vitest::editing_an_advanced_value_moves_the_control_to_custom_visibly`
     - `vitest::selecting_a_preset_updates_prose_and_saves_atomically`
     - `vitest::attributed_readout_shows_size_dimension_and_budget_name`

## Boundaries & Constraints

**Always:**
- The Quality Budget MUST hold exactly one of five named states: `Auto`, `Sharp`, `Balanced`, `Small`, `Custom` (`BR-103`, `DEC-004`).
- `Auto` is the shipped default (`DEC-004`, `BR-111`).
- Under `Auto`, long edge and encoder quality MUST be derived dynamically per capture from the region dimensions (`BR-104`, `LC-003`).
- Small tooltip regions (`312 × 118`) and 4K screen regions (`3840 × 2160`) MUST NOT resolve the same parameter pair under `Auto` (`SCN-03`). A test finding them identical is a failing test.
- Every stored Finding MUST record `resolved_long_edge`, `resolved_encoder_quality`, and `budget_name` in SQLite schema migration `v7` (`NFR-18`, `BR-105`).
- The named budget state and its resolved/custom pair MUST be written as one atomic setting update (`BR-116`).
- `Custom` MUST be entered if and only if the Reviewer explicitly edits an Advanced value directly (`BR-117`). `Auto` resolving an unusual pair MUST NOT transition to `Custom`.
- Out-of-range values in Advanced MUST be refused at entry and MUST NOT move the budget to `Custom` (`BR-117`).
- A reviewer who never opens Advanced MUST NEVER see a raw pixel or quality number (`DEC-004`, `DESIGN.md`).
- Existing findings MUST NEVER be re-encoded when the budget changes (`BR-9`, `FR-5`).
- Reduction and storage MUST happen AFTER capture overlay dismissal (`NFR-2`, `CF-1`).
- All styles MUST use design tokens from `tokens.css` without colour literals (`AD-10`, `W6-S1`).

**Block If:**
- Upstream requirements demand hardcoding a single static compression constant for all captures under `Auto`.
- Migration v7 conflicts with SQLite constraints or disrupts existing finding records.

**Never:**
- Do not re-point legacy constant tests at arbitrary Auto outputs (turns tests into tautologies).
- Do not make `Auto` resolve static constants `1600` and `75` identically for both small tooltips and full screens.
- Do not show raw numbers on screen when Advanced is collapsed.
- Do not re-encode previously captured Findings when the Quality Budget setting changes (`BR-9`).
- Do not perform image reduction before dismissing the capture overlay (`NFR-2`).
- Do not modify corpus documents in `.what/`, `.how/`, or `.constitution/`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| First run / default budget | Fresh install; no `quality_budget` setting in DB | Returns `NamedBudget::Auto`; readout displays preset prose: *"Sizes each capture to what it is. Most captures land near 120 KB."* | Shipped default applied automatically |
| Capture tooltip on Auto | Capture region `312 × 118` with Auto budget | No downscale cap; resolves high encoder quality (`92`); Finding stores `resolved_long_edge: 312`, `resolved_encoder_quality: 92`, `budget_name: "Auto"` | Cleanly encoded without text artifacts |
| Capture 4K screen on Auto | Capture region `3840 × 2160` with Auto budget | Downscaled to long edge `1600` (`1600 × 900`); resolves lower encoder quality (`70`); Finding stores `resolved_long_edge: 1600`, `resolved_encoder_quality: 70`, `budget_name: "Auto"` | Downscaled & compressed |
| Region variance assertion (`SCN-03`) | Capture A (`312 × 118`) vs Capture B (`3840 × 2160`) under Auto | `resolved(A) != resolved(B)` (both long edge and encoder quality differ) | Asserted by automated integration test |
| Fixed preset: Sharp | Reviewer selects `Sharp` in SegmentedControl | Named state `Sharp` saved atomically (`BR-116`); all subsequent captures use fixed `2560 px` long edge and `90` encoder quality | Fixed parameters applied |
| Fixed preset: Balanced | Reviewer selects `Balanced` in SegmentedControl | Named state `Balanced` saved atomically; subsequent captures use fixed `1600 px` and `75` quality | Fixed parameters applied |
| Fixed preset: Small | Reviewer selects `Small` in SegmentedControl | Named state `Small` saved atomically; subsequent captures use fixed `1280 px` and `50` quality | Fixed parameters applied |
| Edit Advanced input | Reviewer opens `▸ Advanced` and types `1920` in Max Long Edge | SegmentedControl immediately shifts to 5th segment `Custom` visibly (`BR-117`); custom pair `{ 1920, current_quality }` persisted atomically | In-place state transition |
| Out-of-range Advanced value | Reviewer enters `100` px in Max Long Edge (valid: `320..=7680`) | Value refused with validation error; budget does NOT move to `Custom`; previous valid budget preserved (`BR-117`) | Error message displayed under input |
| Switch from Custom to Auto | Reviewer on `Custom`, clicks `Auto` segment | State moves to `Auto`; custom overrides discarded; Advanced fields show Auto readout; `Custom` segment deselected | Reset to dynamic derivation |
| Attributed readout with captures | Vault contains previous capture (size: 188416 B, dim: 1408×792, budget: Auto) | Readout renders: `Latest: 184 KB · 1408 px · Auto` | Readout parsed accurately |
| Attributed readout without captures | Vault is empty / 0 findings | Readout renders: `No captures yet` | Displays empty indicator gracefully |

</intent-contract>

## Code Map

- `crates/snapdown-core/src/domain/setting.rs`
  - Define `NamedBudget` enum (`Auto`, `Sharp`, `Balanced`, `Small`, `Custom`).
  - Define `ResolvedPair` struct (`max_long_edge: u32`, `encoder_quality: u8`).
  - Define `QualityBudget` / `QualityBudgetState` struct with named budget and optional custom pair.
  - Implement `QualityBudgetResolver` trait and default curve derivation for `Auto`.
  - Provide preset definitions for `Sharp` (`2560 / 90`), `Balanced` (`1600 / 75`), and `Small` (`1280 / 50`).
- `crates/snapdown-core/src/domain/finding.rs`
  - Add `resolved_long_edge: Option<u32>`, `resolved_encoder_quality: Option<u8>`, and `budget_name: Option<String>` to `Finding` struct.
- `crates/snapdown-core/src/domain/image.rs`
  - Update `compute_reduced_dimensions` to work with `ResolvedPair` or max long edge explicitly.
  - Replace constant 1600 assertion with parameterised test.
- `crates/snapdown-store/src/sqlite/migrations.rs`
  - Add migration `v7`:
    ```sql
    ALTER TABLE finding ADD COLUMN resolved_long_edge INTEGER;
    ALTER TABLE finding ADD COLUMN resolved_encoder_quality INTEGER;
    ALTER TABLE finding ADD COLUMN budget_name TEXT;
    ```
- `crates/snapdown-store/src/sqlite/finding_store.rs`
  - Update `create_finding` to insert `resolved_long_edge`, `resolved_encoder_quality`, and `budget_name` (`NFR-18`, `BR-105`).
  - Update `get_finding` and `list_findings` to read the three new columns into `Finding`.
- `crates/snapdown-store/src/sqlite/settings_store.rs`
  - Update `SettingValue::QualityBudget` handling to serialize and deserialize `QualityBudgetState` atomically (`BR-116`).
- `crates/snapdown-store/src/image/pipeline.rs`
  - Update `ImageReducer::reduce_image` to accept `ResolvedPair` and return `ReducedImageResult` with applied compression metadata.
- `crates/snapdown-store/tests/test_image_reduction.rs`
  - Replace constant 1600 tests with `SCN-03` region variance assertions and fixed preset assertions.
- `crates/snapdown-store/tests/test_sqlite_findings.rs`
  - Integration tests verifying migration v7, storing and retrieving findings with resolved pairs and budget names.
- `apps/desktop/src-tauri/src/commands/settings.rs`
  - Add command `CS-12` `get_quality_budget_presets`.
  - Update command `CS-3` `set_quality_budget` to handle `(budget: NamedBudget, advanced: Option<ResolvedPair>)` atomically (`BR-116`, `BR-117`).
  - Update `CS-1` `get_settings` and `CS-4` `get_latest_finding_size` to provide attributed metadata.
- `apps/desktop/src-tauri/src/commands/capture.rs`
  - Update `CF-1` `capture_screen_region` to resolve budget for region, execute reduction, store resolved parameters in finding store (`NFR-18`), and return them in `CaptureResultDto`.
- `apps/desktop/src-tauri/src/lib.rs`
  - Register `get_quality_budget_presets` in Tauri command invocation handler.
- `apps/desktop/src/types/settings.ts`
  - Update `QualityBudget` and `Settings` TypeScript interfaces to include `NamedBudget`, `ResolvedPair`, `QualityBudgetPresetDto`.
- `apps/desktop/src/types/finding.ts`
  - Update `Finding` TypeScript interface to include `resolved_long_edge`, `resolved_encoder_quality`, `budget_name`.
- `apps/desktop/src/services/settings.ts`
  - Update `setQualityBudget`, `getSettings`, and add `getQualityBudgetPresets`.
- `apps/desktop/src/components/QualityBudgetSection.tsx`
  - Rebuild with `SegmentedControl` (`Auto`, `Sharp`, `Balanced`, `Small`, `Custom`), preset prose readout, attributed finding info (`Latest: 184 KB · 1408 px · Auto`), and collapsible `▸ Advanced` disclosure.
- `apps/desktop/src/test/quality_budget.test.tsx`
  - Comprehensive Vitest suite asserting:
    - Reviewer who never opens Advanced never sees a raw number.
    - Editing an Advanced value moves control to Custom visibly.
    - Selecting presets saves immediately and updates prose/readout.
- `apps/desktop/src/test/settings_layout.test.tsx` & `shell.test.tsx`
  - Update mock settings and assertions to align with named quality budget structure.

## Verification Plan

### Automated Tests
1. **Rust Workspace Verification (`snapdown-core` & `snapdown-store`)**:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```
   - Specific new test assertions:
     - `finding::auto_resolves_a_different_pair_for_a_small_region_than_for_a_full_screen` (`SCN-03`)
     - `finding::auto_resolves_a_higher_encoder_quality_when_no_downscale_applies`
     - `finding::every_stored_finding_carries_the_pair_that_was_applied_to_it` (`NFR-18`)
     - `finding::a_finding_stored_before_a_derivation_change_is_not_re_encoded` (`BR-9`)
     - `finding::a_finding_can_state_which_named_budget_produced_it`
     - `settings::fixed_presets_resolve_pinned_constants`
     - `settings::quality_budget_atomic_write_preserves_consistency` (`BR-116`)
     - `settings::invalid_advanced_values_are_refused_and_do_not_transition_to_custom` (`BR-117`)

2. **Web Package Verification (`web/ui` & `apps/desktop`)**:
   ```bash
   npm --prefix web/ui run typecheck && npm --prefix web/ui run lint && npm --prefix web/ui run test
   npm --prefix apps/desktop run typecheck && npm --prefix apps/desktop run lint
   npm --prefix apps/desktop run test && npm --prefix apps/desktop run build
   ```
   - Specific frontend test assertions:
     - `vitest::a_reviewer_who_never_opens_advanced_never_sees_a_raw_number`
     - `vitest::editing_an_advanced_value_moves_the_control_to_custom_visibly`
     - `vitest::segmented_control_renders_preset_prose_and_attributed_readout`
     - `vitest::selecting_a_preset_persists_immediately_and_closes_custom`

3. **Grep Sweeps**:
   - Verify that `<QualityBudgetSection` is mounted in `SettingsView.tsx`.
   - Verify that no raw color literals exist outside `web/ui/src/styles/tokens.css`.
