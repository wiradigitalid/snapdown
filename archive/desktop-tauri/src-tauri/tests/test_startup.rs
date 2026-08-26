use desktop_lib::commands::startup::{
    get_startup_status_impl, set_startup_status_impl, StartupState,
};
use desktop_lib::hotkey::DesktopHotkeyRegistrar;
use desktop_lib::startup::{reconcile_startup_on_boot, AutoStartBackend, DesktopStartupRegistrar};
use desktop_lib::state::AppState;
use desktop_lib::{format_startup_error_message, init_app_stores, StartupError};
use snapdown_core::domain::setting::{Setting, SettingKey, SettingValue};
use snapdown_core::ports::{Clock, SettingsStore};
use snapdown_store::sqlite::{
    SqliteAccessKeyStore, SqliteBundleStore, SqliteFindingStore, SqlitePublicationStore,
    SqliteSettingsStore,
};
use snapdown_store::system::SystemClock;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;

#[derive(Default)]
struct TestAutoStartBackend {
    enabled: AtomicBool,
    queries: std::sync::atomic::AtomicUsize,
    fail_on_enable: AtomicBool,
}

impl AutoStartBackend for TestAutoStartBackend {
    fn is_enabled(&self) -> Result<bool, String> {
        self.queries.fetch_add(1, Ordering::SeqCst);
        Ok(self.enabled.load(Ordering::SeqCst))
    }

    fn enable(&self) -> Result<(), String> {
        if self.fail_on_enable.load(Ordering::SeqCst) {
            return Err("OS error: autostart permission denied".to_string());
        }
        self.enabled.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn disable(&self) -> Result<(), String> {
        self.enabled.store(false, Ordering::SeqCst);
        Ok(())
    }
}

fn create_test_app_state(
    store: Arc<SqliteSettingsStore>,
    backend: Arc<TestAutoStartBackend>,
) -> AppState {
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();
    let finding_store = Arc::new(SqliteFindingStore::open(&db_path).unwrap());
    let bundle_store = Arc::new(SqliteBundleStore::open(&db_path).unwrap());
    let access_key_store = Arc::new(SqliteAccessKeyStore::open(&db_path).unwrap());
    let publication_store = Arc::new(SqlitePublicationStore::open(&db_path).unwrap());
    let hotkey_registrar = Arc::new(Mutex::new(DesktopHotkeyRegistrar::new(store.clone(), None)));
    let startup_registrar = Arc::new(Mutex::new(DesktopStartupRegistrar::new(backend)));

    AppState {
        settings_store: store,
        finding_store,
        bundle_store,
        access_key_store,
        publication_store,
        hotkey_registrar,
        startup_registrar,
    }
}

fn create_valid_header_corrupt_db(path: &Path) -> Vec<u8> {
    // Create standard SQLite database with schema and multiple pages in rollback journal mode
    {
        let conn = rusqlite::Connection::open(path).unwrap();
        conn.execute(
            "CREATE TABLE test_table (id INTEGER PRIMARY KEY, content TEXT);",
            [],
        )
        .unwrap();
        for i in 0..200 {
            conn.execute(
                "INSERT INTO test_table (content) VALUES (?1);",
                [format!(
                    "test content row with long padding text data to allocate multiple pages {i}"
                )],
            )
            .unwrap();
        }
    }

    let mut bytes = std::fs::read(path).unwrap();
    assert!(bytes.len() >= 8192, "Database must span multiple pages");
    assert_eq!(&bytes[0..16], b"SQLite format 3\0");

    // Keep page 1 (schema) intact, corrupt page 2 and beyond (4096..)
    for b in &mut bytes[4096..] {
        *b = 0xAA;
    }

    std::fs::write(path, &bytes).unwrap();
    bytes
}

#[test]
fn a_store_that_cannot_be_opened_yields_an_error_not_a_panic() {
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();
    create_valid_header_corrupt_db(&db_path);

    // init_app_stores returns Err instead of panicking
    let result = init_app_stores(&db_path);
    assert!(
        result.is_err(),
        "Corrupt database must yield Err, not a panic"
    );
}

#[test]
fn the_startup_error_names_the_path_of_the_file_that_failed() {
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();
    create_valid_header_corrupt_db(&db_path);

    let open_res = init_app_stores(&db_path);
    assert!(
        open_res.is_err(),
        "Opening a corrupt database must fail with error"
    );

    let err = match open_res {
        Err(e) => e,
        Ok(_) => unreachable!(),
    };

    match &err {
        StartupError::DatabaseOpen { path, source: _ } => {
            assert_eq!(path, &db_path);
            let formatted = format_startup_error_message(&err);
            assert!(
                formatted.contains(&db_path.display().to_string()),
                "Error message must contain exact database path"
            );
            assert!(
                formatted.contains(
                    "Snapdown will not recreate or overwrite this file to prevent data loss."
                ),
                "Error message must reassure data preservation"
            );
        }
    }
}

#[test]
fn a_corrupt_store_is_never_recreated_beside_itself() {
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();
    let corrupt_bytes = create_valid_header_corrupt_db(&db_path);

    let open_res = init_app_stores(&db_path);
    assert!(open_res.is_err());

    // BR-118: Verify file on disk is left untouched and not overwritten or recreated
    let remaining_bytes = std::fs::read(&db_path).unwrap();
    assert_eq!(
        remaining_bytes, corrupt_bytes,
        "Corrupted database file on disk must remain unmodified"
    );
}

#[test]
fn a_readable_store_still_starts_normally() {
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();

    let stores = init_app_stores(&db_path).expect("Opening valid store must succeed");

    let clock = SystemClock::new();
    let setting = Setting::new(
        SettingKey::VaultPath,
        SettingValue::String("C:/Users/test/Vault".into()),
        clock.now_rfc3339(),
    );
    stores
        .settings_store
        .set(&setting)
        .expect("Setting should save successfully");

    let loaded = stores
        .settings_store
        .get(&SettingKey::VaultPath)
        .expect("Get should succeed");
    assert!(loaded.is_some());
    assert_eq!(
        loaded.unwrap().value,
        SettingValue::String("C:/Users/test/Vault".into())
    );
}

#[test]
fn a_valid_header_with_corrupt_pages_is_not_written_to_before_it_is_checked() {
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();
    let original_corrupt_bytes = create_valid_header_corrupt_db(&db_path);

    let res = init_app_stores(&db_path);
    assert!(res.is_err(), "Store open must fail on corrupt pages");

    let bytes_after_attempt = std::fs::read(&db_path).unwrap();
    assert_eq!(
        bytes_after_attempt, original_corrupt_bytes,
        "Database bytes must be byte-identical on disk (no WAL pragma mutation on page 1)"
    );
}

#[test]
fn a_failed_open_leaves_no_wal_or_shm_file_beside_the_database() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let db_path = tmp_dir.path().join("library.db");
    create_valid_header_corrupt_db(&db_path);

    let wal_path = tmp_dir.path().join("library.db-wal");
    let shm_path = tmp_dir.path().join("library.db-shm");

    assert!(!wal_path.exists());
    assert!(!shm_path.exists());

    let res = init_app_stores(&db_path);
    assert!(res.is_err(), "Store open must fail");

    assert!(
        !wal_path.exists(),
        "Failed open must not create -wal auxiliary file on disk"
    );
    assert!(
        !shm_path.exists(),
        "Failed open must not create -shm auxiliary file on disk"
    );
}

#[test]
fn a_store_failure_exits_without_panicking() {
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();
    create_valid_header_corrupt_db(&db_path);

    // Opening corrupt db yields Err without panicking
    let result = std::panic::catch_unwind(|| init_app_stores(&db_path));

    assert!(result.is_ok(), "init_app_stores must not panic on failure");
    let open_res = result.unwrap();
    assert!(
        open_res.is_err(),
        "init_app_stores must return Err on failure"
    );

    let err = match open_res {
        Err(e) => e,
        Ok(_) => unreachable!(),
    };

    let msg = format_startup_error_message(&err);
    assert!(
        msg.contains("Snapdown could not open its library database"),
        "Error message must describe database open failure"
    );
    assert!(
        msg.contains(&db_path.display().to_string()),
        "Error message must contain exact database path"
    );
}

#[test]
fn a_first_run_with_nothing_configured_registers_at_startup() {
    // SCN-02 Run 1: Fresh install with nothing configured
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();
    let store = Arc::new(SqliteSettingsStore::open(&db_path).unwrap());
    let backend = Arc::new(TestAutoStartBackend::default());
    let mut registrar = DesktopStartupRegistrar::new(backend.clone());
    let clock = SystemClock::new();

    assert!(!backend.enabled.load(Ordering::SeqCst));
    assert!(store.get(&SettingKey::StartupRegistered).unwrap().is_none());

    // Boot reconciliation
    reconcile_startup_on_boot(store.as_ref(), &mut registrar, &clock).unwrap();

    // BR-112: First-run default registers at startup
    assert!(backend.enabled.load(Ordering::SeqCst));
    let stored_marker = store.get(&SettingKey::StartupRegistered).unwrap();
    assert!(stored_marker.is_some());

    // Query status via app state command implementation
    let app_state = create_test_app_state(store, backend);
    let status = get_startup_status_impl(&app_state).unwrap();
    assert!(status.enabled);
    assert_eq!(status.state, StartupState::On);
}

#[test]
fn a_run_after_the_reviewer_disabled_it_does_not_re_register() {
    // SCN-02 Run 3: The critical regression test.
    // Naive implementation ("if not registered, register") fails here.
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();
    let store = Arc::new(SqliteSettingsStore::open(&db_path).unwrap());
    let backend = Arc::new(TestAutoStartBackend::default());
    let mut registrar = DesktopStartupRegistrar::new(backend.clone());
    let clock = SystemClock::new();

    // Run 1: Boot reconciliation registers default
    reconcile_startup_on_boot(store.as_ref(), &mut registrar, &clock).unwrap();
    assert!(backend.enabled.load(Ordering::SeqCst));

    // Run 2: Reviewer disables autostart via UI command
    let app_state = create_test_app_state(store.clone(), backend.clone());
    let toggle_result = set_startup_status_impl(false, &app_state).unwrap();
    assert!(!toggle_result.enabled);
    assert_eq!(toggle_result.state, StartupState::Off);
    assert!(!backend.enabled.load(Ordering::SeqCst));

    // Run 3: Next boot / sign-in reconciliation
    let mut next_boot_registrar = DesktopStartupRegistrar::new(backend.clone());
    reconcile_startup_on_boot(store.as_ref(), &mut next_boot_registrar, &clock).unwrap();

    // MUST NOT re-register!
    assert!(
        !backend.enabled.load(Ordering::SeqCst),
        "Run 3: Snapdown MUST NOT silently re-register autostart after reviewer disabled it"
    );

    let status = get_startup_status_impl(&app_state).unwrap();
    assert!(!status.enabled);
    assert_eq!(status.state, StartupState::Off);
}

#[test]
fn a_registration_removed_outside_snapdown_shows_off_and_is_not_restored() {
    // SCN-02 Run 4: Registration was removed externally (e.g. Task Manager / Group policy).
    // Store has startup.registered = "expressed", but Windows OS registration is gone.
    let tmp = NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_path_buf();
    let store = Arc::new(SqliteSettingsStore::open(&db_path).unwrap());
    let backend = Arc::new(TestAutoStartBackend::default());
    let mut registrar = DesktopStartupRegistrar::new(backend.clone());
    let clock = SystemClock::new();

    // Run 1: Boot reconciliation registers default
    reconcile_startup_on_boot(store.as_ref(), &mut registrar, &clock).unwrap();
    assert!(backend.enabled.load(Ordering::SeqCst));

    // External removal: Task Manager disables or deletes autostart key
    backend.enabled.store(false, Ordering::SeqCst);

    // Boot reconciliation runs on next sign-in
    let mut next_boot_registrar = DesktopStartupRegistrar::new(backend.clone());
    reconcile_startup_on_boot(store.as_ref(), &mut next_boot_registrar, &clock).unwrap();

    // Must NOT silently restore registration (FR-18, BR-114)
    assert!(
        !backend.enabled.load(Ordering::SeqCst),
        "Run 4: External removal must NOT be silently restored by boot reconciliation"
    );

    // Status queries ground truth from OS and shows Off
    let app_state = create_test_app_state(store, backend);
    let status = get_startup_status_impl(&app_state).unwrap();
    assert!(!status.enabled);
    assert_eq!(status.state, StartupState::Off);
}
