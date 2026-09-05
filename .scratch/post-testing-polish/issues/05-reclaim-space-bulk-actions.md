# 05: Reclaim space — select-all and a bulk "Delete both"

**What to build:** A select-all checkbox in Reclaim space, ticking every listed row (rows already
exclude a Bundle whose originals are gone, per `FR-42`'s second bullet — select-all operates only over
what's shown). A bulk "Delete both" action extends `FR-42`'s current promise (reclaim disk space,
originals only) to also remove the Bundles themselves for the whole selected set, following the exact
confirmation discipline `FR-41`/`BUG-104` already established for one Bundle: one dialog, naming the
whole selected set and what is destroyed, "this cannot be undone." Reads `BR-122` live at confirmation
time (`bundles_sharing_findings` in `main.rs`) so a Bundle outside the selection that shares a Finding
with one inside it is still named as something the act will seal. Write ordering per Bundle follows
`AD-2` (row before its own folder) — do not re-derive the partial-failure handling from scratch, extend
`test_bundle_deletion.rs`'s existing single-Bundle pattern.

**This widens `FR-42`'s current wording** (originals-only → originals-and-Bundles for the bulk path).
Once built and green, this is a "code right, document wrong" case: run a quick `wdi-product` pass
afterward to fold the widened promise back into the PRD, per `AGENTS.md`'s "documents follow the code"
rule — do this as part of closing the ticket, not as a separate follow-up nobody does.

**Blocked by:** None (can start immediately). Independent of every other ticket; touches
`finding`-component Reclaim-space UI and `crates/snapdown-store`'s bulk-delete path.

**Status:** done

Realizes `FR-41`, `FR-42`, `BR-122`. See `.scratch/post-testing-polish/spec.md` Implementation
Decisions § "Reclaim space bulk actions" for the full design.

## Seam

Extends `crates/snapdown-store/tests/test_bundle_deletion.rs`'s existing
`remove_bundle_row_and_folder` + `delete_finding_everywhere` pattern to a set of two-or-more Bundles,
including the case where two selected Bundles share one Finding — asserting the shared Finding is
deleted exactly once and reported exactly once, not twice. Mutation-tested (broken deliberately, seen
red, restored) per the spec's own Testing Decisions.

## Acceptance

- [x] A "select all" checkbox ticks every listed row in Reclaim space
- [x] A bulk "Delete both" action removes the selected Bundles' originals AND the Bundles themselves
- [x] One confirmation dialog names the whole selected set and what is destroyed, "cannot be undone" —
      matching `FR-41`/`BUG-104`'s existing single-Bundle discipline
- [x] A Bundle outside the selection sharing a Finding with one inside it is still named in the
      confirmation (`BR-122` read live, not cached)
- [x] A shared Finding across two selected Bundles is deleted exactly once, reported exactly once
- [x] Write ordering per Bundle follows `AD-2`; a partial failure partway through the batch leaves
      prior state intact for what hasn't been touched yet
- [x] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [x] `wdi-product` pass run to fold the widened `FR-42` promise into the PRD, once this ticket's code
      is green — resolved as a new sibling `FR-44` (component: `bundle`, `defers_to: [FR-25, FR-42]`)
      rather than widening `FR-42` itself, per `entity-one-writer` (`bundle` and `finding` own
      different entities; `FR-42` stays unchanged)
- [ ] **Look at:** select several Bundles including two that share a Finding, confirm the dialog names
      everything correctly before confirming — not yet done manually; left for the owner/reviewer to
      run the app and look (see report)
