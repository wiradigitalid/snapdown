# 05: Settings and Properties sliders update their textbox in realtime and accept manual entry

**What to build:** Every slider in Settings, and the font-size slider in the Properties tab (next to marker notes), updates its paired textbox live as the slider is dragged, not only after release. That textbox is also directly editable — typing a value into it sets the slider. Also fix the "fine-tune size and quality" icon, which currently renders invisibly/broken.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Dragging any Settings slider (percent-based) updates its textbox continuously during the drag
- [x] Dragging the Properties panel's font-size slider updates its textbox continuously during the drag
- [x] Typing a value directly into any of these textboxes updates the corresponding slider and takes effect
- [x] The "fine-tune size and quality" icon renders correctly, no longer blank/broken

## Comments

Confirmed working by the owner. The manual-entry-then-drag desync this left behind is `09`, closed
separately. `09` also added committing the textbox on losing focus, not only on Enter.
