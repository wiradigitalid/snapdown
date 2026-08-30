# 04: Hotkey recording feedback shows inline in the Hotkeys tab, not the main Editor window

**What to build:** While recording a new hotkey in Settings → Hotkeys, any validation error (e.g. "a shortcut needs at least one of Ctrl/Alt/Shift/Win") or conflict message appears inline in the Hotkeys tab itself, sticky at the bottom of that panel — the same way wira-desk's hotkey capture panel behaves — never in the main Snapdown Editor window. The panel should visibly detect modifier keys (Ctrl, Alt, Shift, Win) as they're held during recording. Rebinding a shortcut to the exact combination it already holds is accepted silently, not treated as a conflict; binding to a combination already used by a *different* action shows a conflict message naming that action's owner.

Study wira-desk's hotkey capture panel directly before implementing — the brief is "mimic it", not "approximate it."

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Recording an invalid shortcut (e.g. a bare letter with no modifier) shows its rejection message inside the Hotkeys tab's sticky bottom panel, not in the main Editor window
- [x] The recording panel visibly reflects Ctrl/Alt/Shift/Win as they're pressed, matching wira-desk's hotkey capture UX
- [x] Rebinding a shortcut to the combination it already has is accepted without an error
- [x] Binding to a combination already owned by a different action shows a conflict message naming that action

## Comments

Implemented in `f66ca36` alongside ticket 08, but this file's status was left at
`ready-for-agent` — found stale during a later check. `hotkey_feedback()` (`main.rs:2093`) writes
to the `hotkey_feedback` property, which `settings.slint`'s own sticky bottom panel displays
(never the main window's toast). Live modifier chips are `rec-ctrl`/`rec-alt`/`rec-shift`/`rec-win`
(`settings.slint:106-114`). `HotkeyRegistrar::validate_and_rebind` (`hotkey.rs:193-266`) accepts a
rebind to the same combination silently and refuses a combination held by a different action with
`"{action} already uses this combination"` (`hotkey.rs:216`).
