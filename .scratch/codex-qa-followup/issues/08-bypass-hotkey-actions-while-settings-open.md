# 08: Bypass real hotkey actions while listening or Settings is open, and cancel listening by repeating the same combo

**What to build:** While the Settings screen is open — and especially while a row is actively "listening" for a new shortcut — pressing the real Capture or Open Editor hotkey combination does not trigger the real action (no capture overlay opens, the Editor is not force-raised). The "Pressed just now" confirmation may still fire for the row under test. Additionally, while a row is listening, pressing the exact combination it already holds cancels listening, the same as Escape, so setting a hotkey to itself doesn't get stuck. Hotkey tab copy is also brought in line with wira-desk's conventions where applicable.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Pressing the currently-bound Capture shortcut while Settings is open does not open the capture overlay
- [x] Pressing the currently-bound Open Editor shortcut while Settings is open does not force the Editor to the front beyond what Settings already shows
- [x] While a row is listening, pressing the combination it already holds cancels listening, same as Escape
- [x] Hotkey tab copy (instructions, messages) matches wira-desk's wording conventions where applicable

## Comments

Implemented in `f66ca36` alongside ticket 04, but this file's status was left at
`ready-for-agent` — found stale during a later check. `main.rs:5291-5323` gates both real actions
on `win.get_settings_open()` (`if !settings_open { win.invoke_capture_clicked(); }` and the
matching `OpenEditor` arm), while `hotkey_last_fired`/`hotkey_last_fired_text` still update so the
"Pressed just now" confirmation fires regardless. The same block ends `listening` when the row's
own already-bound combination arrives as the global `WM_HOTKEY` event, matching Escape's behaviour.
