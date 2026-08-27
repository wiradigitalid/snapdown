---
scope: project
purpose: Keeps every surface in both windows composing from one token set and one component set, so a design decision is made once rather than re-litigated per screen.
status: Draft            # Article 4: Draft MAY be read as guidance, MUST NOT reject a change
ratified_by: null        # the commit whose content ratifies this file
---

# design system — project guide

**Loaded when:** building or changing any user interface in this product, in either window.

## Why this file exists

The owner raised design-system inconsistency **five separate times** across one week of capture work.
Every time, the agent fixed the instance and left the cause, so the next screen drifted the same way.
Recorded here rather than in a defect row, because it is not a defect: it is the absence of a written
rule, and a rule is what stops the sixth time.

The instances, and what they had in common:

| Reported | What it was |
|---|---|
| The capture note field "didn't look like an input" | A `std-widgets` `LineEdit` styling itself |
| The Observation Summary rendered white-on-white, grey when focused | A `std-widgets` `TextEdit` styling itself |
| The Save action was a different size and colour from the Editor's buttons | A `std-widgets` `Button` styling itself |
| The overlay "reads as a different product" | Colour literals instead of `theme.slint` |
| The capture panel's label and spacing did not match the inspector's | Numbers chosen for one panel in isolation |

None of them was a number picked badly. In four of the five, **the numbers were not ours to pick** —
a `std-widgets` component paints from Slint's own style palette and ignores `theme.slint` entirely.

## Rules

- Colour, radius and spacing MUST come from `apps/desktop/ui/theme.slint`. A colour literal MUST NOT
  appear anywhere else. The capture overlay is guarded mechanically by
  `the_capture_overlay_takes_every_colour_from_the_token_set`.
- `LineEdit`, `TextEdit` and `Button` from `std-widgets.slint` MUST NOT be used. They paint from
  Slint's palette, not this product's. Only `ScrollView` may be imported from `std-widgets`.
- New UI MUST be composed from `apps/desktop/ui/components/`. A new visual treatment MUST become a
  component there before it is used twice — the second use is what makes drift possible.
- Two places showing the same kind of thing MUST reference the same component, not the same numbers.
  Matching numbers drift the first time one is touched; that is what the two section headings did.
- The **Editor is the reference**, not the overlay. Where they disagree, the overlay changes. The
  owner has said this repeatedly and it settles every such question without further discussion.

## The components, and what they fix

| Component | Use for | Replaces |
|---|---|---|
| `SdTextField` | Every text entry, single- or multi-line | `LineEdit`, `TextEdit` |
| `SdActionButton` | Every button with a label | `Button` |
| `SdSectionLabel` | Every panel section heading | A hand-set `Text` |
| `SdCaption` | Hints, timestamps, dimension readouts | A hand-set `Text` |
| `IconButton` | Icon-only toolbar controls | — |

## The scale

Taken from the Editor's inspector and toolbar, which are the surfaces the owner has approved.

| Role | Value |
|---|---|
| Section heading | 11px · weight 800 · `letter-spacing: 0.5px` · upper case · `text-secondary` |
| Body / field text | 13px · `text-primary` |
| Button label | 10–11px · weight 700 · `text-on-accent` on accent |
| Caption / hint | 10px · `text-dim` |
| Monospace readout | 10–11px · `IBM Plex Mono` |
| Panel padding | 16px |
| Label → its field | 6px |
| Section → section | 16px |
| Corner radius | `Theme.radius-sharp` for every control. `radius-pill` only for a status chip, never a button |
| Accent button | `accent-primary` at rest · `accent-hover` · `accent-pressed` · no border · accent-tinted shadow |
| Icon in a control | 12–13px, `image-fit: contain` |

## Overlay-specific

The capture overlay is drawn over a frozen screenshot of the operator's own desktop, not over app
chrome. Its scrim, ring, grid and floating chrome are therefore **theme-invariant** and live in
`theme.slint` group 6 with no `is-dark` branch — a light scrim over a dark screenshot is invisible.
The note panel is a document surface and does follow `bg-card` into light mode.

This is the same reasoning as the four deliberately theme-invariant groups in
`web/ui/src/styles/tokens.css`, and it is an exception to `NFR-17`'s "defined for both themes"
wording rather than a violation of its intent.

## What is still open

`BUG-37` remains open for a reason unrelated to consistency: the owner asked for a look-and-feel
**study**, naming Graphite as a reference. Everything above makes the product consistent *with the
system it already has*. Whether that system is the right one is the question `wdi-ux` exists to
answer, and it has not been run.
