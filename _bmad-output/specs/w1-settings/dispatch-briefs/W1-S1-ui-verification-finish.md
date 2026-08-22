UI VERIFICATION — FINISH THE JOB. Snapdown wave W1, story W1-S1.

You are in the worktree D:/Developer/orca-workspaces/snapdown/w1-settings, branch
kodesh87/w1-settings. You are running ALONE — one desktop has one keyboard focus and one
accessibility tree, so do not fan this out to subagents that also drive the UI.

WHAT HAPPENED
A previous worker ran this verification and produced eight artifacts in
`_bmad-output/specs/w1-settings/ui-verify-W1-S1/`, then its session ended **without writing the
report**. It cleaned up correctly — the Vite dev server is stopped and no `desktop` process is left —
but the deliverable is missing. If you want its context back, that session is resumable with
`opencode -s ses_fd55e5854ffevd6yskOK1UdWqe`; your call, and not required.

The coordinator **cannot** read those screenshots or that accessibility tree itself — a UI snapshot
returns a whole tree into an orchestrator's context and exhausts it. That is the whole reason this
work is dispatched. So the report is not paperwork; it is the only way the evidence reaches anyone.

WHAT EXISTS ALREADY
```
claim1_tray_overflow_icons.png
claim2_tray_menu.png
claim4_6_uia_tree.txt
claim4_window_rendered.png
claim4_window_rendered_live.png
claim5_single_instance_log.txt
claim6_tab_focused_edit.png
claim6_vault_path_typed.png
```

Two claims already look settled by evidence the coordinator could read as text, and you should
confirm rather than assume:
- **Claim 5, single instance** — `claim5_single_instance_log.txt` shows the second launch exited 0,
  the process count stayed at 1, and the original PID was unchanged.
- **Claim 4, the window renders** — `claim4_6_uia_tree.txt` carries the "Snapdown Settings" heading
  and a `text-field-input` with `IsKeyboardFocusable='True'`.

WHAT YOU OWE

1. **Write `report.md`** in that folder: one section per claim 1 to 7, each with **verified** /
   **failed** / **could not check**, what was actually observed, which artifact backs it, and for a
   failure what was seen instead plus the file and line you think is responsible. An unsupported
   "verified" is worse than an honest "could not check" — the coordinator judges from artifacts, never
   from prose, so a claim with no artifact behind it MUST be marked "could not check".

2. **Write `commands.md`** in the same folder: every command run, in order, with its real output. If
   you cannot reconstruct the previous worker's commands, say so plainly and record only your own.

3. **Cover the two claims that have no artifact yet**, by actually running the app:
   - **Claim 3 — left-clicking the tray icon shows the window.** `main.rs` sets
     `show_menu_on_left_click(false)` and installs an explicit left-click handler, so a left click
     should open the window rather than the menu. Capture evidence.
   - **Claim 7 — button states.** Hover and active on the primary button produce a visible change.
     `web/ui/src/styles/components.css` carries `.btn-primary:hover` and `:active`. Capture before and
     after.
   Name any new artifact so it is obvious which claim it belongs to.

4. **Re-check claim 1 if your evidence is thin.** `claim1_tray_overflow_icons.png` is named for the
   tray overflow, and the claim is that the app starts to the **tray and not to a window**. Note that
   this build opens Settings on every launch on purpose — a known accepted deviation recorded at
   `apps/desktop/src-tauri/src/main.rs`, closing in W1-S2 — so "a window appeared at startup" is
   **not** a failure of claim 1. What claim 1 actually asserts is that a tray icon exists and that the
   tray is how the app lives after that window is closed. Judge it that way and say so.

HOW TO RUN IT
`cargo tauri dev` from `apps/desktop`, or build and run `target/debug/desktop.exe` with the Vite dev
server up. Say which you did.

RULES THAT BIND YOU
- Change NOTHING outside `_bmad-output/specs/w1-settings/ui-verify-W1-S1/`. This is verification, not
  repair. Do not edit application code. `.what/`, `.how/`, `.control/`, `.constitution/` are
  read-only. Do not commit and do not push.
- If you find a defect, report it — that is the point. Do not fix it.
- Kill every process you start. Leave no Snapdown instance and no Vite server running, and say in
  `commands.md` how you confirmed that.
- **Write the two files before you end your session.** The previous run's whole cost was that it did
  the work and left no record.

WHEN DONE
Report with:
  orca orchestration send --type worker_done --subject "<n verified, m failed, k unchecked>"
  --body "<per claim: verdict and what you saw>" --task-id <task_id> --dispatch-id <dispatch_id>
  --outcome succeeded --files-modified "<paths>" --json
Orca may reject that message — its readiness probe does not match this OpenCode build. If it does,
say so and rely on the two files; the coordinator reads artifacts.
