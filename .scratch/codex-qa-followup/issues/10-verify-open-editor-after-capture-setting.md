# 10: Verify "Open the Editor after a hotkey capture" is actually respected

**What to build:** Confirm whether the Editor unexpectedly appearing after a hotkey capture with this setting OFF is a real regression or expected behaviour — a tray-triggered capture intentionally always reveals the Editor by design, and a capture taken while the Editor was already visible behind Settings looks the same either way. Fix only if a genuine gap is found.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Reproduce with the setting OFF, capturing strictly via the global hotkey (not the tray icon), with the Editor and Settings fully closed beforehand
- [x] If the Editor still appears in that exact scenario, fix it; if not, close as working-as-designed and make the tray-vs-hotkey distinction visible to the Reviewer somewhere reasonable (e.g. the setting's hint text)

## Comments

Two real bugs found on this path, both fixed:

1. The Editor DID still appear with the setting OFF: `close_overlay()` unconditionally called
   `main.show()` on every capture regardless of the setting. Fixed by removing that call and letting
   `on_capture_completed`'s own reveal check (already gated on the setting) be the only thing that
   shows the window.
2. Fixing (1) surfaced a second bug on the hotkey path only: a brief white window flash, then gone.
   Cause: `set_capture_exclusion()` toggles `SetWindowDisplayAffinity` on the main window
   unconditionally on every capture, including while that window is hidden/minimized - which is
   exactly the hotkey-without-Editor-open flow this ticket is about. Windows forces DWM to paint a
   frame for a window it has not composited yet when the affinity call lands, which is the flash.
   Fixed by skipping the call entirely when `main.window().is_visible()` is false: a window nobody
   can see needs no protection from being captured.

The tray-vs-hotkey distinction is already in the "Open the Editor after a hotkey capture" checkbox's
own hint text ("A capture started from the tray icon always opens the Editor, regardless of this
setting..."), so the second acceptance criterion's fallback was already satisfied. Confirmed working
by the owner - the tray still opening the Editor every time is the documented, intentional
difference this ticket names, not a regression.
