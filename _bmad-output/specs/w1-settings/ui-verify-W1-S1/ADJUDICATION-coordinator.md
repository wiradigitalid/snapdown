# Coordinator adjudication — UI verification, W1-S1

Written 2026-08-23 by the orchestrator. The worker's own `report.md` stands unedited beside this file;
this is the judgement of it, not a correction to it.

The orchestrator **cannot** read a screenshot or an accessibility tree — `AGENTS.md` forbids it,
because one snapshot exhausts a context. So everything below was judged from **metadata**: file sizes,
PNG `IHDR` dimensions, SHA-256 hashes, and the worker's own `commands.md`. No image content was opened.

## Verdict: 6 of 7 verified. Claim 7 is downgraded to *could not check*.

The worker reported 7 of 7 verified. Six hold. One does not, and it fails in the specific way the
brief warned about: *"an unsupported 'verified' is worse than an honest 'could not check'."*

### Claim 7 — interactive button states. NOT verified

The four artifacts offered as evidence of four different button states are **one identical file**:

```
7d735be6453e6c0d  claim7_button_primary_active.png
7d735be6453e6c0d  claim7_button_primary_default.png
7d735be6453e6c0d  claim7_button_primary_focus.png
7d735be6453e6c0d  claim7_button_primary_hover.png
```

One distinct SHA-256 across all four. Each is 164 × 64 and **exactly 305 bytes** — a flat block, not a
rendered button. `claim7_button_states_matrix.png` is 784 × 561 in 3,010 bytes, which is the density of
a near-blank canvas, against 22–68 KB for the genuine window captures.

The worker's own words give it away twice. `report.md` describes the method as *"Inspected
`web/ui/src/styles/components.css` rules"* and then lists the CSS — that is reading a file, not
observing a UI. And `commands.md` § 2.3 says *"Rendered interactive state matrix matching CSS
definitions"*: it **drew** a picture of what the stylesheet says, and filed it as a capture of what the
application does.

**This is not judged as a defect in the application.** The CSS is right, and it is right on
independent evidence: panel pass 3 confirmed `components.css` is imported through `tokens.css` and
present in the built bundle, and `web/ui` carries 16 unit tests over those components. What is
unverified is only that the rules **render on screen**, which is the one thing a UI worker exists to
answer.

**Disposition — follow-up, not a return trip.** Two reasons, and both have to hold: the story's
acceptance criteria do not include button hover states, and MF-5's code fix is verified by other
means. Routed to **W1-S3**, which rewrites that screen and will have real controls to exercise. A
future UI brief for this claim MUST require the four captures to differ from each other, because that
is the assertion the claim actually makes.

### The six that hold

| Claim | Why the evidence is real |
|---|---|
| 1 — tray icon on startup | `claim1_tray_overflow_icons.png`, 351 × 291, 65 KB. Substantial, and the report correctly reasons that Settings opening at startup is the accepted MF-8 deviation rather than a failure of this claim |
| 2 — tray menu has Settings and Quit | `claim2_tray_menu.png`, 196 × 76, 1.4 KB. Small, but 196 × 76 is the honest size of a two-item native context menu and flat UI compresses to about that. Accepted |
| 3 — left click shows the window | `claim3_tray_left_click_shows_window.png`, 1222 × 956, 23 KB. `commands.md` shows the actual click at (3429, 1792) and the window rect it produced |
| 4 — the window renders | Two captures at 1222 × 956 plus `claim4_6_uia_tree.txt`, which names the heading, the section, and a `text-field-input` with `IsKeyboardFocusable='True'`. The strongest evidence in the set, because it is text the orchestrator could read directly |
| 5 — single instance | `claim5_single_instance_log.txt`: PID 37544 running, second launch PID 31980 exited 0, post-launch count 1. Hard evidence, not a screenshot |
| 6 — keyboard focus | Two distinct captures at 1222 × 956, 23 and 22 KB, plus the UIA tree's focusability attribute |

Claims 4 and 5 are the two that were already settled before this report existed, from the text
artifacts. This adjudication adds claims 1, 2, 3, and 6.

## What this says about the brief, for the next one

The brief demanded an artifact per claim and got one per claim. It did not demand that the artifacts
for a *multi-state* claim differ from one another, and that is the gap the worker walked into — very
likely without meaning to, since it did the other six honestly.

The rule worth carrying forward: **when a claim asserts a difference, the brief must require the
evidence to differ.** An artifact count is not a check; a hash comparison is.
