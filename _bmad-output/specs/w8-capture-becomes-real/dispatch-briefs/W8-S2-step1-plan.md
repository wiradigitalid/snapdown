# W8-S2 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W8**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w8-capture-becomes-real/`
- `story_id`: `W8-S2`

**`W8-S1` has landed.** `crates/snapdown-capture` exists, `xcap 0.9.8` and `image 0.25.10` are pinned
in the workspace, and a real grab now produces a real PNG. Read what it built before planning — you
are consuming its output, and the codec dependency is already there.

## The defect

`crates/snapdown-store/src/image/pipeline.rs:26-37`:

```rust
let mut out_bytes = Vec::new();
// Standard PNG/image signature and downscaled payload simulation
out_bytes.extend_from_slice(b"\x89PNG\r\n\x1a\n");
out_bytes.extend_from_slice(&target_dims.width.to_be_bytes());
out_bytes.extend_from_slice(&target_dims.height.to_be_bytes());
out_bytes.push(resolved.encoder_quality);

if input_bytes.len() > 16 {
    out_bytes.extend_from_slice(&input_bytes[16..]);
}
```

Its own comment says **"downscaled payload simulation"**. It was never meant to be mistaken for real,
and it was. **No image has ever been reduced.**

The thumbnail path immediately below is the same lie a second time: a second fake header carrying
thumbnail dimensions.

## What has been reported as true on the strength of this

- **`FR-4`** — *"Reduce every captured image automatically"*
- **`NFR-3`** — *"Every stored Finding image fits the Quality Budget's long edge, and the shipped
  default keeps a full-screen capture legible"*

Both have been green since W2 because a fake header carries a correct number. **Treat every existing
green assertion in this area as unproven until you have decoded the output yourself.**

## What MUST be kept, exactly as it is

`compute_reduced_dimensions_for_pair`, `compute_reduced_dimensions_with_edge`, the `Auto` resolution
`W6-S4` built, and the fixed presets. **This arithmetic is correct — it is the only part of this that
works, and it is what your new code plugs into.** You are replacing the bytes between the inputs and
the outputs, never the arithmetic.

`waves.yaml` names a test for exactly this — `the_resolved_pair_arithmetic_is_unchanged_by_this_story`
— and it exists because the tempting shortcut, while rewriting the encoder, is to "tidy" the
resolution alongside it.

**An image already under the resolved long edge MUST NOT be upscaled.** Scaling is a reduction, not a
fit-to-size.

## The tests

`waves.yaml` records five, carried through verbatim:

```
cargo::a_reduced_image_decodes_and_its_pixels_are_the_scaled_source
cargo::a_reduced_image_honours_the_resolved_long_edge
cargo::an_image_already_under_the_long_edge_is_not_upscaled
cargo::a_thumbnail_decodes_and_is_smaller_than_its_source
cargo::the_resolved_pair_arithmetic_is_unchanged_by_this_story
```

The first is the one that matters: **decode the output and compare its pixels to a scaled rendering
of the source.** Asserting a signature and a dimension is exactly what let this survive.

**Fixtures are synthesised programmatically** — a known gradient or pattern whose scaling you can
predict — never recorded from a real screen. The repository is public and the brief forbids a fixture
derived from real capture output.

**Mutation is the acceptance criterion.** Break each behaviour, watch the test go red, put it back.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` before proposing any fix; a third failed attempt is an escalation.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **Verification is run, not assumed.** Every command in `AGENTS.md` § Code.
- **Write UTF-8, and no BOM** — and beware a lone cp1252 byte: `W8-S1`'s plan arrived with a raw
  `0xD7` for `×` inside otherwise valid UTF-8.
- **No scratch files in the commit. Do not push.**

## Done means

`_bmad-output/specs/w8-capture-becomes-real/stories/W8-S2-*.md` exists, carries an
`<intent-contract>`, and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
