# 06: Settings checkboxes are clickable again

**What to build:** Checkboxes in the Settings screen respond to clicks and toggle their checked state — currently they don't react at all.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Every checkbox in Settings toggles when clicked
- [x] The toggled state persists the same way other Settings controls already do

## Comments

Root cause: `SdCheckbox` inherited `HorizontalLayout`, which has no hit-testable area of its own,
so clicks landed on nothing. Changed to inherit `Rectangle` with its own `TouchArea` and
`clicked => { root.toggled(!root.checked); }`. Confirmed working by the owner.
