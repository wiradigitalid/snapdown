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

**Status:** ready-for-agent

- [ ] Double-clicking `Snapdown.exe` while Snapdown is already running switches the OS to the Virtual
      Desktop the Editor is on (if it differs from the current one) and brings the window to the
      front — it no longer silently exits with no visible effect
- [ ] The tray icon's "Open Editor" does the same: switches Virtual Desktop first if needed, then
      brings the window to front with real OS foreground focus (not just `show()` + un-minimize)
- [ ] The matching global hotkey for opening the Editor behaves identically to the tray action
- [ ] Choosing to open the Editor right after a capture (the existing "open Editor after capture"
      path) behaves identically
- [ ] All four entry points route through one shared function, so a future change to "bring the
      Editor to front" only has to be made once
- [ ] When the Editor is already on the Reviewer's current Virtual Desktop, nothing about the desktop
      switch is visible — only the window comes to front, exactly as today
- [ ] When Snapdown has no window yet to reopen (first launch, or launched via Windows startup with
      no window shown per `FR-18`/`BR-121`), none of this applies — first launch is unaffected
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0. Automated tests cover what they can
      (the shared function is called from all four sites; the right HWND is targeted; a
      Virtual-Desktop API call is attempted before the foreground call) — the actual OS-level desktop
      switch cannot be exercised by CI and needs a manual pass
- [ ] **Look at:** open the Editor, switch to a second Virtual Desktop, then (a) double-click
      `Snapdown.exe`, (b) use the tray icon's "Open Editor", (c) use the matching hotkey, and (d) take
      a capture with "open Editor after capture" on — each of the four must switch Windows back to the
      first desktop and bring the Editor to the front. Then repeat any one of them while already on
      the Editor's own desktop, to confirm nothing changes except the window coming to front.
