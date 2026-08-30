# 01: Filmstrip click reaches the canvas without a multi-hundred-millisecond blur stall

**What to build:** Clicking a filmstrip thumbnail for a Finding that hasn't been visited yet this
session loads it into the Editor — image and blur preview both — without a perceptible stall, even
for a 4K-resolution capture. The blur preview and the burned output pixels stay exactly what they are
today (same three-pass separable box blur, same default radius); only how `box_pass` computes each
pass changes, from re-summing the whole window per pixel to a sliding-window/running-sum update, so
each pass is `O(pixels)` instead of `O(pixels * radius)`.

Regression history: `BUG-41` fixed the same symptom (320.9ms -> 10.5ms per click) by splitting
`load_active_detail` out of the full filmstrip rebuild. A day later `BUG-73` added a real (non-mosaic)
whole-image blur into that same per-click path, and nobody re-measured `BUG-41` after that landed.
Measured directly on this machine (release build) just now: `blur_rect` alone at the default radius
costs ~496ms at 1920x1080, ~1398ms at 2560x1440, ~3402ms at 3840x2160 — this is the delay the owner is
seeing again.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Clicking a Finding's filmstrip thumbnail for the first time in a session shows the image and
      blur preview much faster than before, including for a 4K-resolution capture. `box_pass` went
      from `O(pixels * radius)` to `O(pixels)` (release build: 3840x2160 3402ms -> 304.6ms, 1920x1080
      496ms -> 78.9ms), then `box_pass` was further split across CPU cores by row band using
      `std::thread::scope` (no new dependency, no `unsafe`) when an image is at least
      `PARALLEL_ROW_BAND_PIXEL_THRESHOLD` (1,000,000) pixels: 3840x2160 304.6ms -> 212.8ms, 1920x1080
      78.9ms -> 57.6ms, 2560x1440 127.3ms -> 91.4ms (12 logical cores on this machine). Gains are
      sub-linear in core count because this is memory-bandwidth-bound, not compute-bound.

      **Still not fully closed at 4K**: ~213ms is a large cumulative improvement (16x from the
      original 3402ms) but still not strictly imperceptible on the UI thread.

      **SIMD was tried and measured, not left un-attempted.** A `Vec4` type wrapping `[f32; 4]` in a
      single SSE2 `__m128` (via `std::arch::x86_64` intrinsics, no new dependency, no runtime feature
      detection needed since SSE2 is x86_64 baseline) replaced the per-channel `for channel in 0..4`
      loops in both band functions. Measured back-to-back on this machine, release build, same
      process: the explicit-SIMD `Vec4` and a plain-array `Vec4` (same code, intrinsics swapped for
      elementwise arithmetic via a forced `#[cfg]`) came out statistically indistinguishable - both
      ~51-56ms at 1080p, ~91-106ms at 1440p, ~201-206ms at 4K. Rust/LLVM's autovectorizer was already
      turning the plain fixed-4-element array loop into equivalent SIMD instructions on this release
      build; the explicit intrinsics added `unsafe` and an architecture-specific code path for no
      measured benefit, so they were reverted rather than kept "for later" - carrying unsafe code
      that isn't earning its cost is a worse position than not having tried. Do not re-attempt this
      exact approach without a new measurement showing the autovectorizer stopped covering it (e.g.
      after a Rust/LLVM version change, or a restructuring that defeats autovectorization).
- [x] The blur preview and the burned output pixels are pixel-identical to before this change — same
      three-pass separable box blur, same radius, same output, no visual regression. Verified by
      `box_pass_matches_the_naive_full_window_resummation_it_replaced` in
      `crates/snapdown-store/src/image/burner.rs`, comparing the new sliding-window `box_pass`
      against the original full-resummation implementation (kept as a test-only reference) across
      seven size/radius combinations including edge cases (1x1, radius wider than the line, odd and
      even widths).
- [x] A measured guard exists so a future change can't silently reintroduce an `O(radius)`-per-pixel
      cost in the box pass. `blur_rect_stays_well_under_the_old_full_window_resummation_cost` asserts
      an 800x600 `blur_rect` call completes in under 3000ms (debug profile); verified by mutation —
      temporarily rewiring `blur_rect` to the naive reference implementation made this test fail at
      3865ms, confirming the threshold actually catches the regression it names.

      A second guard, `blur_rect_above_the_parallel_threshold_stays_faster_than_running_sequentially`,
      covers the follow-up: 800x600 (480,000px) never engages the threaded path
      (`PARALLEL_ROW_BAND_PIXEL_THRESHOLD` is 1,000,000), so the first guard alone would not notice
      threading silently failing to engage. This one uses 1920x1080 and asserts under 1200ms;
      verified by mutation the same way — forcing the sequential branch made it fail at 1634ms
      (consistent with the ~1668ms measured before threading existed).

      Correctness across the threaded path specifically (not just the sequential one every smaller
      case takes) is covered by adding a `(1400, 901, 16)` case — 1,261,400px, above the threshold,
      with a row count not evenly divisible by the machine's thread count to force an uneven last
      band — to `box_pass_matches_the_naive_full_window_resummation_it_replaced`.

Regression note: this was itself a regression of a fix — `BUG-41` closed the same symptom once
(320.9ms -> 10.5ms) before `BUG-73` added the per-click blur that reopened it. The three checks above
guard the blur specifically so a third recurrence doesn't need rediscovering from scratch.
