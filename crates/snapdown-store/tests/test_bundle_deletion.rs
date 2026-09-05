use snapdown_core::domain::bundle::{Bundle, BundleItem};
use snapdown_core::domain::finding::{Finding, Note};
use snapdown_core::ports::{BlobStore, BundleStore, FindingStore};
use snapdown_store::sqlite::{SqliteBundleStore, SqliteFindingStore};
use snapdown_store::vault::VaultBlobStore;
use tempfile::{NamedTempFile, TempDir};

/// **Order corrected 2026-09-02, ticket 16 of the Bundle Library spec.** This test used to delete
/// the files first and the row second - the ORIGINAL ticket 02 decision, before `wdi-review` found
/// it contradicted `AD-2` and the map recorded the correction: the record goes first, then the
/// files. Files-first leaves a row whose Markdown points at images that are gone, which `AD-2`
/// itself names as the state nothing on disk can tell apart from "meant to survive" - row-first
/// leaves only files nothing points at, which the Vault sweeper already owns. This test's own
/// ordering was the trap: an implementer copying "the pattern to test bundle deletion by" would have
/// copied the wrong order along with it.
#[test]
fn bundle_deletion_with_file_synchronization_and_cascade() {
    let tmp_vault = TempDir::new().unwrap();
    let vault_store = VaultBlobStore::new(tmp_vault.path()).unwrap();

    let db_file = NamedTempFile::new().unwrap();
    let bundle_store = SqliteBundleStore::open(db_file.path()).unwrap();
    let finding_store = SqliteFindingStore::open(db_file.path()).unwrap();

    let fid = "018f2345-6789-7abc-8def-0123456789aa";
    let bid = "018f2345-6789-7abc-8def-0123456789bb";

    // 1. Create finding in shared database
    let finding = Finding {
        id: fid.into(),
        image_path: "findings/f1.png".into(),
        image_width: 800,
        image_height: 600,
        captured_at: "2026-08-23T10:00:00Z".into(),
        source_monitor: "DISPLAY1".into(),
        region: "0,0,800,600".into(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note = Note {
        id: "n1".into(),
        finding_id: fid.into(),
        body: "Note".into(),
        updated_at: "2026-08-23T10:00:00Z".into(),
    };
    finding_store.create_finding(&finding, &note, &[]).unwrap();

    // 2. Write bundle files on disk. Both under the SAME `bundles/<id>/` folder, the real layout
    // `plan_bundle` always uses (`main.rs`: `markdown_path = "bundles/{bundle_id}/bundle.md"`,
    // `image_path = "bundles/{bundle_id}/finding_{position}_burned.png"`) - the original fixture put
    // the Markdown one level up from its own image, which no real Bundle ever does and which
    // `delete_folder` below would not have cleaned up correctly.
    let md_path = "bundles/test_bundle/bundle.md";
    let img_path = "bundles/test_bundle/burned_1.png";
    vault_store.write_blob(md_path, b"# Test Bundle").unwrap();
    vault_store
        .write_blob(img_path, b"burned image bytes")
        .unwrap();

    assert!(vault_store.blob_exists(md_path).unwrap());
    assert!(vault_store.blob_exists(img_path).unwrap());

    // 3. Create bundle in store
    let bundle = Bundle::new(
        bid.into(),
        "Test Bundle".into(),
        "# Test Bundle".into(),
        md_path.into(),
        "2026-08-23T10:00:00Z".into(),
    )
    .unwrap();

    let item = BundleItem::new("bi-1".into(), bid.into(), fid.into(), 1, img_path.into()).unwrap();
    bundle_store.create_bundle(&bundle, &[item]).unwrap();

    // Verify bundle created
    assert!(bundle_store.get_bundle(bid).unwrap().is_some());

    // 4. Perform synchronized deletion, AD-2's order: the row first, then the folder holding both
    // files - the whole-folder delete ticket 16 added, not two blob deletes picked one at a time.
    bundle_store.delete_bundle(bid).unwrap();
    vault_store.delete_folder("bundles/test_bundle").unwrap();

    // 5. Verify neither DB record nor files exist
    assert!(bundle_store.get_bundle(bid).unwrap().is_none());
    assert!(!vault_store.blob_exists(md_path).unwrap());
    assert!(!vault_store.blob_exists(img_path).unwrap());
}

/// Ticket `05` of `post-testing-polish` (`FR-41`/`FR-42`/`BR-122`): the bulk "Delete both" action
/// widens the single-Bundle pattern above to a SET of two-or-more Bundles, including the case where
/// two of them share one Finding. Reimplements the same store-level ordering
/// `bundle_deletion_with_file_synchronization_and_cascade` already proves for one Bundle
/// (`AD-2`: the row before its own folder, per Bundle) rather than calling
/// `apps/desktop/src/main.rs`'s own `remove_bundle_row_and_folder`/`delete_finding_everywhere` -
/// those are private functions in a separate (binary) crate this test cannot reach, exactly why the
/// single-Bundle test above already reimplements the pattern with the real store/blob primitives
/// instead of calling them.
///
/// The one new thing this test proves that the single-Bundle test cannot: a Finding two SELECTED
/// Bundles both reference (`fid_shared`, held by both `bundle_a` and `bundle_b`) is deleted exactly
/// once and counted exactly once, never attempted a second time for the second Bundle that also
/// names it. `main.rs`'s real `on_reclaim_space_delete_both_confirmed` handler guards this with an
/// `already_discarded: HashSet<String>` skip, checked BEFORE a Finding is re-read - this test
/// reimplements that same skip and proves it is load-bearing: `get_finding` on a Finding this batch
/// already deleted returns `Ok(None)` (SQLite's own `DELETE` is unconditional, so a naive
/// process-every-item-in-every-Bundle loop would not error on the SQL delete itself - it would
/// error one line earlier, on the re-read `delete_finding_everywhere` performs before it ever
/// deletes anything, exactly the shape the `.expect` below stands in for).
///
/// **Mutation-tested 2026-09-05**: the `if discarded.contains(&finding_id) { continue; }` guard
/// below was commented out, the test was run and observed to panic on the `.expect` inside the loop
/// (the second, undeduped visit to `fid_shared` found no Finding left to re-read), then the guard
/// was restored and the test observed green again.
#[test]
fn bulk_delete_both_removes_every_selected_bundle_and_deletes_a_shared_finding_exactly_once() {
    let tmp_vault = TempDir::new().unwrap();
    let vault_store = VaultBlobStore::new(tmp_vault.path()).unwrap();

    let db_file = NamedTempFile::new().unwrap();
    let bundle_store = SqliteBundleStore::open(db_file.path()).unwrap();
    let finding_store = SqliteFindingStore::open(db_file.path()).unwrap();

    let fid_a_only = "018f2345-6789-7abc-8def-1111111111aa";
    let fid_shared = "018f2345-6789-7abc-8def-2222222222bb";
    let fid_b_only = "018f2345-6789-7abc-8def-3333333333cc";
    let bid_a = "018f2345-6789-7abc-8def-4444444444dd";
    let bid_b = "018f2345-6789-7abc-8def-5555555555ee";

    let make_finding = |id: &str, image_path: &str| Finding {
        id: id.into(),
        image_path: image_path.into(),
        image_width: 800,
        image_height: 600,
        captured_at: "2026-09-05T10:00:00Z".into(),
        source_monitor: "DISPLAY1".into(),
        region: "0,0,800,600".into(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let make_note = |id: &str, finding_id: &str| Note {
        id: id.into(),
        finding_id: finding_id.into(),
        body: "Note".into(),
        updated_at: "2026-09-05T10:00:00Z".into(),
    };

    // Three Findings, each with its own image file under `findings/` - the ORIGINALS Reclaim space
    // measures and Delete both removes.
    for (fid, img) in [
        (fid_a_only, "findings/f_a_only.png"),
        (fid_shared, "findings/f_shared.png"),
        (fid_b_only, "findings/f_b_only.png"),
    ] {
        let finding = make_finding(fid, img);
        let note = make_note(&format!("n-{fid}"), fid);
        finding_store.create_finding(&finding, &note, &[]).unwrap();
        vault_store
            .write_blob(img, b"original capture bytes")
            .unwrap();
        assert!(vault_store.blob_exists(img).unwrap());
    }

    // Two Bundles, each in its own `bundles/<id>/` folder (the real `plan_bundle` layout) - `A`
    // holds `fid_a_only` and `fid_shared`, `B` holds `fid_shared` and `fid_b_only`. Both are in the
    // SELECTED (ticked) set for this bulk act.
    let md_a = "bundles/bundle_a/bundle.md";
    let img_a1 = "bundles/bundle_a/finding_1_burned.png";
    let img_a2 = "bundles/bundle_a/finding_2_burned.png";
    let md_b = "bundles/bundle_b/bundle.md";
    let img_b1 = "bundles/bundle_b/finding_1_burned.png";
    let img_b2 = "bundles/bundle_b/finding_2_burned.png";
    for (md, img1, img2) in [(md_a, img_a1, img_a2), (md_b, img_b1, img_b2)] {
        vault_store.write_blob(md, b"# Bundle").unwrap();
        vault_store.write_blob(img1, b"burned image bytes").unwrap();
        vault_store.write_blob(img2, b"burned image bytes").unwrap();
    }

    let bundle_a = Bundle::new(
        bid_a.into(),
        "Bundle A".into(),
        "# Bundle A".into(),
        md_a.into(),
        "2026-09-05T10:00:00Z".into(),
    )
    .unwrap();
    let items_a = vec![
        BundleItem::new(
            "bi-a1".into(),
            bid_a.into(),
            fid_a_only.into(),
            1,
            img_a1.into(),
        )
        .unwrap(),
        BundleItem::new(
            "bi-a2".into(),
            bid_a.into(),
            fid_shared.into(),
            2,
            img_a2.into(),
        )
        .unwrap(),
    ];
    bundle_store.create_bundle(&bundle_a, &items_a).unwrap();

    let bundle_b = Bundle::new(
        bid_b.into(),
        "Bundle B".into(),
        "# Bundle B".into(),
        md_b.into(),
        "2026-09-05T10:00:00Z".into(),
    )
    .unwrap();
    let items_b = vec![
        BundleItem::new(
            "bi-b1".into(),
            bid_b.into(),
            fid_shared.into(),
            1,
            img_b1.into(),
        )
        .unwrap(),
        BundleItem::new(
            "bi-b2".into(),
            bid_b.into(),
            fid_b_only.into(),
            2,
            img_b2.into(),
        )
        .unwrap(),
    ];
    bundle_store.create_bundle(&bundle_b, &items_b).unwrap();

    assert!(bundle_store.get_bundle(bid_a).unwrap().is_some());
    assert!(bundle_store.get_bundle(bid_b).unwrap().is_some());

    // THE BULK ACT - `main.rs`'s own `on_reclaim_space_delete_both_confirmed` shape: per Bundle,
    // `AD-2`'s order (row before its own folder), then over that Bundle's own Findings a Finding a
    // PRIOR Bundle in this same batch already discarded is skipped, never processed twice.
    let batch = [
        (
            bid_a,
            md_a,
            "bundles/bundle_a",
            vec![fid_a_only, fid_shared],
        ),
        (
            bid_b,
            md_b,
            "bundles/bundle_b",
            vec![fid_shared, fid_b_only],
        ),
    ];

    let mut discarded: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut discard_count = 0usize;
    let mut bundles_removed = 0usize;

    for (bid, _md, folder, finding_ids) in &batch {
        // AD-2: the Bundle's row before its own folder.
        bundle_store.delete_bundle(bid).unwrap();
        vault_store.delete_folder(folder).unwrap();
        bundles_removed += 1;

        for finding_id in finding_ids {
            // THE GUARD BEING MUTATION-TESTED. Comment this out to watch the test go red: the
            // second, undeduped visit to `fid_shared` (from `bundle_b`'s own list) then reaches
            // the `.expect` below with nothing left to find.
            if discarded.contains(finding_id) {
                continue;
            }

            let detail = finding_store.get_finding(finding_id).unwrap().expect(
                "a Finding not yet in `discarded` must still be readable - finding it gone \
                     here means an earlier Bundle in this batch already removed it WITHOUT the \
                     dedup guard recording that fact, i.e. the shared Finding was about to be \
                     processed a second time",
            );
            finding_store.delete_finding(finding_id).unwrap();
            vault_store.delete_blob(&detail.finding.image_path).unwrap();
            discarded.insert(finding_id);
            discard_count += 1;
        }
    }

    // Both Bundles are gone - row and folder.
    assert_eq!(bundles_removed, 2);
    assert!(bundle_store.get_bundle(bid_a).unwrap().is_none());
    assert!(bundle_store.get_bundle(bid_b).unwrap().is_none());
    assert!(!vault_store.blob_exists(md_a).unwrap());
    assert!(!vault_store.blob_exists(img_a1).unwrap());
    assert!(!vault_store.blob_exists(img_a2).unwrap());
    assert!(!vault_store.blob_exists(md_b).unwrap());
    assert!(!vault_store.blob_exists(img_b1).unwrap());
    assert!(!vault_store.blob_exists(img_b2).unwrap());

    // All three Findings are gone - row and file - including the shared one.
    assert!(finding_store.get_finding(fid_a_only).unwrap().is_none());
    assert!(finding_store.get_finding(fid_shared).unwrap().is_none());
    assert!(finding_store.get_finding(fid_b_only).unwrap().is_none());
    assert!(!vault_store.blob_exists("findings/f_a_only.png").unwrap());
    assert!(!vault_store.blob_exists("findings/f_shared.png").unwrap());
    assert!(!vault_store.blob_exists("findings/f_b_only.png").unwrap());

    // THE ACCEPTANCE CRITERION: three DISTINCT Findings existed across two Bundles that together
    // named four `BundleItem`s (2 + 2) over them - `fid_shared` deleted and counted EXACTLY ONCE,
    // not twice, is what keeps this at 3 rather than 4.
    assert_eq!(
        discard_count, 3,
        "the shared Finding must be deleted exactly once and counted exactly once, not once per \
         Bundle that names it"
    );
}
