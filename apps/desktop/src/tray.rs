use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

/// Action requested through the tray icon or its menu, matching the tray app's four
/// entries: Capture, Open Editor, Settings, Exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Capture,
    OpenEditor,
    Settings,
    Quit,
}

/// Owns the tray icon and its menu for the lifetime of the app. Must be constructed on the
/// same thread that runs the win32 event loop (Slint's `AppWindow::run()` thread), per
/// tray-icon's platform notes.
pub struct AppTray {
    _tray: TrayIcon,
    capture_id: MenuId,
    open_editor_id: MenuId,
    settings_id: MenuId,
    quit_id: MenuId,
}

impl AppTray {
    pub fn new(icon_rgba: Vec<u8>, icon_width: u32, icon_height: u32) -> Result<Self, String> {
        let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height)
            .map_err(|e| format!("Failed to build tray icon: {e}"))?;

        let capture_item = MenuItem::new("Capture Region", true, None);
        let open_editor_item = MenuItem::new("Open Editor", true, None);
        let settings_item = MenuItem::new("Settings", true, None);
        let quit_item = MenuItem::new("Exit", true, None);

        let capture_id = capture_item.id().clone();
        let open_editor_id = open_editor_item.id().clone();
        let settings_id = settings_item.id().clone();
        let quit_id = quit_item.id().clone();

        let menu = Menu::new();
        menu.append(&capture_item)
            .map_err(|e| format!("Failed to build tray menu: {e}"))?;
        menu.append(&open_editor_item)
            .map_err(|e| format!("Failed to build tray menu: {e}"))?;
        menu.append(&settings_item)
            .map_err(|e| format!("Failed to build tray menu: {e}"))?;
        menu.append(&PredefinedMenuItem::separator())
            .map_err(|e| format!("Failed to build tray menu: {e}"))?;
        menu.append(&quit_item)
            .map_err(|e| format!("Failed to build tray menu: {e}"))?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Snapdown")
            .with_icon(icon)
            .with_menu_on_left_click(false)
            .build()
            .map_err(|e| format!("Failed to build tray icon: {e}"))?;

        Ok(Self {
            _tray: tray,
            capture_id,
            open_editor_id,
            settings_id,
            quit_id,
        })
    }

    /// Non-blocking poll for the next tray action, if any. Meant to be called from a
    /// short-interval `slint::Timer` on the UI thread.
    pub fn poll_action(&self) -> Option<TrayAction> {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.capture_id {
                return Some(TrayAction::Capture);
            }
            if event.id == self.open_editor_id {
                return Some(TrayAction::OpenEditor);
            }
            if event.id == self.settings_id {
                return Some(TrayAction::Settings);
            }
            if event.id == self.quit_id {
                return Some(TrayAction::Quit);
            }
        }

        if let Ok(TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }) = TrayIconEvent::receiver().try_recv()
        {
            return Some(TrayAction::OpenEditor);
        }

        None
    }
}
