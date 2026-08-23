---
type: component
lc: LC-003
name: image-reducer
component: finding
container: desktop-app
created: "2026-08-23"
---

# LC-003 — image-reducer

The Control object `DEC-004` changes. It is the only place in the product that decides how much of a
Capture survives.

## Responsibility

Given raw pixels and the region they came from, produce exactly one stored image and the record of how
it was produced.

1. Resolve the Quality Budget for **this** region (`BR-104`).
2. Downscale to the resolved long edge, if it is under the region's.
3. Encode at the resolved quality.
4. Hand `LC-005` the bytes, and hand `LC-004` the resolved pair and budget name (`NFR-18`).

## The resolution rule, stated as a requirement rather than an algorithm

`Auto` must satisfy three things. The specific curve is an implementation choice; these are not.

- **It varies with the region.** A `312 × 118` tooltip and a `3840 × 2160` screen must not resolve the
  same pair (`SCN-03`). This is the assertion a constant cannot pass.
- **Where no downscale applies, quality is high.** Under the cap, the encoder is the only lever and
  small text is where its artefacts show.
- **Where a hard downscale applies, quality is lower.** The downscale has already removed the detail
  quality would have preserved, and `BG-3` wants the bytes back.

`Sharp`, `Balanced`, and `Small` resolve fixed pairs. `Custom` resolves what the Reviewer typed.

## What it must not do

- **Keep the original.** `AD-4`. The raw pixels are dropped when this object returns.
- **Re-encode an existing Finding.** `BR-9`. This object is only ever called at capture.
- **Block the overlay's dismissal.** It runs after (`06-flows/flow-capture.md`).
- **Read the budget more than once per Capture.** The named state and its resolved pair are one read;
  `BR-116` makes them one write, and a second read could straddle a change.

## Boundaries

| Direction | With | Contract |
|---|---|---|
| in | `LC-002 region-capturer` | Raw pixels and the region |
| in | `settings` | The Quality Budget, read-only (`BR-110`) |
| out | `LC-005 vault-blobs` | The encoded bytes |
| out | `LC-004 finding-store` | The resolved pair and budget name |

## As-built

`[MISSING]` — step 1 does not exist. Two constants are read from
`crates/snapdown-core/src/domain/setting.rs` and applied to every Capture.

`[MISSING]` — step 4's second half does not exist. Nothing carries the resolved pair to the store, and
`05-model/data-model.md` has no columns for it.

Steps 2 and 3 are built and tested. The delta is entirely the resolution and the record — which is to
say, `DEC-004` and `NFR-18`, and nothing about the encode itself.
