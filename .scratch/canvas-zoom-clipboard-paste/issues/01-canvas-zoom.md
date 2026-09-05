# 01: Zoom the canvas in, out, and back to natural size

**What to build:** In the Editor's canvas toolbar, the Reviewer gets a zoom-in control, a zoom-out
control, and a "return to natural size" control. Using any of them changes only how big the canvas is
drawn on screen — every Marker and every visual annotation stays visually in place on the image at
every zoom level, because they are already stored as `[0,1]` fractions and rescale automatically with
the viewport. Nothing in `crates/snapdown-core`, `library.db`, or the Vault changes as a result of
zooming, at any level, ever.

**Blocked by:** None (can start immediately).

**Status:** done

Realizes `FR-34`. Governs `UC-2`, `UC-27` (control, not a use case of its own — `no_uc`). See
`.scratch/canvas-zoom-clipboard-paste/spec.md` for the full design (Implementation Decisions §
"Zoom").

## Seam

`AppWindow.canvas-zoom: <float>` (new, default `1.0`), read only by `canvas-viewport`'s width/height
expression in `apps/desktop/ui/appwindow.slint` (currently `active-image.width/height * 1px` at lines
~2078-2079). Three new callbacks: `zoom-in-clicked()`, `zoom-out-clicked()`, `zoom-reset-clicked()`,
handled in `apps/desktop/src/main.rs` via `.on_zoom_in_clicked(`/`.on_zoom_out_clicked(`/
`.on_zoom_reset_clicked(`, each ultimately calling `win.set_canvas_zoom(...)`.

## Acceptance criteria

- [ ] `canvas-zoom` is an `AppWindow` property, initialized to `1.0`, and `canvas-viewport`'s size
      expression multiplies the existing `active-image.width/height * 1px` by it.
- [ ] Three new `IconButton`s sit in the toolbar's existing "Canvas Action" group: zoom in, zoom out,
      return to natural size — no new toolbar group is created.
- [ ] Rust registers real handlers for all three new callbacks (not stubs — `println!`-only bodies are
      not acceptable per this repo's `KNOWN_STUBS` convention). Zoom-in/out apply a fixed step and
      clamp to a sane range (cannot reach zero, negative, or an unbounded runaway value); reset sets
      `canvas-zoom` to exactly `1.0`.
- [ ] `canvas-zoom` is never read by `finding_store`, never serialized to `library.db` or a `Setting`,
      and resets to `1.0` whenever a different Finding becomes active — it is pure per-view UI state.
- [ ] The stale "one canvas pixel is one image pixel, unconditionally" comments at
      `apps/desktop/ui/components/annotation.slint:20-22` and `appwindow.slint` (~1245-1246, ~3014-3016)
      are corrected to note this holds only at `canvas-zoom == 1.0` ("natural size").
- [ ] A wiring test (new, following `test_annotation_wiring.rs`'s idioms — `flat()`, comment-stripping
      for absence assertions) proves: the three callbacks are declared in `appwindow.slint`;
      `canvas-viewport`'s size expression contains `root.canvas-zoom`; each of
      `on_zoom_in_clicked(`/`on_zoom_out_clicked(`/`on_zoom_reset_clicked(` exists in `main.rs` and its
      body calls `set_canvas_zoom(`.
- [ ] `test_ui_callbacks_reach_rust.rs`'s existing generic scan passes for all three new callbacks with
      no changes to that test file itself (they were never excused, so nothing there needs editing).
- [ ] A pure Rust unit test over the zoom-step/clamp arithmetic proves: zooming in then out by the same
      number of steps returns to exactly the starting value; reset always yields exactly `1.0`
      regardless of the current value; the clamp bounds hold at both extremes (repeated zoom-in beyond
      the max stays at the max; repeated zoom-out below the min stays at the min).
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` are all green, including every new test above actually
      seen to fail first for the reason it's meant to catch (per this repo's own test-writing rule),
      then pass.
