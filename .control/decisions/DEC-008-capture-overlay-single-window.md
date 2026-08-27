---
type: decision
id: DEC-008
status: draft
touches: []
supersedes: null
superseded_by: null
created: "2026-08-26"
---

# DEC-008 — The capture overlay is one GPU-rendered window over the whole virtual desktop, built at start-up and reused

## Decision

The capture overlay MUST be exactly one window, covering the whole virtual desktop, sized and
positioned in physical pixels, rendered by Slint's default GPU renderer, created during application
start-up and reused for every capture until the desktop layout changes. A selected region MAY span
monitors; the crosshair guides MUST stay confined to the monitor under the pointer.

## Why

**This decision exists because four fixes aimed at the symptom failed.** The overlay blinked — the
whole screen flashing the window's clear colour for a frame — and the attempts to stop it were: an
opaque instead of transparent window background, rewriting the magnifier to sample through
`source-clip`, replacing every `if` gate with `visible` so no element subtree was rebuilt, and
`image-rendering: pixelated`. The first three changed nothing; the fourth made it measurably worse.
Hiding the magnifier entirely left the blink intact, ruling it out as the cause. What the blink
actually *was* has never been identified, and this decision does not claim to know. It removes the
conditions that produced it instead. That is the signal `wdi-systematic-debugging`'s three-fix rule
exists to raise, and it is recorded here rather than buried.

**One Win32 window has exactly one DPI.** The development machine runs a 3840x2160 landscape display
at 150% beside a portrait 2160x3840 at 175%. Microsoft's own high-DPI guidance is explicit that a
window spanning displays of differing scale factors is bitmap-scaled and appears blurry. The first
overlay was a single window stretched over a stitched canvas and rendered as one enormous zoomed
image, which read as proof that one window could not represent two DPIs.

**It was not proof.** The zoom came from how the window was sized, not from what one window can
represent. For a per-monitor-DPI-aware process — this app's manifest declares PerMonitorV2 — a
window's surface maps 1:1 onto desktop pixels, so a canvas at native physical resolution drawn
full-bleed into a window sized in *physical* pixels is pixel-exact on every monitor whatever their
scale factors. Saved output is unaffected either way, because crops are taken from that canvas.

**The intervening design — one overlay window per monitor — fixed the rendering and bought a defect
class.** Each window has its own renderer to warm up, so the overlay blinked on the first pointer
move of every capture; and each has its own event-loop turn, so Escape cleared one screen and left
the other on screen for a visible beat. Both are structural to having N windows. Measured, not
inferred: `SLINT_DEBUG_PERFORMANCE=refresh_lazy,console` reported one renderer initialisation per
window. Flameshot reached the opposite conclusion for the same class of problem — moving *from* one
canvas *to* one monitor at a time (`flameshot-org/flameshot#4495`, PR `#4498`) — but it was solving
mixed-DPI correctness, which the paragraph above makes obtainable without giving up a single window.

**One measured fact governs the geometry, and it caused every geometry bug this overlay had.**
`show()` does not create the native window; the event loop does, on its next turn. Immediately after
`show()`, `with_winit_window()` returns `None` and `window().scale_factor()` still reports `1.0`.
Therefore:

- Sizing from `scale_factor()` inflates the window by exactly the scale factor on the first capture
  of a session and is correct from the second onwards. This shipped twice.
- Given no size at all, the window is sized from the snapshot's intrinsic pixel count treated as
  *logical* — a 6000x3840 canvas asking for a 9000x5760 window at 150%.
- Slint's `set_size(Physical)` is no escape: it divides by that same not-yet-known scale factor.
- The only reliable sizing is winit's `request_inner_size` and `set_outer_position`, which take
  device pixels, applied on a later event-loop turn.

**The GPU renderer is kept deliberately.** Slint's software renderer removes the blink completely —
confirmed by testing — but cannot repaint a full-screen 8-megapixel overlay per pointer move, so
dragging out a region becomes choppy. Every attempt to make that cheap cost visual correctness: a
static scrim plus a `source-clip`ped bright cut-out slides against the backdrop while dragging,
because `source-clip-*` are integer *source* pixels while the selection sits at fractional logical
coordinates. Dragging is the core interaction of this feature; a brief blink is not worth trading it
for.

**Creating the overlay at start-up follows from the same measured fact.** A window's renderer surface
is built, and its geometry corrected, only at creation. Doing that on first Capture is what appeared
as the overlay growing into place on the non-primary monitor. On Windows Slint's `hide()` is
`set_visible(false)` and leaves the window and its renderer alive, so one window can serve every
capture. Verified after start-up and before any capture: the overlay exists hidden at
`pos=(-2160,-838) size=6000x3840`, already its final geometry.

**Cross-monitor regions are permitted on the owner's instruction, reversing an earlier constraint.**
The owner first required that a capture must not combine two monitors, then found that Snagit allows
exactly that, filling any gap with black, and asked for the same. Permitting it removed code rather
than adding it: the drag was pinned to the monitor it started on and clamped to that monitor's
bounds, and is now bounded by the canvas. The black gap needs no code at all — the canvas is the
desktop's bounding box, and the parts no monitor covers were never written to.

## Cost

- **Latency gets worse, and this is the accepted price.** A single window needs the whole desktop in
  one buffer, which the per-monitor design did not — on a 6000x3840 desktop, 92MB of it. The owner
  reports the overlay takes a visible moment to appear. Recorded as `BUG-28`, deliberately left open.

  Corrected after the fact, because the first version of this bullet named a cost that has since
  been removed: it read "a ~92MB allocation plus a blit per monitor, then a copy into a Slint
  buffer". There is no longer a stitched canvas at all — the monitors are blitted straight into the
  buffer that gets presented, which took preparation from 83-91ms to 36-38ms. What survives of the
  cost is the buffer itself: 33-37ms of that 36-38ms is `SharedPixelBuffer::new` alone. So the cost
  this decision incurs is *a full-desktop buffer*, not *a stitch*. `BUG-28` holds the measurements
  and what is still open.
- **Two behaviours that used to be free now need explicit arithmetic.** With one window per monitor,
  confining the crosshair and preventing a cross-monitor region were properties of the geometry —
  each window *was* one monitor. Both now depend on monitor rectangles carried into the overlay and
  reasoned about there, which is code that can be wrong.
- **The geometry path is coupled to `i-slint-backend-winit` internals.** `request_inner_size`,
  `set_outer_position` and the window-attributes hook are reached through `WinitWindowAccessor`, not
  through Slint's own public window API, because the public API cannot express this correctly before
  a window exists. A Slint or winit upgrade can break it, and the failure mode is visual rather than
  a compile error.
- **A full-desktop window and its canvas are resident for the process lifetime.** Reuse is what keeps
  the warm-up paid once; the memory is the price.
- **Start-up carries a 400ms timer.** Not tunable by feel: scheduled before `run()` starts the event
  loop, a 0ms timer fires before the window exists — measured, the overlay stayed at Slint's default
  800x600. The capture path can use 0ms because the loop is already running there.

## Alternatives

| Alternative | Why not |
| --- | --- |
| One overlay window per monitor | What this replaces. Correct rendering, but each window has its own renderer warm-up (the blink) and its own event-loop turn (Escape clearing one screen at a time). Four fixes aimed at those symptoms failed; they are structural to N windows |
| Single window, software renderer | Removes the blink entirely — confirmed — but cannot repaint a full-screen 8-megapixel overlay per pointer move, so dragging becomes choppy, and the optimisations that would make it cheap break visual correctness. Trades the core interaction for a transient artefact |
| Single window, each monitor's pixels resampled to the window's scale factor | Was the plan, and turned out unnecessary: a PerMonitorV2 window's surface already maps 1:1 onto desktop pixels, so resampling would have introduced softness to solve a problem that does not exist |
| Keep per-monitor windows and patch each symptom | Attempted. The blink survived four fixes and the staggered close survived two. Not converging, because each patch addressed a symptom while the structure that produced it remained |
| Build the overlay outside Slint — a raw Win32 layered window drawn with Direct2D or GDI | Maximum control, and roughly what ShareX and Snagit do. Rejected as disproportionate: it means a second UI technology in a codebase `DEC-007` just consolidated onto Slint, for one screen. Remains the escape hatch if the reversal triggers below fire |
| Capture only the monitor under the pointer, one small window | Cheapest and fastest, and Flameshot's answer. Rejected because the owner requires being able to move to another monitor and select there without re-triggering, and now also requires regions that span monitors |

## Reversal trigger

Any of these makes revisiting correct:

- `BUG-28`'s latency proves unacceptable in real use and every cheaper remedy is exhausted. Read
  `BUG-28` for what those are before concluding this decision is the thing that has to give — the
  original version of this trigger asserted that "dropping the stitch means dropping the single
  window", and that turned out to be **false**: the stitch was removed while the single window
  stayed, for a 47-53ms saving. Reusing one buffer across captures (measured at 4.3-4.6ms against
  the 36-38ms now shipping) and enabling xcap's `wgc` feature are both still untaken, and the grab
  itself — 132-167ms — now dominates what remains. This decision is the last thing to reconsider
  here, not the first.
- A monitor arrangement appears where the bounding-box canvas is mostly empty — widely separated or
  diagonally offset displays — making the stitch allocate far more than it captures.
- Slint gains a shared renderer across windows, or an atomic multi-window show/hide. Both costs this
  decision exists to avoid would disappear, and per-monitor windows would become the simpler option
  again.
- The `WinitWindowAccessor` geometry path is removed or changed by a Slint upgrade and no equivalent
  exists, since no Slint-level API can size a window correctly before it is created.

## Trace

| | |
| --- | --- |
| Defect register | `BUG-26` (mixed-DPI overlay, resolved) · `BUG-27` (the per-capture blink, resolved by removing its conditions, with the ruled-out list) · `BUG-28` (overlay latency, open, and the accepted cost of this decision) |
| Related decision | `DEC-007` moved the desktop UI to Slint, and its own reversal trigger already named "per-monitor DPI-correct overlay capture" as the capability that could reopen the framework choice. This decision settles that capability in Slint's favour without reopening it |
| Handover | `_bmad-output/handover-capture-refinement.md` carries the same architecture and its reasons, plus the dead-ends table, for the session continuing this work |
| Commits | `9277ecc` (per-monitor windows, reused) · `ab16569` (simultaneous close) · `31e391f` (single window) · `030f886` (cross-monitor regions, start-up pre-warm) |
| Note | `touches` is intentionally empty at `draft`. At `apply` this reaches `.how/_platform/ARCHITECTURE-SPINE.md`, `.how/_platform/c4-l3-desktop-app.md`, and the `capture` component's own `.what/`/`.how/` slots, each through its owning skill — none of it hand-edited to reach this record. It contradicts no `AD-N`: `AD-11` requires one process to own the tray, hotkeys, overlay and Editor, which one window strengthens rather than breaks, and `AD-3` governs stored Marker coordinates, whereas the canvas-pixel selection here is transient input to a crop |
