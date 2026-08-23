# Task — Visual + accessibility audit of Snapdown Desktop (current build)

You are a testing-ui worker. You DRIVE THE REAL UI. Do not edit application code.

## Gate context
This repo uses WDI Method. This work sits BEFORE a re-run of G2 (`wdi-ux`). Its only
purpose is to give the UX gate a factual record of what the app looks like TODAY.
Bind method docs: `.constitution/method/document/ux-guide.md`,
`.how/_platform/inventory-screen.md`, `.how/_platform/design-system.md`.

## Goal
Produce a complete visual and accessibility record of every screen of the built
Snapdown desktop app, plus a verified defect list.

## The binary
`D:\Developer\wiradigital.id\snapdown\target\release\Snapdown.exe`
Already built and current. Do NOT rebuild. It starts with its main window HIDDEN and a
tray icon; open the window from the tray if it does not appear.

## What to capture — write everything under `.work/ux-audit/`
For EACH of the four tabs (Findings, Bundles, Agent Access, Settings):
1. A PNG screenshot -> `.work/ux-audit/shot-<tab>.png`
2. The accessibility tree -> `.work/ux-audit/tree-<tab>.txt`

Then exercise and screenshot these interactions:
- Settings: click the Vault folder **Browse** button. Does a native folder picker open?
- Settings: click a hotkey box, press Ctrl+Alt+9. Does it record? Does Save work?
  Then try binding a combination already held by Windows (e.g. Ctrl+Alt+Delete is
  unavailable; try Alt+Tab) and record the exact error text shown.
- Settings: toggle "Run at Windows startup" both ways, record the toast text.
- Settings: measure how far the Settings tab scrolls (window height vs content height).
- Trigger the capture hotkey (default Ctrl+Shift+S). Screenshot the capture overlay.
  Select a small region, type a note, confirm. -> `shot-overlay.png`, `shot-capture-note.png`
- Go to Findings, open the finding you just made, place two markers.
  -> `shot-finding-detail.png`, `shot-marker-canvas.png`

## Defects to check explicitly (the owner reported these)
- **Contrast**: any text whose colour is near-identical to its background. The owner
  reported the "Capture Region" and "Open Workspace / Editor" labels in the Hotkeys
  section rendering white-on-white. Confirm or refute, and record the OS theme
  (light or dark) you tested under. TEST BOTH THEMES if you can switch Windows theme.
- **Settings length**: is it denser than one screen? By how much?
- **Navigation**: are the four tabs visible and reachable? The owner said they could
  only ever see Settings.

## Report -> `.work/ux-audit/AUDIT.md`
A table of every screen: what renders, what is broken, contrast pass/fail per text
element you could measure. Then a numbered defect list, each with: screenshot
filename, exact element, observed vs expected, and severity.
State plainly anything you could NOT test and why. Do not guess.

## Done means
`.work/ux-audit/AUDIT.md` exists, every screenshot named above exists (or is listed as
impossible with the reason), and the three owner-reported defects each have a verdict.
