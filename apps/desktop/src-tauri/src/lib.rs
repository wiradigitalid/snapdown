use snapdown_core::domain::setting::HotkeyAction;
use snapdown_core::ports::Clock;
use snapdown_store::error::StoreError;
use snapdown_store::sqlite::{
    SqliteAccessKeyStore, SqliteBundleStore, SqliteFindingStore, SqlitePublicationStore,
    SqliteSettingsStore,
};
use snapdown_store::system::SystemClock;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_global_shortcut::{
    Builder as GlobalShortcutBuilder, Shortcut, ShortcutEvent, ShortcutState,
};

pub mod commands;
pub mod hotkey;
pub mod overlay;
pub mod publish;
pub mod server;
pub mod startup;
pub mod state;
pub mod vault_migration;

use commands::agent_access::{generate_access_key, get_access_key_status, revoke_access_key};
use commands::bundle::{
    copy_bundle_to_clipboard, create_bundle, delete_bundle, get_bundle_detail, list_bundles,
};
use commands::capture::{capture_screen_region, dismiss_overlay, trigger_overlay};
use commands::finding::{
    add_marker, clean_orphans, delete_finding, delete_marker, get_finding_detail, list_findings,
    save_note, scan_orphans, update_marker,
};
use commands::hotkey::{clear_hotkey, get_hotkeys, set_hotkey};
use commands::settings::{
    get_latest_finding_size, get_quality_budget_presets, get_settings, open_vault_folder,
    pick_vault_folder, set_quality_budget, set_vault_path,
};
use commands::sharing::{
    get_publication_status, publish_bundle, reconcile_publication, unpublish_bundle,
};
use commands::startup::{get_startup_status, set_startup_status};
use hotkey::{DesktopHotkeyRegistrar, TauriGlobalShortcutBackend};
use startup::{reconcile_startup_on_boot, DesktopStartupRegistrar, TauriAutoStartBackend};
use state::AppState;

#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("Database open failed at {path}: {source}")]
    DatabaseOpen {
        path: PathBuf,
        #[source]
        source: StoreError,
    },
}

pub struct StoresBundle {
    pub settings_store: SqliteSettingsStore,
    pub finding_store: SqliteFindingStore,
    pub bundle_store: SqliteBundleStore,
    pub access_key_store: SqliteAccessKeyStore,
    pub publication_store: SqlitePublicationStore,
}

pub fn init_app_stores(db_path: &Path) -> Result<StoresBundle, StartupError> {
    let settings_store =
        SqliteSettingsStore::open(db_path).map_err(|source| StartupError::DatabaseOpen {
            path: db_path.to_path_buf(),
            source,
        })?;
    let finding_store =
        SqliteFindingStore::open(db_path).map_err(|source| StartupError::DatabaseOpen {
            path: db_path.to_path_buf(),
            source,
        })?;
    let bundle_store =
        SqliteBundleStore::open(db_path).map_err(|source| StartupError::DatabaseOpen {
            path: db_path.to_path_buf(),
            source,
        })?;
    let access_key_store =
        SqliteAccessKeyStore::open(db_path).map_err(|source| StartupError::DatabaseOpen {
            path: db_path.to_path_buf(),
            source,
        })?;
    let publication_store =
        SqlitePublicationStore::open(db_path).map_err(|source| StartupError::DatabaseOpen {
            path: db_path.to_path_buf(),
            source,
        })?;

    Ok(StoresBundle {
        settings_store,
        finding_store,
        bundle_store,
        access_key_store,
        publication_store,
    })
}

pub fn format_startup_error_message(err: &StartupError) -> String {
    match err {
        StartupError::DatabaseOpen { path, source } => {
            format!(
                "Snapdown could not open its library database at {}.\n\nError: {}\n\nSnapdown will not recreate or overwrite this file to prevent data loss.",
                path.display(),
                source
            )
        }
    }
}

#[cfg(windows)]
pub fn show_native_message_dialog(title: &str, message: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;

    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxW(
            hWnd: *mut std::ffi::c_void,
            lpText: *const u16,
            lpCaption: *const u16,
            uType: u32,
        ) -> i32;
    }

    let wide_title: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0)).collect();
    let wide_msg: Vec<u16> = OsStr::new(message).encode_wide().chain(Some(0)).collect();

    const MB_OK: u32 = 0x00000000;
    const MB_ICONERROR: u32 = 0x00000010;

    unsafe {
        MessageBoxW(
            null_mut(),
            wide_msg.as_ptr(),
            wide_title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(not(windows))]
pub fn show_native_message_dialog(title: &str, message: &str) {
    eprintln!("{title}: {message}");
}

pub fn report_startup_error(err: &StartupError, app_data_dir: &Path) {
    let msg = format_startup_error_message(err);
    let log_path = app_data_dir.join("startup-error.log");
    let clock = SystemClock::new();
    let _ = std::fs::write(
        &log_path,
        format!("{}\nTimestamp: {}\n", msg, clock.now_rfc3339()),
    );
    show_native_message_dialog("Snapdown - Database Error", &msg);
}

fn show_settings_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn check_is_first_run(store: &SqliteSettingsStore) -> bool {
    store.is_empty().unwrap_or(true)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_settings_window(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(
            GlobalShortcutBuilder::new()
                .with_handler(
                    |app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent| {
                        if event.state() == ShortcutState::Pressed {
                            let shortcut_str = shortcut.into_string();
                            let action = {
                                let state = app.state::<AppState>();
                                let registrar = state.hotkey_registrar.lock().ok();
                                registrar.and_then(|r| r.action_for_shortcut_str(&shortcut_str))
                            };

                            if let Some(action) = action {
                                match action {
                                    HotkeyAction::Capture => {
                                        let _ = app.emit("capture-requested", ());
                                        let _ = trigger_overlay(app.clone());
                                    }
                                    HotkeyAction::OpenEditor => {
                                        show_settings_window(app);
                                    }
                                }
                            }
                        }
                    },
                )
                .build(),
        )
        .setup(|app| {
            let handle = app.handle().clone();

            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));

            if !app_data_dir.exists() {
                let _ = std::fs::create_dir_all(&app_data_dir);
            }

            let db_path = app_data_dir.join("library.db");
            let stores = match init_app_stores(&db_path) {
                Ok(s) => s,
                Err(err) => {
                    report_startup_error(&err, &app_data_dir);
                    return Err(Box::new(std::io::Error::other(err.to_string())));
                }
            };

            let is_first_run = check_is_first_run(&stores.settings_store);
            let arc_store = Arc::new(stores.settings_store);
            let arc_finding_store = Arc::new(stores.finding_store);
            let arc_bundle_store = Arc::new(stores.bundle_store);
            let arc_access_key_store = Arc::new(stores.access_key_store);
            let arc_publication_store = Arc::new(stores.publication_store);

            let backend = Arc::new(TauriGlobalShortcutBackend::new(handle.clone()));
            let mut registrar = DesktopHotkeyRegistrar::new(arc_store.clone(), Some(backend));
            let _ = registrar.init_from_store();

            let arc_registrar = Arc::new(Mutex::new(registrar));

            let autostart_backend = Arc::new(TauriAutoStartBackend::new(handle.clone()));
            let mut startup_registrar = DesktopStartupRegistrar::new(autostart_backend);

            // Reconcile startup on boot (BR-112, SCN-02)
            let clock = SystemClock::new();
            let _ = reconcile_startup_on_boot(arc_store.as_ref(), &mut startup_registrar, &clock);

            let arc_startup_registrar = Arc::new(Mutex::new(startup_registrar));

            app.manage(AppState {
                settings_store: arc_store,
                finding_store: arc_finding_store,
                bundle_store: arc_bundle_store,
                access_key_store: arc_access_key_store,
                publication_store: arc_publication_store,
                hotkey_registrar: arc_registrar,
                startup_registrar: arc_startup_registrar,
            });

            // Setup Tray Menu
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "settings" => {
                        show_settings_window(app);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        show_settings_window(app);
                    }
                })
                .build(app)?;

            // Per FR-18: Starting Snapdown via Windows startup opens NO window (tray icon only).
            // Check command-line arguments for "--autostart" flag.
            let is_autostart_launch = std::env::args().any(|arg| arg == "--autostart");

            // Per MF-8: First run is defined as the setting table holding zero rows.
            if is_first_run && !is_autostart_launch {
                show_settings_window(&handle);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            get_quality_budget_presets,
            set_vault_path,
            set_quality_budget,
            get_latest_finding_size,
            open_vault_folder,
            pick_vault_folder,
            get_hotkeys,
            set_hotkey,
            clear_hotkey,
            get_startup_status,
            set_startup_status,
            capture_screen_region,
            trigger_overlay,
            dismiss_overlay,
            list_findings,
            get_finding_detail,
            save_note,
            delete_finding,
            scan_orphans,
            clean_orphans,
            add_marker,
            update_marker,
            delete_marker,
            create_bundle,
            list_bundles,
            get_bundle_detail,
            delete_bundle,
            copy_bundle_to_clipboard,
            get_access_key_status,
            generate_access_key,
            revoke_access_key,
            get_publication_status,
            publish_bundle,
            unpublish_bundle,
            reconcile_publication
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
