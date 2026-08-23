use snapdown_core::domain::setting::{HotkeyAction, Setting, SettingValue};
use snapdown_core::error::CoreError;
use snapdown_core::ports::{Clock, HotkeyRegistrar, SettingsStore};
use snapdown_store::sqlite::SqliteSettingsStore;
use snapdown_store::system::SystemClock;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tauri_plugin_global_shortcut::Shortcut;

pub trait GlobalShortcutBackend: Send + Sync {
    fn register_shortcut(&self, shortcut: &str) -> Result<(), String>;
    fn unregister_shortcut(&self, shortcut: &str) -> Result<(), String>;
    fn is_registered(&self, shortcut: &str) -> bool;
}

pub struct DesktopHotkeyRegistrar {
    backend: Option<Arc<dyn GlobalShortcutBackend>>,
    settings_store: Arc<SqliteSettingsStore>,
    bindings: HashMap<HotkeyAction, String>,
    startup_failures: HashMap<HotkeyAction, String>,
}

impl DesktopHotkeyRegistrar {
    pub fn new(
        settings_store: Arc<SqliteSettingsStore>,
        backend: Option<Arc<dyn GlobalShortcutBackend>>,
    ) -> Self {
        Self {
            backend,
            settings_store,
            bindings: HashMap::new(),
            startup_failures: HashMap::new(),
        }
    }

    pub fn set_backend(&mut self, backend: Arc<dyn GlobalShortcutBackend>) {
        self.backend = Some(backend);
    }

    pub fn init_from_store(&mut self) -> Result<(), CoreError> {
        let actions = [HotkeyAction::Capture, HotkeyAction::OpenEditor];

        for action in actions {
            let setting_key = action.to_setting_key();
            let shortcut = match self.settings_store.get(&setting_key)? {
                Some(Setting {
                    value: SettingValue::String(s),
                    ..
                }) => s,
                _ => {
                    // Fall back to shipped default per BR-28
                    let default_shortcut = action.default_shortcut().to_string();
                    let clock = SystemClock::new();
                    let _ = self.settings_store.set(&Setting::new(
                        setting_key,
                        SettingValue::String(default_shortcut.clone()),
                        clock.now_rfc3339(),
                    ));
                    default_shortcut
                }
            };

            if shortcut.trim().is_empty() {
                // Action is disabled/cleared
                self.bindings.insert(action, String::new());
                continue;
            }

            self.bindings.insert(action, shortcut.clone());

            // Attempt OS registration
            if let Some(backend) = &self.backend {
                if let Err(err_msg) = backend.register_shortcut(&shortcut) {
                    self.startup_failures.insert(action, err_msg);
                }
            }
        }

        Ok(())
    }

    pub fn get_bindings(&self) -> &HashMap<HotkeyAction, String> {
        &self.bindings
    }

    pub fn get_startup_failures(&self) -> &HashMap<HotkeyAction, String> {
        &self.startup_failures
    }

    pub fn action_for_shortcut_str(&self, shortcut_str: &str) -> Option<HotkeyAction> {
        let parsed_event_sc = Shortcut::from_str(shortcut_str).ok();

        for (action, bound_sc) in &self.bindings {
            if bound_sc.eq_ignore_ascii_case(shortcut_str) {
                return Some(*action);
            }
            if let Some(ref ev_sc) = parsed_event_sc {
                if let Ok(b_sc) = Shortcut::from_str(bound_sc) {
                    if *ev_sc == b_sc {
                        return Some(*action);
                    }
                }
            }
        }
        None
    }

    pub fn validate_and_rebind(
        &mut self,
        action: HotkeyAction,
        new_shortcut: &str,
    ) -> Result<(), CoreError> {
        let new_shortcut = new_shortcut.trim();
        if new_shortcut.is_empty() {
            return self.clear(action);
        }

        // Validate shortcut format by parsing with Shortcut::from_str
        let parsed_new = Shortcut::from_str(new_shortcut).map_err(|e| {
            CoreError::Validation(format!("Invalid shortcut format '{new_shortcut}': {e}"))
        })?;

        // BR-27: Two Snapdown actions cannot share the same hotkey combination
        for (other_action, bound_sc) in &self.bindings {
            if *other_action != action {
                if bound_sc.eq_ignore_ascii_case(new_shortcut) {
                    return Err(CoreError::Validation(
                        "Two actions cannot share the same hotkey combination".to_string(),
                    ));
                }
                if let Ok(other_parsed) = Shortcut::from_str(bound_sc) {
                    if parsed_new == other_parsed {
                        return Err(CoreError::Validation(
                            "Two actions cannot share the same hotkey combination".to_string(),
                        ));
                    }
                }
            }
        }

        let old_shortcut = self.bindings.get(&action).cloned();

        // If shortcut didn't change and is already registered, no-op or update string
        if let Some(ref old) = old_shortcut {
            let is_same = if old.eq_ignore_ascii_case(new_shortcut) {
                true
            } else if let Ok(old_parsed) = Shortcut::from_str(old) {
                parsed_new == old_parsed
            } else {
                false
            };

            if is_same && !self.startup_failures.contains_key(&action) {
                // Update string representation in store and bindings if needed
                if old != new_shortcut {
                    let clock = SystemClock::new();
                    self.settings_store.set(&Setting::new(
                        action.to_setting_key(),
                        SettingValue::String(new_shortcut.to_string()),
                        clock.now_rfc3339(),
                    ))?;
                    self.bindings.insert(action, new_shortcut.to_string());
                }
                return Ok(());
            }

            if is_same && self.startup_failures.contains_key(&action) {
                // If it was failed on startup, try re-registering now
                if let Some(backend) = &self.backend {
                    backend.register_shortcut(new_shortcut).map_err(|e| {
                        CoreError::Validation(format!(
                            "Hotkey combination '{new_shortcut}' is already held by another application or the operating system: {e}"
                        ))
                    })?;
                }
                self.startup_failures.remove(&action);
                if old != new_shortcut {
                    let clock = SystemClock::new();
                    self.settings_store.set(&Setting::new(
                        action.to_setting_key(),
                        SettingValue::String(new_shortcut.to_string()),
                        clock.now_rfc3339(),
                    ))?;
                    self.bindings.insert(action, new_shortcut.to_string());
                }
                return Ok(());
            }
        }

        // BR-26: A combination held by another application/OS is refused at binding time
        if let Some(backend) = &self.backend {
            backend.register_shortcut(new_shortcut).map_err(|e| {
                CoreError::Validation(format!(
                    "Hotkey combination '{new_shortcut}' is already held by another application or the operating system: {e}"
                ))
            })?;

            // Unregister old shortcut if registration succeeded
            if let Some(ref old) = old_shortcut {
                let _ = backend.unregister_shortcut(old);
            }
        }

        // Persist to store and update internal state
        let clock = SystemClock::new();
        self.settings_store.set(&Setting::new(
            action.to_setting_key(),
            SettingValue::String(new_shortcut.to_string()),
            clock.now_rfc3339(),
        ))?;

        self.bindings.insert(action, new_shortcut.to_string());
        self.startup_failures.remove(&action);

        Ok(())
    }

    pub fn clear(&mut self, action: HotkeyAction) -> Result<(), CoreError> {
        if let Some(old_shortcut) = self.bindings.insert(action, String::new()) {
            if !old_shortcut.trim().is_empty() {
                if let Some(backend) = &self.backend {
                    let _ = backend.unregister_shortcut(&old_shortcut);
                }
            }
        }

        self.startup_failures.remove(&action);

        let clock = SystemClock::new();
        self.settings_store.set(&Setting::new(
            action.to_setting_key(),
            SettingValue::String(String::new()),
            clock.now_rfc3339(),
        ))?;

        Ok(())
    }
}

impl HotkeyRegistrar for DesktopHotkeyRegistrar {
    fn register(&mut self, action_str: &str, shortcut: &str) -> Result<(), CoreError> {
        let action = HotkeyAction::from_action_str(action_str)
            .ok_or_else(|| CoreError::Validation(format!("Unknown hotkey action: {action_str}")))?;
        self.validate_and_rebind(action, shortcut)
    }

    fn unregister(&mut self, action_str: &str) -> Result<(), CoreError> {
        let action = HotkeyAction::from_action_str(action_str)
            .ok_or_else(|| CoreError::Validation(format!("Unknown hotkey action: {action_str}")))?;
        self.clear(action)
    }

    fn is_registered(&self, action_str: &str) -> bool {
        let action = match HotkeyAction::from_action_str(action_str) {
            Some(a) => a,
            None => return false,
        };

        if self.startup_failures.contains_key(&action) {
            return false;
        }

        if let Some(sc) = self.bindings.get(&action) {
            if sc.is_empty() {
                return false;
            }
            if let Some(backend) = &self.backend {
                backend.is_registered(sc)
            } else {
                true
            }
        } else {
            false
        }
    }

    fn get_shortcut(&self, action_str: &str) -> Option<String> {
        let action = HotkeyAction::from_action_str(action_str)?;
        self.bindings.get(&action).cloned()
    }
}

pub struct TauriGlobalShortcutBackend {
    app_handle: tauri::AppHandle,
}

impl TauriGlobalShortcutBackend {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

impl GlobalShortcutBackend for TauriGlobalShortcutBackend {
    fn register_shortcut(&self, shortcut: &str) -> Result<(), String> {
        use tauri_plugin_global_shortcut::GlobalShortcutExt;
        self.app_handle
            .global_shortcut()
            .register(shortcut)
            .map_err(|e| e.to_string())
    }

    fn unregister_shortcut(&self, shortcut: &str) -> Result<(), String> {
        use tauri_plugin_global_shortcut::GlobalShortcutExt;
        self.app_handle
            .global_shortcut()
            .unregister(shortcut)
            .map_err(|e| e.to_string())
    }

    fn is_registered(&self, shortcut: &str) -> bool {
        use tauri_plugin_global_shortcut::GlobalShortcutExt;
        self.app_handle.global_shortcut().is_registered(shortcut)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use snapdown_core::domain::setting::{
        SettingKey, DEFAULT_HOTKEY_CAPTURE, DEFAULT_HOTKEY_OPEN_EDITOR,
    };
    use std::collections::HashSet;
    use std::sync::Mutex;

    pub struct MockGlobalShortcutBackend {
        registered: Mutex<HashSet<String>>,
        conflicts: Mutex<HashSet<String>>,
    }

    impl Default for MockGlobalShortcutBackend {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockGlobalShortcutBackend {
        pub fn new() -> Self {
            Self {
                registered: Mutex::new(HashSet::new()),
                conflicts: Mutex::new(HashSet::new()),
            }
        }

        pub fn with_conflict(self, shortcut: &str) -> Self {
            self.conflicts.lock().unwrap().insert(shortcut.to_string());
            self
        }
    }

    impl GlobalShortcutBackend for MockGlobalShortcutBackend {
        fn register_shortcut(&self, shortcut: &str) -> Result<(), String> {
            let conflicts = self.conflicts.lock().unwrap();
            if conflicts.contains(shortcut) {
                return Err("Hotkey combination is already held by another application".to_string());
            }
            self.registered.lock().unwrap().insert(shortcut.to_string());
            Ok(())
        }

        fn unregister_shortcut(&self, shortcut: &str) -> Result<(), String> {
            self.registered.lock().unwrap().remove(shortcut);
            Ok(())
        }

        fn is_registered(&self, shortcut: &str) -> bool {
            self.registered.lock().unwrap().contains(shortcut)
        }
    }

    #[test]
    fn a_combination_held_elsewhere_is_refused_at_binding() {
        let store = Arc::new(SqliteSettingsStore::open_in_memory().unwrap());
        let backend = Arc::new(MockGlobalShortcutBackend::new().with_conflict("Ctrl+Alt+Z"));
        let mut registrar = DesktopHotkeyRegistrar::new(store.clone(), Some(backend.clone()));
        registrar.init_from_store().unwrap();

        let res = registrar.validate_and_rebind(HotkeyAction::Capture, "Ctrl+Alt+Z");
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), CoreError::Validation(_)));

        // Ensure database setting and active bindings were NOT mutated to the failing shortcut
        assert_ne!(registrar.get_shortcut("capture"), Some("Ctrl+Alt+Z".into()));
        assert!(!backend.is_registered("Ctrl+Alt+Z"));
    }

    #[test]
    fn two_actions_cannot_share_one_combination() {
        let store = Arc::new(SqliteSettingsStore::open_in_memory().unwrap());
        let backend = Arc::new(MockGlobalShortcutBackend::new());
        let mut registrar = DesktopHotkeyRegistrar::new(store.clone(), Some(backend));
        registrar.init_from_store().unwrap();

        // Bind Capture to Ctrl+Shift+S
        registrar
            .validate_and_rebind(HotkeyAction::Capture, "Ctrl+Shift+S")
            .unwrap();

        // Attempting to bind OpenEditor to the exact same shortcut or equivalent
        let res = registrar.validate_and_rebind(HotkeyAction::OpenEditor, "ctrl+shift+s");
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), CoreError::Validation(_)));

        // Verify OpenEditor did not overwrite Capture
        assert_eq!(
            registrar.get_shortcut("capture"),
            Some("Ctrl+Shift+S".into())
        );
        assert_ne!(
            registrar.get_shortcut("open_editor"),
            Some("ctrl+shift+s".into())
        );
    }

    #[test]
    fn a_cleared_hotkey_disables_its_action() {
        let store = Arc::new(SqliteSettingsStore::open_in_memory().unwrap());
        let backend = Arc::new(MockGlobalShortcutBackend::new());
        let mut registrar = DesktopHotkeyRegistrar::new(store.clone(), Some(backend.clone()));
        registrar.init_from_store().unwrap();

        // Clear Capture action
        registrar.clear(HotkeyAction::Capture).unwrap();

        // Verification
        assert!(!registrar.is_registered("capture"));
        assert_eq!(registrar.get_shortcut("capture"), Some("".into()));
        assert!(!backend.is_registered(HotkeyAction::Capture.default_shortcut()));

        // Verify store has empty string persisted
        let stored = store.get(&SettingKey::HotkeyCapture).unwrap().unwrap();
        assert_eq!(stored.value, SettingValue::String("".into()));
    }

    #[test]
    fn rebinding_takes_effect_without_a_restart() {
        let store = Arc::new(SqliteSettingsStore::open_in_memory().unwrap());
        let backend = Arc::new(MockGlobalShortcutBackend::new());
        let mut registrar = DesktopHotkeyRegistrar::new(store.clone(), Some(backend.clone()));
        registrar.init_from_store().unwrap();

        let old_sc = HotkeyAction::Capture.default_shortcut();
        let new_sc = "Ctrl+Alt+C";

        registrar
            .validate_and_rebind(HotkeyAction::Capture, new_sc)
            .unwrap();

        // Old shortcut is unregistered, new shortcut is registered immediately
        assert!(!backend.is_registered(old_sc));
        assert!(backend.is_registered(new_sc));
        assert_eq!(registrar.get_shortcut("capture"), Some(new_sc.into()));
        assert!(registrar.is_registered("capture"));
    }

    #[test]
    fn a_failed_startup_registration_is_reported_not_swallowed() {
        let store = Arc::new(SqliteSettingsStore::open_in_memory().unwrap());

        // Simulate initial default shortcut conflicting with an existing system hotkey
        let backend = Arc::new(
            MockGlobalShortcutBackend::new()
                .with_conflict(HotkeyAction::Capture.default_shortcut()),
        );
        let mut registrar = DesktopHotkeyRegistrar::new(store.clone(), Some(backend));

        // init_from_store records the failure rather than returning a fatal error or swallowing it
        assert!(registrar.init_from_store().is_ok());

        let failures = registrar.get_startup_failures();
        assert!(failures.contains_key(&HotkeyAction::Capture));
        assert!(!registrar.is_registered("capture"));

        // Shortcut remains recorded in bindings for user visibility, but is marked inactive
        assert_eq!(
            registrar.get_shortcut("capture"),
            Some(HotkeyAction::Capture.default_shortcut().into())
        );
    }

    #[test]
    fn action_resolution_and_event_dispatch_matching() {
        let store = Arc::new(SqliteSettingsStore::open_in_memory().unwrap());
        let backend = Arc::new(MockGlobalShortcutBackend::new());
        let mut registrar = DesktopHotkeyRegistrar::new(store.clone(), Some(backend));
        registrar.init_from_store().unwrap();

        // Resolves Capture and OpenEditor
        assert_eq!(
            registrar.action_for_shortcut_str(DEFAULT_HOTKEY_CAPTURE),
            Some(HotkeyAction::Capture)
        );
        assert_eq!(
            registrar.action_for_shortcut_str(DEFAULT_HOTKEY_OPEN_EDITOR),
            Some(HotkeyAction::OpenEditor)
        );

        // Unknown shortcut returns None
        assert_eq!(registrar.action_for_shortcut_str("Ctrl+Alt+Shift+X"), None);

        // After clearing Capture, resolving its shortcut returns None
        registrar.clear(HotkeyAction::Capture).unwrap();
        assert_eq!(
            registrar.action_for_shortcut_str(DEFAULT_HOTKEY_CAPTURE),
            None
        );
    }

    #[test]
    fn rebinding_failure_preserves_active_old_shortcut_in_os() {
        let store = Arc::new(SqliteSettingsStore::open_in_memory().unwrap());
        let backend = Arc::new(MockGlobalShortcutBackend::new().with_conflict("Ctrl+Alt+Taken"));
        let mut registrar = DesktopHotkeyRegistrar::new(store.clone(), Some(backend.clone()));
        registrar.init_from_store().unwrap();

        let initial_capture = HotkeyAction::Capture.default_shortcut();
        assert!(backend.is_registered(initial_capture));

        // Attempting to rebind to taken shortcut fails
        let res = registrar.validate_and_rebind(HotkeyAction::Capture, "Ctrl+Alt+Taken");
        assert!(res.is_err());

        // Old shortcut remains registered in OS and active in store
        assert!(backend.is_registered(initial_capture));
        assert_eq!(
            registrar.get_shortcut("capture"),
            Some(initial_capture.into())
        );
        assert_eq!(
            registrar.action_for_shortcut_str(initial_capture),
            Some(HotkeyAction::Capture)
        );
    }
}
