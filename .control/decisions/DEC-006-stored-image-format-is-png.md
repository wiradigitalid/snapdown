---
id: DEC-006
title: Every stored image Snapdown writes is PNG
status: draft
opened: '2026-08-24'
type: technical
touches: []
---

## Decision

Every image Snapdown writes — a captured Finding, its thumbnail, and a Bundle's burned copy — is
**PNG**. The `.webp` extension at `bundle.rs:41` is a mistake and is corrected to `.png`.

## Why

`OQ-26` recorded that the corpus names **no image format anywhere**: a search for `png` and `webp`
across `.what/` and `.how/` returns nothing outside the registries. So this is not a promise being
changed — it is a build detail that was chosen twice, inconsistently, and never written down.

The evidence is the build, not the corpus:

- The workspace pins `image = { version = "0.25.10", default-features = false, features = ["png"] }`.
  **WebP is not compiled in.** `.webp` is not a format this binary can produce, so the recorded path
  was never reachable — it was a string, not a decision.
- Every producer already emits PNG: `RegionCapturer::crop_and_encode_image`,
  `ImageReducer::reduce_image`, and `MarkerBurner::burn_markers`.
- Both serving surfaces already map or default to `image/png`.

Choosing WebP instead would mean adding a codec feature, re-encoding three producers, and moving
every golden — to change a format nothing has ever written.

`AD-4` is the constraint this has to respect and it does: an image is reduced exactly once, at
capture. PNG being lossless means the burn can draw on the stored bytes and re-encode **without
compounding loss**, which is what makes `AD-4`'s "no later stage may re-encode" survivable at the one
place a later stage legitimately must — drawing a Marker.

## Cost

**PNG is larger than WebP for photographic content**, and Snapdown's payload is screenshots that
travel to an agent. `BG-3` is about agent reading cost, and `OQ-2` already assumes pixel area rather
than encoder choice is the dominant lever. If `OQ-2` turns out to be wrong, this decision is the
first thing to revisit — and revisiting it means a codec feature flag plus a golden regeneration, not
a redesign.

The format also becomes load-bearing for `AD-9` once a Bundle's Markdown references the burned copy:
a lossless format is what lets the no-Markers case return the source bytes **unchanged** rather than
merely equivalent.

## Reversal trigger

A measured agent-reading-cost figure showing encoder choice, not pixel area, dominates — which is
`OQ-2` answered in the negative.

## Trace

- `OQ-26` — the question this answers.
- `BUG-19` — where the `.webp` path was found.
- `AD-4`, `AD-9` — the decisions this has to respect.
- `Cargo.toml:37` — the pin that makes WebP unproducible today.
