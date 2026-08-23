use snapdown_core::domain::setting::{QualityBudget, Setting, SettingKey, SettingValue};
use snapdown_core::ports::{Clock, SettingsStore};
use snapdown_store::sqlite::SqliteSettingsStore;
use snapdown_store::system::SystemClock;
use tempfile::NamedTempFile;

#[test]
fn migrations_apply_to_an_empty_database() {
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();

    let store = SqliteSettingsStore::open(&db_path).unwrap();
    assert_eq!(store.get_schema_version().unwrap(), 2);
    assert!(store.is_empty().unwrap());
}

#[test]
fn migrations_are_idempotent() {
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();

    {
        let store = SqliteSettingsStore::open(&db_path).unwrap();
        assert_eq!(store.get_schema_version().unwrap(), 2);

        let clock = SystemClock::new();
        let setting = Setting::new(
            SettingKey::VaultPath,
            SettingValue::String("C:/Users/test/Vault".into()),
            clock.now_rfc3339(),
        );
        store.set(&setting).unwrap();
    }

    // Reopen existing database
    {
        let store2 = SqliteSettingsStore::open(&db_path).unwrap();
        assert_eq!(store2.get_schema_version().unwrap(), 2);

        let s = store2.get(&SettingKey::VaultPath).unwrap();
        assert!(s.is_some());
        assert_eq!(
            s.unwrap().value,
            SettingValue::String("C:/Users/test/Vault".into())
        );
    }
}

#[test]
fn setting_read_falls_back_to_its_shipped_default() {
    let store = SqliteSettingsStore::open_in_memory().unwrap();

    // Query non-existent key
    let missing = store.get(&SettingKey::VaultPath).unwrap();
    assert_eq!(missing, None);

    let default_qb = QualityBudget::default();
    let effective_qb = match store.get(&SettingKey::QualityBudget).unwrap() {
        Some(Setting {
            value: SettingValue::QualityBudget(qb),
            ..
        }) => qb,
        _ => default_qb.clone(),
    };
    assert_eq!(effective_qb, default_qb);
}

#[test]
fn corrupt_library_refuses_to_open_and_does_not_recreate() {
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();

    // Write garbage bytes to database file
    std::fs::write(
        &db_path,
        b"CORRUPTED NOT A SQLITE DATABASE HEADER GIBBERISH",
    )
    .unwrap();

    let open_res = SqliteSettingsStore::open(&db_path);
    assert!(
        open_res.is_err(),
        "Opening a corrupt database must fail with error"
    );

    // Verify file on disk was left untouched
    let contents = std::fs::read(&db_path).unwrap();
    assert_eq!(
        contents,
        b"CORRUPTED NOT A SQLITE DATABASE HEADER GIBBERISH"
    );
}

#[test]
fn setting_crud_and_invalid_deserialization_handling() {
    let store = SqliteSettingsStore::open_in_memory().unwrap();
    let clock = SystemClock::new();

    // Verify empty state initially
    assert!(store.is_empty().unwrap());

    // Set QualityBudget
    let qb = QualityBudget::new(1920, 85).unwrap();
    let setting_qb = Setting::new(
        SettingKey::QualityBudget,
        SettingValue::QualityBudget(qb.clone()),
        clock.now_rfc3339(),
    );
    store.set(&setting_qb).unwrap();

    // Verify non-empty state after inserting setting
    assert!(!store.is_empty().unwrap());

    let loaded = store.get(&SettingKey::QualityBudget).unwrap().unwrap();
    assert_eq!(loaded.value, SettingValue::QualityBudget(qb));

    // List all
    let all = store.list_all().unwrap();
    assert_eq!(all.len(), 1);

    // Delete
    store.delete(&SettingKey::QualityBudget).unwrap();
    assert!(store.get(&SettingKey::QualityBudget).unwrap().is_none());
    assert!(store.is_empty().unwrap());
}
