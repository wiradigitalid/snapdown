# 02: "Show in Explorer" opens the actual Vault directory

**What to build:** Choosing "Show in Explorer" (or equivalent) for a Vault opens a File Explorer window rooted at that Vault's own folder, not a generic Explorer window that leaves the user to navigate there manually.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Triggering "Show in Explorer" for a Vault opens Explorer with that Vault's directory already open/selected
- [x] Still correct after a Vault has been moved to a new location (opens the new path, not the stale old one — coordinate with ticket 03)

## Comments

`main.rs` now opens the Vault's own directory directly via `explorer.exe`, distinct from the
existing file-location opener. Coordinated with `03`'s relaunch so the path used is always the
current Vault path. Confirmed working by the owner.
