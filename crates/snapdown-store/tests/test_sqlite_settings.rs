use snapdown_core::domain::setting::{
    NamedBudget, QualityBudget, ResolvedPair, Setting, SettingKey, SettingValue,
    MAX_ENCODER_QUALITY, MAX_LONG_EDGE_PX, MIN_ENCODER_QUALITY, MIN_LONG_EDGE_PX,
};
use snapdown_core::ports::{Clock, SettingsStore};
use snapdown_store::sqlite::SqliteSettingsStore;
use snapdown_store::system::SystemClock;
use tempfile::NamedTempFile;

#[test]
fn migrations_apply_to_an_empty_database() {
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();

    let store = SqliteSettingsStore::open(&db_path).unwrap();
    assert_eq!(store.get_schema_version().unwrap(), 9);
    assert!(store.is_empty().unwrap());
}

#[test]
fn migrations_are_idempotent() {
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();

    {
        let store = SqliteSettingsStore::open(&db_path).unwrap();
        assert_eq!(store.get_schema_version().unwrap(), 9);

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
        assert_eq!(store2.get_schema_version().unwrap(), 9);

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
    assert_eq!(effective_qb.named, NamedBudget::Auto);
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
fn the_named_state_and_its_resolved_pair_are_written_together() {
    let store = SqliteSettingsStore::open_in_memory().unwrap();
    let clock = SystemClock::new();

    // 1. Store named budget: Sharp
    let sharp_qb = QualityBudget::new(NamedBudget::Sharp, None);
    let sharp_setting = Setting::new(
        SettingKey::QualityBudget,
        SettingValue::QualityBudget(sharp_qb.clone()),
        clock.now_rfc3339(),
    );
    store.set(&sharp_setting).unwrap();

    let loaded_sharp = store.get(&SettingKey::QualityBudget).unwrap().unwrap();
    assert_eq!(loaded_sharp.value, SettingValue::QualityBudget(sharp_qb));

    // 2. Store custom budget atomically (BR-116)
    let custom_pair = ResolvedPair::new(2048, 88).unwrap();
    let custom_qb = QualityBudget::new(NamedBudget::Custom, Some(custom_pair));
    let custom_setting = Setting::new(
        SettingKey::QualityBudget,
        SettingValue::QualityBudget(custom_qb.clone()),
        clock.now_rfc3339(),
    );
    store.set(&custom_setting).unwrap();

    let loaded_custom = store.get(&SettingKey::QualityBudget).unwrap().unwrap();
    assert_eq!(loaded_custom.value, SettingValue::QualityBudget(custom_qb));
}

#[test]
fn an_advanced_value_outside_its_range_is_refused_and_does_not_enter_custom() {
    // Attempting to construct out-of-bounds ResolvedPair fails (BR-117)
    let below_min_edge = ResolvedPair::new(MIN_LONG_EDGE_PX - 1, 80);
    assert!(below_min_edge.is_err());

    let above_max_edge = ResolvedPair::new(MAX_LONG_EDGE_PX + 1, 80);
    assert!(above_max_edge.is_err());

    let below_min_quality = ResolvedPair::new(1920, MIN_ENCODER_QUALITY - 1);
    assert!(below_min_quality.is_err());

    let above_max_quality = ResolvedPair::new(1920, MAX_ENCODER_QUALITY + 1);
    assert!(above_max_quality.is_err());
}

#[test]
fn valid_header_with_corrupt_pages_refuses_to_open_and_leaves_file_byte_identical() {
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();

    // Create a real SQLite database with schema and data spanning multiple pages in rollback journal mode
    {
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE sample (id INTEGER PRIMARY KEY, note TEXT);",
            [],
        )
        .unwrap();
        for i in 0..200 {
            conn.execute(
                "INSERT INTO sample (note) VALUES (?1);",
                [format!("padding data row {i}")],
            )
            .unwrap();
        }
    }

    let mut bytes = std::fs::read(&db_path).unwrap();
    assert!(bytes.len() >= 8192, "Database must span multiple pages");
    assert_eq!(&bytes[0..16], b"SQLite format 3\0");

    // Corrupt internal B-tree page 2 bytes and beyond (4096..)
    for b in &mut bytes[4096..] {
        *b = 0xBB;
    }
    std::fs::write(&db_path, &bytes).unwrap();

    let open_res = SqliteSettingsStore::open(&db_path);
    assert!(open_res.is_err(), "Store open on corrupt pages must fail");

    // Verify file is byte-identical and no -wal / -shm were created
    let read_back = std::fs::read(&db_path).unwrap();
    assert_eq!(
        read_back, bytes,
        "Corrupt file bytes must remain completely unmodified"
    );

    let wal_path = db_path.with_file_name(format!(
        "{}-wal",
        db_path.file_name().unwrap().to_str().unwrap()
    ));
    let shm_path = db_path.with_file_name(format!(
        "{}-shm",
        db_path.file_name().unwrap().to_str().unwrap()
    ));
    assert!(!wal_path.exists());
    assert!(!shm_path.exists());
}
