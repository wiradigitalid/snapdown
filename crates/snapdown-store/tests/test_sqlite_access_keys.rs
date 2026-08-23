use snapdown_core::domain::access_key::{AccessKey, AuthResult};
use snapdown_core::ports::AccessKeyStore;
use snapdown_store::sqlite::SqliteAccessKeyStore;
use tempfile::NamedTempFile;

#[test]
fn migrations_apply_cleanly_and_create_access_key_table() {
    let temp = NamedTempFile::new().unwrap();
    let store = SqliteAccessKeyStore::open(temp.path()).expect("open access key store");

    assert_eq!(store.get_schema_version().unwrap(), 7);
}

#[test]
fn access_key_issuance_rotation_and_verification() {
    let store = SqliteAccessKeyStore::open_in_memory().expect("open memory store");

    // 1. Initial state -> No key configured
    assert_eq!(
        store.verify_key("any_secret").unwrap(),
        AuthResult::NoKeyConfigured
    );
    assert!(store.get_active_key().unwrap().is_none());

    // 2. Issue first key
    let secret1 = "sd_key_secret_one_123456789";
    let hash1 = AccessKey::sha256_hex(secret1.as_bytes());
    let key1 = AccessKey::new("k-1".into(), hash1, "2026-08-23T10:00:00Z".into(), None).unwrap();

    store.save_key(&key1).expect("save key 1");

    // Verify key 1 works
    assert_eq!(store.verify_key(secret1).unwrap(), AuthResult::Valid);
    assert_eq!(
        store.verify_key("wrong_secret").unwrap(),
        AuthResult::Invalid
    );

    let active = store.get_active_key().unwrap().unwrap();
    assert_eq!(active.id, "k-1");

    // 3. Issue second key -> should automatically revoke key 1
    let secret2 = "sd_key_secret_two_987654321";
    let hash2 = AccessKey::sha256_hex(secret2.as_bytes());
    let key2 = AccessKey::new("k-2".into(), hash2, "2026-08-23T11:00:00Z".into(), None).unwrap();

    store.save_key(&key2).expect("save key 2");

    // Secret 2 is now valid, secret 1 is invalid
    assert_eq!(store.verify_key(secret2).unwrap(), AuthResult::Valid);
    assert_eq!(store.verify_key(secret1).unwrap(), AuthResult::Invalid);

    let active2 = store.get_active_key().unwrap().unwrap();
    assert_eq!(active2.id, "k-2");

    // 4. Explicitly revoke active key
    store
        .revoke_active_key("2026-08-23T12:00:00Z")
        .expect("revoke active key");

    assert!(store.get_active_key().unwrap().is_none());
    assert_eq!(
        store.verify_key(secret2).unwrap(),
        AuthResult::NoKeyConfigured
    );
}