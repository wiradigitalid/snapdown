# 14: Fine-tune textboxes commit on losing focus, not only on Enter

**What to build:** The three Fine-tune size and quality textboxes (resize percent, max long edge,
encoder quality) commit their typed value when the field loses focus - clicking the slider beside
it, or clicking anywhere else on the screen - not only when Enter is pressed.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Typing a value into any of the three Fine-tune textboxes and then clicking away (not pressing
      Enter) commits it the same way Enter does
- [x] Enter still works as before

## Comments

Added as a new opt-in `commit-on-blur` property on `SdTextField` (default `false`), so every other
caller of the shared component - the note field, the Properties panel's font-size field, the vault
path field - keeps its exact current behaviour with no code change on their part. Wired to `true`
only on the three Fine-tune fields in `settings.slint`. Confirmed working by the owner.
