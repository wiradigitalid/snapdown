# W8-S1 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W8**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first — its
`## Code` section carries the verification commands and the pitfalls this repository has paid for.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w8-capture-becomes-real/`
- `story_id`: `W8-S1`

## The defect

`apps/desktop/src-tauri/src/commands/capture.rs:197`:

```rust
fn generate_placeholder_image(width: u32, height: u32, encoder_quality: u8) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.push(encoder_quality);
    bytes
}
```

**Seventeen bytes.** The PNG signature, the width, the height, the quality. No IHDR, no IDAT, no
IEND. It is not a PNG; it is a label that says PNG. **Snapdown has never taken a screenshot.**

## This is not a design gap — you are executing a decision, not making one

`ARCHITECTURE-SPINE.md:245` already lays out the workspace:

```
snapdown-capture/    # screen capture, overlay geometry, image reduction
```

and assigns `CAP-1` and `CAP-2` to it. `components.yaml` already registers **`LC-002`
region-capturer**, `container: desktop-app`, `area: capture-pipeline`, with no implementation.

`crates/` holds `snapdown-bridge`, `snapdown-core`, `snapdown-store`. **Build the crate where the
spine already put it.** It costs nothing extra and closes the gap between the document and the tree.

## The one real decision this story makes, and it must be recorded

**Two third-party dependencies: a screen-capture crate and an image codec.** These are the first
dependencies added since the workspace was created. The current list is exactly:

```
serde serde_json thiserror uuid chrono rusqlite rand tempfile tiny_http ureq base64
tauri tauri-build tauri-plugin-{single-instance,global-shortcut,autostart} winreg
```

In the plan you MUST:

1. **Name both crates and pin their versions.**
2. **Say why each was chosen** — this is the part a reader in six months needs.
3. **Check the licence is compatible with a public repository**, and say what it is.
4. Say what happens on a machine with **no display** — CI is one. Do not let that case silently pass
   as a green test.

`xcap` and `image` are the obvious candidates and both are on crates.io; you are not obliged to pick
them, but if you pick something else, say why.

## Constraints that bind this story

- **`snapdown-core` must stay free of IO.** There is a test — `snapdown_core_has_no_io_dependency` —
  that enforces it. The capture crate MUST NOT be pulled into `snapdown-core`.
- **`BR-31` — a region smaller than 8×8 is refused.** Already enforced at `capture.rs:42` and in the
  overlay. It must survive.
- **The dimension arithmetic already written is correct.** `compute_reduced_dimensions_for_pair` and
  the `Auto` resolution `W6-S4` built are what your output feeds into. Do not touch them; `W8-S2`
  owns the reduction itself.
- **A region may span monitors, and monitors may have different scale factors.** `capture.rs` already
  carries a `source_monitor`. Say in the plan how a region is mapped onto a physical grab, and what
  happens when the requested region is larger than the monitor — the test list requires it be
  **refused, not silently clamped**.

## Never commit a captured screenshot, and this shapes the tests

The repository is public and the product brief forbids *"a captured screenshot, a token, a client's
name, or any test fixture derived from real capture output."* CI refuses tracked images.

**So fixtures must be synthesised programmatically in the test** — draw a known pattern, encode it,
assert on it — never recorded from a real screen. `a_captured_image_is_not_uniformly_one_colour`
exists because a fake or blank grab is the failure mode that has actually happened here: an earlier
UI audit produced a blank white overlay image and reported it as a dark scrim.

## The tests

`waves.yaml` records four, carried through verbatim:

```
cargo::a_captured_region_decodes_as_a_real_image
cargo::a_captured_region_has_the_dimensions_that_were_requested
cargo::a_captured_image_is_not_uniformly_one_colour
cargo::a_region_larger_than_the_monitor_is_refused_not_clamped_silently
```

**`a_captured_region_decodes_as_a_real_image` is the test this whole wave exists for.** Nothing in
this repository has ever decoded an image; that is precisely why a 17-byte fake passed five waves and
three audits. Decoding is the assertion — asserting on a signature and a dimension is what got us
here.

**Mutation is the acceptance criterion.** For each test, break the thing it claims to cover, watch it
go red, put it back. `W7` proved this is not optional: a test that read correctly and asserted a
plausible thing still passed with its own defect reinstated.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` **before** proposing any fix. A third failed fix attempt is the signal
  to escalate, not to try a fourth.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation is **reported** and becomes a `DEC-`.
- **Verification is run, not assumed.** Every command in `AGENTS.md` § Code, and read the four ways a
  verification run lies recorded there.
- **Write UTF-8, and no BOM.**
- **No scratch files in the commit. Do not push.**

## Done means

`_bmad-output/specs/w8-capture-becomes-real/stories/W8-S1-*.md` exists, carries an
`<intent-contract>`, and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
