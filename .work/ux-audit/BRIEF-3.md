# Task — Prove BUG-4 is fixed in the running product

You drive the real UI. Do not edit application code. Do not rebuild — the binary is already built.

## Why this exists

`BUG-4` was critical: `capture.rs` opened the overlay window at `index.html?overlay=true`, nothing in
the frontend read `window.location.search`, and so **pressing the capture hotkey showed the Settings
shell instead of a capture overlay.** The capture path — the reason the product exists — did nothing.

`W6-S2` fixed it. `mount.test.tsx` proves the *mount decision* in jsdom. **Nothing proves it in the
running application**, and a unit test passing is exactly what let the defect live for four waves.

Two earlier audit attempts never photographed the capture overlay: the first exhausted its context,
the second correctly determined there was nothing to photograph. **This is the first attempt where
there should be something to see.**

## Protect your own context — a previous run died of this

- **Never take an accessibility tree.** Not once. That is what killed the first attempt.
- **Four screenshots, maximum.** Take each once; do not re-shoot to check your work.
- Do not screenshot to confirm a click landed.
- Write findings as text as you go, so they survive if you run out of room.

If you are about to take a fifth screenshot, stop and write your report instead.

## The binary

`D:\Developer\wiradigital.id\snapdown\target\release\Snapdown.exe`

Built from the W6-S2 fix. It starts with the main window **hidden** and a tray icon; open the window
from the tray if it does not appear.

**Sanity check first, before anything else:** the window title bar must read **`Snapdown Editor`**,
and the left side must be a vertical navigation rail — not a row of tabs across the top. If you see
tabs, you are running a stale binary; stop and report that rather than continuing.

## The four captures — write to `.work/ux-audit/`

1. **`shot-overlay.png`** — press `Ctrl+Shift+S`. **This is the whole point of the task.** Shoot
   whatever appears. If it is the Settings shell, `BUG-4` is not fixed and that is the finding —
   report it and stop.
2. **`shot-capture-note.png`** — drag a region roughly 400×200 and release. Shoot whatever appears
   for typing the Note. Then type `overlay works` and press Enter.
3. **`shot-findings-after-capture.png`** — open the Editor, go to Findings. Shoot it. Is the Finding
   you just made there?
4. **`shot-rail-focus.png`** — click into the window, then press `Tab` until a navigation rail item
   has keyboard focus. Shoot it. **Is there a visible focus ring?** That was `W6-S2`'s must-fix and
   no test proves it renders.

## The questions the report must answer, in text

- **Does the capture hotkey now show an overlay?** Yes or no. This is the headline.
- Does the overlay dim every monitor, or only one?
- Is there a live `W × H` readout while dragging? Where does it sit?
- Where does the Note field appear — anchored to the region, or somewhere fixed?
- Does `Enter` save? Does a Finding appear in the Findings list afterwards?
- Does `Esc` cancel cleanly, leaving no Finding?
- **Is the rail's focus ring visible when tabbing?**
- Anything unreadable: text whose colour is close to its background.

## What you are NOT testing

The Findings surface is being rebuilt right now by another story (`W6-S7`), and today it still shows
no image and no marker canvas — that is `BUG-5`, already known and already scheduled. **Do not report
it as a finding.** Look only at whether the Finding *appears in the list*.

## Report → `.work/ux-audit/AUDIT-3.md`

Answer every question above in order. Say plainly what you could not test and why. Do not describe a
screen you did not open, and do not infer behaviour from source code — this task exists precisely
because source-level reasoning is not the same as watching it run.

## Done means

`AUDIT-3.md` exists and answers the headline question, and the four screenshots exist or are listed
as impossible with a reason.

**Do not commit the screenshots.** This repository is public, the product brief forbids it, and CI
now refuses them. `.gitignore` already covers them; do not override it.
