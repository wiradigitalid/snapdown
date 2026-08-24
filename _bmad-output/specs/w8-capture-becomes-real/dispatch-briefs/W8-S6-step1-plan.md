# W8-S6 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W8**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w8-capture-becomes-real/`
- `story_id`: `W8-S6`

`W8-S1` through `W8-S5` have landed. **This story is the one that connects W8-S3's work to the
running application.**

## The defect — `BUG-19`, and it is the fifth of its kind

`apps/desktop/src-tauri/src/commands/bundle.rs:41-48`:

```rust
let burned_path = format!("bundles/{bundle_id}/finding_{pos}_burned.webp");

bundle_items.push(BundleItem {
    ...
    image_path: burned_path,
});
```

**Nothing ever writes that file.** The only `write_blob` calls in the entire desktop crate are
`bundle.rs:75` (the Markdown) and `capture.rs:107` (the reduced capture).

And a repo-wide grep for `MarkerBurner` outside its own file and its tests returns exactly two hits,
both re-exports:

```
crates/snapdown-store/src/image/mod.rs:4  pub use burner::MarkerBurner;
crates/snapdown-store/src/lib.rs:8        pub use image::{ImageReducer, MarkerBurner, ...};
```

So the burn `W8-S3` built is **unreachable from the running application**, and the path recorded in
the database points at a file that has never existed.

## Why this one is worse than its four predecessors

After `BUG-4` (`CaptureOverlay`), `BUG-5` (`MarkerLayer`), `BUG-6` (`OrphanReportView`) and the
`EmptyState` sweep, this is the **fifth** unit built, unit-tested, and mounted nowhere.

What makes it worse: **the caller writes a path that implies the call happened.** A dangling image
reference reads as a bug in whoever reads it, not as a missing call in whoever wrote it.

The miss is hidden a second time on the deletion path. `bundle.rs:171-175` checks `blob_exists`
before deleting each item image — so **the guard that would have surfaced the gap is the same guard
that conceals it.**

`W8-S3`'s five tests are green and honest. They prove the burner draws correctly. They cannot prove
anybody calls it. There is still no composition test class (`OQ-23`), and `V12` does not help: it
checks that an `LC` is *registered*, not that it is *reached*.

## What it breaks

`FR-8` promises a handed-over Bundle carries its Markers **on the image**. The Bundle records that it
does, and it does not. A Reviewer exporting a Bundle gets Markdown plus a set of dangling references.

## Two constraints, and one open question you must resolve in the plan

**1. `AD-4` — an image is reduced exactly once, at capture, and no original is kept.** The export
burn takes the **already-reduced** stored bytes and MUST NOT re-reduce them. There is a named test
for it below.

**2. `SCN-04` still binds.** A Marker with no Note line is never drawn. `W8-S3` implements this as a
comment filter in `burner.rs`; you are calling that code, not reimplementing the rule.

**The open question: the recorded path says `.webp` and the burner emits PNG.** The two currently
disagree. Read the corpus before choosing — `.how/bundle/`, `.what/bundle/`, and the image inventory
— and say in the plan which is right and what evidence settles it. If nothing in the corpus settles
it, that is a `DEC-` through `wdi-decision`, not a coin flip.

## The tests

`waves.yaml` records three, carried through verbatim:

```
cargo::a_bundle_item_image_path_holds_a_file_that_decodes
cargo::an_exported_bundle_image_carries_the_markers_of_its_finding
cargo::a_bundle_export_does_not_re_reduce_the_stored_image
```

**The first is the whole point and it must be written to fail today.** It asserts that the file at
`BundleItem.image_path` **exists and decodes** — never that the path string was constructed. A test
that checks the format string is a test that cannot fail, and it is what this repository keeps
writing.

The second is what makes it a burn rather than a copy: decode the exported image and confirm the
marker pixels are there.

The third guards `AD-4`: assert the exported image's dimensions equal the stored image's, so a second
reduction would be visible.

## The seam

`tauri::test::mock_app` yields `STATUS_ENTRYPOINT_NOT_FOUND` on this platform, so the Tauri commands
here are split into an `_impl(&AppState)` inner function a test can call directly. Follow that
existing pattern rather than inventing a new one.

**Mutation is the acceptance criterion.** Break each behaviour, watch the test go red, put it back.
Use `--no-fail-fast`: cargo stops at the first failing binary otherwise, later tests never run, and a
live test reads as dead. That produced a false result in `W8-S2`.

## Before you close, grep

**`grep -rn "MarkerBurner" --include=*.rs crates apps`**, excluding `image/burner.rs` and `tests/`.

If the only hits are still re-exports, the story has not been done — whatever the tests say. That is
the check this entire story exists because nobody ran.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** Unknown cause → `wdi-systematic-debugging` first; a
  third failed attempt is an escalation.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation is **reported** and becomes a `DEC-`.
- **Verification is run, not assumed.** Every command in `AGENTS.md` § Code, and read the four ways a
  verification run lies recorded there.
- **`let _ =` on a Result an invariant depends on is a defect.** Five were found in this crate on
  2026-08-23, one of which left a published Bundle live on the internet after deletion. The blob
  writes you add here MUST NOT be swallowed.
- **Write UTF-8, no BOM, and watch for a lone cp1252 byte.**
- **No scratch files in the commit. Do not push.**

## Done means

`_bmad-output/specs/w8-capture-becomes-real/stories/W8-S6-*.md` exists, carries an
`<intent-contract>`, and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
