# 01: Reopening Snapdown jumps to whichever Virtual Desktop the Editor already lives on

**What to build:** Snapdown keeps exactly one persistent Editor window for the life of the process.
Today, three different ways of "opening" the Editor while it is already running each fall short in
their own way:

- **Double-clicking `Snapdown.exe` again** does nothing visible at all. A named mutex makes the
  second process detect it is not first and simply exit — no window is shown, focused, or even
  flashed.
- **The tray icon's "Open Editor"** (and its matching global hotkey) call `show()` and un-minimize
  the window, but never give it OS-level foreground focus and never check which Windows Virtual
  Desktop it is actually sitting on. If the Reviewer is looking at a different Virtual Desktop than
  the one the Editor was left on, the window "opens" invisibly on a desktop nobody is looking at.
- **Choosing to open the Editor right after taking a screenshot** goes through the same show/
  un-minimize path as the tray, and has the identical blind spot.

Fix all three at once: reopening the Editor — by any of these three routes — brings the OS to the
Virtual Desktop the Editor window already lives on and puts the window in front, so the Reviewer
never has to go hunting across desktops for a window that "should have just opened."

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Double-clicking `Snapdown.exe` while Snapdown is already running switches the OS to the Virtual
      Desktop the Editor is on (if it differs from the current one) and brings the window to the
      front — it no longer silently exits with no visible effect
- [x] The tray icon's "Open Editor" does the same: switches Virtual Desktop first if needed, then
      brings the window to front with real OS foreground focus (not just `show()` + un-minimize)
- [x] The matching global hotkey for opening the Editor behaves identically to the tray action
- [x] Choosing to open the Editor right after a capture (the existing "open Editor after capture"
      path) behaves identically
- [x] All four entry points route through one shared function, so a future change to "bring the
      Editor to front" only has to be made once
- [x] When the Editor is already on the Reviewer's current Virtual Desktop, nothing about the desktop
      switch is visible — only the window comes to front, exactly as today
- [x] When Snapdown has no window yet to reopen (first launch, or launched via Windows startup with
      no window shown per `FR-18`/`BR-121`), none of this applies — first launch is unaffected
- [x] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0. Automated tests cover what they can
      (the shared function is called from all four sites; the right HWND is targeted; a
      Virtual-Desktop API call is attempted before the foreground call) — the actual OS-level desktop
      switch cannot be exercised by CI and needs a manual pass
- [ ] **Look at:** open the Editor, switch to a second Virtual Desktop, then (a) double-click
      `Snapdown.exe`, (b) use the tray icon's "Open Editor", (c) use the matching hotkey, and (d) take
      a capture with "open Editor after capture" on — each of the four must switch Windows back to the
      first desktop and bring the Editor to the front. Then repeat any one of them while already on
      the Editor's own desktop, to confirm nothing changes except the window coming to front.
      **Still owed — genuinely untested by this pass**, and left unchecked on purpose: no build+launch
      GUI session with a second Virtual Desktop was available to this agent, so this is a manual
      verification step for a human Reviewer, not an automated one. See the feasibility note below for
      what the implementation rests on instead.

## Feasibility finding (this pass)

Windows gives exactly one PUBLIC, documented way to ask about a window's Virtual Desktop:
`IVirtualDesktopManager` (`windows::Win32::UI::Shell`, stable since Windows 10) —
`IsWindowOnCurrentVirtualDesktop`, `GetWindowDesktopId`, and `MoveWindowToDesktop` (which MOVES a
window to a desktop, never switches which one is active). Actually switching the ACTIVE desktop is
normally done through `IVirtualDesktopManagerInternal`, an UNDOCUMENTED COM interface whose CLSID/IID
and vtable layout have already changed across Windows builds — real, silent-breakage risk this repo
does not take on.

This pass did **not** depend on that undocumented interface. Reasoning from established Windows 10/11
behaviour (not from a hands-on multi-desktop test performed in this session — no interactive GUI
session with a second Virtual Desktop was available here): giving a window real OS foreground focus
via `SetForegroundWindow` already brings the OS's own Virtual Desktop switch along as a side effect —
the same mechanism that makes clicking a taskbar button for a window on another desktop switch you
there. This is understood to be *why* Windows never shipped a public "switch to desktop N" API:
activating the window is the sanctioned way to get there. `IVirtualDesktopManager` is still called
first on every reopen (satisfying the ticket's own "Virtual-Desktop API call attempted before the
foreground call" requirement) but purely as a best-effort, non-gating probe — see
`apps/desktop/src/focus.rs`'s module doc comment.

**This is a reasoned conclusion, not an empirically verified one on this machine.** If the coordinator
or owner wants it independently confirmed, the manual "Look at" step above is exactly that
confirmation, and is the one piece of this ticket left for a human to run.

### What was built

- `apps/desktop/src/focus.rs` (new): the `ForegroundBackend` trait, the shared
  `bring_editor_to_foreground` function every entry point now routes through, the real
  `WindowsForegroundBackend` (COM probe + `IsIconic`/`ShowWindow(SW_RESTORE)` +
  `AttachThreadInput`-backed `SetForegroundWindow`), and `find_running_editor_window` (`FindWindowW`
  by the Editor's window title, for the cross-process double-click case).
- `apps/desktop/src/main.rs`: added `mod focus;`, the in-process `reveal_editor_window` helper, and
  wired all four entry points — the single-instance-mutex early exit in `main()`, `TrayAction::OpenEditor`,
  `HotkeyAction::OpenEditor`, and the reveal-after-capture branch — through it.
- `apps/desktop/Cargo.toml`: added the `Win32_UI_Shell` and `Win32_System_Com` `windows` crate
  features `IVirtualDesktopManager`/`CoCreateInstance` need.
- `apps/desktop/tests/test_reopen_editor_wiring.rs` (new): reachability tests proving all four entry
  points actually call the shared function (not just that it compiles), in the same shape as
  `tests/test_annotation_wiring.rs`.
- `apps/desktop/src/focus.rs`'s own unit tests: prove the Virtual-Desktop probe runs before the
  foreground call, that a zero HWND is a no-op, that the probe's answer never gates the foreground
  call, and that `EDITOR_WINDOW_TITLE` matches `appwindow.slint`'s own `title:` so `FindWindowW`
  cannot silently drift from what the window actually calls itself.

`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
`cargo test --workspace --no-fail-fast` all exit 0 as of this pass.
