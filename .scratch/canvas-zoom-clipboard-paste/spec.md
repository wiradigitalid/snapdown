Status: done

Corrected 2026-09-05 under `DEC-023`: both tickets (`01-canvas-zoom`, `02-paste-from-clipboard`) were
already shipped to `main` on 2026-09-04 (`396550c`, `d97b82e2`) before this mandate opened. This
`ready-for-agent` line was simply never updated after that work landed — verified independently by two
separate builders and by the coordinator (`git merge-base --is-ancestor` against `origin/main`), not
re-implemented.

# Canvas zoom and paste-from-clipboard

Synthesized from the corpus, no interview held — both requirements are already fully specified and
carry `no_uc` (they are controls inside existing use cases, not new promises). Unattended-run
authority (`DEC-018`, this repo's `AGENTS.md` § Unattended runs) applies: the seam check below and
the ticket breakdown were decided by this run rather than by a human quiz, and are recorded as such
in this file rather than in a separate `owner-questions.md`, since neither choice was one only the
owner could make — both follow directly from what the corpus and the existing code already commit to.

## Problem Statement

The Reviewer works with a canvas that is always drawn at exactly one image pixel per canvas pixel.
Inspecting a fine detail — a single character of a tooltip, a thin border — means squinting at a
1:1 image with no way to get closer, and there is no way back to natural size once any other zoom
mechanism (OS accessibility zoom, monitor scaling) is used to compensate. Separately, an image that
already exists on the Windows clipboard — copied from a browser, a chat client, anywhere — has no way
into Snapdown except saving it to a file first and using Import; there is a dead "Paste" button in
the toolbar that visibly does nothing, which is worse than no button at all.

## Solution

**Zoom (`FR-34`).** The Reviewer can zoom the canvas in and out and return it to natural size, from a
new control in the canvas toolbar's existing "Canvas Action" group. Zoom is a pure view-layer
transform: it changes how big the canvas is drawn, never what is stored. Markers and annotations are
already stored as `[0,1]` fractions of the image (`AD-3`) and are already positioned in
`apps/desktop/ui/components/annotation.slint` and the marker `ReticleMarker` wrapper as a fraction of
their immediate parent's rendered size — so scaling the one property that currently pins the canvas to
`active-image.width/height * 1px` (`appwindow.slint:2078-2079`) rescales the image, every annotation,
and every marker together, for free, with no change anywhere in `crates/snapdown-core`. Zoom realizes
no use case of its own (`no_uc`); it is how the Reviewer looks at a Finding inside `UC-2` (repeated
Capture) and `UC-27` (visual annotation).

**Paste (`FR-35`).** The Reviewer can press the existing (currently inert) Paste button, and if the
Windows clipboard holds an image, Snapdown reads it, reduces it under the Quality Budget exactly the
way a Capture or an Import does, and it appears as a new Finding at the top of the filmstrip — reusing
`persist_finding`, the same function the existing Import feature (`on_open_file_clicked`) already
calls, so there is exactly one reduction path in the product (`AD-4`: an image is reduced exactly once,
at capture, and no original is kept) rather than a second one built for Paste. Paste is a second entry
point into `UC-1` (`no_uc`): from the point pixels arrive onward — reduction, storage, filmstrip — the
flow is UC-1's own, unchanged.

## Seams (recorded, not quizzed — see header)

- **Zoom** is tested at the Slint-declaration seam (`test_annotation_wiring.rs`'s own convention): a
  wiring test asserts the new zoom control is instantiated in `appwindow.slint` and that
  `canvas-viewport`'s width/height binding is the one that reads the new `canvas-zoom` property (proof
  the transform is wired to the actual viewport, not decorative), plus the callback-reachability seam
  (`test_ui_callbacks_reach_rust.rs`) for its Rust handler(s). This is the highest seam available:
  there is no lower seam, because `crates/snapdown-core` has no concept of zoom at all and must not
  gain one.
- **Paste** is tested at two seams: the same callback-reachability seam for `paste-clicked` (moving it
  out of `DELIBERATELY_UNHANDLED`), and a decode-and-persist seam mirroring the existing
  `encode_region_for_clipboard` split (`main.rs:4003-4032`) — a new function takes already-fetched
  clipboard bytes (never the live OS clipboard) and calls `persist_finding`, so the test can fabricate
  BMP bytes in memory exactly the way the existing clipboard-write test does, with no dependency on
  what happens to be on a CI machine's real clipboard.

## User Stories

1. As a Reviewer inspecting a Finding with fine detail, I want to zoom the canvas in, so that I can
   read small text or see a thin border precisely.
2. As a Reviewer who has zoomed in, I want to zoom back out, so that I can see the whole capture again.
3. As a Reviewer at any zoom level, I want a single action that returns the canvas to exactly natural
   size (one image pixel per canvas pixel), so that I don't have to hunt for the exact original level.
4. As a Reviewer, I want every Marker and every visual annotation to stay visually on the same spot of
   the image at every zoom level, so that zooming never looks like it moved my markup.
5. As a Reviewer, I want zooming to never change the Finding's stored image or any Marker/annotation's
   stored coordinates, so that zoom is safe to use freely and never risks the underlying data.
6. As a Reviewer, I want the zoom control discoverable in the canvas toolbar without hunting, so that
   I don't need to be told it exists.
7. As a Reviewer, I want an image already on my Windows clipboard to become a new Finding when I press
   Paste, so that I don't have to save it to a file first and use Import.
8. As a Reviewer, I want a pasted image reduced under the same Quality Budget a Capture would use, so
   that pasted Findings behave and cost disk the same way captured ones do.
9. As a Reviewer, I want a pasted Finding to appear in the filmstrip immediately, selected, the same
   way a fresh Capture or Import does, so that I can start writing its Note right away.
10. As a Reviewer, I want to be told clearly if I press Paste with no image on the clipboard, rather
    than have nothing visibly happen, so that the button never again looks broken.
11. As a Reviewer, I want the Paste button to actually work now, so that the toolbar has no dead
    controls in it.

## Implementation Decisions

**Zoom**

- A new `in-out property <float> canvas-zoom: 1.0` on `AppWindow`, read only by `canvas-viewport`'s
  size expression (`appwindow.slint` ~2078-2079), changed from
  `root.active-image.width * 1px` / `...height * 1px` to the same expression multiplied by
  `root.canvas-zoom`. `main-img`, `AnnotationItem` (100%/100% of the viewport), and the marker wrapper
  Rectangles all already read percentages of their immediate parent, so no other Slint file changes
  position math.
- Zoom UI sits in the toolbar's existing "Canvas Action" group (`appwindow.slint` ~1985 onward),
  reusing `IconButton` (zoom-in / zoom-out, matching the existing Undo/Redo icon-button pair's sizing)
  plus a control that returns to natural size in one action — a third `IconButton` is sufficient and
  keeps parity with how Undo/Redo are already two buttons plus nothing extra; a slider is not required
  by the proof line and is more surface than the requirement asks for.
- New callbacks on `AppWindow`: `zoom-in-clicked()`, `zoom-out-clicked()`, `zoom-reset-clicked()`.
  Rust registers `on_zoom_in_clicked` / `on_zoom_out_clicked` / `on_zoom_reset_clicked`, each reading
  `win.get_canvas_zoom()`, computing the next value (a fixed step; clamp to a sane range so zoom-out
  cannot reach zero or negative and zoom-in cannot run away unbounded), and calling
  `win.set_canvas_zoom(next)`. `zoom-reset-clicked` sets it to exactly `1.0` — the corpus's own phrase
  for this is "natural size" (`annotation.slint:20-22`, `appwindow.slint:1245/3014`,
  `requirements.yaml` FR-34's own title) and that phrase is used in code/comments/UI text in preference
  to "100%" or "fit", to stay consistent with what the rest of the codebase already calls it.
  `canvas-zoom` is pure UI state: it is never read by `finding_store`, never serialized, and is not a
  `Setting` — it resets to `1.0` each time a different Finding becomes active (view state belongs to
  the view being looked at, not to the Reviewer's Library-wide preference; nothing in `FR-34`'s proof
  asks it to persist across Findings).
- Nothing in `crates/snapdown-core` changes. `writes: []` on `FR-34`'s own registry row already says
  so, and this is a mechanical consequence of markers/annotations already being normalized fractions —
  it is not a design choice made for this ticket, it is a fact the existing schema already established
  (`AD-3`).
- The stale "one canvas pixel is one image pixel, unconditionally" comments at
  `annotation.slint:20-22` and `appwindow.slint:1245-1246`/`3014-3016` are corrected in the same change
  to say that this holds only when `canvas-zoom == 1.0` — the code changed under them and the comments
  must catch up, per this repo's own "documents follow the code" convention (this is a code comment,
  not a corpus document, so no `wdi-*` skill is involved).

**Paste**

- `paste-clicked()` already exists as a Slint callback (`appwindow.slint:1467`) and an `IconButton` in
  the toolbar (`appwindow.slint:1888-1894`); neither changes shape. Only its Rust wiring and its
  listing in `test_ui_callbacks_reach_rust.rs`'s `DELIBERATELY_UNHANDLED` change.
- A new `#[cfg(any(windows, test))]` function, `decode_clipboard_image_bytes(bmp: &[u8]) -> Result<(Vec<u8>, u32, u32), String>`, decodes already-fetched clipboard bytes into RGBA8 (`image::load_from_memory(bmp)?.to_rgba8()`), mirroring `encode_region_for_clipboard`'s existing split
  (`main.rs:4003-4032`) between "bytes in hand → testable transform" and "touch the real OS clipboard".
  It never calls into the Windows clipboard itself, so it is unit-testable with fabricated BMP bytes
  exactly the way `encode_region_for_clipboard`'s own test already fabricates them
  (`main.rs` test near line 7217).
- A new `#[cfg(windows)]` function, `paste_clipboard_image(ctx: &AppContext) -> Result<String, String>`,
  does the real work: checks the clipboard actually holds `CF_BITMAP`
  (`clipboard_win::raw::is_format_avail(clipboard_win::formats::CF_BITMAP)`) and returns a clear,
  specific error if not (addresses User Story 10 — no silent no-op); otherwise reads it
  (`clipboard_win::get_clipboard::<Vec<u8>, _>(clipboard_win::formats::Bitmap)`), decodes via
  `decode_clipboard_image_bytes`, and calls the existing `persist_finding(ctx, rgba.as_ref(), (w,h),
  (0,0,w,h), "Pasted", "")` — the exact call shape `on_open_file_clicked` (Import) already uses at
  `main.rs:6566`. On `Some(finding_id)` the caller (the callback registration) calls
  `load_findings_into_window(&win, ctx, Some(&finding_id))` (the same full filmstrip-refresh-and-select
  call every other Finding-creating path already uses — no new filmstrip API is invented) and a
  success `toast`; on `None` or an `Err`, an error `toast`. A `#[cfg(not(windows))]` stub mirrors the
  existing `copy_burned_image`/`copy_region_to_clipboard` non-Windows fallbacks: returns a fixed
  "implemented on Windows only" error string, for the same reason those already do (this whole feature
  is a Windows-clipboard feature; the product does not build for another OS today).
- `paste-clicked`'s entry is deleted from `DELIBERATELY_UNHANDLED` in
  `test_ui_callbacks_reach_rust.rs:64-67` in the same commit that wires the handler — leaving it listed
  once handled fails that test's own release-half-of-the-ratchet check
  (`every_ui_callback_has_a_rust_handler_or_is_declared_unreachable`, lines 82-114).
- Nothing about the Quality Budget, `ImageReducer`, or `prepare_region` is touched. Paste calls
  `persist_finding`, which already calls `prepare_region` internally exactly as Capture and Import do;
  a second reduction pipeline is exactly what `AD-4` and this dispatch's brief forbid, and nothing here
  introduces one.

## Testing Decisions

Only test **external behavior** (a Finding lands in the store, a Slint property is bound to the
control that changes it, a callback is reachable) — never a copy of the implementation's own internal
values, per this repo's own hard-won convention against exactly that failure mode.

- **Zoom wiring test** (new, in `test_annotation_wiring.rs`'s style, either appended there or as its
  own `test_zoom_wiring.rs` — same idioms: `flat()` to survive `rustfmt` rewrapping,
  `code_only()`/comment-stripping where an absence is asserted): asserts (a) the three zoom callbacks
  are declared in `appwindow.slint`, (b) `canvas-viewport`'s width/height expression contains
  `root.canvas-zoom` (proof the transform reaches the actual viewport, not a decorative property that
  nothing reads), and (c) each of `on_zoom_in_clicked(`, `on_zoom_out_clicked(`,
  `on_zoom_reset_clicked(` exists in `main.rs` and its body calls `set_canvas_zoom(` (not a stub).
- **`test_ui_callbacks_reach_rust.rs`** gains no new content for zoom beyond what already checks every
  declared callback automatically — the three new callbacks are covered by its existing generic scan,
  which is exactly the point of that test.
- A **behavioral zoom-in/zoom-out/reset test** (Rust unit test over the pure clamp/step arithmetic that
  computes the next `canvas-zoom` value from a click, independent of any Slint runtime) — asserts
  zooming in then out returns to the exact starting value (not a lossy round-trip), reset always
  produces exactly `1.0` regardless of the current value, and the clamp bounds hold at the extremes.
  This is the seam-appropriate proof for User Story 4/5's "never moves" claim: since markers/
  annotations are stored as fractions and the Slint layer already proves (by construction, per the
  research above) that a percentage-of-parent position is invariant under a parent resize, the
  Rust-side test only needs to prove the *arithmetic* Rust owns (the next zoom value) is correct;
  proving Slint's percentage layout math is out of scope (it is the Slint engine's own behavior, not
  this feature's code).
- **Paste callback reachability**: `test_ui_callbacks_reach_rust.rs`'s existing generic test now
  passes for `paste-clicked` once wired, and its own ratchet (a handled callback must not still be
  listed as deliberately unhandled) is the proof the exclusion was correctly removed — no separate new
  test is needed for this half.
- **`decode_clipboard_image_bytes` unit tests** (prior art: the existing `encode_region_for_clipboard`
  test near `main.rs:7217`, which fabricates RGBA bytes, encodes, then decodes and asserts equality):
  fabricate valid BMP bytes in memory (via the `image` crate's own BMP encoder, or by round-tripping
  through the existing write-side encoder) and assert the function decodes them to the expected
  RGBA8 dimensions/pixels; separately assert it returns an `Err` (not a panic) on garbage bytes.
- **`paste_clipboard_image`-level integration test**: following the pattern already used by Import's
  own test (find and reuse `library_test_ctx`, the same in-memory/tempdir `AppContext` builder every
  other persistence test in `main.rs` already uses), call the paste path with fabricated clipboard-BMP
  bytes injected at the `decode_clipboard_image_bytes` seam (never touching the real OS clipboard) and
  assert: (a) exactly one Finding now exists in `ctx.finding_store` carrying those pixels, (b) its
  stored image's long edge fits the active Quality Budget, matching the existing image-reduction test
  pattern for `NFR-3`/`NFR-18` rather than a fresh assertion invented for this ticket, and (c) the
  Finding's `Note` is empty and its origin marker (whatever short label Import already uses, e.g.
  "Imported: …") is set to a Paste-specific label instead, so a Reviewer can tell a pasted Finding from
  an imported one if that distinction is ever surfaced. Decode the actual output image rather than
  asserting a signature and a size, per this repo's own standing rule against exactly that mistake.
- **No-image-on-clipboard behavior**: a test asserting `paste_clipboard_image` (or the seam just below
  the real clipboard read) returns a clear, specific `Err` rather than silently doing nothing, and that
  no Finding is created by it — addresses User Story 10.

## Out of Scope

- Any change to `crates/snapdown-core` — both requirements are proven to need none.
- Persisting `canvas-zoom` as a `Setting`, across app restarts, or across switching Findings — not
  asked for by `FR-34`'s proof line.
- A zoom keyboard shortcut, scroll-wheel zoom, or pinch gesture — the proof line asks only that the
  canvas *can* be zoomed in/out/reset; the corpus does not name an input method and none of these three
  requirements-yaml entries do either. A future `FR-37`-style right-click/keyboard entry point is a
  separate, later requirement if the owner asks for one.
- A drag-to-pan capability when zoomed past the viewport's visible area — the existing `ScrollView`
  already scrolls a canvas larger than the window (this is exactly how an unzoomed large capture is
  handled today), so nothing new is needed for panning; this spec does not add one.
- Any clipboard format other than a Windows bitmap/DIB (`CF_BITMAP`) — e.g. `CF_HDROP` (a copied file)
  or `CF_UNICODETEXT` — is out of scope. `FR-35`'s proof line says "an image held on the Windows
  clipboard"; a copied file path is a different, unspecified feature.
- Non-Windows builds of paste — the whole clipboard-write side is already Windows-only
  (`#[cfg(windows)]` throughout `main.rs`), and this stays consistent with that existing boundary.
- Reversing `crop`/destructive resize (named non-goals, `OQ-29`) — unrelated to either FR here and not
  reopened by this work.

## Further Notes

- `FR-34` and `FR-35` are both `no_uc` by design (no `UC-` entry is created or edited for either); this
  spec cites `UC-1`, `UC-2`, and `UC-27` only as the existing flows each control sits inside, per the
  registry's own `no_uc` explanations — nothing in `.what/finding/04-usecases/` is touched.
  `UC-2`/`UC-27` are catalogue-only rows (`.how/_platform/inventory-screen.md` /
  `.control/generated/rtm.md`), not full narrative use-case files, at this component's current mode —
  that is expected and is not a gap this work needs to close.
- Both requirements are `component: finding`, `mode: guarded`/`deep` (the corpus's `c4-l3-desktop-app`
  currently records `finding` at `mode: deep`, stricter than the `guarded` named in this work's
  dispatch brief; the stricter of the two governs, and nothing here is written at less than `deep`
  would demand), `risk_accepted: low` — a two-reviewer code panel convention applies; this spec's
  author will request a separately-dispatched `/code-review` pass in addition to any self-review before
  calling either ticket done.
- Neither FR's proof line, nor anything found during exploration, conflicts with the corpus. No
  conflict is reported per this run's rule 2 (the corpus is not the builder's to change) because none
  was found.
