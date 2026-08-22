---
id: SPEC-w1-settings
companions:
  - .control/registry/index.yaml
  - .control/registry/components.yaml
  - .control/product-glossary.md
  - .what/settings/SRS-settings.md
  - .what/settings/03-domain/domain-model.md
  - .what/business-rules.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/c4-l2-containers.md
  - .how/_platform/c4-l3-desktop-app.md
  - .how/_platform/inventory-db.md
  - .how/_platform/inventory-screen.md
  - .how/_platform/cross-cutting.md
  - .how/_platform/design-system.md
  - .how/settings/SDD-settings.md
  - .control/decisions/DEC-001-stack.md
sources:
  - .what/_prd/capture-to-markdown/prd.md
  - .control/registry/requirements.yaml
  - .control/registry/usecases.yaml
  - .control/registry/waves.yaml
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete,
> preservation-validated contract for what to build, test, and validate. Source documents listed in
> frontmatter are for traceability — consult them only if you need narrative rationale or prose
> colour this contract intentionally omits.

# W1 — Workspace, the two stores, and Settings complete

## Why

**A vision to realize, and nothing is standing yet.** Snapdown does not exist as code. This wave is
the first one, so it carries two things at once: the workspace every later wave builds inside, and the
one Product Component that every other component reads from.

`settings` was chosen as the first component deliberately. Four of the five components read a Setting
before they can do anything — `finding` needs the Vault location and the Quality Budget, `bundle`
needs the Vault location, `sharing` needs the web service address — so building it first means no
later wave has to stub it. It is also the component at `mode: catalog`, which makes it the cheapest
place to get the workspace shape wrong and fix it.

Who is affected: the Reviewer, on their first run. Nothing in this wave is visible to an agent.

## Capabilities

- **CAP-6** — Keep the tool out of the way: folder, hotkeys, startup
  - **intent:** The Reviewer can choose where image files are kept, how much picture quality a
    screenshot is worth, which key combinations set Snapdown off, and whether Snapdown is running
    when they sign in — and set all four once, on first run.
  - **success:** On a clean Windows 11 machine, a fresh install opens Settings, accepts a Vault
    folder, refuses a hotkey combination that another process already holds while naming the
    conflict, accepts a different one that then works without a restart, and turns on run-at-sign-in
    without an administrator prompt. After signing out and back in, Snapdown is in the tray with its
    hotkeys registered and no window open.

This wave also lays the workspace and the two stores. Neither is a capability — nothing a Reviewer
can do — which is why stories W1-S1 and W1-S2 satisfy no use case. They are the substrate CAP-6 needs,
and their success criterion is that the tests named in `waves.yaml` pass.

## Constraints

- **The stack is fixed by `DEC-001` and MUST NOT be varied.** Rust 1.96, Tauri v2, React 19 + Vite 7
  + TypeScript 5, Go 1.25 with `chi`, embedded SQLite. **Next.js and Express are excluded**; a
  dependency on either is a violation, not a shortcut. Nothing in this wave needs Go — do not create
  `web/`.
- **The tree shape in `ARCHITECTURE-SPINE.md` § Structural Seed is a seed, not a rule.** Follow it
  unless something on disk makes it wrong, and say so in the completion report if you depart from it.
- **`snapdown-core` performs no I/O and makes no OS call.** Not filesystem, not network, not clock,
  not `std::env`. The named test `cargo::snapdown_core_has_no_io_dependency` is what holds this, and
  it MUST be a real check of the dependency graph, not an assertion of a literal.
- **AD-6 — nothing leaves the machine.** No component may open an outbound network connection. In
  this wave that means: no telemetry, no update check, no crash reporter, no analytics, and no
  dependency that phones home. There is no network code in this wave at all.
- **AD-2 — a record and its files live or die together.** It reaches this wave through BR-29: moving
  the Vault moves every file or none.
- **Secrets are not Settings.** `cross-cutting.md` § Secrets governs. No Setting holds a secret, and
  nothing in this wave writes to the Windows credential store.
- **Every identifier, file name, database table, and config key is English.** Prose inside code is
  not governed. See `.constitution/method/language-guide.md`.
- **The corpus is not yours to change.** `.what/`, `.how/`, `.control/`, and `.constitution/` are
  read-only for this wave. A deviation from the SDD, an `AD-N`, or a `BR-` is **reported in the
  completion report**, never absorbed as a code patch and never fixed by editing the document.
- **This repository is public.** No captured screenshot, no token, no client name, and no fixture
  derived from real capture output may be committed. Test fixtures are synthetic.
- **File names MUST survive every OS.** No `\ / : * ? " < > |`, no trailing space or dot. See
  `.constitution/method/structure-guide.md`.

## Non-goals

- **Capturing anything.** No overlay, no screen grab, no image reduction. `LC-009 hotkey-registrar`
  registers the Capture hotkey and raises a capture-requested event; **nothing listens to it yet**,
  and that is the correct end state for this wave. Do not build a placeholder capture.
- **The Editor.** No Finding list, no Bundle list, no Marker canvas. Screen 12 (`/settings`) is the
  only screen in this wave. Screen 13 (`/settings/agent-access`) belongs to W4.
- **The `finding`, `bundle`, `agent-access`, and `sharing` components.** Their tables are in
  `inventory-db.md` and are **not** created in this wave — only `setting` and `schema_version`.
- **The Go web service and the browser reader.** W5.
- **The MCP bridge.** W4. Do not create `crates/snapdown-mcp`.
- **An installer, code signing, or auto-update.** A `cargo tauri dev` run and a debug build are
  enough. OQ-15 is unresolved and is go-live only.
- **Any appearance option** — theme, density, layout. Not a Setting, because leaving it fixed breaks
  nothing.
- **Import or export of Settings.**
- **A second Vault, or switching between Vaults.** One at a time (OQ-11).

## Success signal

On a clean Windows 11 machine, the Reviewer installs Snapdown, sees Settings open on first run,
chooses a folder on a different drive, sets the Quality Budget, binds a Capture hotkey — being told
plainly when their first choice is already taken — and turns on run-at-sign-in. They sign out, sign
back in, and Snapdown is in the tray with its hotkeys registered, no window open, in under three
seconds, with no administrator prompt at any point.

Nothing captures yet, and pressing the Capture hotkey does nothing visible. That is the wave's
honest end state.

## Story order and what each one owes

Execution is top to bottom; every story depends on the one before it. All five touch
`settings-store`, so they MUST NOT be parallelised — V11 in `validate.py` is what says so.

### W1-S1 — Cargo workspace, Tauri v2 shell, React webview, tray, and CI

Build the substrate. A Cargo workspace with `crates/snapdown-core` and `crates/snapdown-store` as
library crates, and `apps/desktop/src-tauri` as the Tauri v2 binary. `apps/desktop/src` is a React 19
+ Vite 7 + TypeScript 5 front end.

- `snapdown-core` holds the domain entities named in `.what/settings/03-domain/domain-model.md` for
  this wave — `Setting` and nothing else yet — plus the port traits the spine's Design Paradigm names.
  It has **no** I/O dependency. Add `snapdown-store` and the Tauri app as its only consumers.
- The Tauri app starts to a **tray icon**, not a window. Single-instance: a second launch focuses the
  existing one rather than starting a second process. The tray menu opens Settings and quits.
- On first run only, Settings opens. Afterwards, the tray icon is the only trace until the Reviewer
  opens something.
- `web/ui/src/styles/tokens.css` and the base element components in
  `.how/_platform/design-system.md` are needed by this wave's one screen. **Create them under
  `web/ui/` as that file specifies**, and import them from `apps/desktop/src`. Do not create
  `web/api/` — this wave needs no Go.
- CI: a GitHub Actions workflow set. `korpus.yml` runs
  `uv run .constitution/method/scripts/validate.py --check` and is expected to be **red** on V24 and
  V25 until this wave's code lands — after it lands, the four V25 findings for the containers with
  code MUST be gone. A second workflow builds and tests the Rust workspace and the front end on
  `windows-latest`.
- Also refresh `.control/structure-codebase.md`? **No.** That is the coordinator's, at wave close.
  Report the tree you created instead.

Tests: `cargo::workspace_builds`, `cargo::snapdown_core_has_no_io_dependency`,
`vitest::app_renders_shell`, `ci::korpus_workflow_runs_validate`.

### W1-S2 — `library.db` with migrations, the `setting` table, and the Vault blob adapter

`crates/snapdown-store` holds two adapters, both behind ports declared in `snapdown-core`.

- **`LC-025 settings-store`.** SQLite via `rusqlite`, `journal_mode=WAL`. Create only two tables this
  wave: `setting` and `schema_version`, exactly as `inventory-db.md` rows 8 and 9 specify. Migrations
  are forward-only, numbered, idempotent, and recorded in `schema_version`.
- A **corrupt or unreadable `library.db` MUST NOT be replaced by a fresh empty one.** It refuses to
  open, says so, and leaves the file alone. This is stated in `SDD-finding.md` § Failure Behaviour and
  it binds here because this story writes the opener.
- **Setting reads fall back to the shipped default** when no value is stored — BR-28, and it is what
  makes capture work before configuration in W2. An out-of-range stored value is rejected on read,
  the default is used for that run, and which Setting was rejected is surfaced. Never log the value:
  a Vault path can carry a person's name (`cross-cutting.md` § Logging).
- **`LC-005 vault-blobs`.** Create, read, delete, and stat a blob by path relative to the Vault root.
  It **resolves** the path and refuses anything that escapes the root — resolve, do not string-match.
  This is the single place that check lives for the whole product; W4 and W5 both rely on it.
  Nothing in this wave writes a blob, and the adapter is still fully tested.
- Ids are UUIDv7, lowercase hyphenated, generated by one helper in `snapdown-core`. Timestamps are
  RFC 3339 UTC with an explicit `Z`. `cross-cutting.md` § Identifiers and § Timestamps.

Tests: `cargo::migrations_apply_to_an_empty_database`, `cargo::migrations_are_idempotent`,
`cargo::setting_read_falls_back_to_its_shipped_default`,
`cargo::vault_refuses_a_path_that_escapes_its_root`,
`cargo::corrupt_library_refuses_to_open_and_does_not_recreate`.

### W1-S3 — Settings screen: the Vault folder and the Quality Budget

Screen 12, route `/settings`, per `inventory-screen.md`. One screen with sections, not four screens.

- **Vault folder** (FR-16, UC-14). A default location applies until the Reviewer picks one, so
  everything works before setup (BR-28). A folder that cannot be written to is refused **at the
  moment of choosing**, not at the next use. Changing it offers to move existing files and **moves
  every file or none** (BR-29). Opening the current folder in Explorer is one action.
- **Quality Budget** (FR-5, UC-13). Two values: maximum long edge in pixels, and encoder quality.
  Shipped defaults are **1600 px** and **quality 75** — write them as named constants with a comment
  pointing at OQ-3, because they are a working answer and not a measured one. A value outside a sane
  range is refused at the point of entry. Settings shows the stored size of the most recent Finding so
  the effect of a change is visible; with no Findings yet it says so rather than showing zero.
- A Quality Budget change applies only to later Captures. **Nothing re-encodes a stored image, ever**
  (BR-9). There is nothing to re-encode this wave; do not build a path that could.
- Use the base elements from `design-system.md`. No literal colour, spacing, radius, or font size
  anywhere — tokens only.

Tests: `cargo::unwritable_vault_folder_is_refused_at_choosing`,
`cargo::changing_the_vault_moves_every_file_or_none`,
`cargo::quality_budget_outside_range_is_refused_on_entry`,
`vitest::settings_shows_the_stored_size_of_the_latest_finding`.

### W1-S4 — Hotkey binding, OS registration, and honest conflict reporting

`LC-009 hotkey-registrar`, in `settings` because this component owns the binding it registers.

- Two actions are bindable: **Capture** and **Open Editor**. Both are listed with their current
  combination. Shipped defaults: pick sensible ones and say in the report which you chose and why.
- **A combination another process already holds is refused at binding time, naming the conflict**
  (BR-26). If Windows cannot be made to report that reliably, **do not fake it** — report the
  limitation in the completion report and escalate; FR-17's promise would then need renegotiating,
  and that is the owner's call, not a code decision.
- Two Snapdown actions cannot share one combination (BR-27).
- A hotkey can be **cleared**, which disables that action rather than leaving a broken binding.
- **Rebinding takes effect without restarting Snapdown**: the old combination stops working and the
  new one starts.
- A registration that fails **at startup** is reported — a tray badge plus one line in Settings
  naming which hotkey is not active. Never swallowed (BR-26).
- The registrar raises a **capture-requested** event when the Capture hotkey fires. **Nothing
  subscribes to it in this wave.** Wire the event, leave it unconsumed, and do not add a placeholder
  capture.
- Registration MUST succeed **without administrator rights** (NFR-7, OQ-5). If it turns out it cannot,
  that is the single most important thing in this wave's report.

Tests: `cargo::a_combination_held_elsewhere_is_refused_at_binding`,
`cargo::two_actions_cannot_share_one_combination`, `cargo::a_cleared_hotkey_disables_its_action`,
`cargo::rebinding_takes_effect_without_a_restart`,
`cargo::a_failed_startup_registration_is_reported_not_swallowed`.

### W1-S5 — Run at Windows startup, reflecting the real registration

`LC-026 startup-registrar`. Use `tauri-plugin-autostart` unless something makes it wrong.

- Enabling it needs **no administrator rights** (NFR-7).
- Starting this way opens **no window** — the tray icon only.
- **The setting reflects the actual OS registration, not a remembered intention** (FR-18). Read it
  back from the OS every time the screen opens. Do not cache it.
- Disabling it **removes** the registration rather than leaving it and ignoring it.

Tests: `cargo::startup_registration_needs_no_administrator_rights`,
`cargo::the_setting_is_read_back_from_the_os_not_remembered`,
`cargo::disabling_removes_the_registration`.

## Verification — run it, do not assume it

`.constitution/project/codebase-stack-guide.md` is still empty; it is filled from this wave at close.
Until then these are the commands, and every one of them MUST actually be run before a story is
reported done:

```bash
# from the repository root
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# from apps/desktop
npm run typecheck
npm run lint
npm run test

# from the repository root — the corpus, not the code
uv run .constitution/method/scripts/validate.py --check
```

A green `validate.py` is **not** evidence that the code compiles; the two answer different questions.
Report both.

`validate.py` is expected to stay red on V24 for `web/ui/` files not yet created and on V25 for
containers with no code yet. After this wave, `desktop-app` MUST no longer appear in V25; `mcp-bridge`
and `web-api` still will, correctly.

## Debugging

When a test or a build fails and you do not know why, run `wdi-systematic-debugging` **before**
proposing a fix. A third failed fix attempt is the signal to escalate, not to try a fourth. Do not
change a test, an assertion, or a guard to turn something green — a failing guard is a finding about
the content.

## Assumptions

- Windows global hotkeys register from a user-level process without administrator rights — **OQ-5**.
  This wave is where that is settled. If it is false, say so loudly.
- The shipped Quality Budget defaults (1600 px, quality 75) are usable without being changed —
  **OQ-3**. Unmeasured, and deliberately written as named constants so the number is easy to move.
- One Vault at a time is enough — **OQ-11**.
- `tauri-plugin-global-shortcut` and `tauri-plugin-autostart` are current and behave as documented on
  Windows 11. Neither has been verified on this machine.

## Open Questions

- **Which combinations should the shipped hotkeys be?** Choose sensible ones and state the choice in
  the completion report; the owner may change it, and it is one constant either way.
- **Can a hotkey conflict be detected reliably at binding time on Windows 11?** BR-26 assumes yes. If
  not, FR-17's promise weakens to "reported at the next registration attempt", which is the owner's
  decision and not a code workaround.
- **Does moving the Vault all-or-nothing survive a file held open by another process?** BR-29 says all
  or none. If the move cannot be made atomic, **refusing the move** is the acceptable answer and the
  safer one; report which you implemented.
