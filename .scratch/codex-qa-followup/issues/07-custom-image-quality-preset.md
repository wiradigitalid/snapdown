# 07: Custom image quality/size preset

**What to build:** Alongside the existing image size/quality presets, add a "Custom" option where the user picks their own combination of settings (e.g. resize percentage and encoder quality) rather than being limited to the built-in presets.

**Blocked by:** None — confirmed during implementation that Custom is selected via the existing preset `SdSegmented` control, not a checkbox, so ticket 06 does not gate this one.

**Status:** done

- [x] A "Custom" preset is selectable alongside the existing presets
- [x] Selecting "Custom" exposes editable controls for the settings that make up a preset (at minimum resize percentage and encoder quality)
- [x] A capture taken while "Custom" is selected uses the user's chosen combination, not a built-in preset's values

## Comments

Added `{ id: "custom", label: "Custom" }` to the existing preset `SdSegmented` control in
`settings.slint`; picking it opens the "Fine-tune size and quality" disclosure automatically so the
Reviewer isn't left looking at a preset with no visible way to customise it. Confirmed working by
the owner.
