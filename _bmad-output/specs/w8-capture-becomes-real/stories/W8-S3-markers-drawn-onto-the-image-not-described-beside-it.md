---
id: W8-S3
title: "W8-S3: Markers drawn onto the image, not described beside it"
type: 'feature'
wave: W8
status: ready-for-dev
created: '2026-08-24'
review_loop_iteration: 0
followup_review_recommended: false
dependencies:
  - W8-S2
files:
  - crates/snapdown-store/Cargo.toml
  - crates/snapdown-store/src/image/burner.rs
  - crates/snapdown-store/tests/test_marker_burner.rs
context:
  - _bmad-output/specs/w8-capture-becomes-real/SPEC.md
  - _bmad-output/specs/w8-capture-becomes-real/stories.yaml
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .control/registry/components.yaml
  - .control/reports/ASSESS-BUG-14.md
  - .what/business-rules.md
  - .what/finding/05-scenarios/SCN-04-the-note-line-deleted-without-its-marker.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/bundle/SDD-bundle.md
  - .how/finding/SDD-finding.md
  - .constitution/project/codebase-stack-guide.md
  - web/ui/src/styles/tokens.css
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Context & Defect Rationale (`BUG-14`, `CAP-3`, `LC-012`):**
Snapdown's bundle export promises that a composed bundle carries numbered marker badges drawn directly onto the screenshot images (`FR-8`), ensuring that annotations survive leaving Snapdown and are legible on any external platform or reader without requiring access to Snapdown's internal state.

However, since Wave W2, `crates/snapdown-store/src/image/burner.rs:20` has been a simulation:
```rust
pub fn burn_markers(
    input_bytes: &[u8],
    dimensions: &ImageDimensions,
    markers: &[Marker],
) -> Result<Vec<u8>, CoreError> {
    let mut output = Vec::new();
    output.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    output.extend_from_slice(&dimensions.width.to_be_bytes());
    output.extend_from_slice(&dimensions.height.to_be_bytes());
    output.push(markers.len() as u8);
    for m in markers {
        output.push(m.ordinal as u8);
        let px = ((m.x * (dimensions.width as f64)).round() as u32).to_be_bytes();
        let py = ((m.y * (dimensions.height as f64)).round() as u32).to_be_bytes();
        output.extend_from_slice(&px);
        output.extend_from_slice(&py);
    }
    if input_bytes.len() > 16 {
        output.extend_from_slice(&input_bytes[16..]);
    } else {
        output.extend_from_slice(input_bytes);
    }
    Ok(output)
}
```
This writes a fake header, appends marker ordinals and normalized pixel coordinates as unformatted binary metadata bytes, copies raw payload bytes, and **never draws anything onto the image**. Because previous tests only checked the PNG magic signature and length (`test_marker_burner.rs`), this simulation survived undetected.

This story replaces `MarkerBurner` with a genuine image badge rasterizer implementing `LC-012 marker-burner`.

---

### Architectural Invariants & Governing Constraints

1. **`AD-4` — Single Reduction at Capture, No Re-Reduction During Burn:**
   An image is reduced exactly once, at capture time, under the Quality Budget. No original unreduced image is retained. The marker burner operates on the **already-reduced image bytes** and MUST NOT re-scale or re-reduce them. Re-scaling or altering the resolution during marker burning would violate `AD-4` and destroy `AD-9`'s byte-identity guarantees.
2. **`AD-9` — Byte-Identity for Unmarked Images:**
   When `markers` is empty, or when no markers qualify to be drawn (e.g. all markers have empty note lines under `SCN-04`), `MarkerBurner::burn_markers` MUST return the input `input_bytes` **completely unchanged byte-for-byte** (`input_bytes.to_vec()`). Re-encoding an unmarked image through a codec alters compression tables and chunk timestamps, breaking byte-identity for clean findings.
3. **`AD-10` — Theme-Invariant Marker Styling:**
   Marker badge colors are deliberately theme-invariant across light and dark desktop themes because burned images are exported artifacts shared across different machines, viewers, and lighting environments. Marker colors MUST match the theme-invariant token values in `web/ui/src/styles/tokens.css`:
   - Badge Fill: `--color-marker` (`#f59e0b` / RGB `[245, 158, 11]`)
   - Text / Number Glyph: `--color-marker-text` (`#000000` / RGB `[0, 0, 0]`)
   - Outer Ring / Border: `--color-marker-ring` (`#ffffff` / RGB `[255, 255, 255]`)
   The burn process MUST NOT consult system or application theme preferences.
4. **`SCN-04` — Asymmetry: Markers with No Note Line Are NEVER Drawn:**
   Under scenario `SCN-04`, when a Reviewer deletes a note line in the note editor, the marker remains on the interactive canvas in the application, and the note pane reports the mismatch. However, **a marker with no note line (`comment.trim().is_empty()`) is NEVER drawn onto the exported image**. An app-only editing state must not be burned into an external artifact.
5. **Dimensional Integrity (`a_burned_image_keeps_the_dimensions_of_its_source`):**
   The burned output image must have the exact same pixel width and height as the input image.
6. **Pixel Modification & Spatial Locality:**
   - Burning a valid marker with a note line MUST produce an image that decodes as a valid image and differs from the source image in pixel values (`a_burned_image_decodes_and_differs_from_its_source_in_pixels`).
   - The pixel modifications MUST occur specifically at the pixel coordinates corresponding to the marker's normalized `(x, y)` location (`a_burned_marker_changes_pixels_at_its_own_coordinates`).
7. **No Committed Screenshots / Programmatic Test Fixtures:**
   All test images must be generated programmatically in code (e.g., solid color or grid pattern `RgbaImage` buffers encoded to PNG in memory). No real screenshot files may be committed to git.
8. **Mutation Testing Acceptance Criterion:**
   All 5 named tests must be validated by mutation testing: invert or alter the logic, verify that the test fails (red), and restore the implementation (green).

---

## Approach

1. **Add `image` Dependency to `crates/snapdown-store`:**
   In `crates/snapdown-store/Cargo.toml`, add `image = { workspace = true }`.
2. **Implement Real Marker Rasterization in `crates/snapdown-store/src/image/burner.rs`:**
   - **Filter Eligible Markers:**
     Filter `markers` to keep only those with non-empty comments:
     `let active_markers: Vec<&Marker> = markers.iter().filter(|m| !m.comment.trim().is_empty()).collect();`
   - **Fast Path for Zero Active Markers (`AD-9`):**
     If `active_markers.is_empty()`, immediately return `Ok(input_bytes.to_vec())`.
   - **Decode Input Image:**
     Decode `input_bytes` using `image::load_from_memory(input_bytes)`. Return `CoreError::Validation` if decoding fails or if the decoded dimensions do not match the expected `dimensions`. Convert to `image::RgbaImage`.
   - **Rasterize Badges:**
     For each active marker:
     - Map normalized coordinates `(m.x, m.y)` to pixel center coordinates:
       `cx = (m.x * (img_width as f64)).round() as i32`, `cy = (m.y * (img_height as f64)).round() as i32`.
     - Define badge dimensions relative to image scale (e.g. fixed radius ~14-16px with clamp, matching the 28px/1.75rem UI badge, with 2px white ring).
     - Draw outer white circular ring (`#ffffff`).
     - Draw inner amber circular disc (`#f59e0b`).
     - Draw centered black digit glyph (`#000000`) for the marker ordinal (e.g. `1`..`99`) using a built-in bitmap glyph rasterizer (e.g., standard 5x7 or 3x5 font table) centered at `(cx, cy)`.
   - **Encode Output PNG:**
     Encode the modified `RgbaImage` to PNG bytes in memory via `image::codecs::png::PngEncoder` (or `image::ImageOutputFormat::Png`).
3. **Implement Full Test Suite in `crates/snapdown-store/tests/test_marker_burner.rs`:**
   Implement all 5 tests declared in `waves.yaml`:
   1. `cargo::a_burned_image_decodes_and_differs_from_its_source_in_pixels`
   2. `cargo::a_burned_marker_changes_pixels_at_its_own_coordinates`
   3. `cargo::a_burned_image_keeps_the_dimensions_of_its_source`
   4. `cargo::a_marker_with_no_note_line_is_never_drawn_on_the_image`
   5. `cargo::burning_no_markers_returns_the_source_bytes_unchanged`

---

## Boundaries & Constraints

**Always:**
- Return input bytes byte-for-byte unchanged when no markers or no eligible markers are present (`AD-9`).
- Render badges with theme-invariant colors (`#f59e0b` disc, `#ffffff` ring, `#000000` text) (`AD-10`).
- Skip markers with empty or whitespace-only comments/note lines (`SCN-04`).
- Keep output image dimensions identical to input image dimensions (`AD-4`).
- Encode output as standard, compliant PNG byte streams with valid IHDR, IDAT, and IEND chunks.
- Programmatically synthesise all image test fixtures in code.

**Block If:**
- Upstream changes attempt to re-reduce or resize the image during marker burning (`AD-4` violation).

**Never:**
- Never commit screenshot image files or binary fixtures to the repository.
- Never write fake 17-byte headers or append raw marker coordinates as trailer bytes.
- Never draw a marker that has no note line (`SCN-04`).
- Never consult system theme or CSS theme variables during server/store-side marker rasterization.
- Never modify or re-encode image bytes when zero markers are burned (`AD-9`).

---

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Behavior / Output | Invariants & Guarantees |
|---|---|---|---|
| Zero Markers | `markers: &[]`, valid PNG bytes | Returns `Ok(input_bytes.to_vec())` identical byte-for-byte | `AD-9`, `burning_no_markers_returns_the_source_bytes_unchanged` |
| Marker with Empty Comment | Marker with `comment: ""` or `"   "` | Marker is ignored; if no other markers, returns input bytes unchanged | `SCN-04`, `a_marker_with_no_note_line_is_never_drawn_on_the_image` |
| Single Marker with Note Line | Marker at `(0.5, 0.5)` with `ordinal: 1`, `comment: "Fix CTA"` | Output decodes as PNG, dimensions match, pixels at center changed to badge colors | `CAP-3`, `a_burned_image_decodes_and_differs_from_its_source_in_pixels` |
| Multi-Marker Burn | Multiple markers at distinct coordinates | All eligible markers rasterized at respective `(x, y)` locations | Spatial accuracy, `a_burned_marker_changes_pixels_at_its_own_coordinates` |
| Edge Coordinates | Marker at `(0.0, 0.0)` or `(1.0, 1.0)` | Badge rasterization safely clamped within canvas bounds without buffer overflow | Pixel safety and boundary clamping |
| Invalid / Corrupt Input Bytes | Corrupt or non-image byte slice | Returns `Err(CoreError::Validation(...))` | Clean error reporting, no panic |

</intent-contract>

## Code Map

- `crates/snapdown-store/Cargo.toml` — Add `image = { workspace = true }` dependency.
- `crates/snapdown-store/src/image/burner.rs` — Implementation of `MarkerBurner` (`LC-012`):
  - In-memory rasterization of theme-invariant circular badges (`#f59e0b`, `#ffffff`, `#000000`).
  - Embedded numeric digit bitmap renderer for badge ordinals (`1`..`99`).
  - `SCN-04` marker filter (omitting markers without note lines).
  - `AD-9` zero-marker passthrough.
  - Standard PNG image decoding and re-encoding.
- `crates/snapdown-store/tests/test_marker_burner.rs` — Test suite implementing the 5 required tests from `waves.yaml`.

## Tasks & Acceptance

**Execution:**
1. `crates/snapdown-store/Cargo.toml` — Add `image` dependency from workspace.
2. `crates/snapdown-store/src/image/burner.rs` — Implement `MarkerBurner::burn_markers`:
   - Filter out markers where `comment.trim().is_empty()` (`SCN-04`).
   - If empty, return `input_bytes.to_vec()` (`AD-9`).
   - Decode PNG, rasterize circular ring, filled disc, and centered number glyphs.
   - Encode to standard PNG and return bytes.
3. `crates/snapdown-store/tests/test_marker_burner.rs` — Implement and verify all 5 tests:
   - `cargo::a_burned_image_decodes_and_differs_from_its_source_in_pixels`
   - `cargo::a_burned_marker_changes_pixels_at_its_own_coordinates`
   - `cargo::a_burned_image_keeps_the_dimensions_of_its_source`
   - `cargo::a_marker_with_no_note_line_is_never_drawn_on_the_image`
   - `cargo::burning_no_markers_returns_the_source_bytes_unchanged`
4. Validate test sensitivity through mutation testing.

**Acceptance Criteria:**
- `MarkerBurner` no longer writes 17-byte synthetic headers or raw coordinate byte trailers.
- All 5 named tests pass in `cargo test -p snapdown-store --test test_marker_burner`.
- Zero-marker burn returns identical bytes without codec roundtrip (`AD-9`).
- Markers with no note line are excluded from image burn (`SCN-04`).
- `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` pass cleanly with 0 errors/warnings.

## Spec Change Log

### 2026-08-24 — Initial Story Specification (Step 1: Plan Only)

- Created specification for `W8-S3` executing `CAP-3` and implementing `LC-012 marker-burner`.
- Formulated the plan to replace the fake header / raw coordinate trailer simulation in `MarkerBurner` with true image badge rasterization.
- Embedded governing architectural constraints: `AD-4` (single reduction, no re-scaling), `AD-9` (strict byte-identity for zero markers), `AD-10` (theme-invariant token colors `#f59e0b`, `#ffffff`, `#000000`), and `SCN-04` (never drawing markers with no note lines).
- Defined the 5 required tests from `waves.yaml` and mutation validation criteria.

## Design Notes

**Embedded Bitmap Glyphs for Ordinals:**
To keep `snapdown-store` free of heavy TrueType font rendering engines or font file assets while ensuring deterministic pixel output across all operating systems, `MarkerBurner` uses a simple, clean bitmap glyph table for digits `0`..`9`. This allows crisp, centered numbers `1`..`99` within the circular badge.

**Zero-Marker Fast-Path for AD-9:**
Codecs (such as PNG encoders) can produce variations in compressed bytes across encoder versions or runs even on identical pixel data. Returning `input_bytes.to_vec()` when no active markers are present guarantees exact byte-for-byte preservation for unmarked findings.

## Verification

**Commands:**
- `cargo fmt --all -- --check` — Clean formatting across workspace crates.
- `cargo clippy --workspace --all-targets -- -D warnings` — 0 compiler or linter warnings.
- `cargo test -p snapdown-store --test test_marker_burner` — All 5 marker burner tests pass.
- `cargo test --workspace` — Full workspace test suite passes.