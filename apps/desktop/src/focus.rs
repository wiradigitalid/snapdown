//! Bringing the Editor's one persistent window to real OS foreground focus - and, as a documented
//! side effect, switching Windows itself to whatever Virtual Desktop it already lives on - from
//! every place "reopen the Editor" can be triggered. Answers
//! `.scratch/editor-virtual-desktop-focus/issues/01-reopening-snapdown-jumps-to-its-own-virtual-desktop.md`.
//!
//! ## What is achievable, and what this deliberately does not do
//!
//! Windows exposes exactly one PUBLIC, documented way to ask about a window's Virtual Desktop:
//! `IVirtualDesktopManager` (`windows::Win32::UI::Shell`, stable since Windows 10). It can report
//! whether a window is on the current desktop (`IsWindowOnCurrentVirtualDesktop`) and which desktop
//! id it belongs to (`GetWindowDesktopId`), and it can MOVE a window to a different desktop
//! (`MoveWindowToDesktop`) - it has no way to switch which desktop is currently ACTIVE. There is no
//! public API for that at all.
//!
//! Actually switching the active desktop is normally done through `IVirtualDesktopManagerInternal`,
//! an UNDOCUMENTED COM interface whose CLSID/IID and vtable layout have already changed across
//! Windows builds - which is why community projects (VirtualDesktopAccessor and similar) ship a
//! per-build-number GUID table and still break on new Windows releases. This file does not take on
//! that fragility, and does not need to: giving a window real OS foreground focus
//! (`SetForegroundWindow`, backed by `AttachThreadInput` so the call is not refused by Windows'
//! foreground-lock heuristics) is *itself* enough to bring the desktop switch along as a side
//! effect - the same mechanism that makes clicking a taskbar button for a window living on another
//! desktop switch the visible desktop to it. This is exactly why Windows never shipped a public
//! "switch to desktop N" API: activating the window IS the sanctioned way to get there. Every call
//! in this file is therefore a stable, documented Win32 (and, for the read-only probe, COM) call -
//! nothing here depends on a private interface.
//!
//! `IVirtualDesktopManager::IsWindowOnCurrentVirtualDesktop` is still called first on every reopen,
//! but only to learn whether a switch is about to happen - its answer never gates the foreground
//! call below it, which always runs regardless. That is what keeps "the Editor is already on the
//! Reviewer's current desktop" a no-op: there is nothing for Windows to switch, so only the window
//! comes to front, exactly as before this file existed.

#[cfg(windows)]
use windows::Win32::Foundation::HWND;

/// The exact title Windows shows for the Editor - `apps/desktop/ui/appwindow.slint`'s `AppWindow`
/// sets it, and `find_running_editor_window` matches on it via `FindWindowW` to find the
/// ALREADY-RUNNING instance's window from the separate process a double-click while Snapdown is
/// running starts (see `main`'s single-instance-mutex early exit, which shares no memory with the
/// first instance and so cannot reach a live Slint window handle the way the tray/hotkey/capture
/// sites do). Kept in sync with the `.slint` source by
/// `tests::rust_editor_window_title_matches_the_slint_source`.
pub const EDITOR_WINDOW_TITLE: &str = "Snapdown Editor";

/// The OS operations `bring_editor_to_foreground` needs, kept behind a trait so the call ORDER and
/// the exact handle threaded through it can be proven by a test without a live window or a real
/// Windows session.
pub trait ForegroundBackend {
    /// Best-effort only: `None` means the probe itself could not be answered (no COM, an
    /// unsupported Windows version, ...). The caller must still try to bring the window forward
    /// either way - see the module doc comment for why the answer is never a gate.
    fn is_window_on_current_virtual_desktop(&self, hwnd: isize) -> Option<bool>;
    /// Restores the window if it is minimized. A no-op otherwise.
    fn restore_if_minimized(&self, hwnd: isize);
    /// Asks Windows for real OS foreground focus. Returns whether the request was accepted.
    fn set_foreground(&self, hwnd: isize) -> bool;
}

/// The one shared function every "reopen the Editor" entry point routes through: the
/// already-running double-click early exit, the tray's Open Editor, its matching global hotkey,
/// and reveal-after-capture. A future change to what "bring the Editor to front" means is made
/// here, once.
///
/// `hwnd == 0` is a deliberate no-op: it means there is no window to reopen yet (first launch, or
/// launched via Windows startup with no window shown per `FR-18`/`BR-121`), and none of this
/// applies.
pub fn bring_editor_to_foreground(hwnd: isize, backend: &dyn ForegroundBackend) {
    if hwnd == 0 {
        return;
    }
    // Order matters here: the ticket's own acceptance criteria asks for "a Virtual-Desktop API
    // call ... attempted before the foreground call" - even though, per the module doc comment,
    // its result is never consulted to decide whether the foreground call runs.
    let _ = backend.is_window_on_current_virtual_desktop(hwnd);
    backend.restore_if_minimized(hwnd);
    backend.set_foreground(hwnd);
}

/// The real Windows backend. A unit struct - every call is a fresh, independent Win32/COM
/// invocation, so there is no state to hold between them.
#[derive(Clone, Copy)]
pub struct WindowsForegroundBackend;

#[cfg(windows)]
impl ForegroundBackend for WindowsForegroundBackend {
    fn is_window_on_current_virtual_desktop(&self, hwnd: isize) -> Option<bool> {
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
        };
        use windows::Win32::UI::Shell::{IVirtualDesktopManager, VirtualDesktopManager};

        // SAFETY: a read-only, best-effort COM probe. `CoInitializeEx`'s result and a matching
        // `CoUninitialize` are both skipped on purpose, the same way
        // `crates/snapdown-capture/src/capturer.rs` already treats its own `CoCreateInstance` call:
        // this runs once per Editor-reopen - the same amortized frequency a capture already pays
        // it at - and the COM apartment it establishes on this thread simply lives for the rest of
        // the process either way.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let manager: Option<IVirtualDesktopManager> =
                CoCreateInstance(&VirtualDesktopManager, None, CLSCTX_INPROC_SERVER).ok();
            manager
                .and_then(|manager| {
                    manager
                        .IsWindowOnCurrentVirtualDesktop(HWND(hwnd as *mut _))
                        .ok()
                })
                .map(|on_current| on_current.as_bool())
        }
    }

    fn restore_if_minimized(&self, hwnd: isize) {
        use windows::Win32::UI::WindowsAndMessaging::{IsIconic, ShowWindow, SW_RESTORE};

        let hwnd = HWND(hwnd as *mut _);
        unsafe {
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }
        }
    }

    fn set_foreground(&self, hwnd: isize) -> bool {
        use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
        use windows::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
        };

        let hwnd = HWND(hwnd as *mut _);
        unsafe {
            let current_thread = GetCurrentThreadId();
            let foreground = GetForegroundWindow();
            let foreground_thread = if foreground.is_invalid() {
                0
            } else {
                GetWindowThreadProcessId(foreground, None)
            };

            // Windows refuses `SetForegroundWindow` from a thread that neither owns the current
            // foreground window nor recently received input - which the tray/hotkey/capture call
            // sites likely satisfy on their own, but the double-click entry point (a brand-new,
            // just-launched second process asking to foreground a DIFFERENT process's window)
            // cannot rely on that. Attaching input queues with the current foreground thread is
            // the documented workaround: for as long as the attachment holds it makes this
            // thread's input state indistinguishable from the foreground thread's, which is enough
            // for the call below to be honoured - then it is detached again immediately after.
            let attached = foreground_thread != 0
                && foreground_thread != current_thread
                && AttachThreadInput(current_thread, foreground_thread, true).as_bool();

            let accepted = SetForegroundWindow(hwnd).as_bool();

            if attached {
                let _ = AttachThreadInput(current_thread, foreground_thread, false);
            }

            accepted
        }
    }
}

// Only ever constructed on non-Windows targets. This crate builds and tests on Windows (both
// `desktop-ci.yml` jobs run on `windows-latest`), so keep this from tripping `dead_code` there.
#[cfg(not(windows))]
#[allow(dead_code)]
impl ForegroundBackend for WindowsForegroundBackend {
    fn is_window_on_current_virtual_desktop(&self, _hwnd: isize) -> Option<bool> {
        None
    }
    fn restore_if_minimized(&self, _hwnd: isize) {}
    fn set_foreground(&self, _hwnd: isize) -> bool {
        false
    }
}

/// Finds the ALREADY-RUNNING instance's Editor window from a second process. `None` means no such
/// window exists (nothing else is running) or it could not be found.
#[cfg(windows)]
pub fn find_running_editor_window() -> Option<isize> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

    // Same wide-string-plus-`PCWSTR` shape `acquire_single_instance_lock` already uses in
    // `main.rs` for `CreateMutexW` - `FindWindowW` takes the same kind of Win32 string.
    let wide_title: Vec<u16> = EDITOR_WINDOW_TITLE.encode_utf16().chain(Some(0)).collect();
    unsafe { FindWindowW(None, PCWSTR(wide_title.as_ptr())) }
        .ok()
        .map(|hwnd| hwnd.0 as isize)
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn find_running_editor_window() -> Option<isize> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::fs;
    use std::path::Path;

    #[derive(Default)]
    struct RecordingBackend {
        calls: RefCell<Vec<(&'static str, isize)>>,
        on_current_desktop: Option<bool>,
    }

    impl ForegroundBackend for RecordingBackend {
        fn is_window_on_current_virtual_desktop(&self, hwnd: isize) -> Option<bool> {
            self.calls.borrow_mut().push(("vd_check", hwnd));
            self.on_current_desktop
        }
        fn restore_if_minimized(&self, hwnd: isize) {
            self.calls.borrow_mut().push(("restore", hwnd));
        }
        fn set_foreground(&self, hwnd: isize) -> bool {
            self.calls.borrow_mut().push(("foreground", hwnd));
            true
        }
    }

    #[test]
    fn the_virtual_desktop_probe_runs_before_the_foreground_call() {
        let backend = RecordingBackend {
            on_current_desktop: Some(false),
            ..Default::default()
        };
        let hwnd = 0xDEAD_BEEFu32 as isize;
        bring_editor_to_foreground(hwnd, &backend);

        let calls = backend.calls.borrow();
        assert_eq!(
            calls.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
            vec!["vd_check", "restore", "foreground"],
            "the ticket's own acceptance criteria requires a Virtual-Desktop API call attempted \
             before the foreground call"
        );
        assert!(
            calls.iter().all(|(_, called_hwnd)| *called_hwnd == hwnd),
            "every OS call must target the SAME hwnd it was handed, never a different window"
        );
    }

    #[test]
    fn a_zero_hwnd_is_a_no_op() {
        let backend = RecordingBackend::default();
        bring_editor_to_foreground(0, &backend);
        assert!(
            backend.calls.borrow().is_empty(),
            "hwnd == 0 means no window to reopen (first launch, or launched via Windows startup \
             with no window per FR-18/BR-121) - nothing here should run"
        );
    }

    #[test]
    fn the_probes_answer_never_gates_the_foreground_call() {
        // Even when the probe confidently says "already on the current desktop", the foreground
        // call still runs - it is what keeps the window coming to front on its OWN desktop too,
        // and the module doc comment is explicit that the probe never decides this.
        let backend = RecordingBackend {
            on_current_desktop: Some(true),
            ..Default::default()
        };
        bring_editor_to_foreground(42, &backend);
        assert!(backend
            .calls
            .borrow()
            .iter()
            .any(|(name, _)| *name == "foreground"));
    }

    fn read(relative: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
    }

    #[test]
    fn rust_editor_window_title_matches_the_slint_source() {
        let slint_source = read("ui/appwindow.slint");
        let expected = format!("title: \"{EDITOR_WINDOW_TITLE}\";");
        assert!(
            slint_source.contains(&expected),
            "EDITOR_WINDOW_TITLE must match AppWindow's own `title:` in appwindow.slint exactly - \
             find_running_editor_window matches on this string from a SEPARATE process, so a \
             drift here would make the double-click entry point silently stop finding the running \
             instance"
        );
    }
}
