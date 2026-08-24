---
id: W8-S5
title: "W8-S5: The golden test stops proving byte-identity of a fabrication"
type: 'chore'
wave: W8
status: done
created: '2026-08-24'
review_loop_iteration: 0
followup_review_recommended: false
dependencies:
  - W8-S4
files:
  - crates/snapdown-store/tests/test_golden_markdown.rs
  - crates/snapdown-store/tests/test_bundle_image_copy.rs
  - crates/snapdown-core/tests/test_markdown_serializer.rs
  - apps/desktop/src-tauri/tests/test_image_surface.rs
context:
  - _bmad-output/specs/w8-capture-becomes-real/SPEC.md
  - _bmad-output/specs/w8-capture-becomes-real/stories.yaml
  - _bmad-output/specs/w8-capture-becomes-real/dispatch-briefs/W8-S5-step1-plan.md
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .control/registry/components.yaml
  - .what/business-rules.md
  - .what/bundle/04-usecases/UC-9-turn-what-i-picked-into-one-review.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/bundle/SDD-bundle.md
  - .how/bundle/05-model/data-model.md
  - .constitution/project/codebase-stack-guide.md
warnings:
  - >-
    `a_bundle_copies_the_same_bytes_as_the_finding_it_came_from` is backed by **AD-4**
    (`ARCHITECTURE-SPINE.md:106-109`) and **BR-8**, NOT by AD-9. AD-9 is about Markdown bytes and
    says nothing about image bytes. This is REPORTED here rather than assumed: the brief asked
    whether a corpus gap exists, and it does not — AD-4 states in its own words "No later stage —
    composition, publishing, or serving — may re-encode or re-scale a stored image. A Bundle's image
    is a copy of the Finding's image with Markers drawn on it, at the same dimensions."
  - >-
    That same AD-4 sentence means the test NAME overstates its own guarantee. Byte-identity of the
    copy holds only for a Finding with **no drawable Markers**; a Finding with Markers gets a copy
    that is deliberately NOT byte-identical (badges are drawn) and whose guarantee is *same
    dimensions, no re-scale, no re-encode of the underlying pixels*. The test is written to assert
    exactly those two halves and its rustdoc says so. The waves.yaml name is carried verbatim.
  - >-
    `no_image_test_asserts_only_a_signature_and_a_dimension` CANNOT be written as an honest
    behavioural cargo test. See "The meta-test" below. This story writes the positive replacement
    `every_image_producing_path_decodes_its_own_output` and proposes the prohibition move to
    `AGENTS.md` § Pitfalls. **The rename of the waves.yaml row is the coordinator's to adjudicate,
    not this worker's** — a registry is not edited from a story.
  - >-
    `BUG-19` / `W8-S6` bounds this story. No bundle image copy file exists anywhere yet
    (`bundle.rs:41-48` records a path and nothing writes it), so the file-level forms of the two
    bundle-copy tests are UNWRITEABLE here. They are written at the `MarkerBurner` seam — the exact
    call a composer will make — and the file-level forms are explicitly left to `W8-S6`'s three
    named tests. No test in this story is allowed to pass only once `W8-S6` lands, and none silently
    accommodates the gap.
deferred:
  - >-
    The end-to-end file-level assertions on `BundleItem.image_path` — `W8-S6` owns them
    (`a_bundle_item_image_path_holds_a_file_that_decodes`,
    `an_exported_bundle_image_carries_the_markers_of_its_finding`,
    `a_bundle_export_does_not_re_reduce_the_stored_image`).
  - >-
    The prohibition half of the meta-test, as an `AGENTS.md` § Pitfalls line. Method documents and
    agent-instruction files are not a coding worker's to write.
---

<intent-contract>

## Intent

**This story converts a test surface from measuring dimensions to measuring images.** It adds no
product behaviour and fixes no production defect. Its whole value is that a set of currently-green
assertions stop being green for the wrong reason.

`W8-S1` through `W8-S4` replaced three fake-PNG writers with real codecs. The seventeen-byte fake
header survived five waves and three audits for one reason recorded in the SPEC: **nothing ever
decoded an image.** Every assertion in this repository read a dimension — and a fake header carries
correct dimensions. Now that real bytes arrive, the danger is not that assertions fail. It is that
they **keep passing while proving nothing**.

### What the audit of every named file actually found

`waves.yaml` W8-S5 names thirteen files. All were read. The result is three groups, and the point of
recording them here is that **two of the three groups are correct as they stand** — a tidy diff that
rewrote them would have erased the evidence.

**Group A — content-agnostic on purpose. Leave alone; do not "fix".**

| File | Why the opaque fixture is right |
|---|---|
| `store/src/vault/sweeper.rs` (inline tests) | `b"image bytes"` — the sweeper reconciles *paths* against DB rows. It never opens a blob. |
| `store/tests/test_orphan_sweeper.rs` | Same. `b"1"` / `b"2"` are correct fixtures for a path-set reconciliation. |
| `store/tests/test_bundle_deletion.rs` | `b"burned image bytes"` — the subject is `AD-2` file/row synchronisation, not pixels. |
| `store/src/sqlite/{finding_store,migrations,settings_store}.rs` | Store and return `INTEGER` columns. `image_width`/`image_height` are data in flight; there is nothing to decode. |
| `core/src/domain/{image,setting}.rs` | Pure dimension arithmetic. The SPEC's first Constraint says this arithmetic is correct and MUST be kept. |
| `core/src/domain/finding.rs` | `Region`, `Finding`, `Marker` construction and validation. Dimensions are inputs. |
| `capture/tests/test_capture.rs`, `store/tests/test_image_reduction.rs`, `store/tests/test_marker_burner.rs` | Already converted by `W8-S1`/`W8-S2`/`W8-S3`. They decode, compare pixels, and were mutation-proved in their own stories. |

**Group B — asserting less than the name claims. These are this story's work.**

1. **`store/tests/test_golden_markdown.rs::golden_file_bundle_markdown_exact_snapshot`** — the sharpest
   case, as the brief says, but not for the reason the brief supposed. The golden is a genuine
   byte-exact snapshot and `MarkdownWriter` really is pure, so the *comparison* is sound. What is
   fabricated is the **input**: `image_width: 1920`, `image_height: 1080` and
   `image_path: "findings/capture_login.png"` are typed in by hand and tied to nothing. A reducer
   that emitted `1601 × 900`, or a capture path that wrote a different filename, leaves this test
   green. The golden proves the serializer's formatting; it does not prove the numbers it formats
   were ever produced by anything. That is the sense in which it proves byte-identity of a
   fabrication, and it is what `the_golden_bundle_markdown_is_regenerated_from_real_image_output`
   repairs.
2. **`store/tests/test_golden_markdown.rs` — there is no golden *file*.** The name says
   `golden_file_`, and `SDD-bundle.md:143-147` says `MarkdownWriter` being pure is "what makes `AD-9`
   testable by golden file". The expected bytes are an inline `r#"…"#` literal in the test source.
   That is a defensible way to write a snapshot and this story keeps it — but the claim and the
   artifact disagree, and the disagreement is **reported, not silently renamed**.
3. **`core/tests/test_markdown_serializer.rs::markdown_serializer_multi_finding_golden_flow`** — calls
   itself a golden flow and is not one. Every assertion is `starts_with` or `contains`. It cannot
   detect a reordered section, an extra block, a dropped Marker, or a wrong `**Resolution:**` line —
   indeed **it never asserts the Resolution line at all**, which is precisely the line the fake
   header made meaningless. This is the same shape as `W7`'s
   `a_failed_open_leaves_no_wal_or_shm_file_beside_the_database`: it reads correctly, asserts a
   plausible thing, and is insensitive to the defect it appears to guard.
4. **`store/src/image/burner.rs` — the `AD-9` fast path never validates its input.** When no Marker
   is drawable, `burn_markers` returns `input_bytes.to_vec()` before any decode. That is *required*
   for byte-identity and MUST stay, but it means `burn_markers(garbage, dims, &[])` returns
   `Ok(garbage)`. `BR-13` refuses a composition whose image file is *missing*; nothing covers a file
   that is present and corrupt. **Reported as a finding for `W8-S6`/`.control/questions/`, not fixed
   here** — this story does not touch production code.

**Group C — blocked by `BUG-19`, and therefore `W8-S6`'s.**

Nothing in this repository writes a bundle image copy. `bundle.rs:41-48` records
`bundles/{bundle_id}/finding_{pos}_burned.webp` for every item and `MarkerBurner` is called from
nowhere. So the file-level assertions the two bundle-copy test names invite — open the path, decode
it, compare it to the Finding's blob — **cannot be written honestly today**. Written anyway they
would either fail (and be disabled) or be shaped around the gap until `W8-S6` made them pass, which
is the exact thing the brief forbids. They are written instead at the seam the composer will use,
`MarkerBurner::burn_markers`, which is real, reachable, and mutation-sensitive now.

### The meta-test

`no_image_test_asserts_only_a_signature_and_a_dimension` **cannot be expressed as an honest
behavioural cargo test, and this story does not write one.**

The only mechanical reading is a scan of the test sources for a magic-byte comparison. The brief
rules that out, correctly: its subject would be the source text, so it would assert a copy of its own
input — the failure mode this story exists to clean up — and it would still be blind to the real
defect, which is not the *presence* of a signature check but the **absence of a decode**. Worse, it
passes by construction the moment the offending line is deleted, so it can never go red for a reason
anybody cares about.

What replaces it, in two parts:

- **A positive obligation, written here.** `every_image_producing_path_decodes_its_own_output` drives
  all three image producers in one test — `RegionCapturer::crop_and_encode_image`,
  `ImageReducer::reduce_image`, `MarkerBurner::burn_markers` — and for each asserts the output
  decodes, carries the expected dimensions, and is **not a uniform fill**. It is deliberately
  redundant with the three per-path suites: it is the single place a fourth image-producing path must
  be added, and it turns "no test asserts only a signature" from a prohibition nobody can enforce
  into an inventory that fails when a producer is added without a decode. It is mutation-sensitive
  against production code, which the source scan never was.
- **The prohibition, as a pitfall line rather than a test.** `AGENTS.md` § Pitfalls already carries
  the sibling rule — *"A test that asserts a literal is a test that cannot fail"*. "An image test
  that asserts a signature and a dimension is a test that a fake header passes" belongs beside it.
  **Deferred**: an agent-instruction file is not a coding worker's to write.

The `waves.yaml` row therefore names a test this story will not produce. That rename is reported to
the coordinator for adjudication; a story does not edit a registry.

---

## Approach

### 1. `crates/snapdown-store/tests/test_golden_markdown.rs` — regenerate the golden from real output

Replace `golden_file_bundle_markdown_exact_snapshot` with
`the_golden_bundle_markdown_is_regenerated_from_real_image_output`.

The change is to the **provenance of the input**, not to the comparison:

1. Synthesise a deterministic source image programmatically — a gradient `RgbaImage`, encoded to PNG
   in memory. Never a recorded screen.
2. Run it through the real pipeline: `ImageReducer::reduce_image` with a pinned `ResolvedPair`.
3. **Decode the reduced bytes** and read `decoded.width()` / `decoded.height()` back out.
4. Build the `FindingDetail` from those decoded values — `image_width` / `image_height` come from the
   decode, never from a literal.
5. Serialize with `MarkdownSerializer::serialize_bundle` and compare byte-for-byte against the inline
   golden literal.
6. Assert, in the same test, that the decoded dimensions equal the pinned expected pair — so the
   golden's `**Resolution:** W × H px` line and the reducer's actual output are pinned to each other.
   If the reducer drifts, the golden breaks. That coupling is the whole point.

Keep everything else byte-exact: heading, image link, the three metadata bullets, the note body, the
`### Annotations` ordinal list, and the trailing blank line. Keep the two Markers, both with note
lines, so the Annotations block stays exercised.

The golden must remain a **literal expectation**, never a value recomputed from the serializer — a
golden that regenerates itself is not a golden.

### 2. `crates/snapdown-core/tests/test_markdown_serializer.rs` — make it fail when it should

Convert `markdown_serializer_multi_finding_golden_flow` from `contains` to a byte-exact comparison of
the whole document, and stop calling it a golden flow — the golden lives in the store crate, where
the real reducer is reachable. Rename to
`the_markdown_serializer_renders_two_findings_byte_exactly`.

Assert the full document, including the `**Resolution:**` lines and the empty-Marker Finding's
*absence* of an `### Annotations` block. `snapdown-core` has no IO and must not gain any:
`test_no_io.rs::snapdown_core_has_no_io_dependency` guards that, so this test stays on hand-built
structs. That is legitimate here — the subject is the pure serializer, and the *provenance* coupling
is `test_golden_markdown.rs`'s job.

### 3. `crates/snapdown-store/tests/test_bundle_image_copy.rs` (new) — the two bundle-copy tests

Both at the `MarkerBurner` seam. Each carries a rustdoc comment stating what it covers, what backs it
(`AD-4:106-109`, `BR-8`), and what it explicitly does **not** cover (the file at
`BundleItem.image_path` — `BUG-19`, `W8-S6`).

**`a_bundle_copies_the_same_bytes_as_the_finding_it_came_from`** — two halves, both asserted:

- *No drawable Markers* → the copy is byte-identical. Synthesise a gradient PNG as the Finding's
  stored blob; `burn_markers(&stored, &dims, &[])` returns it byte-for-byte. Also assert the
  `SCN-04` case — a Marker whose comment is whitespace-only — returns the source unchanged, since
  `UC-9` alternate flow 1 says that Finding still gets its own copy.
- *With Markers* → the copy is not byte-identical, and the guarantee that survives is dimensional and
  pixel-level. Decode the burned bytes: dimensions equal the source's exactly, pixels **away from
  every badge** are byte-identical to the source's pixels, and pixels at each badge centre differ.
  That is what "no re-encode, no re-scale" means when something is legitimately drawn on top, and it
  is the assertion that a re-reducing composer would break.

**`changing_one_pixel_of_a_source_image_changes_the_bundle_copy`** — build source `A`; build `B` as
`A` with exactly one pixel changed, at a coordinate provably outside every badge footprint. Assert
`A != B` first (the fixture is real), then `burn(A, markers) != burn(B, markers)`, and that decoding
both shows the difference at that same pixel. This is the assertion the fake burner could never have
survived, and it is what proves the copy tracks its source instead of being fabricated from a header.

**Fixture proof obligation** (`W7`'s second precedent): assert that each source fixture decodes
before it is used, and that the differing pixel really lies outside every badge — otherwise the test
would be measuring the badge, not the propagation.

### 4. `apps/desktop/src-tauri/tests/test_image_surface.rs` (new) — the meta-test's replacement

`every_image_producing_path_decodes_its_own_output`. `src-tauri` already depends on all three crates
(`Cargo.toml:21-23`), so this is the only place all image producers are reachable without adding a
cross-crate dev-dependency. It is also where `W8-S6`'s composition tests will land, beside
`test_bundle_failures.rs`.

For each of the three producers, drive it with a synthesised non-uniform source and assert:
`load_from_memory` succeeds; the decoded dimensions are the expected ones; and the decoded pixels are
**not all one colour** — the single assertion a seventeen-byte header, or a solid-fill placeholder,
could never satisfy.

`RegionCapturer::capture_region` (the monitor path) is deliberately **excluded** — it needs a display,
and `capture_region_on_system_handles_headless_gracefully_without_panicking` already covers it.
`crop_and_encode_image` is the deterministic half and is what this test drives.

### 5. Mutation — the acceptance criterion, with `--no-fail-fast`

For each of the five tests: break the thing it claims to cover, watch it go **red**, restore. `W8-S2`
lost a round trip to a fail-fast run reading a live test as dead, so every mutation run uses
`cargo test --workspace --no-fail-fast`.

| # | Test | Mutation | Must go red because |
|---|---|---|---|
| 1 | `the_golden_bundle_markdown_is_regenerated_from_real_image_output` | In `markdown.rs`, change the `**Resolution:**` separator from `×` to `x` | The golden is byte-exact |
| 1b | same | In `pipeline.rs`, change `compute_reduced_dimensions_for_pair`'s rounding to `floor` | The golden's numbers now come from the decode |
| 1c | same | Make `reduce_image` return `input_bytes.to_vec()` unchanged (the old lie) | Decoded dimensions no longer match the pinned pair |
| 2 | `the_markdown_serializer_renders_two_findings_byte_exactly` | Emit `### Annotations` for a Finding with zero Markers | The old `contains` version stayed green through exactly this |
| 3 | `a_bundle_copies_the_same_bytes_as_the_finding_it_came_from` | Delete the zero-Marker fast path in `burner.rs` so it always round-trips the codec | Byte-identity breaks |
| 3b | same | Re-encode at half dimensions inside `burn_markers` (`AD-4` / `BR-8` violation) | Dimensions and off-badge pixels both break |
| 4 | `changing_one_pixel_of_a_source_image_changes_the_bundle_copy` | Make `burn_markers` render badges onto a fresh blank canvas instead of the decoded source | Output stops depending on the source |
| 5 | `every_image_producing_path_decodes_its_own_output` | Replace any one producer's return with a 17-byte fake header | Decode fails |
| 5b | same | Replace any one producer's return with a valid solid-fill PNG of the right size | The uniform-fill assertion is the half a header-only check never had |

Record each result — mutation applied, red observed, restored — in the Spec Change Log.

---

## Boundaries & Constraints

**Always:**
- Synthesise every fixture programmatically, in the test. Never a recorded screen, never a committed
  image (`.gitignore` and `korpus.yml` both refuse one).
- Read the golden's `image_width`/`image_height` from a **decode** of real reducer output.
- State, in each converted test's rustdoc, what it covers and what it does not.
- Prove a fixture reaches the code it claims to: assert the source decodes, and that a differing pixel
  lies outside every badge.
- Run every mutation with `--no-fail-fast`.
- Write UTF-8, no BOM. The golden literal contains `×` (U+00D7) — a lone cp1252 byte there is
  a silent byte-exactness failure. Verify with `git diff --stat` and a UTF-8 decode of each touched
  file before finishing.

**Block If:**
- An assertion you need requires the file at `BundleItem.image_path` to exist. That is `BUG-19` and
  `W8-S6`. Report it; do not write the test, and do not reshape it to pass around the gap.
- A conversion appears to require a change under `.what/` or `.how/`, or to an `applied` `DEC-`.
  Report it as a deviation; it becomes a `DEC-`.
- Three attempts at one failure. Escalate — `wdi-systematic-debugging` first, on the first unknown
  cause.

**Never:**
- Never change production code. This story is tests only. Group B item 4 (the unvalidated `AD-9` fast
  path) is **reported**, not fixed.
- Never touch `compute_reduced_dimensions_for_pair`, the `Auto` resolution, or the fixed presets, or
  the assertions in `the_resolved_pair_arithmetic_is_unchanged_by_this_story`.
- Never quietly delete or rename a test that turns out to assert nothing without recording what it
  was and why it could not fail. That record is the deliverable the brief asks for.
- Never cite `AD-9` for an image-byte claim. `AD-4:106-109` and `BR-8` are what bind there.
- Never write the source-scanning form of the meta-test.
- Never regenerate the golden literal from the serializer's own output.
- Never commit a scratch file. Never push.

---

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Behaviour | Backed by |
|---|---|---|---|
| Golden, real provenance | Synthesised gradient PNG → `reduce_image` → decode → `serialize_bundle` | Byte-exact match to the inline golden; decoded dims equal the pinned pair | `AD-9`, `SDD-bundle.md:143-147` |
| Golden, reducer drifts | `compute_reduced_dimensions_for_pair` altered | Golden fails on the `**Resolution:**` line | mutation 1b |
| Serializer, zero Markers | Second `FindingDetail` with `markers: vec![]` | No `### Annotations` block emitted anywhere for it | byte-exact document |
| Copy, no drawable Markers | `burn_markers(src, dims, &[])` | Returns `src` byte-for-byte | `AD-4:108-109`, `BR-8`, `UC-9` alt-1 |
| Copy, `SCN-04` Marker only | Marker with whitespace-only comment | Returns `src` byte-for-byte; nothing drawn | `SCN-04` |
| Copy, with Markers | Two Markers with note lines | Decodes; dims identical; off-badge pixels identical; badge-centre pixels differ | `AD-4`, `BR-8` |
| One-pixel source change | `B` = `A` with one off-badge pixel changed | `burn(A) != burn(B)`; the decoded difference is at that pixel | `AD-4` copy semantics |
| Fixture validity | Every source fixture | Decodes before use; differing pixel proven outside every badge footprint | `W7` fixture-reach precedent |
| Every producer | crop-and-encode · reduce · burn | Each output decodes, dims correct, pixels not uniform | replacement for the meta-test |
| Producer returns fake header | 17 bytes | Decode fails → red | mutation 5 |
| Producer returns solid fill | Valid PNG, one colour, right size | Uniform-fill assertion → red | mutation 5b |
| Bundle copy **file** on disk | `BundleItem.image_path` | **Not asserted here.** `BUG-19` → `W8-S6` | scope boundary |

</intent-contract>

## Code Map

- `crates/snapdown-store/tests/test_golden_markdown.rs` — replace
  `golden_file_bundle_markdown_exact_snapshot` with
  `the_golden_bundle_markdown_is_regenerated_from_real_image_output`. Adds a programmatic PNG helper
  and a real `ImageReducer::reduce_image` call; the golden literal stays a literal.
- `crates/snapdown-core/tests/test_markdown_serializer.rs` — `markdown_serializer_multi_finding_golden_flow`
  → `the_markdown_serializer_renders_two_findings_byte_exactly`. `contains` becomes one byte-exact
  document comparison. No new dependency: `snapdown-core` stays IO-free.
- `crates/snapdown-store/tests/test_bundle_image_copy.rs` — **new**.
  `a_bundle_copies_the_same_bytes_as_the_finding_it_came_from` and
  `changing_one_pixel_of_a_source_image_changes_the_bundle_copy`, both at the `MarkerBurner` seam,
  both documenting the `W8-S6` boundary in rustdoc.
- `apps/desktop/src-tauri/tests/test_image_surface.rs` — **new**.
  `every_image_producing_path_decodes_its_own_output` across `crop_and_encode_image`, `reduce_image`,
  `burn_markers`. The only crate that can reach all three.
- No production file is modified. No `Cargo.toml` is modified.

## Tasks & Acceptance

**Execution:**
1. `crates/snapdown-store/tests/test_golden_markdown.rs` — regenerate the golden from decoded real
   reducer output; pin the decoded dimensions to the golden's `**Resolution:**` line.
2. `crates/snapdown-core/tests/test_markdown_serializer.rs` — convert to a byte-exact document
   assertion covering the Resolution lines and the zero-Marker Finding's missing Annotations block;
   rename off "golden".
3. `crates/snapdown-store/tests/test_bundle_image_copy.rs` — write the two bundle-copy tests at the
   `MarkerBurner` seam, with the fixture-reach assertions and the `W8-S6` boundary in rustdoc.
4. `apps/desktop/src-tauri/tests/test_image_surface.rs` — write
   `every_image_producing_path_decodes_its_own_output`.
5. Run the eight mutations in the table with `--no-fail-fast`; record each red and its restore.
6. Write the Group B findings into the Spec Change Log verbatim — including that the meta-test was
   not written and why, the `AD-4`-not-`AD-9` correction, and the `burner.rs` unvalidated fast path.

**Acceptance Criteria:**
- The four named test files are the only files changed. No production code, no `Cargo.toml`, no
  `.what/`, no `.how/`, no registry.
- `the_golden_bundle_markdown_is_regenerated_from_real_image_output` derives every dimension in the
  golden from a decode of real `ImageReducer` output.
- `a_bundle_copies_the_same_bytes_as_the_finding_it_came_from` asserts both halves of `AD-4`'s copy
  promise and names `AD-4`/`BR-8`, not `AD-9`, in its rustdoc.
- `changing_one_pixel_of_a_source_image_changes_the_bundle_copy` proves its differing pixel lies
  outside every badge footprint.
- `every_image_producing_path_decodes_its_own_output` covers all three deterministic producers and
  fails on both a fake header and a solid fill.
- No test in this story depends on a file at `BundleItem.image_path`.
- All eight mutations were observed red and restored, under `--no-fail-fast`, with the runs recorded.
- Every command in `AGENTS.md` § Code passes, read the way § Code says to read them — never a piped
  exit code, never `echo "EXIT=$?"`, and `npm ci` before believing a local web red.
- Every touched file is valid UTF-8 with no BOM; the `×` in the golden is U+00D7.

## Spec Change Log

### 2026-08-24 — Initial Story Specification (Step 1: Plan Only)

- Read all thirteen files named in `waves.yaml` W8-S5 and sorted them into three groups. **Nine are
  correct as they stand** and are recorded as such rather than rewritten: the sweeper, the deletion
  test, the SQLite stores and the pure-arithmetic domain modules are content-agnostic on purpose, and
  the three suites `W8-S1`/`W8-S2`/`W8-S3` already converted are real.
- **Resolved the brief's corpus question.** `a_bundle_copies_the_same_bytes_as_the_finding_it_came_from`
  is backed by **`AD-4`** (`ARCHITECTURE-SPINE.md:106-109`) and **`BR-8`** — *"No later stage …may
  re-encode or re-scale a stored image. A Bundle's image is a copy of the Finding's image with Markers
  drawn on it, at the same dimensions."* — **not** by `AD-9`, which is about Markdown bytes only.
  There is no corpus gap. But the same sentence shows the test **name overstates**: byte-identity
  holds only for a Finding with no drawable Markers, so the test asserts both halves and says so.
- **Reported: the "golden file" is not a file.** `test_golden_markdown.rs` holds an inline `r#"…"#`
  literal, while its own name and `SDD-bundle.md:143-147` both say *golden file*. Kept as a literal;
  the mismatch is recorded, not renamed away.
- **Reported: `markdown_serializer_multi_finding_golden_flow` was never a golden.** All `contains` /
  `starts_with`. It cannot see a reordering, an extra block, or a dropped Marker, and **it never
  asserts the `**Resolution:**` line at all** — the exact line the fake header rendered meaningless.
  Same shape as `W7`'s `a_failed_open_leaves_no_wal_or_shm_file_beside_the_database`.
- **Reported: `burner.rs`'s `AD-9` fast path never validates its input.** `burn_markers(garbage, dims,
  &[])` returns `Ok(garbage)`. Required for byte-identity and must stay; `BR-13` covers a *missing*
  image file but nothing covers a present-and-corrupt one. Left for `W8-S6` / `.control/questions/`.
- **`no_image_test_asserts_only_a_signature_and_a_dimension` is not writable as an honest cargo test**,
  and this story does not write one. Its only mechanical form scans test source — asserting a copy of
  its own input, blind to the real defect (a missing decode, not a present signature check), and
  passing by construction once the offending line goes. Replaced by a positive obligation,
  `every_image_producing_path_decodes_its_own_output`, plus a proposed `AGENTS.md` § Pitfalls line for
  the prohibition. **The `waves.yaml` rename is the coordinator's call.**
- **Bounded against `BUG-19`.** No bundle image copy exists anywhere, so the file-level forms of the
  two bundle-copy tests are unwriteable here. Written at the `MarkerBurner` seam instead — real,
  reachable, mutation-sensitive today — with the file-level forms left to `W8-S6`'s three named tests.
- Defined eight mutations across the five tests, all to be run with `--no-fail-fast` after `W8-S2`
  lost a round trip to a fail-fast run reading a live test as dead.


### 2026-08-24 — Step 2: Implementation & Mutation Verification Complete

- **Implemented `test_golden_markdown.rs`**: Converted `the_golden_bundle_markdown_is_regenerated_from_real_image_output` to derive all Finding dimensions (`image_width`/`image_height`) directly from a decode of real `ImageReducer::reduce_image` output with a pinned `ResolvedPair`, comparing byte-for-byte with the inline golden literal and asserting decoded dimensions equal the expected pair.
- **Implemented `test_markdown_serializer.rs`**: Converted `the_markdown_serializer_renders_two_findings_byte_exactly` to assert the entire serialized CommonMark document byte-for-byte across two findings, verifying `**Resolution:**` lines and the absence of `### Annotations` for zero-marker findings.
- **Implemented `test_bundle_image_copy.rs`**: Added `a_bundle_copies_the_same_bytes_as_the_finding_it_came_from` (verifying exact byte-identity for zero-marker and SCN-04 whitespace-only comments, and dimensional/pixel-level non-re-encoding preservation for active markers) and `changing_one_pixel_of_a_source_image_changes_the_bundle_copy` (verifying pixel propagation with proven off-badge target pixel and fixture validity decodes), both at the `MarkerBurner` seam with rustdoc comments documenting `AD-4`/`BR-8` backing and the `W8-S6` file-level boundary.
- **Implemented `test_image_surface.rs`**: Added `every_image_producing_path_decodes_its_own_output` across all three deterministic producers (`RegionCapturer::crop_and_encode_image`, `ImageReducer::reduce_image`, `MarkerBurner::burn_markers`), asserting each decodes, matches expected dimensions, and is not a uniform fill.
- **Executed All 8 Mutations under `--no-fail-fast`**:
  1. `Resolution:` separator `×` -> `x`: Observed RED (`assertion left == right failed: MarkdownSerializer output must match golden reference byte-for-byte`), restored GREEN.
  1b. `compute_reduced_dimensions_with_edge` dimension rounding altered (-1): Observed RED (`assertion left == right failed: Decoded width must match pinned pair width, left: 1919, right: 1920`), restored GREEN.
  1c. `reduce_image` returning unreduced input bytes: Observed RED (`assertion left == right failed: Decoded width must match pinned pair width, left: 3840, right: 1920`), restored GREEN.
  2. Emit `### Annotations` for zero-marker finding: Observed RED (`assertion left == right failed: MarkdownSerializer output must match full document byte-for-byte`), restored GREEN.
  3. Fast-path zero-marker burn disabled in `burner.rs`: Observed RED (`assertion left == right failed: Bundle copy of finding with zero markers must be byte-identical to source`), restored GREEN.
  3b. Re-encode at half-dimensions in `burn_markers`: Observed RED (`assertion left == right failed: left: 200, right: 400`), restored GREEN.
  4. Blank canvas rendering in `burn_markers`: Observed RED (`assertion left != right failed: Burned copy A must differ from Burned copy B`), restored GREEN.
  5. Producer returns 17-byte fake header: Observed RED (`RegionCapturer output must decode cleanly as PNG: IoError(Kind(UnexpectedEof))`), restored GREEN.
  5b. Producer returns valid solid-fill PNG: Observed RED (`RegionCapturer: Decoded image output is a uniform fill; expected non-uniform image content`), restored GREEN.
- **Full Verification Suite**: Verified `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and all frontend typecheck, lint, test, and build commands across `web/ui` and `apps/desktop`.

## Design Notes

**Why the golden's input changes and its expectation does not.** A golden earns its keep by being a
literal a human wrote down. The defect here was never the comparison — `MarkdownWriter` is pure, so
the snapshot is exactly the right instrument, which is what `SDD-bundle.md:143-147` says. The defect
was that the *numbers being formatted* came from nowhere. Sourcing them from a decode of real reducer
output couples the golden to the pipeline; regenerating the golden literal from the serializer would
uncouple it from everything and is forbidden.

**Why the bundle-copy tests sit at the burner and not at the composer.** The composer does not call
the burner (`BUG-19`). Writing at the seam the composer *will* call keeps the assertion honest about
what exists: it proves the copy semantics `AD-4` promises, and proves nothing about whether anybody
invokes them. `W8-S6` closes exactly that gap, with `assert on the file, never on the path string`.
Two stories, two failure modes, neither hiding behind the other.

**Why the aggregate test lives in `src-tauri` and is deliberately redundant.** It needs
`snapdown-capture`, `snapdown-store` and `snapdown-core` at once; `src-tauri/Cargo.toml:21-23` already
has all three, and adding `snapdown-capture` to `snapdown-store`'s dev-dependencies to avoid that
would invert a layer for a test's convenience. The redundancy with the three per-path suites is the
feature: one file that enumerates every image producer is a place a fourth producer's absence is
visible, which is the only durable form the meta-test's intent can take.

**A note on `image`'s feature set.** The workspace pins `image = { version = "0.25.10",
default-features = false, features = ["png"] }` — WebP is not compiled in at all, in either
direction. That is an observation relevant to `OQ-26` and `W8-S6`'s `.webp`-versus-PNG question, and
it is **not** this story's to settle.

## Verification

**Commands** — every one from `AGENTS.md` § Code, run from the repo root:

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
cargo test -p snapdown-store --test test_golden_markdown
cargo test -p snapdown-store --test test_bundle_image_copy
cargo test -p snapdown-core --test test_markdown_serializer
cargo test -p snapdown --test test_image_surface
```

**How to read these runs** — `AGENTS.md` § Code records four ways a verification run lies, and three
apply here. Never pipe a cargo command into `tail` and read the exit code; never append
`echo "EXIT=$?"` and trust the harness's reported code over the echoed value; run `npm ci` before
believing a local web-side red. The frontend is untouched by this story, so a web-side failure is a
stale worktree until proven otherwise.

**Mutation:** each of the eight rows in the mutation table, applied one at a time under
`cargo test --workspace --no-fail-fast`, red observed, then restored and the suite confirmed green.
