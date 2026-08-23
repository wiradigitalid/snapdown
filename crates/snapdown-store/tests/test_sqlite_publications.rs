use snapdown_core::domain::bundle::Bundle;
use snapdown_core::domain::publication::Publication;
use snapdown_core::ports::{BundleStore, PublicationStore};
use snapdown_store::sqlite::{SqliteBundleStore, SqlitePublicationStore};
use tempfile::NamedTempFile;

#[test]
fn migrations_apply_cleanly_and_create_publication_table() {
    let temp = NamedTempFile::new().unwrap();
    let store = SqlitePublicationStore::open(temp.path()).expect("open pub store");

    assert_eq!(store.get_schema_version().unwrap(), 7);
}

#[test]
fn publication_store_lifecycle_and_slug_uniqueness() {
    let temp = NamedTempFile::new().unwrap();
    let pub_store = SqlitePublicationStore::open(temp.path()).expect("open pub store");
    let bundle_store = SqliteBundleStore::open(temp.path()).expect("open bundle store");

    let bid = "018f2345-6789-7abc-8def-0123456789aa";
    let bundle = Bundle::new(
        bid.into(),
        "Test Bundle".into(),
        "# Markdown".into(),
        "bundles/b.md".into(),
        "2026-08-23T10:00:00Z".into(),
    )
    .unwrap();
    bundle_store.create_bundle(&bundle, &[]).unwrap();

    // 1. Publish bundle
    let pub_record = Publication::new(
        "pub-1".into(),
        bid.into(),
        "testslug123".into(),
        "https://snapdown.dev".into(),
        "2026-08-23T10:00:00Z".into(),
        None,
        None,
    )
    .unwrap();

    pub_store.save(&pub_record).expect("save publication");

    // 2. Query by bundle_id and slug
    let fetched = pub_store
        .get_by_bundle_id(bid)
        .expect("get by bid")
        .expect("must exist");
    assert_eq!(fetched.slug, "testslug123");
    assert!(fetched.is_live());

    let fetched_slug = pub_store
        .get_by_slug("testslug123")
        .expect("get by slug")
        .expect("must exist");
    assert_eq!(fetched_slug.bundle_id, bid);

    // 3. Set last error
    pub_store
        .set_last_error(bid, Some("Connection timeout on upload"))
        .expect("set last error");
    let err_fetched = pub_store.get_by_bundle_id(bid).unwrap().unwrap();
    assert_eq!(
        err_fetched.last_error,
        Some("Connection timeout on upload".into())
    );

    // 4. Mark unpublished
    pub_store
        .mark_unpublished(bid, "2026-08-23T11:00:00Z")
        .expect("unpublish");
    let unpub_fetched = pub_store.get_by_bundle_id(bid).unwrap().unwrap();
    assert!(!unpub_fetched.is_live());
    assert_eq!(
        unpub_fetched.unpublished_at,
        Some("2026-08-23T11:00:00Z".into())
    );

    // 5. Delete by bundle ID
    pub_store.delete_by_bundle_id(bid).expect("delete pub");
    assert!(pub_store.get_by_bundle_id(bid).unwrap().is_none());
}
