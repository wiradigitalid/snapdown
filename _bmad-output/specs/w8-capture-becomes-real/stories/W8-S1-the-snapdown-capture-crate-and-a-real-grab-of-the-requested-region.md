---
id: W8-S1
title: "W8-S1: The snapdown-capture crate, and a real grab of the requested region"
type: 'feature'
wave: W8
status: ready-for-dev
created: '2026-08-24'
review_loop_iteration: 0
followup_review_recommended: false
dependencies: []
files:
  - Cargo.toml
  - crates/snapdown-capture/Cargo.toml
  - crates/snapdown-capture/src/lib.rs
  - crates/snapdown-capture/src/error.rs
  - crates/snapdown-capture/src/capturer.rs
  - crates/snapdown-capture/tests/test_capture.rs
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/commands/capture.rs
context:
  - _bmad-output/specs/w8-capture-becomes-real/SPEC.md
  - _bmad-output/specs/w8-capture-becomes-real/stories.yaml
  - _bmad-output/specs/w8-capture-becomes-real/dispatch-briefs/W8-S1-step1-plan.md
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .control/registry/components.yaml
  - .control/reports/ASSESS-BUG-14.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/finding/SDD-finding.md
  - .how/finding/04-components/LC-003-image-reducer.md
  - .how/finding/06-flows/flow-capture.md
  - .what/finding/SRS-finding.md
  - .what/business-rules.md
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Context & Defect Rationale (`BUG-14`, `CAP-1`, `LC-002`):**
Snapdown has never taken a real screenshot. Since Wave W2, the desktop capture command in `apps/desktop/src-tauri/src/commands/capture.rs:197` has relied on `generate_placeholder_image`, writing a 17-byte synthetic payload:
```rust
fn generate_placeholder_image(width: u32, height: u32, encoder_quality: u8) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"\x89PNG\r\n\x1a\n");   // 8-byte PNG signature
    bytes.extend_from_slice(&width.to_be_bytes());   // 4 bytes width
    bytes.extend_from_slice(&height.to_be_bytes());  // 4 bytes height
    bytes.push(encoder_quality);                     // 1 byte quality
    bytes
}
```
This payload contains no IHDR, no IDAT chunks, and no IEND trailer. It is not an image and cannot be decoded by any image reader. Because every test in prior waves checked only width/height dimensions from headers rather than decoding pixels, this simulation persisted undetected across 5 waves and 3 audits.

Furthermore, `ARCHITECTURE-SPINE.md:245` explicitly allocated `crates/snapdown-capture` (`CAP-1`, `CAP-2`) and `components.yaml` registered logical component `LC-002 region-capturer` in area `capture-pipeline`. However, `crates/snapdown-capture` was never created.

This story executes the G3 architectural decision: it introduces `crates/snapdown-capture` to implement `LC-002`, selects and pins the third-party dependencies for the entire wave, removes `generate_placeholder_image`, and wires real screen capture and real PNG encoding into `capture_screen_region`.

---

### Dependency Decision for Wave W8

This story makes and records the dependency decision for the entire wave. These are the first third-party dependencies added to the workspace since its creation.

| Crate | Pinned Version | License | Role & Rationale |
|---|---|---|---|
| `xcap` | `0.9.8` | `Apache-2.0` | **Cross-Platform Screen Capture:** Native Rust monitor and window capture engine. On Windows, leverages DXGI / GDI / WinRT Graphics Capture. Clean, safe idiomatic Rust API providing monitor enumeration (`Monitor::all()`) and monitor grabs returning RGBA pixel buffers. Permissive Apache-2.0 license is fully compatible with Snapdown's MIT public repository. |
| `image` | `0.25.10` | `MIT OR Apache-2.0` | **Image Processing & Real Codec:** The de facto standard Rust imaging toolkit. Provides standard-compliant PNG encoding with proper chunk framing (IHDR, IDAT, IEND), dynamic region cropping (`image::imageops::crop`), pixel buffer operations, and image decoding. Dual MIT/Apache-2.0 license is fully compatible with Snapdown. Workspace configuration uses `default-features = false` with `features = ["png"]` (and image operations) to avoid compiling unneeded formats. |

**Headless & Displayless Environment Behavior (CI / Virtual Machines):**
In automated CI environments (such as GitHub Actions runners or containerized test hosts) or virtual machines without an active display monitor:
- Hardware monitor enumeration (`xcap::Monitor::all()`) returns an empty list `Ok(vec![])` or encounters an environment where no display server/adapter is attached.
- When no monitor is found, hardware grab operations MUST NOT panic and MUST NOT fall back to generating fake byte sequences. Instead, the capture engine returns an explicit, structured error: `CaptureError::NoDisplayFound`.
- For tests that validate capturing, cropping, encoding, and dimension invariants, tests MUST NOT depend on hardware framebuffers. They must use programmatic synthetic test frames (drawing multi-color test patterns, noise grids, or gradients into an `image::RgbaImage`) to test cropping and real PNG encoding deterministically without requiring a physical monitor.
- Live hardware grab tests must explicitly handle headless environments by asserting `CaptureError::NoDisplayFound` or testing device discovery gracefully, never silently passing on dummy data.

---

### Key Architectural Constraints & Invariants

1. **`snapdown-core` Zero-I/O Invariant:**
   `crates/snapdown-core` is strictly pure domain and port interfaces. It has no I/O, no OS API access, and no screen capture dependencies. The existing workspace integrity test `snapdown_core_has_no_io_dependency` (`crates/snapdown-core/tests/test_no_io.rs`) MUST continue to pass cleanly. `crates/snapdown-capture` depends on `snapdown-core`, never the reverse.
2. **`BR-31` Region Validation:**
   Any requested capture region smaller than 8×8 pixels is invalid and MUST be rejected with an error (`CaptureError::InvalidRegion` / `Err("Region must be at least 8x8 pixels")`).
3. **Preservation of Existing Dimension Arithmetic:**
   The resolution arithmetic in `snapdown_core::domain::image::ImageDimensions` (`compute_reduced_dimensions_for_pair`, `compute_reduced_dimensions_with_edge`, `Auto` budget derivation from `W6-S4`, fixed presets `Sharp`, `Balanced`, `Small`) is correct and MUST be preserved without alteration. `W8-S1` produces the unscaled real capture of the requested region; `W8-S2` will handle the decode-scale-encode pipeline.
4. **Monitor Bounds & Multi-Monitor Geometry:**
   The capture request specifies a `source_monitor` (e.g. `"DISPLAY1"` or monitor identifier) and physical pixel bounding box `(x, y, width, height)`.
   - The region is mapped onto the physical pixel coordinate space of the specified monitor.
   - If the requested region extends beyond the physical bounds of the monitor (`x < 0`, `y < 0`, `x + width > monitor.width`, `y + height > monitor.height`), the capture MUST be **refused with an explicit error (`CaptureError::RegionExceedsMonitorBounds`), not silently clamped**.
5. **No Tracked Screenshots / Programmatic Test Fixtures:**
   Per repository policy and the product brief, no captured screenshot, token, or fixture derived from a real capture may be committed. All test fixtures must be programmatically synthesised in memory (e.g. generating distinct RGBA geometric patterns or gradients).
6. **Mutation Testing Acceptance Criterion:**
   Each test written must be verified by mutation: invert or break the assertion / logic, observe the test fail (red), and restore the correct implementation (green).

---

## Approach

1. **Update Workspace `Cargo.toml`:**
   - Add `"crates/snapdown-capture"` to `workspace.members`.
   - Add `xcap = "0.9.8"` to `[workspace.dependencies]`.
   - Add `image = { version = "0.25.10", default-features = false, features = ["png"] }` to `[workspace.dependencies]`.
2. **Create `crates/snapdown-capture`:**
   - Define `crates/snapdown-capture/Cargo.toml` importing `snapdown-core`, `xcap`, `image`, `thiserror`.
   - Implement `CaptureError` in `error.rs` covering `NoDisplayFound`, `MonitorNotFound(String)`, `RegionExceedsMonitorBounds { requested: String, monitor: String }`, `InvalidRegion(String)`, `CaptureFailed(String)`, `EncodingFailed(String)`.
   - Implement `RegionCapturer` in `capturer.rs` implementing `LC-002`:
     - Method `capture_region(region: &Region, source_monitor: Option<&str>) -> Result<Vec<u8>, CaptureError>`.
     - Method `crop_and_encode_image(source: &RgbaImage, region: &Region) -> Result<Vec<u8>, CaptureError>` (enabling programmatic testing and headless validation).
     - Standard PNG encoder using `image::codecs::png::PngEncoder` or `image::ImageOutputFormat::Png`.
3. **Integrate into `apps/desktop/src-tauri`:**
   - Add `snapdown-capture = { path = "../../../crates/snapdown-capture" }` to `apps/desktop/src-tauri/Cargo.toml` dependencies.
   - In `apps/desktop/src-tauri/src/commands/capture.rs`:
     - Remove `generate_placeholder_image`.
     - Invoke `snapdown_capture::capture_region` (or `RegionCapturer`) to capture the requested screen region as real PNG bytes.
     - Write the real PNG bytes to the Vault via `vault_store.write_blob`.
4. **Implement the 4 Required Tests:**
   In `crates/snapdown-capture/tests/test_capture.rs` (and integration tests):
   1. `cargo::a_captured_region_decodes_as_a_real_image` — Assert that captured and encoded bytes decode via `image::load_from_memory` into a valid `DynamicImage` / `RgbaImage` with valid dimensions.
   2. `cargo::a_captured_region_has_the_dimensions_that_were_requested` — Assert that cropping a region of `W × H` yields an image whose decoded pixel buffer is exactly `W × H`.
   3. `cargo::a_captured_image_is_not_uniformly_one_colour` — Assert that capturing a multi-color patterned canvas yields non-uniform pixel values across the decoded image buffer (preventing blank/uniform grab regressions).
   4. `cargo::a_region_larger_than_the_monitor_is_refused_not_clamped_silently` — Assert that requesting a region exceeding monitor dimensions returns `Err(CaptureError::RegionExceedsMonitorBounds)` rather than silently clamping to monitor dimensions.

## Boundaries & Constraints

**Always:**
- Keep `snapdown-core` strictly free of I/O or capture dependencies.
- Pin `xcap = "0.9.8"` and `image = "0.25.10"` in `Cargo.toml`.
- Encode real, valid PNGs (with valid IHDR, IDAT, and IEND chunks).
- Reject regions smaller than 8×8 (`BR-31`).
- Refuse regions larger than the target monitor with an explicit error.
- Programmatically synthesise all image test fixtures in code.

**Block If:**
- Upstream changes require `snapdown-core` to perform I/O.

**Never:**
- Never commit real screenshot image files or binaries to git.
- Never write 17-byte placeholder headers or fake PNG signatures.
- Never silently clamp out-of-bounds capture regions.
- Never let headless/no-display environments silently pass fake data as a real grab.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Behavior / Output | Invariants & Guarantees |
|---|---|---|---|
| Valid Region Capture | Region `(x: 100, y: 100, w: 400, h: 300)` on active monitor | Captures frame, crops `400 × 300` pixel rect, encodes standard PNG | Decodable real image, exact dimensions; `CAP-1`, `LC-002` |
| Small Region Refusal | Region `(w: 4, h: 4)` | Returns `Err("Region must be at least 8x8 pixels")` | `BR-31` enforced before grab |
| Region Exceeds Monitor | Region `(w: 5000, h: 4000)` on a `1920 × 1080` monitor | Returns `Err(CaptureError::RegionExceedsMonitorBounds)` | Must NOT clamp silently; `a_region_larger_than_the_monitor_is_refused_not_clamped_silently` |
| Headless / No Display (CI) | `Monitor::all()` returns empty slice | Returns `Err(CaptureError::NoDisplayFound)` | No panic, no fake image buffer |
| Multi-Color Pattern Image | Programmatic test pattern (e.g. stripes/gradients) | Decoded image contains variance across pixel values | `a_captured_image_is_not_uniformly_one_colour` |
| Image Decoding Roundtrip | Valid captured PNG bytes | `image::load_from_memory(&bytes)` returns `Ok(DynamicImage)` with matching dimensions | `a_captured_region_decodes_as_a_real_image` |

</intent-contract>

## Code Map

- `Cargo.toml` — Add `crates/snapdown-capture` to workspace members; add `xcap` and `image` to workspace dependencies.
- `crates/snapdown-capture/Cargo.toml` — Package manifest for `snapdown-capture` with dependencies on `snapdown-core`, `xcap`, `image`, `thiserror`.
- `crates/snapdown-capture/src/lib.rs` — Module declarations and public API re-exports for `RegionCapturer` and `CaptureError`.
- `crates/snapdown-capture/src/error.rs` — Domain error type `CaptureError` implementing `thiserror::Error`.
- `crates/snapdown-capture/src/capturer.rs` — Implementation of `RegionCapturer` (`LC-002`) providing monitor resolution, screen grabbing, cropping, and PNG encoding.
- `crates/snapdown-capture/tests/test_capture.rs` — Unit and integration test suite implementing the 4 required tests from `waves.yaml`.
- `apps/desktop/src-tauri/Cargo.toml` — Add `snapdown-capture` dependency.
- `apps/desktop/src-tauri/src/commands/capture.rs` — Replace `generate_placeholder_image` with real capture call from `snapdown-capture`.

## Tasks & Acceptance

**Execution:**
1. `Cargo.toml` — Register `crates/snapdown-capture` member and declare workspace dependencies `xcap = "0.9.8"`, `image = "0.25.10"`.
2. `crates/snapdown-capture/` — Create crate implementing `RegionCapturer` (`LC-002`) and `CaptureError`.
3. `apps/desktop/src-tauri/src/commands/capture.rs` — Remove `generate_placeholder_image` and connect `capture_screen_region` to `snapdown_capture::RegionCapturer`.
4. `crates/snapdown-capture/tests/test_capture.rs` — Implement the 4 named tests:
   - `cargo::a_captured_region_decodes_as_a_real_image`
   - `cargo::a_captured_region_has_the_dimensions_that_were_requested`
   - `cargo::a_captured_image_is_not_uniformly_one_colour`
   - `cargo::a_region_larger_than_the_monitor_is_refused_not_clamped_silently`

**Acceptance Criteria:**
- `generate_placeholder_image` is completely removed from `apps/desktop/src-tauri/src/commands/capture.rs`.
- `crates/snapdown-capture` exists and builds cleanly as part of the workspace.
- `cargo test --test test_no_io` continues to pass cleanly (verifying `snapdown-core` remains free of I/O).
- All 4 tests declared in `waves.yaml` pass cleanly in `cargo test --workspace`.
- `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` pass with 0 errors/warnings.

## Spec Change Log

### 2026-08-24 — Initial Story Specification (Step 1: Plan Only)

- Created specification for `W8-S1` executing `CAP-1` and implementing `LC-002 region-capturer`.
- Recorded the third-party dependency decisions: `xcap = "0.9.8"` (Apache-2.0) for native cross-platform screen capture, and `image = "0.25.10"` (MIT OR Apache-2.0) for standard PNG encoding and pixel buffer operations.
- Defined explicit error handling for headless / displayless execution environments (`CaptureError::NoDisplayFound`).
- Specified out-of-bounds region behavior (`CaptureError::RegionExceedsMonitorBounds`) to ensure out-of-bounds regions are refused rather than silently clamped.
- Outlined the 4 tests required by `waves.yaml` and established mutation testing acceptance criteria.

## Design Notes

**Separation of Hardware Grab vs Codec Operations:**
`RegionCapturer` exposes both `capture_region` (which queries active hardware monitors via `xcap`) and `crop_and_encode_image` (which operates on an in-memory `image::RgbaImage`). This allows automated unit tests to thoroughly verify region cropping, boundary checks, non-uniform pixel validation, and PNG encoding roundtrips deterministically in headless CI environments without hardware dependencies.

**Real PNG Encoding:**
Instead of 17 synthetic bytes, `snapdown-capture` encodes RGBA pixel buffers into valid PNG byte streams with standard headers, compression chunks, and end markers, enabling downstream decoders (`image::load_from_memory`) to parse and manipulate images reliably.

## Verification

**Commands:**
- `cargo fmt --all -- --check` — Clean formatting across all workspace crates.
- `cargo clippy --workspace --all-targets -- -D warnings` — 0 compiler or linter warnings.
- `cargo test -p snapdown-core --test test_no_io` — Confirms `snapdown-core` maintains zero I/O.
- `cargo test -p snapdown-capture` — All 4 capture tests pass.
- `cargo test --workspace` — Full workspace test suite passes.
