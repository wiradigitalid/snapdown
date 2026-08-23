# Task — Narrow UI audit: the capture path and the marker canvas

You drive the real UI. Do not edit application code. Do not rebuild.

## Why this brief is narrow

A previous run of this audit **exhausted its 1M-token context** and was stopped. Screenshots are
expensive in context. This brief asks for **four screenshots and nothing else**, and you must protect
your own budget:

- **Never take an accessibility tree.** Not once. The earlier run died on those.
- **Take each screenshot once.** Do not re-shoot to check your work.
- **Do not screenshot to confirm a click landed** — click, then act, then shoot only the four below.
- Write findings as text as you go, so that if you run out of room the findings survive.

If you find yourself about to take a fifth screenshot, stop and write your report instead.

## The binary

`D:\Developer\wiradigital.id\snapdown\target\release\Snapdown.exe`

Already built. It starts with the main window **hidden** and a tray icon; open the window from the
tray if it does not appear.

## The four captures — write to `.work/ux-audit/`

1. **`shot-overlay.png`** — press the capture hotkey (default `Ctrl+Shift+S`). Shoot the armed
   overlay: the dim, the crosshair, whether a live `W × H` readout follows the pointer.
2. **`shot-capture-note.png`** — drag a small region (roughly 400×200) and release. Shoot whatever
   appears for typing the Note. Then type `marker test` and press Enter.
3. **`shot-finding-detail.png`** — open the Editor, go to Findings, select the finding you just made.
   Shoot the detail view.
4. **`shot-marker-canvas.png`** — click three places on the image to place three Markers, then write
   three numbered lines in the Note. Shoot it.

## Questions the report must answer, in text

- Does the overlay appear to arrive quickly, and does it dim **every** monitor or only one?
- Is there a live dimension readout while dragging? Where does it sit — over the region or outside it?
- Where does the Note field appear? Anchored to the region, or somewhere fixed?
- Does `Enter` save? Does `Esc` cancel cleanly?
- On the Findings detail: do the three panels fill the window, or is there dead space below them?
- Do the Marker badges stay legible over the screenshot content?
- **Delete the second Marker.** Do the remaining ones renumber contiguously? What happens to the
  second numbered line in the Note — does it move, vanish, or stay and become unbound?
- **Delete a numbered line from the Note instead**, without touching the image. Is the now-unbound
  Marker reported anywhere, or silently tolerated?
- Anything unreadable: text whose colour is close to its background.

That last pair matter most. `SCN-04` says the two directions must behave **asymmetrically** —
deleting a Marker removes its line and renumbers; deleting a line leaves the Marker in place and
reports it as unbound. Nobody has ever checked which the code does.

## One more, if and only if you have room

Try binding a hotkey Windows already holds (e.g. `Alt+Tab`) in Settings → Hotkeys. Record the **exact
error text**. No screenshot — just the text.

## Report → `.work/ux-audit/AUDIT-2.md`

Answer every question above in order. Say plainly what you could not test and why. Do not guess, and
do not describe a screen you did not open.

## Done means

`AUDIT-2.md` exists, and the four screenshots exist or are listed as impossible with the reason.

**Do not commit the screenshots.** This repository is public and the product brief forbids committing
captured screenshots. `.gitignore` already covers `.work/**/*.png`; do not override it.
