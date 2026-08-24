---
id: SPEC-w8-capture-becomes-real
companions:
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .control/registry/components.yaml
  - .what/business-rules.md
  - .what/finding/SRS-finding.md
  - .what/finding/03-domain/state-machines.md
  - .what/finding/04-usecases/EXPERIENCE.md
  - .what/finding/05-scenarios/SCN-03-the-quality-budget-that-resolves-differently.md
  - .what/finding/05-scenarios/SCN-04-the-note-line-deleted-without-its-marker.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/cross-cutting.md
  - .how/finding/SDD-finding.md
  - .how/finding/01-ux/DESIGN.md
  - .how/finding/04-components/LC-003-image-reducer.md
  - .how/finding/06-flows/flow-capture.md
  - .how/bundle/SDD-bundle.md
  - .how/bundle/05-model/data-model.md
  - .what/bundle/SRS-bundle.md
  - .what/bundle/04-usecases/UC-9-turn-what-i-picked-into-one-review.md
  - .constitution/project/codebase-stack-guide.md
sources:
  - .control/registry/requirements.yaml
  - .control/reports/ASSESS-BUG-14.md
  - .control/questions/assumptions.md
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete,
> preservation-validated contract for what to build, test, and validate. Source documents listed in
> frontmatter are for traceability — consult them only if you need narrative rationale or prose
> colour this contract intentionally omits.

# W8 — Capture becomes real

## Why

**Snapdown does not take screenshots.** Three separate functions write a seventeen-byte fake PNG
header and call it an image, and a fourth promise — the Note at capture time — was never built at
all. This is the product's central capability, and it has been reported as delivered since W2.

| Where | What it actually does |
|---|---|
| `commands/capture.rs:197` `generate_placeholder_image` | 17 bytes: signature, width, height, quality. No IHDR, IDAT or IEND |
| `store/src/image/pipeline.rs:26` `reduce_image` | Same fake header, then copies `input_bytes[16..]` through. Its own comment says *"downscaled payload simulation"* |
| `store/src/image/burner.rs:20` `burn_markers` | Same fake header. Markers are **never drawn onto anything** |
| `CaptureOverlay.tsx:76-100` | Captures on mouse-up and never asks for a Note. `grep note` returns zero hits |
| `commands/bundle.rs:41-48` | Records `bundles/{id}/finding_{pos}_burned.webp` for every item. **Nothing ever writes that file** |

**This is not a design gap. The design is right and was never built.**
`ARCHITECTURE-SPINE.md:245` already lays out `snapdown-capture/  # screen capture, overlay geometry,
image reduction` and assigns `CAP-1` and `CAP-2` to it. `components.yaml` already registers `LC-002`
region-capturer, `LC-003` image-reducer, `LC-012` marker-burner and `LC-029` capture-note-field.
Every one of those is a decision taken at G3 and never executed. `crates/` holds only
`snapdown-bridge`, `snapdown-core`, `snapdown-store`.

**Why it survived five waves and three audits: nothing ever decoded an image.** Every test here
asserts *dimensions*, and a fake header carries correct dimensions.

**And the burn is reached by nobody** (`BUG-19`, found while writing the `W8-S5` brief). A repo-wide
grep for `MarkerBurner` outside its own file and tests returns exactly two hits, and both are
re-exports. `SDD-bundle.md:36` says *"Markers are burned at compose time, into a copy"* and
`data-model.md:63` says `image_path` is *"the Bundle's own copy"* — the design is written, the burner
is built, and the composer never calls it. This is the **fifth** time this shape has landed here,
after `BUG-4`, `BUG-5`, `BUG-6` and the `EmptyState` sweep, and it is worse than its predecessors in
one respect: the caller writes a path that *implies* the call happened, so a dangling reference reads
as a bug in whoever reads it. `W8-S3`'s five tests are green and honest — they prove the burner draws
correctly, and they cannot prove anybody calls it.

## Capabilities

- **CAP-1** — Capture a screen region with a Note, from a hotkey
  - **intent:** The Reviewer boxes a region of their screen and says what is wrong with it, and both
    the picture and the sentence are kept together.
  - **success:** After a capture, the stored file **decodes as an image**, its pixels are the content
    of the requested region rather than a uniform fill, and the Finding carries the Note the Reviewer
    typed. `Esc` during the note step leaves no Finding at all.

- **CAP-2** — Reduce every captured image under a budget the Reviewer sets
  - **intent:** Every captured image is made smaller automatically, under the budget the Reviewer
    chose, without them having to think about it.
  - **success:** A reduced image **decodes**, its long edge is the resolved `max_long_edge`, and its
    pixels are a scaled rendering of the source — not a copy of the source bytes behind a rewritten
    header. An image already under the long edge is not upscaled.

- **CAP-3** — Hold Findings with their Notes and Markers (the burn half)
  - **intent:** A Bundle handed to someone carries its Markers **on the image**, so the numbered
    badges survive leaving Snapdown.
  - **success:** A burned image decodes, differs from its source **in pixels** at the coordinates
    where Markers sit, and keeps its source's dimensions. Burning no Markers returns the source bytes
    unchanged.

- **CAP-4** — Compose a Bundle whose recorded image copies exist (the composition half)
  - **intent:** When the Reviewer hands a Bundle over, every image reference in it resolves to a file
    that is really there, with that Finding's Markers drawn on it.
  - **success:** After composing a Bundle, the file at each `BundleItem.image_path` **exists and
    decodes**, and it carries the Markers of the Finding it came from — asserted on the file, never on
    the path string. The stored image is not re-reduced on the way through.

## Constraints

- **The dimension arithmetic already written is correct and MUST be kept.**
  `compute_reduced_dimensions_for_pair`, the `Auto` resolution `W6-S4` built, and the fixed presets
  are the only part of this that works, and they are what the new code plugs into. This wave replaces
  the bytes between them, never the arithmetic.
- **`AD-4` — an image is reduced exactly once, at capture, and no original is kept.** The burn
  therefore operates on the already-reduced bytes and MUST NOT re-reduce them. Re-encoding on every
  burn would also break `AD-9`.
- **`AD-9` — a Bundle copies the same bytes.** `test_golden_markdown.rs` currently proves
  byte-identity **of a fabrication**. The golden must be regenerated from real output and shown to
  fail when the bytes change.
- **`SCN-04` — a Marker with no Note line is reported in the note pane and NEVER annotated on the
  image.** An app-only state must not be burned into an artifact that is read on another machine
  under another theme.
- **`BR-31` — a region smaller than 8×8 is refused.** Already enforced; it must survive.
- **`AD-2` — a record MUST NOT be committed before its files exist.** The `bundle_item` row and the
  file at its `image_path` belong to one unit of work, and if any part of it fails the prior state
  stands. `UC-9`'s failure flows are explicit: a failed burn abandons the whole composition, no rows
  and no files (`BR-5`), and a store write that fails after the files are written removes them again.
- **`BR-13` — a selected Finding whose image file is missing refuses the whole composition** and names
  the Finding. No Bundle with a broken image reference is ever written.
- **`AD-3` — normalised coordinates become pixels in `LC-012` and nowhere else**, at the stored
  image's own dimensions. `BR-8` forbids any later step re-encoding or re-scaling, so the composition
  burn reads the already-reduced bytes exactly as `AD-4` left them.
- **A Finding with no Markers still gets its own image copy**, with nothing drawn on it (`UC-9`
  alternate flow 1). “Burning no Markers returns the source bytes unchanged” is what makes that copy
  byte-identical, not a reason to skip writing it.
- **Marker colours are theme-invariant on purpose.** `--color-marker*` is one of the four deliberately
  theme-invariant token groups (`AD-10`), because the burned image is read elsewhere. The burn must
  not consult the running theme.
- **Mutation is the acceptance criterion for every test this wave writes or converts**, not a nicety.
  `W7` proved why: a test that read correctly and asserted a plausible thing still passed with its own
  defect reinstated, and only mutation revealed it.
- **Never commit a captured screenshot, or any test fixture derived from real capture output.** The
  repository is public and the brief forbids it. Fixtures must be **synthesised in the test** — drawn
  programmatically — never recorded from a real screen.

## Non-goals

- **Cross-platform capture.** The spine's Deferred section is explicit: the capture port exists so a
  macOS or Linux adapter is *possible*, and the brief forbids designing against the abstraction
  before the Windows one is proven.
- **Measuring `NFR-1` and `NFR-2`.** The 200 ms overlay and 500 ms return-to-focus budgets become
  measurable for the first time once capture is real, and neither is scheduled here: they need an
  instrument this project does not have. `OQ-24` records four UI verifications and four failures.
- **Re-opening the Quality Budget presets.** `DEC-004` settled them and `W6-S4` built the resolution.
- **Changing the Marker interaction in the Editor.** `W6-S7` built `MarkerLayer` and mounted it. This
  wave changes what the **burn** does with Markers, not how they are placed.
- **A capture history, retake, or multi-region capture.** Not promised anywhere.
- **`BUG-1`, the `finding_id` cascade on `bundle_item`.** `data-model.md` calls it live and wrong and
  `SCN-05` carries the case. `W8-S6` writes the file the row records; it does not touch the schema.
- **Naming the stored image format in the corpus.** See `OQ-26` below.

## Success signal

The Reviewer presses the hotkey, boxes a region, types what is wrong, and presses Enter. What lands
in the Vault is a **picture of the thing they boxed**, smaller than the screen, with their sentence
beside it — and when they hand a Bundle to someone else, every image the Bundle references is a file
that is really there, with the numbered badges visible on it on that person's machine.

## Assumptions

- Two third-party dependencies are unavoidable: a screen-capture crate and an image codec. Neither
  capability can responsibly be written from scratch. **Which crates is `W8-S1`'s decision to make and
  record** — it is the first dependency added since the workspace was created.

## Open Questions

- **The test-surface conversion may reveal that a currently-green assertion was never meaningful.**
  `W8-S5` names the files to read. Where a test turns out to have been asserting nothing, that is a
  finding to report, not a line to quietly rewrite.
- **`OQ-26` — the stored image format is unnamed anywhere in the corpus.** A corpus-wide search for
  `png` and `webp` returns zero hits outside the registries, while `bundle.rs:41` records a `.webp`
  path and `burner.rs` emits PNG. **This SPEC does not resolve it and MUST NOT be read as resolving
  it.** `OQ-26` frames the two branches: if a format is an encoder choice, `W8-S6` picks one and
  records a `DEC-` while the corpus stays silent on purpose; if it is a promise, it belongs in the
  spine as an `AD-N`. What is certain either way is that the present state is not a choice — one half
  of the pipeline writes a path the other half cannot fill.
