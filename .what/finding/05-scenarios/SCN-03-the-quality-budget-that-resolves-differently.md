---
type: scenario
id: SCN-03
component: finding
branches_from: UC-1
created: "2026-08-23"
---

# SCN-03 — Two captures, one budget, two different reductions

Branches from `UC-1` step 8, at the reduction that happens behind the dismissed overlay. It is the
scenario that makes `DEC-004` testable, and without it "Auto" can ship as the old constant wearing a
new label — every other consequence of `FR-5` would still pass.

## Setup

The Quality Budget is `Auto`, the shipped default. The Reviewer takes two Captures a minute apart:

- **A** — a tooltip on a button. `312 × 118` px. Mostly flat colour, four words of 11 px text.
- **B** — a full 4K screen. `3840 × 2160` px. A dense dashboard.

## What must happen

**A is not downscaled at all.** Its long edge is 312 px; every plausible cap is above it, so the cap
does nothing and the encoder decides everything. `Auto` must therefore resolve a **high** encoder
quality for A, because 11 px text is exactly where lossy artefacts are visible and there are no
pixels to spare.

**B is downscaled hard.** 3840 px is far over any cap, the cap decides almost everything, and the
encoder barely matters. `Auto` resolves a **lower** encoder quality for B, because the downscale has
already removed the detail that quality would have preserved.

**The two resolved pairs are different, and both are stored** with their Findings (`NFR-18`,
`BR-105`).

## The assertion that matters

```
assert resolved(A) != resolved(B)
```

A test that captures both and finds identical parameters is a **failing test**. It is stated as a
consequence of `FR-5` for exactly this reason: it is the only assertion that a constant cannot pass.

Everything else `FR-5` promises — a default the Reviewer never changes, a visible stored size, no
re-encoding of existing Findings, refusal of out-of-range values — is satisfied perfectly well by
`1600` and `75`, which is what shipped.

## What this scenario does NOT settle

**Whether `Auto`'s output is legible.** That is `OQ-3`, restated by `DEC-004` and still unmeasured.
This scenario asserts that the derivation *varies*, which is a property of the mechanism. Whether it
varies to the *right* values needs a Reviewer looking at B and deciding they do not want to open the
original, and no test settles it.

The two are worth keeping apart: a derivation that varies wrongly is a tuning problem, and a
derivation that does not vary is a design that was never built.

## The upgrade case

A month later the derivation is tuned. A third Capture, C, of the same tooltip as A, now resolves a
different pair.

**A is not re-encoded** (`BR-9`, `FR-5`), so A and C differ. Both say what produced them, because both
stored their resolved pair — which is the entire reason `NFR-18` exists and the reason it was found by
reading `DEC-004`'s Cost section rather than its Decision.

## Tests this scenario names

- `finding::auto_resolves_a_different_pair_for_a_small_region_than_for_a_full_screen`
- `finding::auto_resolves_a_higher_encoder_quality_when_no_downscale_applies`
- `finding::every_stored_finding_carries_the_pair_that_was_applied_to_it`
- `finding::a_finding_stored_before_a_derivation_change_is_not_re_encoded`
- `finding::a_finding_can_state_which_named_budget_produced_it`
