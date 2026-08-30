# 03: Vault move fully switches the active vault end-to-end

**What to build:** After confirming "Move it" during a Vault relocation, every part of the app that reads or writes the active Vault path switches to the new location immediately and consistently: the folder textbox reflects the new path, the old location's Bundle folders are cleaned up once the move completes, a capture taken afterward is written into the new Vault, and the Findings filmstrip keeps showing existing Findings without needing a restart.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] After "Move it" completes, the Settings folder textbox shows the new Vault path, not the old one
- [x] The old Vault location no longer retains leftover Bundle folders once the move is confirmed complete
- [x] A capture taken immediately after the move is written into the new Vault, not the old one
- [x] The Findings filmstrip still renders existing Findings correctly after the move (no broken thumbnails from the path change)

## Comments

Solved by relaunching the whole process after a move (spawning the next instance of the current
exe, then exiting the same way the tray's Quit path already does) rather than trying to hot-switch
every in-memory reader of the Vault path. That is what makes the textbox, Bundle cleanup, new
captures, and the Findings filmstrip all consistent — they all come up fresh against the new path.
Confirmed working by the owner.
