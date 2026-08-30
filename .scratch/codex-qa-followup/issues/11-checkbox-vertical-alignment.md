# 11: Checkbox rows are not vertically aligned with their label

**What to build:** In Settings, each checkbox's box lines up vertically with the first line of its label text, instead of sitting visibly higher or lower than it.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] "Run Snapdown at Windows startup" and "Open the Editor after a hotkey capture" checkboxes visually line up with their label's first line

## Comments

Root cause was two-part. First, `SdCheckbox`'s box column had an explicit height centred inside
itself while the label column used `alignment: start`; once a row grew taller than one line (any
hint text does that), the two columns cross-aligned differently depending on the row's own height,
so the same component looked fine in the Hotkeys tab and off by ~1.5px in the Startup section.
Fixed by pulling the box out of the layout entirely and positioning it from `label-text`'s own
resolved `y`/`height`, which is deterministic regardless of context.

Second, even anchored correctly, centring on `label-text.height` (the full ascent+cap-height+
descender line box) still read ~1px high against the actual capital letters, because IBM Plex Sans
puts more room below the baseline than above the cap-height and Slint doesn't expose ascent/cap-
height metrics directly. Closed with a measured `+ 0.5px` correction, verified by pixel-sampling a
live screenshot until the checkbox's vertical centre matched the label's cap-height centre exactly.
Confirmed working by the owner.
