---
type: decision
id: DEC-010
status: draft
touches: []
supersedes: null
superseded_by: null
created: "2026-08-28"
---

# DEC-010 — `encoder_quality` means colour precision and a palette, not an encoder dial

## Decision

The Quality Budget's `encoder_quality` stays, keeps its 10–100 range, and **keeps PNG**. What it
controls is:

1. how many bits of each colour channel are retained, and
2. whether the capture is written as an **indexed** PNG — one byte per pixel plus a palette — when
   the result is opaque and fits in 256 colours.

`100` remains lossless and skips both steps.

The alternative readings — switch to WebP lossy, or to JPEG — are **rejected**.

## Why

`BUG-63` recorded that `encoder_quality` was stored and read by nothing, and diagnosed it as an
architecture mismatch: the corpus designed a two-lever budget while `DEC-006` had settled the format
as PNG, and PNG is lossless. That diagnosis was right. The conclusion drawn from it — that closing
the defect required a format change — was not.

The owner restated the requirement in terms of outcome rather than mechanism: *"bagaimana kualitas
gambar sama (kasat mata), tapi secara storage dia makin kecil"*. Measured against that, PNG is not
the obstacle. It was measured on a 1280×800 fixture with antialiased glyph shoulders over a gradient,
which is where a real capture's colour count actually comes from:

| | bytes | of lossless | worst channel error |
|---|---|---|---|
| lossless | 32117 | 100% | 0 |
| 7 bits, RGB | 23562 | 73% | 1 |
| **7 bits, indexed** | **8403** | **26%** | **1** |
| 6 bits, indexed | 7756 | 24% | 2 |

A 74% reduction for an error of one level per channel. That is below what anyone can see, and it is
what the requirement asked for.

It works because of what this product captures. A UI screenshot is flat colour and text; after
rounding, the fixture held 60 distinct colours. A palette is the natural representation for that, and
PNG has supported one since 1996.

## Why not WebP or JPEG

- **WebP lossy** needs `libwebp`, a C dependency. `DEC-006` already declined to add a codec feature
  for a format nothing in the corpus names, and nothing since has changed that. It would also make
  every existing Finding a different format from every new one.
- **JPEG** is the wrong tool for the content. Its block transform is worst exactly where a screenshot
  lives — hard edges between flat regions — so text acquires ringing at precisely the quality settings
  that would save meaningful space. A capture whose text is harder to read is a capture that has
  failed at its one job.

Both would have been a larger change than the one made, for a worse result on this product's own
material.

## Consequences

- **Reduction is now lossy by default**, and that is a change in kind rather than degree. The
  ceiling is stated and tested: at quality 92 no channel moves by more than one step of two.
- **Quantisation must stay idempotent.** The burn re-encodes a Finding that was already reduced at
  capture, so a Bundle composed three times must not be worse than one composed once.
  `quantising_twice_changes_nothing` guards it.
- **The burn takes the Finding's own quality.** Encoding a handoff lossless while its Finding was
  quantised would make the copy larger than the original.
- **A photographic capture is unaffected in kind.** More than 256 colours means the palette path is
  skipped and the RGB path runs, which still gives about 27%.
- `a_reduced_image_decodes_and_its_pixels_are_the_scaled_source` is pinned to quality 100. Its
  subject is the reducer's arithmetic; the perceptual claims belong to `test_png_encoding.rs` where
  they can fail for their own reasons.

## What would reopen this

A capture format that is not a UI screenshot becoming the common case — a product that mostly
photographs things would get little from a palette and would be better served by a real lossy codec.
Nothing in `.what/` points that way today.
