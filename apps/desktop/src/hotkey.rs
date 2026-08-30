use global_hotkey::hotkey::HotKey;
use global_hotkey::GlobalHotKeyManager;
use snapdown_core::domain::setting::{HotkeyAction, Setting, SettingValue};
use snapdown_core::error::CoreError;
use snapdown_core::ports::{Clock, HotkeyRegistrar, SettingsStore};
use snapdown_store::sqlite::SqliteSettingsStore;
use snapdown_store::system::SystemClock;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

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
    // Separate from `bindings` on purpose: switching a hotkey off must not erase the
    // combination it holds, or turning it back on would come back to "Not set" instead of the
    // Reviewer's own choice. Defaults to enabled - an action with no row in the store yet has
    // never been disabled.
    enabled: HashMap<HotkeyAction, bool>,
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
            enabled: HashMap::new(),
        }
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

            let enabled = match self.settings_store.get(&action.enabled_setting_key())? {
                Some(Setting {
                    value: SettingValue::Boolean(b),
                    ..
                }) => b,
                // No row yet: never disabled.
                _ => true,
            };
            self.enabled.insert(action, enabled);

            if shortcut.trim().is_empty() {
                // Nothing assigned yet.
                self.bindings.insert(action, String::new());
                continue;
            }

            self.bindings.insert(action, shortcut.clone());

            // A disabled action keeps its shortcut recorded but is never handed to the OS -
            // that is the entire point of `enabled` existing separately from the shortcut.
            if !enabled {
                continue;
            }

            // Attempt OS registration
            if let Some(backend) = &self.backend {
                if let Err(err_msg) = backend.register_shortcut(&shortcut) {
                    self.startup_failures.insert(action, err_msg);
                }
            }
        }

        Ok(())
    }

    pub fn is_enabled(&self, action: HotkeyAction) -> bool {
        self.enabled.get(&action).copied().unwrap_or(true)
    }

    /// Marks an action enabled after `validate_and_rebind` has already talked to the OS itself -
    /// picking a fresh combination is how a Reviewer says they want it active, and coming back
    /// from disabled with the row still reading "off" would be its own confusing state. Does not
    /// touch the backend again: the caller already registered (or re-registered) the shortcut.
    fn ensure_enabled_persisted(&mut self, action: HotkeyAction) -> Result<(), CoreError> {
        if self.is_enabled(action) {
            return Ok(());
        }
        let clock = SystemClock::new();
        self.settings_store.set(&Setting::new(
            action.enabled_setting_key(),
            SettingValue::Boolean(true),
            clock.now_rfc3339(),
        ))?;
        self.enabled.insert(action, true);
        Ok(())
    }

    /// Flips the enabled toggle without touching the assigned shortcut - `wira-desk`'s own
    /// distinction between "this row is off" and "this row was never bound".
    pub fn set_enabled(&mut self, action: HotkeyAction, enabled: bool) -> Result<(), CoreError> {
        if enabled == self.is_enabled(action) {
            return Ok(());
        }

        let shortcut = self.bindings.get(&action).cloned().unwrap_or_default();

        if enabled {
            if !shortcut.trim().is_empty() {
                if let Some(backend) = &self.backend {
                    backend.register_shortcut(&shortcut).map_err(|_e| {
                        CoreError::Validation(
                            "This combination is already held by Windows or another application"
                                .to_string(),
                        )
                    })?;
                }
                self.startup_failures.remove(&action);
            }
        } else if !shortcut.trim().is_empty() {
            if let Some(backend) = &self.backend {
                let _ = backend.unregister_shortcut(&shortcut);
            }
            self.startup_failures.remove(&action);
        }

        let clock = SystemClock::new();
        self.settings_store.set(&Setting::new(
            action.enabled_setting_key(),
            SettingValue::Boolean(enabled),
            clock.now_rfc3339(),
        ))?;
        self.enabled.insert(action, enabled);

        Ok(())
    }

    pub fn get_bindings(&self) -> &HashMap<HotkeyAction, String> {
        &self.bindings
    }

    // Kept for the Shortcuts settings screen to surface registration failures once that
    // UI is wired up; not yet called from main.rs.
    #[allow(dead_code)]
    pub fn get_startup_failures(&self) -> &HashMap<HotkeyAction, String> {
        &self.startup_failures
    }

    #[allow(dead_code)]
    pub fn action_for_shortcut_str(&self, shortcut_str: &str) -> Option<HotkeyAction> {
        let parsed_event_sc = HotKey::from_str(shortcut_str).ok();

        for (action, bound_sc) in &self.bindings {
            if bound_sc.eq_ignore_ascii_case(shortcut_str) {
                return Some(*action);
            }
            if let Some(ev_sc) = parsed_event_sc {
                if let Ok(b_sc) = HotKey::from_str(bound_sc) {
                    if ev_sc == b_sc {
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

        // Validate shortcut format by parsing with HotKey::from_str
        let parsed_new = HotKey::from_str(new_shortcut).map_err(|e| {
            CoreError::Validation(format!("Invalid shortcut format '{new_shortcut}': {e}"))
        })?;

        // BR-27: Two Snapdown actions cannot share the same hotkey combination
        for (other_action, bound_sc) in &self.bindings {
            if *other_action != action {
                let collides = bound_sc.eq_ignore_ascii_case(new_shortcut)
                    || HotKey::from_str(bound_sc)
                        .is_ok_and(|other_parsed| parsed_new == other_parsed);
                if collides {
                    return Err(CoreError::Validation(format!(
                        "\"{}\" already uses this combination",
                        other_action.label()
                    )));
                }
            }
        }

        let old_shortcut = self.bindings.get(&action).cloned();

        // If shortcut didn't change and is already registered, no-op or update string
        if let Some(ref old) = old_shortcut {
            let is_same = if old.eq_ignore_ascii_case(new_shortcut) {
                true
            } else if let Ok(old_parsed) = HotKey::from_str(old) {
                parsed_new == old_parsed
            } else {
                false
            };

            if is_same {
                // A disabled action was never handed to the backend, so it carries no startup
                // failure of its own - it still needs the SAME re-registration a recovered
                // startup failure does, not the plain no-op path below.
                let needs_registration =
                    self.startup_failures.contains_key(&action) || !self.is_enabled(action);

                if needs_registration {
                    if let Some(backend) = &self.backend {
                        backend.register_shortcut(new_shortcut).map_err(|_e| {
                            CoreError::Validation(
                                "This combination is already held by Windows or another \
                                 application"
                                    .to_string(),
                            )
                        })?;
                    }
                    self.startup_failures.remove(&action);
                }
                if old != new_shortcut {
                    let clock = SystemClock::new();
                    self.settings_store.set(&Setting::new(
                        action.to_setting_key(),
                        SettingValue::String(new_shortcut.to_string()),
                        clock.now_rfc3339(),
                    ))?;
                    self.bindings.insert(action, new_shortcut.to_string());
                }
                self.ensure_enabled_persisted(action)?;
                return Ok(());
            }
        }

        // BR-26: A combination held by another application/OS is refused at binding time
        if let Some(backend) = &self.backend {
            backend.register_shortcut(new_shortcut).map_err(|_e| {
                CoreError::Validation(
                    "This combination is already held by Windows or another application"
                        .to_string(),
                )
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
        self.ensure_enabled_persisted(action)?;

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

        if !self.is_enabled(action) {
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

/// Global-hotkey-backed [`GlobalShortcutBackend`]. Must be constructed on the same thread that
/// runs the win32 event loop (see [`GlobalHotKeyManager::new`] platform notes); this holds for
/// Slint's winit backend, whose event loop runs on the thread that calls `AppWindow::run()`.
pub struct DesktopGlobalHotkeyBackend {
    manager: GlobalHotKeyManager,
}

// SAFETY: `GlobalHotKeyManager` wraps a raw HWND on Windows. Every call into it (register,
// unregister) happens from the Slint UI thread, which also owns the win32 message loop that
// backs it, so cross-thread access never actually occurs despite the raw pointer inside.
unsafe impl Send for DesktopGlobalHotkeyBackend {}
unsafe impl Sync for DesktopGlobalHotkeyBackend {}

impl DesktopGlobalHotkeyBackend {
    pub fn new() -> Result<Self, String> {
        let manager = GlobalHotKeyManager::new().map_err(|e| e.to_string())?;
        Ok(Self { manager })
    }
}

impl GlobalShortcutBackend for DesktopGlobalHotkeyBackend {
    fn register_shortcut(&self, shortcut: &str) -> Result<(), String> {
        let hotkey = HotKey::from_str(shortcut).map_err(|e| e.to_string())?;
        self.manager.register(hotkey).map_err(|e| e.to_string())
    }

    fn unregister_shortcut(&self, shortcut: &str) -> Result<(), String> {
        let hotkey = HotKey::from_str(shortcut).map_err(|e| e.to_string())?;
        self.manager.unregister(hotkey).map_err(|e| e.to_string())
    }

    fn is_registered(&self, shortcut: &str) -> bool {
        // global-hotkey does not expose a registration query; the caller tracks
        // registration state via `startup_failures` instead (see `is_registered` above).
        HotKey::from_str(shortcut).is_ok()
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
        match res.unwrap_err() {
            CoreError::Validation(msg) => {
                assert_eq!(
                    msg,
                    "This combination is already held by Windows or another application"
                );
            }
            other => panic!("Expected CoreError::Validation, got {other:?}"),
        }

        // Ensure database setting and active bindings were NOT mutated to the failing shortcut
        assert_ne!(registrar.get_shortcut("capture"), Some("Ctrl+Alt+Z".into()));
        assert!(!backend.is_registered("Ctrl+Alt+Z"));
    }

    #[test]
    fn a_snapdown_internal_conflict_is_worded_differently_from_an_os_conflict() {
        let store = Arc::new(SqliteSettingsStore::open_in_memory().unwrap());
        let backend = Arc::new(MockGlobalShortcutBackend::new().with_conflict("Ctrl+Alt+T"));
        let mut registrar = DesktopHotkeyRegistrar::new(store.clone(), Some(backend));
        registrar.init_from_store().unwrap();

        // 1. Internal conflict: trying to bind OpenEditor to the combination held by Capture
        let internal_err = registrar
            .validate_and_rebind(HotkeyAction::OpenEditor, DEFAULT_HOTKEY_CAPTURE)
            .unwrap_err();
        let internal_msg = match internal_err {
            CoreError::Validation(msg) => msg,
            other => panic!("Expected CoreError::Validation, got {other:?}"),
        };

        // 2. OS conflict: trying to bind Capture to a combination held by the OS / external app
        let os_err = registrar
            .validate_and_rebind(HotkeyAction::Capture, "Ctrl+Alt+T")
            .unwrap_err();
        let os_msg = match os_err {
            CoreError::Validation(msg) => msg,
            other => panic!("Expected CoreError::Validation, got {other:?}"),
        };

        // Names the action that already holds it, not just "another Snapdown action" - so the
        // Reviewer knows which row to go change instead of guessing.
        assert_eq!(
            internal_msg,
            "\"Capture a region\" already uses this combination"
        );
        assert_eq!(
            os_msg,
            "This combination is already held by Windows or another application"
        );
        assert_ne!(internal_msg, os_msg);
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
        let backend = Arc::new(MockGlobalShortcutBackend::new().with_conflict("Ctrl+Alt+T"));
        let mut registrar = DesktopHotkeyRegistrar::new(store.clone(), Some(backend.clone()));
        registrar.init_from_store().unwrap();

        let initial_capture = HotkeyAction::Capture.default_shortcut();
        assert!(backend.is_registered(initial_capture));

        // Attempting to rebind to taken shortcut fails
        let res = registrar.validate_and_rebind(HotkeyAction::Capture, "Ctrl+Alt+T");
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

    #[test]
    fn disabling_a_hotkey_keeps_its_shortcut_but_stops_it_registering() {
        let store = Arc::new(SqliteSettingsStore::open_in_memory().unwrap());
        let backend = Arc::new(MockGlobalShortcutBackend::new());
        let mut registrar = DesktopHotkeyRegistrar::new(store.clone(), Some(backend.clone()));
        registrar.init_from_store().unwrap();

        let shortcut = HotkeyAction::Capture.default_shortcut();
        assert!(registrar.is_registered("capture"));

        registrar.set_enabled(HotkeyAction::Capture, false).unwrap();

        // Unregistered with the OS, but the shortcut itself is still the Reviewer's own choice -
        // not erased to "Not set" the way `clear` would.
        assert!(!registrar.is_registered("capture"));
        assert!(!backend.is_registered(shortcut));
        assert_eq!(registrar.get_shortcut("capture"), Some(shortcut.into()));

        // Survives a restart: a fresh registrar reading the same store also comes back disabled.
        let mut reloaded = DesktopHotkeyRegistrar::new(store, Some(backend.clone()));
        reloaded.init_from_store().unwrap();
        assert!(!reloaded.is_enabled(HotkeyAction::Capture));
        assert!(!reloaded.is_registered("capture"));
        assert_eq!(reloaded.get_shortcut("capture"), Some(shortcut.into()));
        assert!(!backend.is_registered(shortcut));
    }

    #[test]
    fn re_enabling_a_hotkey_registers_its_kept_shortcut_again() {
        let store = Arc::new(SqliteSettingsStore::open_in_memory().unwrap());
        let backend = Arc::new(MockGlobalShortcutBackend::new());
        let mut registrar = DesktopHotkeyRegistrar::new(store, Some(backend.clone()));
        registrar.init_from_store().unwrap();

        let shortcut = HotkeyAction::Capture.default_shortcut();
        registrar.set_enabled(HotkeyAction::Capture, false).unwrap();
        assert!(!registrar.is_registered("capture"));

        registrar.set_enabled(HotkeyAction::Capture, true).unwrap();

        assert!(registrar.is_registered("capture"));
        assert!(backend.is_registered(shortcut));
        assert_eq!(registrar.get_shortcut("capture"), Some(shortcut.into()));
    }

    #[test]
    fn re_enabling_a_hotkey_can_be_refused_if_something_else_claimed_it_meanwhile() {
        let store = Arc::new(SqliteSettingsStore::open_in_memory().unwrap());
        let backend = Arc::new(MockGlobalShortcutBackend::new());
        let mut registrar = DesktopHotkeyRegistrar::new(store, Some(backend.clone()));
        registrar.init_from_store().unwrap();

        let shortcut = HotkeyAction::Capture.default_shortcut();
        registrar.set_enabled(HotkeyAction::Capture, false).unwrap();

        // Something else takes the combination while Capture sat disabled.
        backend
            .conflicts
            .lock()
            .unwrap()
            .insert(shortcut.to_string());

        let res = registrar.set_enabled(HotkeyAction::Capture, true);
        assert!(res.is_err());
        // Refused, and still disabled - not left in a half-enabled state.
        assert!(!registrar.is_enabled(HotkeyAction::Capture));
        assert!(!registrar.is_registered("capture"));
    }

    #[test]
    fn disabling_a_hotkey_does_not_affect_the_other_action() {
        let store = Arc::new(SqliteSettingsStore::open_in_memory().unwrap());
        let backend = Arc::new(MockGlobalShortcutBackend::new());
        let mut registrar = DesktopHotkeyRegistrar::new(store, Some(backend.clone()));
        registrar.init_from_store().unwrap();

        registrar.set_enabled(HotkeyAction::Capture, false).unwrap();

        assert!(!registrar.is_registered("capture"));
        assert!(registrar.is_registered("open_editor"));
        assert!(backend.is_registered(HotkeyAction::OpenEditor.default_shortcut()));
    }

    #[test]
    fn rebinding_a_disabled_action_registers_it_and_marks_it_enabled() {
        let store = Arc::new(SqliteSettingsStore::open_in_memory().unwrap());
        let backend = Arc::new(MockGlobalShortcutBackend::new());
        let mut registrar = DesktopHotkeyRegistrar::new(store, Some(backend.clone()));
        registrar.init_from_store().unwrap();

        registrar.set_enabled(HotkeyAction::Capture, false).unwrap();
        assert!(!registrar.is_registered("capture"));

        registrar
            .validate_and_rebind(HotkeyAction::Capture, "Ctrl+Alt+Q")
            .unwrap();

        assert!(registrar.is_enabled(HotkeyAction::Capture));
        assert!(registrar.is_registered("capture"));
        assert!(backend.is_registered("Ctrl+Alt+Q"));
    }
}
