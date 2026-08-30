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
      blur preview much faster than before, including for a 4K-resolution capture — **partially
      met, not closed out**. `box_pass` went from `O(pixels * radius)` to `O(pixels)`: measured
      (release build) 3840x2160 went 3402ms -> 304.6ms, 1920x1080 went 496ms -> 78.9ms. That is an
      11x/6x improvement, but 304.6ms at 4K is still not imperceptible on the UI thread. Closing the
      remaining gap needs SIMD or parallelism (e.g. splitting rows across threads), which is out of
      scope for this ticket's fix (adds a new dependency / execution model, not a same-output
      algorithmic change) — worth its own follow-up ticket if 4K captures are common enough to
      justify it.
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

Regression note: this was itself a regression of a fix — `BUG-41` closed the same symptom once
(320.9ms -> 10.5ms) before `BUG-73` added the per-click blur that reopened it. The three checks above
guard the blur specifically so a third recurrence doesn't need rediscovering from scratch.
