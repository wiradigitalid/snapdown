# 09: Settings sliders lose two-way sync with their textbox after manual entry

**What to build:** After typing a value directly into a Settings slider's textbox and committing it, the slider thumb visually reflects the new position, and dragging the slider afterward continues to update the textbox live — the same way both already work before any manual entry.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Typing a value into a slider's textbox and pressing Enter moves the slider thumb to match
- [x] After that, dragging the same slider continues to update its textbox in realtime
- [x] Note for the implementer: reproduce live before fixing — prior investigation could not confirm the mechanism (candidates: the custom slider's internal drag-state getting stuck, or the text field retaining focus and suppressing external re-sync) and computer-use verification was inconclusive in that session's environment

## Comments

Root cause: `SdTextField.text` is a two-way alias onto the native `TextInput.text`, and Slint drops
a property's declarative `text: expr` binding the first time anything assigns to it directly - which
is exactly what typing a key does. Fixed with two-way-aliased mirror properties plus `changed`
handlers on the shared scope that push the text imperatively instead, so it keeps resyncing after
every edit rather than only before the first one. See `settings.slint`'s own comment where the
mirrors are declared.

Follow-up, same underlying field: the textbox also now commits on losing focus (`commit-on-blur` on
`SdTextField`), not only on Enter - confirmed working by the owner.
