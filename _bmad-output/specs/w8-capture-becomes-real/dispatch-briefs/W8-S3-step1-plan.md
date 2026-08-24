# W8-S3 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W8**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w8-capture-becomes-real/`
- `story_id`: `W8-S3`

`W8-S1` and `W8-S2` have landed: real capture, real reduction, `image 0.25.10` in the workspace.

## The defect — the third simulation, and the one nobody registered

`crates/snapdown-store/src/image/burner.rs:20`:

```rust
output.extend_from_slice(b"\x89PNG\r\n\x1a\n");
output.extend_from_slice(&dimensions.width.to_be_bytes());
output.extend_from_slice(&dimensions.height.to_be_bytes());

output.push(markers.len() as u8);
for m in markers {
    output.push(m.ordinal as u8);
    // ... pixel coordinates appended as raw bytes
}
```

**Markers are never drawn onto anything.** The coordinates are computed correctly and then written
into a byte blob nobody renders. Its test asserts only that the first eight bytes are a PNG signature
and that the dimensions are preserved.

`BUG-14` was registered on the first two simulations. **This third one was found while writing the
assessment**, which is itself the lesson: nobody had decoded an image, so nobody found it either.

## What it breaks

`FR-8`'s promise is that a handed-over Bundle carries its Markers **on the image**. `BG-1` — the goal
the product is built on — is that a note is unambiguously attached to the image it describes. The
numbered badge is the attachment, and it currently does not survive leaving Snapdown.

## Three constraints, and the second is the one most likely to be got wrong

**1. `AD-4` — an image is reduced exactly once, at capture, and no original is kept.** The burn
therefore operates on the **already-reduced** bytes and MUST NOT re-reduce them. Re-encoding on every
burn would also break `AD-9`'s byte-identity promise.

**2. `SCN-04` — a Marker with no Note line is reported in the note pane and NEVER annotated on the
image.** Read
`.what/finding/05-scenarios/SCN-04-the-note-line-deleted-without-its-marker.md` before planning. The
reasoning: the image is exported and read on another machine under another theme, so an **app-only
state must not be burned into an artifact**. The unbound Marker still exists, still holds its
position and its number, and is still reported in the note pane — it is simply not drawn.

**3. Marker colour is theme-invariant on purpose.** `--color-marker*` is one of exactly four
deliberately theme-invariant token groups under `AD-10`, and the reason is this burn: the image is
read elsewhere, so this machine's theme is the wrong reference. **The burn must not consult the
running theme.** `web/ui/src/styles/tokens.css` holds those values with a comment saying why.

## The tests

`waves.yaml` records five, carried through verbatim:

```
cargo::a_burned_image_decodes_and_differs_from_its_source_in_pixels
cargo::a_burned_marker_changes_pixels_at_its_own_coordinates
cargo::a_burned_image_keeps_the_dimensions_of_its_source
cargo::a_marker_with_no_note_line_is_never_drawn_on_the_image
cargo::burning_no_markers_returns_the_source_bytes_unchanged
```

The second is the substance: **decode the output and check that pixels changed where the Marker sits,
and did not change somewhere it does not.** The first alone would pass on any re-encode.

The last one is load-bearing for `AD-9`: an unmarked Finding's bytes must come through **unchanged**,
not merely equivalent — a re-encode that produces a visually identical but byte-different image breaks
the Bundle's byte-identity promise.

**Fixtures synthesised programmatically**, never recorded. **Mutation is the acceptance criterion.**

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** Unknown cause → `wdi-systematic-debugging` first; a
  third failed attempt is an escalation.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **Verification is run, not assumed.** Every command in `AGENTS.md` § Code.
- **Write UTF-8, no BOM, and watch for a lone cp1252 byte.**
- **No scratch files in the commit. Do not push.**

## Done means

`_bmad-output/specs/w8-capture-becomes-real/stories/W8-S3-*.md` exists, carries an
`<intent-contract>`, and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
