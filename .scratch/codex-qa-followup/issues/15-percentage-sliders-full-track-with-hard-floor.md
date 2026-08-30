# 15: Percentage sliders draw a full 0-100 track with a hard floor, not a narrowed range

**What to build:** The "Resize every capture to" and "Colour accuracy" sliders draw their track
across the full 0-100 range instead of starting at their domain floor (25 and 10 respectively), so a
value's position on the track matches what a Reviewer expects from a percentage. Dragging below the
floor stops the thumb dead at that point - it does not bounce back, and it does not let the value go
lower. The Properties panel's font-size slider (floor of 8) is explicitly out of scope and unchanged.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] The "Resize every capture to" slider's track visually spans 0-100%, and 50% sits at the visual
      middle
- [x] Dragging that slider's thumb left of the 25% mark stops it there; it does not move further and
      does not bounce
- [x] The "Colour accuracy" slider gets the same treatment, floor 10
- [x] The Properties panel's font-size slider is unchanged

## Comments

`SdSlider` gained two optional properties, `track-minimum`/`track-maximum`, defaulting to
`minimum`/`maximum` - so every existing caller that does not set them (font size, the pixel cap
slider) is pixel-for-pixel unchanged. Where they are set wider than `minimum`/`maximum`, the track
draws across the wider range while `at()` still clamps the resulting value to `[minimum, maximum]`,
which is what produces the hard stop rather than a bounce - the thumb's position is a function of
the clamped value, so a drag past the floor keeps producing the same value and the thumb simply does
not move again until the pointer returns past that point. A short tick mark is drawn on the track at
the floor's position, visible only when `track-minimum < minimum`, so the stop has a visible reason.
Wired to `track-minimum: 0; track-maximum: 100;` on the resize-percent and quality sliders only; the
max-long-edge slider (480-3840px, no natural 0 baseline) and the font-size slider are untouched.
Confirmed working by the owner.
