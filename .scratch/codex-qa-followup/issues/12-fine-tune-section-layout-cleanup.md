# 12: Fine-tune size and quality section is hard to read

**What to build:** The "Fine-tune size and quality" disclosure's labels, sliders, textboxes, and explainer captions are grouped and spaced so it is clear which caption belongs to which control. "Colour accuracy" is clarified as the setting that controls compression (bit depth reduction plus an indexed palette), not a colour-only control.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Each control (Resize every capture to / Never wider or taller than / Colour accuracy) is visually grouped with its own label and explainer, distinguishable from its neighbours at a glance
- [x] Wording makes clear that "Colour accuracy" is the compression lever

## Comments

Relabelled to "Colour accuracy (compression)" and regrouped each control's label/slider/caption
into its own visual cluster in `settings.slint`. Confirmed working by the owner.
