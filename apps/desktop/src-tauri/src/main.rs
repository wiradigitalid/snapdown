// Desktop Tauri shell - Native entry point
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use snapdown_store::sqlite::SqliteSettingsStore;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

fn show_settings_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn check_is_first_run(app: &AppHandle) -> bool {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    if !app_data_dir.exists() {
        let _ = std::fs::create_dir_all(&app_data_dir);
    }

    let db_path = app_data_dir.join("library.db");
    match SqliteSettingsStore::open(&db_path) {
        Ok(store) => store.is_empty().unwrap_or(true),
        Err(e) => {
            eprintln!("Failed to open library.db: {e}");
            true
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_settings_window(app);
        }))
        .setup(|app| {
            let handle = app.handle().clone();

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

            // Per MF-8: First run is defined as the setting table holding zero rows.
            if check_is_first_run(&handle) {
                show_settings_window(&handle);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
