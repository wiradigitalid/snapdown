---
type: decision
id: DEC-009
status: draft
touches: []
supersedes: null
superseded_by: null
created: "2026-08-27"
---

# DEC-009 — The design system stays hand-authored in two theme columns, and takes one idea from Graphite rather than its palette

## Decision

`apps/desktop/ui/theme.slint` MUST remain the single source of colour, radius and spacing, written as
explicit `is_dark ? dark : light` pairs chosen by hand. It MUST NOT be replaced by a ramp derived from
a seed colour.

Two things MUST be taken from the study below instead:

1. A contrast check MUST cover `theme.slint`, not only `web/ui/src/styles/tokens.css`. The Slint
   surfaces are now the whole product's UI and they have no equivalent of `contrast.test.ts`.
2. The capture overlay's pixel lens MUST report the colour under the cursor alongside its coordinates.

Graphite's **layout** conventions MUST NOT be adopted. Snapdown's toolbar-canvas-inspector shape is
already the one this app class uses, and the owner has approved it repeatedly.

## Why

The owner asked for a look-and-feel study, naming Graphite, after raising design-system
inconsistency five times. `BUG-37` held that request open while the mechanical half was fixed.

**The study's first finding is that the inconsistency was never the palette.** Every one of the five
reports had the same cause: a `std-widgets` component painting itself from Slint's own style palette
instead of `theme.slint`. `TextEdit` rendered the Observation Summary white-on-white; `Button` gave
the capture panel a Save action at a different size, colour and type scale from every button in the
Editor. The numbers were not ours to adjust, so no choice of numbers would have fixed it. That is
closed — `SdTextField`, `SdActionButton`, `SdSectionLabel` and `SdCaption` carry the Editor's values,
`std-widgets` exports only `ScrollView` into this product, and the rule is written down in
`.constitution/project/design-system-guide.md`.

So the question this decision answers is narrower than it looked: **now that the product is
consistent, is the system it is consistent with the right one?**

### What Graphite actually offers

Two different things carry the name, and only one of them is about colour.

**Graphite the editor** — the open-source vector and raster editor, in release-candidate stage for
desktop as of 2026 — is dark-themed with a left tool palette, a centre canvas, and a node graph panel
below that collapses when not needed. Its own description of the intent is that the UI sits close
enough to Figma that onboarding is fast. That is the same shape Snapdown already has: toolbar, canvas,
inspector. There is nothing here to adopt, because it has already been adopted independently, and the
owner has signed off on it more than once.

**Graphite UI** — a separate design system — is the interesting one. Its whole premise is that you
input a single hex value and it generates the system: three perceptual ramps (accent, neutral,
neutral-variant) sampled in OKLab at fixed tone stops, semantic tokens mapped onto roles like
`surface` and `on-surface`, paired light and dark themes from one source, 59 CSS custom properties,
and automatic WCAG 2.1 verification in which a failing pairing is walked along its ramp until it
passes. It specifies no radius scale, no typography, no spacing.

That premise is aimed squarely at what `theme.slint` does by hand. Snapdown has roughly thirty tokens,
each a hand-picked pair — `is_dark ? #1c1c1f : #f1f5f9`, thirty times over.

### Why the hand-authored pairs stay anyway

Three reasons, and the third is the one that decides it.

**The pairs are not arbitrary.** They are already a coherent Slate-derived scale with WCAG-checked
accent colours, and they carry information a derived ramp cannot: four groups are *deliberately*
theme-invariant, each for a stated reason. `--color-marker*`, `--color-overlay-scrim`,
`--color-overlay-ring` and `--canvas-checker` in `tokens.css`, and the whole of `theme.slint`'s group 6,
exist because the capture overlay is drawn over a frozen screenshot of the operator's own desktop
rather than over app chrome — a light scrim over a dark screenshot is invisible. A generator that pairs
every token light-and-dark from one seed would have to be told to exempt those, which is the same
hand-authoring it was meant to replace.

**There is nothing to gain that the owner would see.** A derived ramp buys the ability to change the
whole product's colour by changing one input. Nobody has asked to, this is a single-product palette
with no white-labelling requirement, and OKLab derivation for Slint means writing the ramp generator
too — Slint has no colour-space functions, so the tokens would have to be generated at build time into
the file they already occupy.

**The seed-colour approach has no way in.** Graphite UI emits CSS custom properties. Snapdown's UI is
Slint; the React surface `web/ui` builds nothing in the active workspace (`OQ-27`). Taking the system
means taking its output format, and the format does not reach this product.

### What is worth taking

**The contrast gate.** Graphite UI walks a failing pairing along its ramp until it passes; the
principle underneath is that contrast is verified mechanically rather than trusted. Snapdown already
believes that — `contrast.test.ts` parses `tokens.css` and was verified by mutation after an earlier
version hardcoded its own copy of the values and passed a 2:1 ratio. But that test covers
`web/ui/src/styles/tokens.css`, and `web/ui` is not what ships. `theme.slint` is, and nothing checks
it. That is a real gap the study found, and it is the first requirement above.

**Shottr's pixel readout.** Not Graphite, but from the same survey and closer to what Snapdown does:
the tool most praised for measurement in this class lets you hover any UI element and read its
dimensions, spacing *and colours*. Snapdown's overlay now reports dimensions (in source pixels) and
the pixel coordinate under the cursor, and it detects containers. The colour is the one part missing,
it is one lookup into a canvas the product already holds, and it is the second requirement above.

## Cost

Keeping the hand-authored pairs costs a hand edit in two places whenever a token changes — once for
each theme column — and the risk that the two drift. The contrast gate is what makes that risk
detectable rather than invisible, which is why it is a requirement of this decision rather than a
suggestion beside it.

Declining the derived ramp also declines its by-product: light and dark can disagree in ways nobody
notices until a screenshot is taken in the other theme. Only the contrast gate covers that, and only
for contrast — not for hue drift.

## Alternatives

**Adopt Graphite UI's generated tokens directly.** Rejected: it emits CSS custom properties into a
surface that does not ship, and the four theme-invariant groups would each need an exemption written
by hand.

**Write a build-time ramp generator for `theme.slint`.** Rejected for now, not on principle. It is the
right answer if the product ever needs more than one palette, and it is what the reversal trigger
below watches for. Today it is a generator, a build step and a new failure mode, to replace thirty
lines nobody is struggling with.

**Restyle the product to look like Graphite the editor.** Rejected: Snapdown already has that shape,
arrived at independently, and the owner has approved it. Changing it would be motion, not progress.

## Reversal trigger

Any of these makes revisiting correct:

- A second palette is needed — a client theme, a white-label build, or a high-contrast mode. That is
  the case a derived ramp exists for, and hand-authoring three columns is where this decision breaks.
- The contrast gate required above finds a pairing that is wrong in only one theme. One is an
  oversight; a pattern of them says the pairs are being chosen without checking, and a generator that
  cannot make that mistake becomes worth its cost.
- Slint gains colour-space functions, making an OKLab ramp expressible in `theme.slint` itself rather
  than generated into it. Most of the cost above is the generator, not the idea.
- The owner's answer to the study is that the product should look different rather than consistent.
  This decision answers "is the current system right", and it would be the wrong document to bend.

## Trace

| | |
| --- | --- |
| Defect register | `BUG-37` (the overlay did not follow the design system; the mechanical half is closed, this decision closes the study half) · `BUG-45` (the `std-widgets` palette, found through a white-on-white Observation Summary) · `BUG-42` (the full-screen control speaking its own dialect) |
| Project guide | `.constitution/project/design-system-guide.md` records the rules and the scale; this decision records why the scale is hand-authored and what the study rejected |
| Requirement | `NFR-17` puts colour in one file. This decision keeps that and extends its enforcement to `theme.slint`, which `NFR-17`'s stated lint never covered |
| Open question | `OQ-27` — `web/ui` builds and tests nothing in the active workspace, which is why a contrast test living only there does not protect the product |
| Sources | Graphite the editor: [graphite.art](https://graphite.art/), [GraphiteEditor/Graphite](https://github.com/GraphiteEditor/Graphite), [abduzeedo review](https://abduzeedo.com/graphite-free-open-source-vector-editor-procedural-design/) · Graphite UI: [graphite-ui.com](https://www.graphite-ui.com/) · app-class survey including Shottr's measurement tools: [screensnap.pro comparison](https://www.screensnap.pro/blog/cleanshot-x-vs-shottr) |
| Progress | Requirement 1 is BUILT: `apps/desktop/tests/test_theme_contrast.rs`, seventeen pairings in both themes, five mutants killed. It found six AA failures on its first run, including white on the dark theme's `accent-primary` at 3.20 — every primary button label in dark mode, in a file whose own comment claims "WCAG AA Compliant". Those are `BUG-54`, recorded as measured exceptions so the gate is green today and fails if any of them worsens. Requirement 2 — the colour readout — is NOT built |
| Note | `touches` is intentionally empty at `draft`. It contradicts no `AD-N` |
