# 01: Ctrl+Scroll zoom, on top of the already-shipped zoom buttons

**What to build:** `FR-34`'s canvas zoom already shipped (`396550c`, toolbar buttons in the "Canvas
Action" group: zoom-in/zoom-out/reset, `CANVAS_ZOOM_MIN/MAX/STEP` = 0.25/4.0/0.25, resets to `1.0` on
Finding switch). This spec (`.scratch/post-testing-polish/spec.md`, User Stories 1-5) asks for a second
input path: Ctrl+Scroll over the canvas changes the zoom level the same way the existing buttons do;
plain Scroll (no Ctrl) must keep scrolling the canvas viewport exactly as it does today, unchanged.

**Diagnosis already done, do not re-derive:** User Story 4 ("a Marker I place while zoomed lands
exactly where my pointer is") is **already satisfied** by the existing code, not a gap. Marker
placement (`appwindow.slint:2313`) computes `self.mouse-x / parent.width` — both the click position and
`parent.width` scale by `canvas-zoom` together, so the ratio is zoom-invariant by construction. No
change needed here, and no `crates/snapdown-core` change either.

The button placement question (spec asks for them "beside the existing resolution/size readout";
shipped version put them in "Canvas Action") is **not a defect** — the spec itself calls the exact panel
"a builder decision, not a new one," and a working, shipped placement already exists. Leave it as
shipped unless this ticket's own build finds a concrete reason not to.

**Blocked by:** None (can start immediately). Touches only `apps/desktop/ui/appwindow.slint` and
`apps/desktop/src/main.rs` (the existing `zoomed_in`/`zoomed_out`/`CANVAS_ZOOM_*` from `396550c`) — wire
a scroll-wheel event with the Ctrl modifier to the same `set_canvas_zoom(zoomed_in/out(...))` calls the
buttons already use. Reuse, don't reimplement.

**Status:** done

## Seam

Wiring test alongside the existing `test_zoom_wiring.rs`: a Ctrl+Scroll event on the canvas viewport
calls `zoomed_in`/`zoomed_out` (not a copy of their arithmetic — reuse the existing pure functions and
their existing tests), and a plain Scroll event does not change `canvas-zoom`. Decode-based, not a
literal-value assertion.

## Acceptance

- [ ] Ctrl+Scroll up zooms in one step (`zoomed_in`), Ctrl+Scroll down zooms out one step
      (`zoomed_out`), using the exact same functions the toolbar buttons call — not a second
      implementation of the clamp/step arithmetic
- [ ] Plain Scroll (no Ctrl) still scrolls the canvas viewport exactly as before; `canvas-zoom` is
      unaffected by it
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] **Look at:** Ctrl+Scroll over the canvas at various zoom levels, confirm it feels responsive and
      the step size matches the buttons; confirm plain Scroll still pans/scrolls as before
