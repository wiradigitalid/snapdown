---
id: SPEC-w12-panoramic-scrolling
title: Scrolling & Panoramic Screenshot (Auto-Scroll & Stitching)
status: draft
companions:
  - .what/finding/SRS-finding.md
  - .how/finding/SDD-finding.md
  - crates/snapdown-capture/src/capturer.rs
  - crates/snapdown-store/src/image/reducer.rs
  - apps/desktop/src/components/CaptureOverlay.tsx
sources:
  - .control/registry/requirements.yaml
---

# SPEC-w12: Scrolling & Panoramic Screenshot (Auto-Scroll & Stitching)

## 1. Intent & Context
Reviewing long web pages, lengthy terminal output, or multi-page documents requires stitching multiple viewport frames into a single vertical or horizontal finding image without seams or duplicate text lines.

## 2. Functional Requirements & Acceptance Criteria

### FR-SCROLL-1: Scroll Assist Triggers
- When the cursor hovers over a scrollable window or control (identified via `UIA_ScrollPatternId`), display dynamic scroll assist trigger buttons:
  - Downward arrow `?` on bottom edge (Vertical scroll).
  - Rightward arrow `?` on right edge (Horizontal scroll).
  - Diagonal arrow `?` on bottom-right corner (Both dimensions).
- Hovering the assist button previews the expected scroll direction.

### FR-SCROLL-2: Automated Scroll-and-Capture Loop
- Clicking the scroll assist arrow starts the automated capture sequence:
  1. Capture current visible frame `F_0`.
  2. Send synthesized scroll event via Win32 `SendInput` (`MOUSEEVENTF_WHEEL` / delta -120 or `WM_VSCROLL`).
  3. Wait settling period (default `60ms` for smooth-scrolling animations to settle).
  4. Capture next visible frame `F_i`.
  5. Repeat until either:
     - End of scrollable content is reached (subsequent frames show 0 pixel difference), OR
     - User presses `Escape` / `Stop` button, OR
     - Max scroll height cap (e.g. 15,000 pixels or 20 frames) is reached.

### FR-SCROLL-3: Pixel Stitching Engine
- The Rust stitching pipeline in `snapdown-capture::stitcher`:
  - Uses Normalized Cross-Correlation (NCC) / template matching over overlapping horizontal bands to calculate the exact vertical scroll displacement `dy`.
  - Seamlessly trims the overlap and concatenates slice buffers into a single `DynamicImage`.
  - Rejects stitching if alignment confidence is below threshold (<95%), reporting a graceful error rather than corrupted mosaic.

### FR-SCROLL-4: Reduction & Pipeline Integration
- The stitched composite image passes seamlessly into `ImageReducer` and `FindingStore`.
- The final result is saved as a single finding in the Vault with the full panorama dimensions.

## 3. Invariants & Non-Functional Constraints
- **NFR-PAN-1**: The stitched output must decode cleanly with valid dimensions matching `width × sum(slices)`.
- **NFR-PAN-2**: Memory safety: Image slice buffers must stream into a preallocated target buffer without unbounded memory growth.

## 4. Test Obligations
- `cargo::panoramic_stitcher_aligns_two_overlapping_frames_byte_accurately`
- `cargo::panoramic_stitcher_detects_end_of_scroll_on_identical_frames`
- `cargo::panoramic_stitcher_refuses_disjoint_images_without_panicking`
- `vitest::scroll_assist_arrow_mounts_on_scrollable_container_hover`
- `vitest::escape_key_stops_active_scrolling_capture`
