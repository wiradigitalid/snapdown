---
id: W2-S3
title: Quality budget, thumbnail generation, and async reduction pipeline
wave: W2
status: done
created: 2026-08-23
dependencies:
  - W2-S1
  - W2-S2
files:
  - crates/snapdown-core/src/domain/image.rs
  - crates/snapdown-store/src/image/mod.rs
  - crates/snapdown-store/src/image/pipeline.rs
  - crates/snapdown-store/tests/test_image_reduction.rs
---

# W2-S3: Quality budget, thumbnail generation, and async reduction pipeline

## User Story
As a tester creating findings with high-resolution screenshots, I want captured images to be validated and converted (with thumbnails and quality-budget downscaling) through a robust image pipeline so that disk storage remains controlled without blocking the main workflow.

## Acceptance Criteria
- [ ] Implement image reduction helper functions with `image` crate (downscale, quality reduction, thumbnail creation).
- [ ] Support quality budget constraints (e.g. max width/height/file size) according to settings.
- [ ] Maintain aspect ratio during thumbnail generation.
- [ ] Ensure non-blocking async execution or threadpool delegation for image processing.
- [ ] Full automated test suite in Rust (`cargo test`) covering downscaling, compression, and error fallback.
