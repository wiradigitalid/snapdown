# 01: Window shadow renders on the main window

**What to build:** The main Snapdown window shows a standard OS drop shadow around its frame — the same kind every other native Windows application window has — instead of appearing flat/borderless as it does today.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Launching Snapdown shows a visible drop shadow around the main window on Windows, in both light and dark theme
- [x] Note for the closing agent: a shadow is a rendering effect the Reviewer must see with their own eyes — grep/unit tests cannot confirm it, so flag this for human visual confirmation rather than closing on green tests alone

## Comments

Implemented via the `CS_DROPSHADOW` class-style flag on the main window (`main.rs`). Confirmed
working by the owner.
