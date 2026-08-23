# Task — The first look at a properly built Snapdown

You drive the real UI. **Do not edit application code. Do not rebuild** — the binary is already built.

## Why this one is different from the three before it

Every UI claim this project has ever made was made against a Vite dev server. Three audit attempts
have been made and none of them looked at a correctly built application:

- **AUDIT-1** exhausted its context taking accessibility trees and never photographed anything.
- **AUDIT-2** correctly determined there was nothing to photograph.
- **AUDIT-3** launched a binary built with `cargo build`, which is **not** how this app is built. Its
  webview showed `ERR_CONNECTION_REFUSED`. That was the coordinator's error, not the auditor's, and
  it became `BUG-11`.

`W6-S11` fixed the build. **This binary was produced by the Tauri CLI, the frontend is embedded in it,
and it is the first application-shaped artifact this project has had.**

## Protect your own context — a previous run died of exactly this

- **Never take an accessibility tree. Not once.** That is what killed AUDIT-1.
- **Six screenshots, maximum.** Take each once; do not re-shoot to check your work.
- Do not screenshot to confirm a click landed.
- Write findings as text as you go, so they survive if you run out of room.

If you are about to take a seventh screenshot, stop and write your report instead.

## The binary

`D:\Developer\wiradigital.id\snapdown\target\release\Snapdown.exe`

It starts with the main window **hidden** and a tray icon; open the window from the tray if it does
not appear.

**Sanity check first, before anything else.** Any of these means you are running the wrong binary or a
stale one — stop and report that rather than continuing:

- the webview shows a network error instead of a UI
- the window title bar does not read **`Snapdown Editor`**
- the left side is a row of tabs across the top rather than a vertical navigation rail

## The six captures — write to `.work/ux-audit/`

1. **`shot-4-overlay.png`** — press `Ctrl+Shift+S`. **This is the headline.** `BUG-4` was critical:
   the capture hotkey opened `index.html?overlay=true` and nothing in the frontend read
   `window.location.search`, so the hotkey showed the Settings shell instead of a capture overlay. The
   capture path — the reason the product exists — did nothing for four waves. `W6-S2` fixed it and
   **no test outside jsdom has ever seen the fix.** Shoot whatever appears.

2. **`shot-4-note.png`** — drag a region roughly 400×200 and release. Shoot whatever appears for
   typing the Note. Then type `overlay works` and press Enter.

3. **`shot-4-findings.png`** — open the Editor, go to Findings, select the Finding you just made.
   **`BUG-5` is what this shot is for.** `MarkerLayer` was built, unit-tested, and mounted nowhere, so
   the screenshot a Note describes was never on screen and Markers could not be placed at all.
   `W6-S7` fixed it. **Is the screenshot visible?**

4. **`shot-4-marker.png`** — click on the image to place a Marker. Shoot it. Does a numbered badge
   appear on the image, and does a matching numbered line appear in the Note pane? That pairing is
   `AD-1`, the invariant this whole product is built on.

5. **`shot-4-rail-focus.png`** — click into the window, then press `Tab` until a navigation rail item
   has keyboard focus. Shoot it. **Is there a visible focus ring?** That was `W6-S2`'s must-fix and no
   test proves it renders.

6. **`shot-4-settings.png`** — open Settings. Shoot it. The owner reported three things here that this
   shot should settle: white text on a white background around the hotkey area, a Vault folder with no
   Browse button, and a panel that has to be scrolled.

## The questions the report must answer, in text

- **Does the capture hotkey show an overlay?** Yes or no. Headline.
- Does the overlay dim every monitor, or only one?
- Is there a live `W × H` readout while dragging? Where does it sit?
- Where does the Note field appear — anchored to the region, or somewhere fixed?
- Does `Enter` save, and does a Finding appear in the Findings list afterwards?
- Does `Esc` cancel cleanly, leaving no Finding?
- **Is the Finding's screenshot visible in the Editor?**
- **Does clicking the image place a numbered Marker, with a matching numbered Note line?**
- Is the rail's focus ring visible when tabbing?
- In Settings: any text whose colour is close to its background? Is there a Browse button for the
  Vault folder? Does the panel fit without scrolling?
- Anything else unreadable anywhere.

## What you are NOT testing, and must not report

- **Do not test what happens to a corrupt `library.db`.** `BUG-12` is already registered and already
  scheduled; five `.expect()` calls on store opens mean a corrupt database makes the window never
  appear. Do not corrupt anything to find out.
- Publishing and agent-access are frozen by `DEC-005`. Ignore them.
- Do not infer behaviour from source code. This task exists precisely because source-level reasoning
  is not the same as watching it run.

## Report → `.work/ux-audit/AUDIT-4.md`

Answer every question above, in order. Say plainly what you could not test and why. Do not describe a
screen you did not open.

## Done means

`AUDIT-4.md` exists and answers the headline question, and the six screenshots exist or are listed as
impossible with a reason.

**Do not commit the screenshots.** This repository is public, the product brief forbids it, and CI now
refuses them. `.gitignore` already covers them; do not override it.
