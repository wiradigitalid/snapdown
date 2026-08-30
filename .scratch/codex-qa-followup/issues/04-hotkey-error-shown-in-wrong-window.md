# 04: Hotkey recording feedback shows inline in the Hotkeys tab, not the main Editor window

**What to build:** While recording a new hotkey in Settings → Hotkeys, any validation error (e.g. "a shortcut needs at least one of Ctrl/Alt/Shift/Win") or conflict message appears inline in the Hotkeys tab itself, sticky at the bottom of that panel — the same way wira-desk's hotkey capture panel behaves — never in the main Snapdown Editor window. The panel should visibly detect modifier keys (Ctrl, Alt, Shift, Win) as they're held during recording. Rebinding a shortcut to the exact combination it already holds is accepted silently, not treated as a conflict; binding to a combination already used by a *different* action shows a conflict message naming that action's owner.

Study wira-desk's hotkey capture panel directly before implementing — the brief is "mimic it", not "approximate it."

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Recording an invalid shortcut (e.g. a bare letter with no modifier) shows its rejection message inside the Hotkeys tab's sticky bottom panel, not in the main Editor window
- [ ] The recording panel visibly reflects Ctrl/Alt/Shift/Win as they're pressed, matching wira-desk's hotkey capture UX
- [ ] Rebinding a shortcut to the combination it already has is accepted without an error
- [ ] Binding to a combination already owned by a different action shows a conflict message naming that action
