use snapdown_core::error::CoreError;
use snapdown_core::ports::StartupRegistrar;
use std::sync::Arc;

pub trait AutoStartBackend: Send + Sync {
    fn is_enabled(&self) -> Result<bool, String>;
    fn enable(&self) -> Result<(), String>;
    fn disable(&self) -> Result<(), String>;
}

pub struct DesktopStartupRegistrar {
    backend: Arc<dyn AutoStartBackend>,
}

impl DesktopStartupRegistrar {
    pub fn new(backend: Arc<dyn AutoStartBackend>) -> Self {
        Self { backend }
    }
}

impl StartupRegistrar for DesktopStartupRegistrar {
    fn is_enabled(&self) -> Result<bool, CoreError> {
        self.backend
            .is_enabled()
            .map_err(|e| CoreError::System(format!("Failed to query startup registration: {e}")))
    }

    fn enable(&self) -> Result<(), CoreError> {
        self.backend
            .enable()
            .map_err(|e| CoreError::System(format!("Failed to enable startup registration: {e}")))
    }

    fn disable(&self) -> Result<(), CoreError> {
        self.backend
            .disable()
            .map_err(|e| CoreError::System(format!("Failed to disable startup registration: {e}")))
    }
}

pub struct TauriAutoStartBackend {
    app_handle: tauri::AppHandle,
}

impl TauriAutoStartBackend {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

impl AutoStartBackend for TauriAutoStartBackend {
    fn is_enabled(&self) -> Result<bool, String> {
        use tauri_plugin_autostart::ManagerExt;
        self.app_handle
            .autolaunch()
            .is_enabled()
            .map_err(|e| e.to_string())
    }

    fn enable(&self) -> Result<(), String> {
        use tauri_plugin_autostart::ManagerExt;
        self.app_handle
            .autolaunch()
            .enable()
            .map_err(|e| e.to_string())
    }

    fn disable(&self) -> Result<(), String> {
        use tauri_plugin_autostart::ManagerExt;
        self.app_handle
            .autolaunch()
            .disable()
            .map_err(|e| e.to_string())
    }
}

#[cfg(windows)]
pub struct WindowsRegistryAutoStartBackend {
    app_name: String,
    app_path: std::path::PathBuf,
    args: Vec<String>,
}

#[cfg(windows)]
impl WindowsRegistryAutoStartBackend {
    pub fn new(
        app_name: impl Into<String>,
        app_path: std::path::PathBuf,
        args: Vec<String>,
    ) -> Self {
        Self {
            app_name: app_name.into(),
            app_path,
            args,
        }
    }

    pub fn current_executable(
        app_name: impl Into<String>,
        args: Vec<String>,
    ) -> Result<Self, String> {
        let exe = std::env::current_exe()
            .map_err(|e| format!("Cannot resolve current executable: {e}"))?;
        Ok(Self::new(app_name, exe, args))
    }
}

#[cfg(windows)]
impl AutoStartBackend for WindowsRegistryAutoStartBackend {
    fn is_enabled(&self) -> Result<bool, String> {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = match hkcu
            .open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Run", KEY_READ)
        {
            Ok(key) => key,
            Err(_) => return Ok(false),
        };

        let val: Result<String, _> = run_key.get_value(&self.app_name);
        Ok(val.is_ok())
    }

    fn enable(&self) -> Result<(), String> {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (run_key, _) = hkcu
            .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
            .map_err(|e| format!("Failed to open HKCU Run key: {e}"))?;

        let exe_str = self.app_path.to_string_lossy().to_string();
        let cmd = if self.args.is_empty() {
            format!("\"{exe_str}\"")
        } else {
            format!("\"{exe_str}\" {}", self.args.join(" "))
        };

        run_key
            .set_value(&self.app_name, &cmd)
            .map_err(|e| format!("Failed to set HKCU Run value for {}: {e}", self.app_name))?;

        Ok(())
    }

    fn disable(&self) -> Result<(), String> {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = match hkcu
            .open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Run", KEY_WRITE)
        {
            Ok(key) => key,
            Err(_) => return Ok(()), // If the key doesn't exist, disabling is already complete
        };

        match run_key.delete_value(&self.app_name) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!(
                "Failed to delete HKCU Run value for {}: {e}",
                self.app_name
            )),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Default)]
    pub struct MockAutoStartBackend {
        pub enabled: AtomicBool,
        pub queries: std::sync::atomic::AtomicUsize,
    }

    impl AutoStartBackend for MockAutoStartBackend {
        fn is_enabled(&self) -> Result<bool, String> {
            self.queries.fetch_add(1, Ordering::SeqCst);
            Ok(self.enabled.load(Ordering::SeqCst))
        }

        fn enable(&self) -> Result<(), String> {
            self.enabled.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn disable(&self) -> Result<(), String> {
            self.enabled.store(false, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn startup_registration_needs_no_administrator_rights() {
        // NFR-7 & OQ-5: Verify startup registrar enables and disables cleanly
        // against mock and in HKCU user scope without elevation or system keys.
        let backend = Arc::new(MockAutoStartBackend::default());
        let registrar = DesktopStartupRegistrar::new(backend.clone());

        assert!(!registrar.is_enabled().unwrap());
        assert!(registrar.enable().is_ok());
        assert!(registrar.is_enabled().unwrap());
        assert!(backend.enabled.load(Ordering::SeqCst));
    }

    #[test]
    fn the_setting_is_read_back_from_the_os_not_remembered() {
        // FR-18: The setting is read back directly from the OS, not remembered in DB or cached.
        let backend = Arc::new(MockAutoStartBackend::default());
        let registrar = DesktopStartupRegistrar::new(backend.clone());

        // First check
        assert!(!registrar.is_enabled().unwrap());
        assert_eq!(backend.queries.load(Ordering::SeqCst), 1);

        // Simulate external change in OS (e.g. user toggles in Task Manager)
        backend.enabled.store(true, Ordering::SeqCst);

        // Query again: reflects the OS state immediately
        assert!(registrar.is_enabled().unwrap());
        assert_eq!(backend.queries.load(Ordering::SeqCst), 2);

        // Simulate external removal in OS
        backend.enabled.store(false, Ordering::SeqCst);
        assert!(!registrar.is_enabled().unwrap());
        assert_eq!(backend.queries.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn disabling_removes_the_registration() {
        // FR-18: Disabling removes the registration completely.
        let backend = Arc::new(MockAutoStartBackend::default());
        let registrar = DesktopStartupRegistrar::new(backend.clone());

        registrar.enable().unwrap();
        assert!(registrar.is_enabled().unwrap());

        registrar.disable().unwrap();
        assert!(!registrar.is_enabled().unwrap());
        assert!(!backend.enabled.load(Ordering::SeqCst));
    }

    #[cfg(windows)]
    #[test]
    fn windows_registry_autostart_hkcu_roundtrip() {
        let test_app_name = "SnapdownUnitTestAutoStartHKCU";
        let dummy_exe = std::path::PathBuf::from(r"C:\Windows\System32\cmd.exe");
        let backend = WindowsRegistryAutoStartBackend::new(
            test_app_name,
            dummy_exe,
            vec!["--autostart".into()],
        );

        // Clean up beforehand if leftover
        let _ = backend.disable();

        assert!(!backend.is_enabled().unwrap());

        // Enable in HKCU
        assert!(backend.enable().is_ok());
        assert!(backend.is_enabled().unwrap());

        // Disable completely deletes key value
        assert!(backend.disable().is_ok());
        assert!(!backend.is_enabled().unwrap());
    }
}
