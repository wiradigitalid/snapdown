# 13: Settings footer button says "Close", not "Done"

**What to build:** Settings auto-saves every change immediately — there is no staged-edit, Revert/Save flow, and the product keeps it that way. The footer's primary button is relabeled "Close" instead of "Done" so it stops implying there is a pending action to finish.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Footer button reads "Close"
- [x] Behaviour is unchanged — it still closes the Settings screen

## Comments

Label-only change in `settings.slint`; the button's callback was untouched. Confirmed working by
the owner.
