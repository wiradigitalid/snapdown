# 08: Bypass real hotkey actions while listening or Settings is open, and cancel listening by repeating the same combo

**What to build:** While the Settings screen is open — and especially while a row is actively "listening" for a new shortcut — pressing the real Capture or Open Editor hotkey combination does not trigger the real action (no capture overlay opens, the Editor is not force-raised). The "Pressed just now" confirmation may still fire for the row under test. Additionally, while a row is listening, pressing the exact combination it already holds cancels listening, the same as Escape, so setting a hotkey to itself doesn't get stuck. Hotkey tab copy is also brought in line with wira-desk's conventions where applicable.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Pressing the currently-bound Capture shortcut while Settings is open does not open the capture overlay
- [ ] Pressing the currently-bound Open Editor shortcut while Settings is open does not force the Editor to the front beyond what Settings already shows
- [ ] While a row is listening, pressing the combination it already holds cancels listening, same as Escape
- [ ] Hotkey tab copy (instructions, messages) matches wira-desk's wording conventions where applicable
