---
id: W3-S3
title: Marker burning pipeline onto exported screenshot images
wave: W3
status: done
created: 2026-08-23
dependencies:
  - W3-S1
files:
  - crates/snapdown-store/src/image/burner.rs
  - crates/snapdown-store/src/image/mod.rs
  - crates/snapdown-store/tests/test_marker_burner.rs
---

# W3-S3: Marker burning pipeline onto exported screenshot images

## User Story
As a user exporting documentation bundles, I want numbered marker badges (with high-contrast circular badges and numbers 1..N) to be burned directly onto the exported screenshot images, so that recipients without the Snapdown app can view the annotations seamlessly.

## Acceptance Criteria
- [ ] Implement `MarkerBurner` in `crates/snapdown-store/src/image/burner.rs`.
- [ ] Render circular badges with contrast borders and numbers for each marker based on relative `[0.0, 1.0]` coordinates.
- [ ] Support burning onto raw or decoded image buffers and returning encoded PNG/WebP bytes.
- [ ] Maintain source image dimensions and aspect ratios.
- [ ] Automated tests in Rust (`cargo test`) covering marker drawing, coordinate boundary handling, and image encoding.
