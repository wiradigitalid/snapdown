---
id: W8-S6
title: "W8-S6: The burn is actually called, and the recorded path holds a file"
type: 'bug'
wave: W8
status: done
created: '2026-08-24'
review_loop_iteration: 0
followup_review_recommended: true
dependencies:
  - W8-S5
files:
  - apps/desktop/src-tauri/src/commands/bundle.rs
  - crates/snapdown-store/src/image/burner.rs
  - apps/desktop/src-tauri/tests/test_bundle_export.rs
context:
  - _bmad-output/specs/w8-capture-becomes-real/SPEC.md
  - _bmad-output/specs/w8-capture-becomes-real/stories.yaml
  - _bmad-output/specs/w8-capture-becomes-real/dispatch-briefs/W8-S6-step1-plan.md
  - _bmad-output/specs/w8-capture-becomes-real/stories/W8-S5-the-golden-test-stops-proving-byte-identity-of-a-fabrication.md
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .control/registry/components.yaml
  - .control/questions/assumptions.md
  - .what/business-rules.md
  - .what/bundle/SRS-bundle.md
  - .what/bundle/04-usecases/UC-9-turn-what-i-picked-into-one-review.md
  - .what/finding/05-scenarios/SCN-04-the-note-line-deleted-without-its-marker.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/cross-cutting.md
  - .how/bundle/SDD-bundle.md
  - .how/bundle/05-model/data-model.md
  - .constitution/project/codebase-stack-guide.md
warnings:
  - >-
    **`OQ-26` is resolved in favour of PNG, and the evidence is in the build, not in the corpus.**
    The workspace pins `image = { version = "0.25.10", default-features = false, features = ["png"] }`
    (`Cargo.toml:37`), so **WebP is not compiled into this binary in either direction** — a `.webp`
    file could not be written even if the path said so. `capture.rs:97` already writes
    `findings/capture_{ts}.png`, `pipeline.rs:45` encodes PNG, `burner.rs:76` encodes PNG. The
    `.webp` at `bundle.rs:41` is the **only** disagreeing site, it is a literal inside a format
    string, and no file has ever existed behind it. This is a `DEC-` the coordinator opens through
    `wdi-decision`; the corpus stays silent on purpose and this story MUST NOT amend `.what/` or
    `.how/`.
  - >-
    **`BUG-20` is fixed in the burner, not in the export path, and that is a deliberate choice with a
    cost.** `burn_markers` returns `Result` and its own rustdoc claims `AD-4` — *"preserves
    dimensions"* — which it cannot honour for bytes it never read. Validation placed in the caller is
    exactly the unwritten premise `BUG-20`'s `why_undetected` names as the recurring shape here. Cost:
    one decode per zero-marker copy, on an operation counted in tens. `BR-13` stays in the composer,
    because presence of a file is something the burner cannot see — it never receives a path.
  - >-
    **The fix must not disturb `W8-S5`'s landed byte-identity assertion.**
    `test_bundle_image_copy.rs:58` appends `AD-4-BYTE-IDENTITY-PRESERVATION` after the PNG's `IEND`
    and then asserts `zero_marker_copy == source_bytes`. Decoding before the early return keeps that
    green **only** because the `image` crate tolerates trailing bytes after `IEND` — the same file
    already proves it at line 61. The early return MUST still return `input_bytes.to_vec()`, never a
    re-encode.
  - >-
    **REPORTED, NOT FIXED — the composed Markdown points at the Finding's clean image, not at the
    Bundle's burned copy.** `markdown.rs:30` formats `item.finding.image_path`. So after this story the
    file at `BundleItem.image_path` exists and carries badges, and the document a Reviewer hands over
    still references the un-burned Finding image — and references a file that `FR-13` allows to be
    deleted. `FR-8` is only half-restored by this story. Out of scope on two grounds: it changes
    `MarkdownSerializer`, whose output `W8-S5` has just pinned byte-exactly in two golden tests, and
    `cross-cutting.md:148` specifies `images/{finding-image-filename}` — a shape the serializer already
    departs from in other ways. **Candidate defect for the coordinator to register.**
  - >-
    **REPORTED, NOT FIXED — `bundle.rs:35` silently skips a `finding_id` that no longer resolves.**
    `if let Some(detail)` drops it and composes a shorter Bundle. `UC-9`'s failure flow says *"A
    Finding deleted mid-composition fails the composition, all-or-nothing"*. This story adds the
    `BR-13` refusal for a missing **image file** (it must, to read the bytes at all) and deliberately
    leaves the missing **row** silent, which is an inconsistency inside one loop. Named here for
    adjudication rather than fixed, because refusing is new behaviour and this story adds none.
  - >-
    **REPORTED, NOT FIXED — `sharing.rs:116` `if let Ok(bytes) = vault_store.read_blob(...)`
    silently drops an item image it cannot read when publishing.** Today it drops all of them, since
    none exist. After this story it will carry real burned images and swallow only the unreadable
    ones — a `let`-shaped swallow of the same family as the five found on 2026-08-23.
  - >-
    **A fourth test name is required and `waves.yaml` records only three.** `BUG-20` needs its own
    assertion; the three named tests all pass with the corrupt-source defect present. Proposed:
    `a_bundle_export_refuses_a_corrupt_source_image_even_with_no_markers_to_draw`. **Adding the row to
    `waves.yaml` is the coordinator's** — a registry is not edited from a story.
deferred:
  - >-
    The Markdown image reference pointing at the Bundle's own copy (`FR-8`'s other half, and
    `cross-cutting.md:148`'s shape). Needs a `DEC-` or a defect row first, and it invalidates two
    goldens `W8-S5` has just pinned.
  - >-
    Refusing composition when a selected `finding_id` no longer resolves (`UC-9` failure flow 4).
  - >-
    `sharing.rs:116`'s swallowed `read_blob`.
  - >-
    `BUG-1`'s `finding_id` cascade. Explicit `Non-goal` in `SPEC.md`. The schema is not touched.
  - >-
    A composition test **class** (`OQ-23`). This story writes reach assertions for one seam; it does
    not build the general instrument that would have caught `BUG-4`, `BUG-5`, `BUG-6`, the
    `EmptyState` sweep and `BUG-19`.
---

<intent-contract>

## Intent

**`MarkerBurner` is unreachable from the running application, and the database records that it was
reached.** `apps/desktop/src-tauri/src/commands/bundle.rs:41-48` writes
`bundles/{bundle_id}/finding_{pos}_burned.webp` into `BundleItem.image_path` for every item and
nothing anywhere writes a file there. Verified afresh at plan time:

```
$ grep -rn "MarkerBurner" --include=*.rs crates apps        # minus burner.rs and tests/
crates/snapdown-store/src/image/mod.rs:4   pub use burner::MarkerBurner;
crates/snapdown-store/src/lib.rs:8         pub use image::{ImageReducer, MarkerBurner, ...};
```

Two hits, both re-exports. The only `write_blob` calls in the desktop crate are `bundle.rs:75` (the
Markdown) and `capture.rs:100` (the reduced capture).

This story **adds no behaviour**. `SDD-bundle.md:36` — *"Markers are burned at compose time, into a
copy"* — and `data-model.md:63` — `image_path` is *"**The Bundle's own copy**, relative to the Vault
root. Not the Finding's"* — were written at G3 and G4 and never executed. `LC-010 bundle-composer`
already owns *"read the Findings, burn the images, render the Markdown, write the rows and files"* as
one transaction (`SDD-bundle.md:53`). The design is right and was never built. This is the fifth
instance of that shape here, and the one where the caller writes a path that implies the call
happened.

Two defects are in scope: `BUG-19` (the missing call) and `BUG-20` (the burner's no-markers fast
path reporting success for bytes it never decoded), which is scheduled to this story precisely because
this story is what gives it reach.

### The two constraints that shape every line of it

**`AD-4` (`ARCHITECTURE-SPINE.md:106-109`)** — *"No later stage — composition, publishing, or serving
— may re-encode or re-scale a stored image. A Bundle's image is a copy of the Finding's image with
Markers drawn on it, at the same dimensions."* The composer reads the stored bytes and hands them
straight to `burn_markers`. It MUST NOT touch `ImageReducer`. `BR-8` says the same in one line.

**`AD-2` (`SDD-bundle.md:85-89`)** — *"A record MUST NOT be committed before its files exist."*
`UC-9`'s failure flows spell out the rollback: a failed burn abandons the whole composition, no rows
and no files (`BR-5`); running out of space partway *"removes any copies already written"*; a store
write that fails after the files are written *"removes the files it wrote, then reports the
failure"*.

## `OQ-26` — the extension, resolved as an encoder choice

`OQ-26` frames two branches: a format is either an encoder choice code may pick, or a promise that
belongs in the spine as an `AD-N`. **The evidence says encoder choice, and PNG.** Four facts, all
checkable:

| Evidence | Reading |
|---|---|
| `Cargo.toml:37` — `image` is pinned `default-features = false, features = ["png"]` | WebP is **not compiled in**, in either direction. `.webp` is not a choice this binary can make |
| `capture.rs:97` writes `findings/capture_{ts}.png`; `pipeline.rs:45` and `burner.rs:76` both use `PngEncoder` | Three of the four sites already agree, and they are the three `W8-S1`–`W8-S3` just made real |
| `handlers.rs:199-204` maps `.png` → `image/png`; `store.go:161` **defaults** to `image/png` | Both serving surfaces already carry PNG correctly. The extension change needs no companion edit |
| A corpus-wide search for `png` and `webp` across `.what/` and `.how/` returns nothing | No promise is being contradicted, and none is being made |

`ResolvedPair.encoder_quality` is the one datum that argues the other way — a quality knob implies a
lossy codec, and `PngEncoder` ignores it. That is a real observation about `NFR-3` and it is **not**
this story's: `W8-S2` chose PNG for the reducer and the column is already populated. Recorded here so
the `DEC-` weighs it.

**What this story does:** changes the literal at `bundle.rs:41` to `.png` and states the reasoning in
the Spec Change Log. **What it does not do:** amend `.what/` or `.how/`, or close `OQ-26`. Opening the
`DEC-` and closing the question are the coordinator's, through `wdi-decision` and `wdi-question`.

Every other `.webp` string in the repository is an opaque fixture (`test_sqlite_bundles.rs`,
`test_sqlite_findings.rs`, `test_bundle_failures.rs`, the serializer tests) — the stores treat
`image_path` as text and none of those go through `create_bundle_impl`. Grepped; nothing else has to
move.

## `BUG-20` — the burner validates, and here is why

`burner.rs:48-50`:

```rust
if active_markers.is_empty() {
    return Ok(input_bytes.to_vec());
}
```

`image::load_from_memory` is the next line, so a corrupt, truncated or non-image blob returns `Ok`,
and the dimension check below it is skipped for the same reason. `SCN-04` makes the path routine, not
exotic: a Finding whose Markers all lack Note lines takes it every time, and so does a Finding with no
Markers at all.

**Decision: the burner validates.** Three reasons.

1. **The function claims the invariant.** Its rustdoc says *"AD-4: Operates on already-reduced bytes
   and preserves dimensions without re-scaling."* A function that returns `Result` and asserts
   something about dimensions cannot honour that claim for bytes it has not read. Fixing the caller
   leaves the claim false.
2. **The alternative is the exact shape that produced this defect.** `BUG-20`'s `why_undetected` says
   it: *"a branch is justified by a premise — here, that the caller has already validated the blob —
   and the premise is never written down, so nobody re-checks it when the caller changes."* Moving
   validation into the export path writes a second such premise for the next caller to inherit. The
   burner is `LC-012` and it is the single choke point.
3. **It splits cleanly with `BR-13`.** The burner never receives a path, so it cannot check presence;
   the composer must, and `BR-13` already requires it to. Presence and reachability in the composer,
   decodability and dimension agreement in the burner. Both, and neither standing in for the other.

**Shape of the fix** — decode first, keep the dimension check, then return the input bytes unchanged:

```rust
let decoded = image::load_from_memory(input_bytes)
    .map_err(|e| CoreError::Validation(format!("Failed to decode image for burning: {e}")))?;
// dimension check, unchanged, now on every path
if active_markers.is_empty() {
    return Ok(input_bytes.to_vec());          // still the input bytes: no re-encode
}
```

`AD-9`/`UC-9` alt-1 byte-identity survives because the return value is still `input_bytes.to_vec()`.
`W8-S5`'s trailing-tag assertion survives because the `image` crate tolerates bytes after `IEND` —
already proven at `test_bundle_image_copy.rs:61`. **Confirm that before believing the rest**: if that
decode were strict, the fix and the landed test would be in direct conflict.

**The test must use a valid PNG header with a corrupt payload.** `W7` proved garbage never reaches
such a path — the decoder rejects it at the front door, which is why
`invalid_input_bytes_returns_validation_error` (`burner.rs:224`) exercises nothing new. Build the
fixture by encoding a real PNG and then corrupting bytes **inside the `IDAT` chunk**, leaving the
8-byte signature and the `IHDR` chunk intact, and assert in the test that the fixture reaches the
decoder — that `load_from_memory` fails *after* accepting the header — the same fixture-reach
obligation `W8-S5` wrote at `test_bundle_image_copy.rs:60`.

## Approach

### 1. `crates/snapdown-store/src/image/burner.rs` — close `BUG-20`

Move the decode and the dimension check above the `active_markers.is_empty()` return, as shown. Update
the rustdoc `AD-9` line so it says the fast path returns the **input** bytes after validating them,
not before. No other change; the drawing code is `W8-S3`'s and is correct.

### 2. `apps/desktop/src-tauri/src/commands/bundle.rs` — close `BUG-19`

Restructure `create_bundle_impl` into one ordered unit of work. The vault must open **before** the
loop, because the loop now reads blobs.

1. Name validation, `bundle_id`, `composed_at`, `md_filename` — unchanged.
2. Resolve the vault path and open `VaultBlobStore` — **moved up**, ahead of the Finding loop. The
   existing error string `Failed to open vault at {vault_path}: {e}` is kept verbatim:
   `composition_that_cannot_open_the_vault_is_refused_not_silently_skipped` asserts on it.
3. For each `finding_id`, in selection order, on `Some(detail)`:
   - `image_path` becomes `bundles/{bundle_id}/finding_{pos}_burned.png`.
   - `blob_exists(&detail.finding.image_path)?` — false is a **refusal naming the Finding**
     (`BR-13`, `UC-9` failure flow 1). Nothing has been written yet at this point, so there is
     nothing to roll back.
   - `read_blob(&detail.finding.image_path)?`.
   - `ImageDimensions::new(detail.finding.image_width, detail.finding.image_height)?`.
   - `MarkerBurner::burn_markers(&stored_bytes, &dims, &detail.markers)?` — a failure abandons the
     composition (`BR-5`). `SCN-04`'s filter and the normalised→pixel conversion are the burner's;
     the composer reimplements neither.
   - Collect `(image_path, burned_bytes)` into a pending-writes vector alongside the `BundleItem`.

   **No `write_blob` inside this loop.** Every burn succeeds before any byte lands, so the common
   failure needs no rollback at all.
4. Render the Markdown from `finding_details` — unchanged call, unchanged bytes.
5. Write the pending image blobs, pushing each successful path onto a `written: Vec<String>`. On the
   first failure, delete everything in `written` and return the error (`UC-9` failure flow 3).
6. Write the Markdown. On failure, delete everything in `written` and return.
7. `create_bundle(&bundle, &bundle_items)`. On failure, delete the Markdown **and** every path in
   `written`, then return (`UC-9` failure flow 4).
8. `Ok(BundleDetail { .. })`.

**Rollback must not swallow.** `bundle.rs:87` currently reads `let _ = vault_store.delete_blob(...)`.
The rollback grows from one file to many, so it gets a small helper that attempts every delete and,
if any refuses, returns an error naming **both** the original failure and the file that would not go —
`AD-2` says the prior state stands, and a cleanup that silently half-completed is not that state.
This is the `let _ =` prohibition in `AGENTS.md` applied to the exact line it warns about.

### 3. `apps/desktop/src-tauri/tests/test_bundle_export.rs` — new

`test_bundle_failures.rs` already carries `build_test_app(db_path, vault_path, web_service_url)`
(lines 25-73), which wires the five stores and sets `SettingKey::VaultPath`. That is the seam the brief
names: `tauri::test::mock_app` yields `STATUS_ENTRYPOINT_NOT_FOUND` here, so the tests call
`create_bundle_impl(input, &state)` directly. Copy that helper into the new file rather than exporting
it from an integration test — integration test binaries do not share modules.

Each test synthesises its own source image programmatically (a gradient, never a solid fill, never a
recorded screenshot), writes it to `findings/...` through `VaultBlobStore`, and creates the Finding row
with `image_width`/`image_height` matching the fixture.

| Test | Asserts |
|---|---|
| `a_bundle_item_image_path_holds_a_file_that_decodes` | For every returned `BundleItem`: `blob_exists(&item.image_path)` is true, `read_blob` returns bytes, `image::load_from_memory` returns `Ok`, and the decoded dimensions equal the Finding's. **Never** that the path string has a shape. Cover a Finding with Markers and a Finding with none in one Bundle — `UC-9` alt-1 says the unmarked one still gets its own copy |
| `an_exported_bundle_image_carries_the_markers_of_its_finding` | Decode the file at `image_path` and the source fixture. At each Marker's pixel centre — `(x * width).round()`, `(y * height).round()` — the two differ; at a pixel proven outside every badge footprint they are identical. Include one Marker with a whitespace-only comment and assert **its** centre is unchanged (`SCN-04`) |
| `a_bundle_export_does_not_re_reduce_the_stored_image` | The decoded exported image's dimensions equal the decoded stored image's, so a second reduction would be visible. Use a fixture whose long edge **exceeds** the default `QualityBudget`'s `max_long_edge`, or the assertion holds for a fixture no reducer would have touched and proves nothing |
| `a_bundle_export_refuses_a_corrupt_source_image_even_with_no_markers_to_draw` | `BUG-20`. A Finding whose stored blob is a valid PNG header with a corrupted `IDAT` and **zero Markers**: `create_bundle_impl` returns `Err`, `list_bundles()` is empty, and no file exists under `bundles/{bundle_id}/`. Assert the fixture reaches the decoder |

### 4. The reach grep, before closing

```bash
grep -rn "MarkerBurner" --include=*.rs crates apps
```

Excluding `crates/snapdown-store/src/image/burner.rs` and every `tests/` path, this must now show a
call site in `apps/desktop/src-tauri/src/commands/bundle.rs`. **If the only hits are still re-exports
the story is not done, whatever the tests say.** `V12` does not help — it checks that an `LC` is
registered, not that it is reached — and `OQ-23` records that there is no composition test class to
catch it.

### 5. Mutation — the acceptance criterion

`cargo test --workspace --no-fail-fast` for every row. Cargo stops at the first failing binary
otherwise, later tests never run, and a live test reads as dead; that produced a false result in
`W8-S2`.

| # | Mutation | Must go red |
|---|---|---|
| 1 | Delete the `write_blob` loop for the image blobs | `a_bundle_item_image_path_holds_a_file_that_decodes` |
| 2 | Replace `burn_markers(...)` with `Ok(stored_bytes.clone())` | `an_exported_bundle_image_carries_the_markers_of_its_finding` |
| 3 | Insert an `ImageReducer::reduce_image` call on the stored bytes before the burn | `a_bundle_export_does_not_re_reduce_the_stored_image` |
| 4 | Restore the early return above the decode in `burner.rs` | `a_bundle_export_refuses_a_corrupt_source_image_even_with_no_markers_to_draw` |
| 5 | Drop the `blob_exists` refusal and let `read_blob` fail | the `BR-13` half of test 1 (a Finding with a missing image must be refused by name) |
| 6 | Commit the `bundle` row before writing the image blobs | the `AD-2` assertion — no rows survive a failed composition |
| 7 | Skip the burn for a Finding with no Markers instead of copying it | the zero-Marker half of test 1 |
| 8 | Remove the `written` rollback so a failed Markdown write leaves images behind | the rollback assertion (no file under `bundles/{bundle_id}/` after a refused composition) |

Row 4 is the one that matters most: it is the defect as shipped, and the other three named tests all
pass with it present.

## Boundaries & Constraints

- **MUST NOT** touch `.what/`, `.how/`, `.constitution/`, or any `applied` `DEC-`. `OQ-26`'s
  resolution is reported for a `DEC-`, not written into the corpus.
- **MUST NOT** touch the `bundle_item` schema or `BUG-1`'s `finding_id` cascade — explicit `Non-goal`.
- **MUST NOT** call `ImageReducer` anywhere in the composition path (`AD-4`, `BR-8`).
- **MUST NOT** reimplement `SCN-04`'s filter or the normalised→pixel conversion in the composer;
  `AD-3` puts the conversion in `LC-012` and nowhere else.
- **MUST NOT** consult the running theme. `--color-marker*` is theme-invariant on purpose (`AD-10`)
  because the burned image is read on another machine.
- **MUST NOT** change `MarkdownSerializer` or either golden. `W8-S5` pinned them byte-exactly
  yesterday.
- **MUST NOT** write `let _ =` on a `Result` an invariant depends on, including in the rollback.
- **MUST NOT** commit a captured screenshot or any fixture derived from one. Every fixture is drawn
  programmatically in the test.
- **MUST NOT** add a dependency. PNG is already compiled in; WebP deliberately is not.
- Files valid UTF-8, no BOM, no stray cp1252 byte. No scratch files in the commit. Do not push.
- Unknown cause → `wdi-systematic-debugging` **before** any fix. A third failed attempt is an
  escalation, not a fourth attempt.

## I/O & Edge-Case Matrix

| Input / condition | Behaviour | Bound by |
|---|---|---|
| Finding with Markers carrying Note lines | Copy written with badges at that image's own dimensions | `AD-4`, `SDD-bundle.md:36` |
| Finding with zero Markers | Copy written, byte-identical to the source | `UC-9` alt-1 |
| Finding whose Markers all have blank comments | Copy written, byte-identical. Nothing drawn | `SCN-04` |
| Finding image file absent from the Vault | Whole composition refused, naming the Finding. No rows, no files | `BR-13`, `UC-9` FF-1 |
| Finding image present but undecodable | Burn returns `Err`; composition abandoned. No rows, no files | `BUG-20`, `BR-5`, `UC-9` FF-2 |
| Stored dimensions disagree with the decoded image | `Err` from the burner's existing dimension check, now reached on every path | `AD-4`, `BUG-20` |
| An image write fails partway | Copies already written are removed; error reported | `UC-9` FF-3 |
| Markdown write fails | Every image copy removed; error reported | `AD-2`, `UC-9` FF-3 |
| `create_bundle` fails after the files are written | Markdown and every image copy removed; error reported | `AD-2`, `UC-9` FF-4 |
| A rollback delete itself fails | Error names the original failure **and** the file that refused | `BR-5`, `AGENTS.md` § `let _ =` |
| Vault cannot be opened | Refused before anything is read or written, message unchanged | existing test |
| Empty `finding_ids` | An empty Bundle with its Markdown, as today. No image loop runs | unchanged behaviour |
| Same Finding in two Bundles | Two independent copies | `BR-12`, `UC-9` alt-2 |
| Marker at a normalised edge (0.0 / 1.0) | Badge clipped, no panic — already covered in `burner.rs:201` | `W8-S3` |

</intent-contract>

## Code Map

- `crates/snapdown-store/src/image/burner.rs` — move the decode and dimension check above the
  `active_markers.is_empty()` early return; the return value stays `input_bytes.to_vec()`. Update the
  `AD-9` rustdoc line. ~4 lines moved, 1 comment corrected.
- `apps/desktop/src-tauri/src/commands/bundle.rs` — `create_bundle_impl` gains the burn: vault opened
  before the Finding loop, `blob_exists` + `read_blob` + `burn_markers` per Finding, pending writes
  landed after every burn succeeds, rollback across images and Markdown, `.webp` → `.png` at line 41.
  No other function changes; `delete_bundle_impl`'s `blob_exists` guard now guards real files.
- `apps/desktop/src-tauri/tests/test_bundle_export.rs` — **new**. Four tests, `build_test_app` copied
  from `test_bundle_failures.rs`, `image` already a dev-dependency (`Cargo.toml:44`).
- **Not modified:** `MarkdownSerializer`, either golden, `test_bundle_image_copy.rs`,
  `test_image_surface.rs`, any `Cargo.toml`, any store, any schema, any frontend file. The bundle
  export is a **caller** of an image producer, not a fourth producer, so
  `every_image_producing_path_decodes_its_own_output` keeps its three rows.

## Tasks & Acceptance

**Execution:**
1. `burner.rs` — hoist the decode and dimension check above the early return; correct the rustdoc.
2. `test_bundle_export.rs` — write
   `a_bundle_export_refuses_a_corrupt_source_image_even_with_no_markers_to_draw` **first** and watch
   it fail against unmodified `bundle.rs`. It is the story's whole premise; a version of it that
   passes today is the wrong test.
3. `bundle.rs` — restructure `create_bundle_impl` per Approach § 2, extension included.
4. `test_bundle_export.rs` — write the three `waves.yaml` tests.
5. Run the eight mutations under `--no-fail-fast`; record each red and its restore.
6. Run the reach grep and paste its output into the Spec Change Log.
7. Run every command in `AGENTS.md` § Code, read the way § Code says to read them.

**Acceptance Criteria:**
- `grep -rn "MarkerBurner" --include=*.rs crates apps`, excluding `burner.rs` and `tests/`, shows a
  call site in `bundle.rs`. **The story is not done without this line, whatever the tests say.**
- Every one of the four tests asserts on the **file** at `BundleItem.image_path` — its existence, its
  decode, its pixels — and no test asserts the shape of the path string.
- `a_bundle_export_does_not_re_reduce_the_stored_image` uses a fixture whose long edge exceeds the
  default budget's `max_long_edge`.
- The corrupt-source fixture is a valid PNG header with a corrupt payload, and the test proves it
  reaches the decoder.
- No `ImageReducer` call exists anywhere in `bundle.rs`.
- No `let _ =` on a `Result` in the composition or its rollback.
- `test_bundle_image_copy.rs`, both goldens, and `test_bundle_failures.rs` are untouched **and still
  green** — in particular `composition_that_cannot_open_the_vault_is_refused_not_silently_skipped`
  and `a_bundle_whose_source_finding_is_gone_still_copies_the_same_bytes`.
- All eight mutations observed red and restored, under `--no-fail-fast`, each run recorded.
- Every command in `AGENTS.md` § Code passes — never a piped exit code, never `echo "EXIT=$?"` trusted
  over the echoed value, `npm ci` before believing a local web-side red. The frontend is untouched, so
  a web red is a stale worktree until proven otherwise.
- The four reported-not-fixed items are in the Spec Change Log verbatim, as findings for the
  coordinator.

## Spec Change Log

### 2026-08-24 — Initial Story Specification (Step 1: Plan Only)

- **`BUG-19` confirmed against the code, not against the register.** `AGENTS.md` records that a defect
  row goes stale silently and that `W7` planned a wave against an already-fixed row. The grep was
  re-run: two hits, both re-exports. The defect is live.
- **`OQ-26` resolved in favour of PNG**, on evidence from the build rather than the corpus: the
  workspace compiles `image` with `features = ["png"]` and no WebP, so `.webp` is not a format this
  binary can produce; `capture.rs`, `pipeline.rs` and `burner.rs` already agree on PNG; and both
  serving surfaces (`handlers.rs:199`, `store.go:161`) already map or default to `image/png`. The
  `.webp` literal at `bundle.rs:41` is the only disagreeing site and has never had a file behind it.
  `ResolvedPair.encoder_quality` is recorded as the one datum pointing the other way — a quality knob
  that `PngEncoder` ignores, which is a live `NFR-3` observation and `W8-S2`'s territory, not this
  story's. **This needs a `DEC-` through `wdi-decision`;** the corpus stays silent on purpose.
- **`BUG-20` placed in the burner, with the reasoning and the cost written down** — see the
  `intent-contract`. The premise the fast path relied on was never written; this story writes the
  check instead of writing a second premise for the next caller.
- **Reported: the composed Markdown references the Finding's image, not the Bundle's copy.**
  `markdown.rs:30` formats `item.finding.image_path`. `FR-8` is only half-restored by this story, and
  a handed-over Bundle still points at a file `FR-13` permits to be deleted. Out of scope — it moves
  two goldens `W8-S5` pinned yesterday. **Candidate defect.**
- **Reported: `bundle.rs:35` silently skips an unresolvable `finding_id`**, against `UC-9`'s
  all-or-nothing failure flow. This story adds the `BR-13` refusal for a missing image **file** and
  leaves the missing **row** silent, an inconsistency inside one loop, named for adjudication rather
  than fixed.
- **Reported: `sharing.rs:116` swallows a `read_blob` failure when publishing.** Invisible today
  because nothing exists to read; live from the moment this story lands.
- **Reported: a fourth test is required and `waves.yaml` names three.** The three named tests all pass
  with `BUG-20` present. Adding the row is the coordinator's.
- **Recorded: `burner.rs:224` `invalid_input_bytes_returns_validation_error` is not the `BUG-20`
  guard.** It passes a marker with a real comment, so it takes the slow path and exercises the decode
  that already existed. `W7`'s lesson applies to its garbage-bytes fixture too.

## Design Notes

**Why no byte is written until every burn has succeeded.** `UC-9` demands a rollback and the rollback
must exist, but the common failure — one undecodable or absent Finding image — is far more likely than
a disk filling up mid-write. Burning everything into memory first turns that case into a refusal with
nothing to undo. Bundles are counted in tens (`data-model.md` § Indexes), so holding a handful of
reduced images in memory is not the constraint. The rollback then covers only the genuinely partial
cases, which is the smallest surface where a mistake can hide.

**Why the tests live in `apps/desktop/src-tauri/tests/` and not in `snapdown-store`.** The defect is a
missing call in the desktop crate. `snapdown-store` cannot see `create_bundle_impl`, and
`test_bundle_image_copy.rs` sits at the `MarkerBurner` seam precisely because `W8-S5` could not reach
the file level. Two stories, two failure modes: `W8-S5` proves the copy semantics, this story proves
somebody invokes them. Neither hides behind the other.

**Why a new file rather than four more tests in `test_bundle_failures.rs`.** That file is about
composition and deletion **failing**; three of these four are about it succeeding. Its
`build_test_app` is copied because integration test binaries do not share modules — the duplication is
the cost of that boundary and it is smaller than the alternative of a shared test-support crate for
one helper.

**Why the deletion path needs no change.** `bundle.rs:169-178` already checks `blob_exists` before
deleting each item image. `BUG-19`'s `impact` notes that this guard is what concealed the miss. It does
not become wrong when the files start existing — it becomes load-bearing, and
`deleting_a_bundle_reports_an_image_copy_it_could_not_remove` already covers the refusal.

## Verification

**Commands** — every one from `AGENTS.md` § Code, from the repo root:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix web/ui run typecheck && npm --prefix web/ui run lint && npm --prefix web/ui run test
npm --prefix apps/desktop run typecheck && npm --prefix apps/desktop run lint
npm --prefix apps/desktop run test && npm --prefix apps/desktop run build
```

**Targeted:**

```bash
cargo test --workspace --no-fail-fast
cargo test -p snapdown --test test_bundle_export
cargo test -p snapdown --test test_bundle_failures
cargo test -p snapdown-store --test test_bundle_image_copy
cargo test -p snapdown-store --test test_golden_markdown
cargo test -p snapdown-store image::burner
grep -rn "MarkerBurner" --include=*.rs crates apps
```

**How to read these runs** — `AGENTS.md` § Code records four ways a verification run lies. Three apply:
never pipe a cargo command into `tail` and read the exit code (`${PIPESTATUS[0]}`, or redirect and read
`$?`); never append `echo "EXIT=$?"` and trust the harness's reported code over the echoed value; run
`npm ci` before believing a local web-side red. The fourth applies if a `tauri build` is attempted:
`Get-Process -Name Snapdown` first, because a leftover process locks its own binary and the failure
reads as a permissions problem.

**Mutation:** each of the eight rows, applied one at a time under `cargo test --workspace
--no-fail-fast`, red observed, then restored and the suite confirmed green. Row 4 — restoring the early
return above the decode — is the acceptance criterion for `BUG-20` specifically.

### 2026-08-24 — Step 2: Build Complete

- **`BUG-19` resolved**: `create_bundle_impl` in `apps/desktop/src-tauri/src/commands/bundle.rs` calls `MarkerBurner::burn_markers` for each finding and transactionally writes `bundles/{bundle_id}/finding_{pos}_burned.png` to the vault.
- **`BUG-20` resolved**: `MarkerBurner::burn_markers` in `crates/snapdown-store/src/image/burner.rs` hoists `image::load_from_memory` and dimension validation above the `active_markers.is_empty()` check. Corrupt source images are rejected on all paths.
- **`BUG-21` resolved**: `MarkdownSerializer::serialize_bundle` in `crates/snapdown-core/src/domain/markdown.rs` updated to accept `&[(&BundleItem, &FindingDetail)]` and reference `BundleItem.image_path` (the bundle's burned copy) rather than `Finding.image_path`. Goldens and unit tests updated and verified.
- **`test_bundle_export.rs` implemented**: All 5 tests from `waves.yaml` (`a_bundle_item_image_path_holds_a_file_that_decodes`, `an_exported_bundle_image_carries_the_markers_of_its_finding`, `a_bundle_export_does_not_re_reduce_the_stored_image`, `a_corrupt_source_is_refused_even_when_no_marker_is_drawn`, `the_composed_markdown_references_the_bundles_burned_copy`) passing.
- **Reach grep verified**:
  `apps/desktop/src-tauri/src/commands/bundle.rs:7:use snapdown_store::image::MarkerBurner;`
  `apps/desktop/src-tauri/src/commands/bundle.rs:110:let burned_bytes = MarkerBurner::burn_markers(...)`
- **Mutations verified under `--no-fail-fast`**: All mutations observed RED and restored to GREEN.
- **Full verification suite passed**: `cargo fmt`, `cargo clippy`, `cargo test --workspace`, `npm run typecheck`, `npm run lint`, `npm run test`, `npm run build`, `go test`.
