---
reviewer: reviewer-a
model: 9router/combo
date: 2026-08-23
story: W3-S3
verdict: ACCEPTED
---

# Code Review: W3-S3 — Marker burning pipeline onto exported screenshot images

## Scope & Implementation Review
- **Marker Burning Pipeline**: `MarkerBurner` in `crates/snapdown-store/src/image/burner.rs` translates normalized `[0.0, 1.0]` coordinates to pixel offsets on the image dimension and burns marker annotations onto output buffers.
- **Export Integrity**: Preserves original image dimensions and aspect ratios (`INV-IMAGE-001`).
- **Automated Tests**: Integration tests in `crates/snapdown-store/tests/test_marker_burner.rs` verify correct PNG encoding and buffer augmentation for multiple markers.

## Invariant Adherence
- `INV-IMAGE-001` (Aspect Ratio & Dimension Preservation): Verified.
- `INV-MARKER-003` (Export Badge Fidelity): Verified marker coordinate calculation.

## Verdict
ACCEPTED.
