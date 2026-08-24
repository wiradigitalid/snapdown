W8-S6 Step 2 - BUILD. This is the LAST story of wave W8.

WDI Method, G5 Release, wdi-build Phase 3 Step 2. Read AGENTS.md first.

Run bmad-build-auto given the spec path:
  _bmad-output/specs/w8-capture-becomes-real/stories/W8-S6-the-burn-is-actually-called-and-the-recorded-path-holds-a-file.md

It is committed on your base branch wave/w8-capture-becomes-real and it is the contract. Its
<intent-contract> is the owner's and MUST NOT be edited.

THE STORY'S SCOPE HAS GROWN SINCE IT WAS PLANNED. The coordinator adjudicated the planner's report
and waves.yaml now names FIVE tests for W8-S6, not three. Two were added:

  cargo::a_corrupt_source_is_refused_even_when_no_marker_is_drawn
  cargo::the_composed_markdown_references_the_bundles_burned_copy

Read waves.yaml W8-S6 for the comments explaining both. The full list:

  cargo::a_bundle_item_image_path_holds_a_file_that_decodes
  cargo::an_exported_bundle_image_carries_the_markers_of_its_finding
  cargo::a_bundle_export_does_not_re_reduce_the_stored_image
  cargo::a_corrupt_source_is_refused_even_when_no_marker_is_drawn
  cargo::the_composed_markdown_references_the_bundles_burned_copy

YOU ARE CLOSING THREE DEFECTS, NOT ONE:

BUG-19 - MarkerBurner is called from nowhere. bundle.rs:41-48 records
bundles/{id}/finding_{pos}_burned.webp and nothing ever writes that file.

BUG-20 - burn_markers returns Ok(input_bytes.to_vec()) at burner.rs:48-50 BEFORE any decode when no
Marker carries a Note line, so a corrupt blob is reported as success and the dimension check below
is skipped with it. The plan places the fix in the burner: hoist the decode and dimension check
ABOVE the no-markers early return, and still return input_bytes.to_vec() so AD-4 byte identity
survives. A test for it MUST use a VALID PNG HEADER WITH A CORRUPT PAYLOAD - W7 proved garbage bytes
never reach such a path, the decoder rejects them at its front door.

BUG-21 - THE ONE MOST LIKELY TO BE UNDERESTIMATED. crates/snapdown-core/src/domain/markdown.rs:30
builds the image reference from item.finding.image_path - the FINDING'S CLEAN IMAGE - and never
reads BundleItem.image_path. Writing the burned file without referencing it leaves FR-8 UNMET. The
serializer takes &[FindingDetail] and has no BundleItem in hand, so the signature has to carry it;
that is the real work and it is why this is not a one-line change.

TWO GOLDENS W8-S5 JUST PINNED WILL MOVE. That is CORRECT and EXPECTED - they were pinned against the
wrong reference, and a golden pins whatever it is shown. Regenerate them DELIBERATELY and RE-RUN THE
MUTATION so each moved golden is proven to still fail when it should. Do not weaken a golden to make
it pass.

DEC-006 IS OPEN AT DRAFT and it settles the format question: every stored image is PNG. The
workspace pins image with features=["png"] so WebP is not compiled in - the .webp extension at
bundle.rs:41 was never producible and is corrected to .png. Read
.control/decisions/DEC-006-stored-image-format-is-png.md. It is draft, not applied, so cite it as
the reason and report if your work contradicts it rather than editing it.

AD-4 (ARCHITECTURE-SPINE.md:106-109) governs and backs the copy tests, NOT AD-9: "A Bundle's image is
a copy of the Finding's image with Markers drawn on it, at the same dimensions." The export burn
takes the ALREADY-REDUCED stored bytes and MUST NOT re-reduce them. SCN-04 still binds: a Marker with
no Note line is never drawn, and W8-S3 already implements that as a comment filter - you are calling
that code, not reimplementing the rule.

W8-S5 left crates/snapdown-store/tests/test_bundle_image_copy.rs holding the two bundle-copy tests at
the MarkerBurner seam, with their FILE-LEVEL forms explicitly left to you. Read it first.

TWO DEFECTS ARE REGISTERED AND ARE NOT YOURS. Do not fix them and do not write tests that depend on
them: BUG-22 (bundle.rs:34-38 silently skips an unresolvable finding_id) and BUG-23 (sharing.rs:116
swallows a failed read_blob and publishes anyway).

BEFORE YOU CLOSE, GREP: grep -rn "MarkerBurner" --include=*.rs crates apps, excluding image/burner.rs
and tests/. If the only hits are still the two re-exports, THE STORY IS NOT DONE whatever the tests
say. That check is why this story exists.

MUTATION IS THE ACCEPTANCE CRITERION. Break each behaviour, watch the test go red, put it back. USE
--no-fail-fast: cargo stops at the first failing binary otherwise, later tests never run, and a live
test reads as dead. That produced a false result in W8-S2.

let _ = AND if let Ok( ON A RESULT AN INVARIANT DEPENDS ON IS A DEFECT, not a style. BUG-23 is one of
those and five more were found in this crate on 2026-08-23. The blob writes you add MUST NOT be
swallowed.

Rules that bind every worker on this repo:
- Debugging is conditional, never a phase. Unknown cause -> wdi-systematic-debugging FIRST. A third
  failed fix attempt is an escalation, not a fourth attempt.
- The corpus is not yours to change. MUST NOT edit .what/, .how/, or an applied DEC-. A deviation is
  REPORTED and becomes a DEC-.
- Verification is run, not assumed. Every command in AGENTS.md section Code, and read the four ways a
  verification run lies recorded there.
- Never commit a fixture derived from real capture output - synthesise it. The repo is public.
- Write UTF-8, no BOM, and watch for a lone cp1252 byte.
- No scratch files in the commit. Commit locally. DO NOT PUSH.

Done when the story frontmatter reads status: done and the full verification set is green. Report
worker_done with --outcome succeeded and the spec path, or --outcome failed with the blocking reason.
