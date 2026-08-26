# Handover — refining the capture function

Self-contained brief. Do not expect to read the conversation that produced it.

- **Repo**: `D:\Developer\wiradigital.id\snapdown` (public), branch `main`
- **Baseline**: `030f886` — pushed, working tree clean under `apps/desktop` and `crates/`
- **Product**: Snapdown, a Slint desktop app on Windows. Note `AGENTS.md` still describes a
  Tauri/React app under `apps/desktop` — that is stale. It is Slint now; `DEC-007` covers the move
  and the old Tauri code sits in `archive/desktop-tauri/`.

## What the capture flow is

Pressing Capture (or `Ctrl+Shift+S`) grabs the whole virtual desktop, shows a frozen full-screen
overlay, and lets the Reviewer drag out a region, add a note, and save it as a Finding.

Files that matter:

| Path | Role |
|---|---|
| `crates/snapdown-capture/src/capturer.rs` | grabbing; `capture_virtual_desktop`, `capture_each_monitor`, `virtual_desktop_bounds` |
| `apps/desktop/ui/appwindow.slint` | `CaptureOverlayWindow` — scrim, selection, crosshair, loupe, note popup |
| `apps/desktop/src/main.rs` | `prewarm_capture_overlay`, the `on_capture_clicked` handler, `persist_finding` |
| `apps/desktop/tests/test_capture_overlay_fullscreen.rs` | structural guards for everything below |
| `.control/registry/defects.yaml` | `BUG-24`…`BUG-28` — read `BUG-27` and `BUG-28` before starting |

## Architecture, and why — do not undo these without reading the reasons

**ONE overlay window covering the whole virtual desktop.** It was one window per monitor. That
fixed mixed-DPI rendering but bought a defect class: each window had its own renderer to warm up
(the overlay blinked on the first pointer move of every capture) and its own event-loop turn (so
Escape cleared one screen at a time). Flameshot hit the same wall and went the other way, from one
canvas to per-monitor — for us per-monitor was the wrong end. See `BUG-26`, `BUG-27`.

**A single window costs nothing in quality.** For a per-monitor-DPI-aware process (the app manifest
declares PerMonitorV2) a window's surface maps 1:1 onto desktop pixels, so a canvas at native
physical resolution drawn full-bleed into a window sized in *physical* pixels is pixel-exact on
every monitor whatever their scale factors. The machine this was developed on has 3840x2160 @150%
next to a portrait 2160x3840 @175%.

**`show()` does not create the native window — the event loop does, on its next turn.** This one
fact caused every geometry bug this overlay had. Measured: `with_winit_window()` returns `None`
immediately after `show()`, and `window().scale_factor()` still reports `1.0` there. Consequences:

- Sizing from `scale_factor()` inflates the window by exactly the scale factor on the first capture
  of a session, and is correct from the second onwards. This shipped twice.
- With no size at all, the window is sized from the snapshot's intrinsic pixel size treated as
  *logical* — a 6000x3840 canvas asks for a 9000x5760 window at 150%.
- Slint's `set_size(Physical)` is no escape: it divides by that same unknown scale factor.
- The only reliable sizing is winit's `request_inner_size` / `set_outer_position`, which take
  device pixels, applied on a **later** event-loop turn.

**The overlay is created at start-up (`prewarm_capture_overlay`) and reused forever.** Renderer
warm-up and the geometry correction both happen only at creation, so they are paid where nobody is
looking. On Windows Slint's `hide()` is just `set_visible(false)` and leaves the window and its
renderer alive, which is what makes reuse work. It is rebuilt only if the desktop layout changes
(`placement` is the reuse key). The pre-warm timer is **400ms, not 0ms** — it is scheduled before
`run()` starts the loop, and 0ms fires before the window exists (measured: it stayed 800x600).

**GPU renderer, deliberately.** The software renderer removes the blink but cannot repaint a
full-screen 8-megapixel overlay per pointer move, so dragging a region becomes choppy.
`SLINT_BACKEND` is still honoured for A/B (`SLINT_BACKEND=software`).

**A region MAY span monitors** (Snagit does, filling gaps with black). The drag is bounded by the
canvas, not by a monitor. The black gap needs no code: the canvas is the desktop's bounding box and
the parts no monitor covers were never written to. **The crosshair is still confined** to the
monitor under the pointer — that is a separate, still-wanted behaviour, driven by the monitor
rectangles.

**Selection is reported to Rust in canvas pixels, not logical pixels**, so the crop indexes the
snapshot directly with no scale conversion.

## Dead ends — already tried, do not repeat

On the per-capture blink (`BUG-27`), all of these failed:

| Tried | Result |
|---|---|
| Opaque window background instead of transparent | changed only what colour the bad frame was |
| Rewriting the loupe to use `source-clip` | no effect on the blink |
| `image-rendering: pixelated` on the loupe | measurably **worse** |
| Replacing every `if` gate with `visible` (no subtree rebuild) | removed the per-monitor-entry blinks, not this one |
| Hiding the loupe entirely | blink remained — so the loupe is not the cause |
| Software renderer | removes the blink, but makes dragging choppy — rejected |

Also rejected: a static scrim plus a `source-clip`ped bright cut-out for the selection. It looks
like an optimisation but `source-clip-*` are integer **source** pixels while the selection sits at
fractional logical coordinates, so the image inside the selection visibly slides against the
backdrop while dragging. The four-rectangles-around-the-selection scrim is aligned by construction.
There is a comment in the `.slint` saying so; leave it there.

## What is open

**`BUG-28` — the overlay takes a visible moment to appear.** This is the main thing left. The cost
is preparing the image, not the overlay: two ~8.3-megapixel grabs (already parallelised, one thread
per monitor), a ~92MB stitched canvas, a copy into a Slint `SharedPixelBuffer`, then a texture
upload. Single-window made this **worse** — the per-monitor design needed no stitch — and that was
a deliberate trade. Untried ideas, cheapest first:

1. Kill the extra copy: keep one `SharedPixelBuffer` and overwrite its pixels between captures
   instead of building a new `slint::Image` each time. A new texture per capture is also the last
   untested suspicion in `BUG-27`.
2. Skip the stitch when there is only one monitor — the common case for most users.
3. A faster grab than xcap's (DXGI Desktop Duplication).
4. Show the overlay before the image is ready and fill it in. Changes the UX contract (the screen
   would no longer freeze instantly) — needs the owner's decision, do not just do it.

Smaller things noticed and never chased:

- The main window stays visible while the desktop is grabbed, so Snapdown appears in its own
  screenshot. The Tauri predecessor hid it first.
- `persist_finding` attributes the Finding to the monitor containing the region's top-left. For a
  region spanning monitors that is arbitrary but not wrong.

## House rules that bit hard here

- **`wdi-systematic-debugging`'s three-fix rule is real.** Four symptom-aimed fixes at the blink all
  failed; what worked was removing the structure that produced it. If two fixes fail, stop and
  investigate instead of trying a third.
- **Measure, do not reason, about window geometry and frames.** Everything true in this document was
  established by measurement. Two useful tools: `SLINT_DEBUG_PERFORMANCE=refresh_lazy,console` (a
  debug build has a console; release does not, being windows-subsystem), and enumerating the
  process's windows through Win32 with `SetProcessDpiAwarenessContext(-4)` — without that, a
  non-DPI-aware measurement reports virtualised coordinates and lies.
- **Automation cannot drive the release binary.** Synthetic clicks do not reach it. The debug build
  responds to `orca computer click`. Geometry can be verified programmatically; flicker, sharpness
  and drag feel need the owner's eyes — ask, do not assume.
- **Guards must be seen failing.** Every structural guard in the test file was verified by mutation.
  Two of them were themselves wrong first: one matched its own explanatory comment (the helper now
  strips comments), another matched `main.show()` instead of `overlay.show()`.
- Never commit a captured screenshot. The repo is public and `korpus.yml` refuses tracked images.
- A stale `Snapdown.exe` locks its own file and fails the next build with *Access is denied*. Stop
  the process first.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

31 tests pass at `030f886`. `cargo build --release -p snapdown-desktop` for something to hand the
owner — debug is roughly 8x slower in xcap's pixel conversion, enough to change what "slow" means.
