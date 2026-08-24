---
id: W8-S2
title: "W8-S2: A real decode, scale and encode, keeping the arithmetic that already works"
type: 'feature'
wave: W8
status: ready-for-dev
created: '2026-08-24'
review_loop_iteration: 0
followup_review_recommended: false
dependencies:
  - W8-S1
files:
  - crates/snapdown-store/Cargo.toml
  - crates/snapdown-store/src/image/pipeline.rs
  - crates/snapdown-store/tests/test_image_reduction.rs
  - apps/desktop/src-tauri/src/commands/capture.rs
context:
  - _bmad-output/specs/w8-capture-becomes-real/SPEC.md
  - _bmad-output/specs/w8-capture-becomes-real/stories.yaml
  - _bmad-output/specs/w8-capture-becomes-real/dispatch-briefs/W8-S2-step1-plan.md
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .control/registry/components.yaml
  - .control/reports/ASSESS-BUG-14.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/finding/SDD-finding.md
  - .how/finding/04-components/LC-003-image-reducer.md
  - .how/finding/06-flows/flow-capture.md
  - .what/finding/SRS-finding.md
  - .what/finding/05-scenarios/SCN-03-the-quality-budget-that-resolves-differently.md
  - .what/business-rules.md
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Context & Defect Rationale (`BUG-14`, `CAP-2`, `LC-003`, `FR-4`, `NFR-3`):**
In `crates/snapdown-store/src/image/pipeline.rs:26-37`, `ImageReducer::reduce_image` writes a 17-byte synthetic header followed by `input_bytes[16..]`, accompanied by the comment:
```rust
// Standard PNG/image signature and downscaled payload simulation
```
Because of this simulation, **no image in Snapdown has ever been reduced**. Although `FR-4` promises that every captured image is reduced automatically and `NFR-3` promises that every stored image fits within the Quality Budget's resolved long edge, these requirements have passed across 5 waves and 3 audits solely on the strength of numbers written into fake headers. The thumbnail path (`generate_thumbnail: true`) in `pipeline.rs` is the identical simulation repeated twice.

Following `W8-S1` (which established `crates/snapdown-capture` and added `image = "0.25.10"` to the workspace), this story implements the real decode-scale-encode pipeline in `crates/snapdown-store/src/image/pipeline.rs`, fulfilling logical component `LC-003 image-reducer`.

---

### Key Architectural Constraints & Invariants

1. **Preservation of Existing Dimension Arithmetic:**
   `compute_reduced_dimensions_for_pair`, `compute_reduced_dimensions_with_edge`, the `Auto` resolution from `W6-S4`, and the fixed presets (`Sharp`, `Balanced`, `Small`) in `snapdown-core` are mathematically correct and MUST BE KEPT EXACTLY AS THEY ARE. This story replaces the simulated bytes between them, never the arithmetic. Test `cargo::the_resolved_pair_arithmetic_is_unchanged_by_this_story` explicitly locks this behavior.
2. **`BR-40` — No Upscaling:**
   An image whose long edge is already less than or equal to the resolved `max_long_edge` MUST NOT be upscaled. Its original pixel dimensions and buffer are retained.
3. **`BR-41` — Aspect Ratio Preservation:**
   Reduction preserves aspect ratio precisely. The image is scaled proportionally; it is never stretched and never cropped.
4. **`AD-4` — Single Reduction at Capture:**
   An image is reduced exactly once, at capture, and no unreduced original is stored.
5. **Zero I/O in `snapdown-core`:**
   `crates/snapdown-core` remains strictly pure domain logic with zero I/O and zero image codec dependencies (`snapdown_core_has_no_io_dependency` must continue to pass). All decoding, scaling, and PNG encoding operations live in `crates/snapdown-store` and `crates/snapdown-capture`.
6. **No Tracked Screenshot Fixtures:**
   Per repository policy and the product brief, no captured screenshot or fixture derived from real capture output may be committed. All image test fixtures must be programmatically synthesised in memory.
7. **Mutation Testing Acceptance Criterion:**
   Every test added or updated must be validated by mutation (e.g. modifying downscale logic or pixel buffers, verifying test goes red, and reverting to green).

---

## Approach

1. **Update `crates/snapdown-store/Cargo.toml`:**
   - Add `image = { workspace = true }` under `[dependencies]`.
2. **Refactor `crates/snapdown-store/src/image/pipeline.rs`:**
   - Import `image::{load_from_memory, DynamicImage, ExtendedColorType, ImageEncoder, RgbaImage}` and `image::codecs::png::PngEncoder`.
   - In `ImageReducer::reduce_image`:
     - Decode `input_bytes` using `image::load_from_memory(input_bytes)`. Return `CoreError::Validation` if decoding fails.
     - Compute `target_dims = original_dims.compute_reduced_dimensions_for_pair(resolved)`.
     - If `original_dims.long_edge() <= resolved.max_long_edge`:
       - Retain image without downscaling (`BR-40`).
       - Encode the decoded RGBA buffer to valid PNG bytes using `PngEncoder`.
     - If `original_dims.long_edge() > resolved.max_long_edge`:
       - Downscale using `image::imageops::resize(&decoded_rgba, target_dims.width, target_dims.height, image::imageops::FilterType::Lanczos3)`.
       - Encode the resized RGBA buffer to valid PNG bytes using `PngEncoder`.
     - In the thumbnail branch (`if generate_thumbnail`):
       - Compute `thumb_dims = target_dims.compute_thumbnail_dimensions(320)`.
       - Downscale the target image buffer to `(thumb_dims.width, thumb_dims.height)` using `image::imageops::resize`.
       - Encode the thumbnail buffer to valid PNG bytes using `PngEncoder`.
       - Return `(Some(thumb_bytes), Some(thumb_dims))`.
     - Return `ReducedImageResult` with real encoded PNG bytes and dimensions.
3. **Integrate into Desktop Capture Command (`apps/desktop/src-tauri/src/commands/capture.rs`):**
   - In `capture_screen_region`:
     - Pass `captured_png_bytes` through `ImageReducer::reduce_image(&captured_png_bytes, orig_dims, &resolved, false)` (or `reduce_image_with_budget`).
     - Write the resulting reduced PNG bytes (`reduced_result.bytes`) to the Vault via `vault_store.write_blob`.
     - Record the actual reduced dimensions `reduced_result.dimensions` in the stored `Finding`.
4. **Implement / Update Tests in `crates/snapdown-store/tests/test_image_reduction.rs`:**
   - Add programmatic PNG test fixture helper `create_test_png(width: u32, height: u32, pattern: TestPattern) -> Vec<u8>`.
   - Implement the 5 required tests from `waves.yaml`:
     1. `cargo::a_reduced_image_decodes_and_its_pixels_are_the_scaled_source` — Assert that reduced PNG bytes decode back to `DynamicImage` and the pixels match the downscaled source patterns.
     2. `cargo::a_reduced_image_honours_the_resolved_long_edge` — Assert that a 3840x2160 source reduced with a 1600px max long edge produces a decoded image with long edge equal to 1600.
     3. `cargo::an_image_already_under_the_long_edge_is_not_upscaled` — Assert that a 400x300 source reduced with a 1600px max long edge stays 400x300.
     4. `cargo::a_thumbnail_decodes_and_is_smaller_than_its_source` — Assert that thumbnail bytes decode as valid PNG and dimensions are strictly smaller than original and target.
     5. `cargo::the_resolved_pair_arithmetic_is_unchanged_by_this_story` — Assert that pure dimension arithmetic functions (`compute_reduced_dimensions_for_pair`, `compute_reduced_dimensions_with_edge`, presets, `Auto` resolution) produce identical results.
   - Update existing test functions (`auto_derivation_varies_reduction_between_small_tooltip_and_4k_screen`, `fixed_presets_downscale_to_pinned_constants`, `custom_pair_reduction_honors_explicit_limits`) to provide valid programmatic PNG inputs.

## Boundaries & Constraints

**Always:**
- Keep `snapdown-core` strictly free of I/O or image codec dependencies.
- Preserve all existing dimension arithmetic in `snapdown-core::domain::image`.
- Use `image = "0.25.10"` via workspace dependency.
- Ensure all encoded outputs decode cleanly as valid standard PNG files with IHDR, IDAT, and IEND chunks.
- Preserve aspect ratio without cropping or stretching (`BR-41`).
- Refuse upscaling for images under the resolved long edge (`BR-40`).
- Generate all test image fixtures programmatically in memory.

**Block If:**
- Any requirement attempts to change the dimension arithmetic or preset constants.

**Never:**
- Never commit real screenshot image files or binaries to git.
- Never write 17-byte placeholder headers or fake PNG signatures.
- Never upscale an image smaller than the budget long edge.
- Never discard error results from image decoding or encoding.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Behavior / Output | Invariants & Guarantees |
|---|---|---|---|
| Large Image Downscaling (4K) | Valid 3840x2160 PNG, Balanced preset (1600 long edge) | Decodes, resizes to 1600x900 using Lanczos3, encodes valid PNG | Decodable real PNG, long edge == 1600; `FR-4`, `NFR-3`, `CAP-2` |
| Small Image (No Upscale) | Valid 400x300 PNG, Balanced preset (1600 long edge) | Decodes, skips downscaling, encodes valid PNG with 400x300 | `BR-40`; `an_image_already_under_the_long_edge_is_not_upscaled` |
| Thumbnail Generation | Valid 1600x1200 PNG, `generate_thumbnail: true` | Generates full reduced PNG plus 320x240 thumbnail PNG | Decodable thumbnail PNG, max edge <= 320; `a_thumbnail_decodes_and_is_smaller_than_its_source` |
| Non-Image / Corrupt Input | Garbage bytes (e.g. `[0u8; 100]`) | Returns `Err(CoreError::Validation("..."))` | No panic, clean structured error |
| Non-Square Aspect Ratio | Valid 3000x1000 PNG, long edge cap 1500 | Decodes, resizes to 1500x500 (3:1 aspect ratio preserved) | `BR-41` Aspect ratio preserved |
| Zero-Byte Placeholder Pipeline | Valid `VaultBlobStore` path and reduced PNG bytes | Reserves 0-byte file first (`NFR-2`), then writes full PNG bytes | Invariant hold in `reserve_and_write` |

</intent-contract>

## Code Map

- `crates/snapdown-store/Cargo.toml` — Declare dependency on workspace `image` crate.
- `crates/snapdown-store/src/image/pipeline.rs` — Implement real decode-scale-encode and thumbnail generation in `ImageReducer`.
- `crates/snapdown-store/tests/test_image_reduction.rs` — Comprehensive test suite implementing the 5 required tests from `waves.yaml` and updating existing reduction tests.
- `apps/desktop/src-tauri/src/commands/capture.rs` — Wire `ImageReducer::reduce_image` into `capture_screen_region` so real captured images are reduced before being saved to the Vault.

## Tasks & Acceptance

**Execution:**
1. `crates/snapdown-store/Cargo.toml` — Add `image = { workspace = true }`.
2. `crates/snapdown-store/src/image/pipeline.rs` — Replace fake 17-byte header and payload simulation with `image::load_from_memory`, aspect-ratio-preserving downscaling via `image::imageops::resize`, and real PNG encoding via `PngEncoder`.
3. `apps/desktop/src-tauri/src/commands/capture.rs` — Connect `ImageReducer::reduce_image` into `capture_screen_region` before writing blob to Vault.
4. `crates/snapdown-store/tests/test_image_reduction.rs` — Implement programmatic test image synthesiser and the 5 tests declared in `waves.yaml`:
   - `cargo::a_reduced_image_decodes_and_its_pixels_are_the_scaled_source`
   - `cargo::a_reduced_image_honours_the_resolved_long_edge`
   - `cargo::an_image_already_under_the_long_edge_is_not_upscaled`
   - `cargo::a_thumbnail_decodes_and_is_smaller_than_its_source`
   - `cargo::the_resolved_pair_arithmetic_is_unchanged_by_this_story`

**Acceptance Criteria:**
- `crates/snapdown-store/src/image/pipeline.rs` contains zero fake headers or simulated payload slicing.
- All reduced images and thumbnails decode cleanly using standard image readers (`image::load_from_memory`).
- `the_resolved_pair_arithmetic_is_unchanged_by_this_story` passes and proves that dimension calculation logic is 100% preserved.
- All 5 named tests in `waves.yaml` pass cleanly in `cargo test --workspace`.
- Mutation testing confirms tests fail when scaling or encoding invariants are intentionally broken.
- `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` exit with 0 errors/warnings.

## Spec Change Log

### 2026-08-24 — Initial Story Specification (Step 1: Plan Only)

- Created specification for `W8-S2` executing `CAP-2` and implementing `LC-003 image-reducer`.
- Replaced the 17-byte "downscaled payload simulation" in `crates/snapdown-store/src/image/pipeline.rs` with real image decoding (`image::load_from_memory`), Lanczos3 resizing (`image::imageops::resize`), and standard PNG encoding (`PngEncoder`).
- Preserved all dimension arithmetic and resolution algorithms in `snapdown-core`.
- Specified real thumbnail decode-scale-encode path.
- Detailed integration into desktop `capture_screen_region` command.
- Outlined the 5 tests required by `waves.yaml` and established programmatic fixture generation in test memory.

## Design Notes

**Downscaling Filter Selection:**
`image::imageops::FilterType::Lanczos3` is selected for main image downscaling as it produces sharp, high-quality results for text and UI elements across desktop resolutions. For thumbnail generation (320px bounding box), `FilterType::Triangle` (bilinear) or `Lanczos3` provides rapid scaling with clean downsampling.

**Lossless PNG Format:**
Since Snapdown captures and stores PNG images for clarity of desktop UI details and text legibility, standard PNG encoding preserves pixel fidelity while downscaling reduces total pixel footprint in accordance with the Quality Budget.

## Verification

**Commands:**
- `cargo fmt --all -- --check` — Clean formatting across all workspace crates.
- `cargo clippy --workspace --all-targets -- -D warnings` — 0 compiler or linter warnings.
- `cargo test -p snapdown-core --test test_no_io` — Confirms `snapdown-core` maintains zero I/O.
- `cargo test -p snapdown-store --test test_image_reduction` — All 5 image reduction tests pass.
- `cargo test --workspace` — Full workspace test suite passes.
