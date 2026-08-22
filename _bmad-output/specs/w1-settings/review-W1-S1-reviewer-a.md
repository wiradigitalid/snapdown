# Code review — W1-S1 (reviewer A)

- **Change set reviewed:** commit `55490b9` (`feat(w1-s1): cargo workspace, tauri v2 shell, react webview, tray, and ci`).
  The worktree is **clean** — the work was already committed, so `git status --short --untracked-files=all`
  was empty and the diff of `55490b9` against `6a470fd` was reviewed instead.
- **Reviewed against:** `SPEC.md`, `W1-S1-*.md`, `ARCHITECTURE-SPINE.md` (AD-6), `design-system.md`,
  `inventory-db.md`, `DEC-001-stack.md`, `SRS-settings.md`.
- **Verdict: 11 must-fix, 14 follow-up.** The Rust side is sound and every gate the story ran is green,
  but the desktop app **cannot be built or run**, and the two guards this wave exists to install — the
  no-I/O check and the design system's focus/state contract — do not hold.

## Commands actually run (not taken from the story's report)

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo test --workspace` | 5 pass |
| `npm --prefix apps/desktop run typecheck` | clean |
| `npm --prefix apps/desktop run lint` | clean — **but lints only 6 files, none in `web/ui`** |
| `npm --prefix apps/desktop run test` | 1 pass |
| `npm --prefix apps/desktop run build` | **FAILS — `Could not resolve entry module "index.html"`** |
| `npm --prefix web/ui run typecheck` | clean when run by hand; **no gate runs it** |
| `uv run .constitution/method/scripts/validate.py --check` | RED, 12 findings (V13/V18/V24/V25) — matches the story's report |
| `cargo tree -p snapdown-core -e normal` | shows `getrandom v0.4.3` transitively |

`npm run build` was **not** in the story's verification list, which is why the missing entry point survived.

---

## MUST-FIX

### M1 — The desktop app cannot be built. `apps/desktop/index.html` does not exist

`apps/desktop/vite.config.ts` · `apps/desktop/package.json:8`

There is no `index.html` anywhere in the repository (`find . -name index.html` returns nothing), yet
`apps/desktop/src/main.tsx:6` mounts into `#root` and `tauri.conf.json:10` sets
`beforeBuildCommand: "npm run build"`.

```
> tsc && vite build
error during build: Could not resolve entry module "index.html".
```

Consequence: `vite build` fails, therefore `cargo tauri build` fails, therefore `frontendDist: "../dist"`
is never produced. `cargo tauri dev` serves `http://localhost:5173` with no document. The story's
acceptance criterion "the system tray icon is created and the Settings window is displayed" and the SPEC's
"a `cargo tauri dev` run and a debug build are enough" are both unmet. The vitest suite passes only
because jsdom supplies its own document.

**Must-fix:** breaks a story acceptance criterion; the app does not run.

### M2 — `snapdown_core_has_no_io_dependency` cannot fail on the violation that is present

`crates/snapdown-core/tests/test_no_io.rs:16-21`

The test reads `core_pkg.dependencies` — the **direct manifest** entries only — and never touches
`metadata.resolve`, the actual dependency graph. The SPEC is explicit: *"it MUST be a real check of the
dependency graph, not an assertion of a literal."*

The escape is not theoretical. `cargo tree -p snapdown-core -e normal`:

```
└── uuid v1.25.0
    └── getrandom v0.4.3     <-- OS entropy syscall, a normal transitive dependency
```

`getrandom` never appears in `snapdown-core/Cargo.toml`, so the allowlist at line 51 never sees it and the
test passes. Any future direct dependency can smuggle in `tokio`, `rusqlite`, or a network stack the same
way — the blocklist at lines 26-41 is a list of literal names, which is exactly what the SPEC forbade.

**Must-fix:** a test that cannot fail, on the one constraint this story was asked to hold.

### M3 — `snapdown-core` reads the system clock

`crates/snapdown-core/src/util/id.rs:4`

```rust
pub fn new_id() -> String { Uuid::now_v7().to_string() }
```

`Uuid::now_v7()` calls `SystemTime::now()`. The SPEC constraint reads *"`snapdown-core` performs no I/O and
makes no OS call. Not filesystem, not network, not clock, not `std::env`."* The same story's own test
excludes `chrono` with the comment *"chrono has clock/system time calls"* (`test_no_io.rs:40`) and then
commits the identical call through `uuid`.

The fix that keeps the port shape: take the timestamp (or the 48-bit unix-millis) as a parameter, the way
`Setting::updated_at` already takes a `String` (`domain/setting.rs:106`), and let the adapter in
`snapdown-store` supply the clock.

**Must-fix:** contradicts a SPEC constraint, in the crate the constraint names.

### M4 — The main window points at a route nothing serves

`apps/desktop/src-tauri/tauri.conf.json:20`

`"url": "/settings"`. There is no router in the front end — `App.tsx:5` holds `activeRoute` in a
`useState` that is never written — and there is no `/settings/index.html` in the bundle. Tauri serves
`frontendDist` as static files, so in a built app the window resolves `/settings` to nothing and shows a
blank frame or a 404. It is masked today only because M1 means no bundle exists at all.

**Must-fix:** wrong behaviour reachable from the running app.

### M5 — Every base element removes or omits its focus-visible state

`web/ui/src/components/Button.tsx:42` · `TextField.tsx:49` · `TextArea.tsx:64`

All three set `outline: 'none'` and nothing replaces it. `grep -rn "focus" web/ui/src/components` returns
**zero matches** across all ten components.

`design-system.md` § Base elements requires `focus-visible` on `Button`, `TextField`, `TextArea` and
`Checkbox`, and § Rules that bind every screen states: *"Every interactive element has a visible
`focus-visible` state — Prevents a capture loop that cannot be driven from the keyboard, which FR-2
requires."* `Button` is also required to support `hover` and `active`, likewise absent.

Note the structural cause, because it decides the fix: every component styles through the inline `style`
prop, and inline styles cannot express `:hover`, `:active`, or `:focus-visible` at all. The state matrix
`design-system.md` mandates is unreachable without a stylesheet or CSS modules alongside `tokens.css`.
This is the substrate wave — the right place to settle it.

**Must-fix:** actively weakens an accessibility guard the corpus states as a MUST, and blocks FR-2.

### M6 — `Modal` has no focus trap and no focus restore

`web/ui/src/components/Modal.tsx:83-157`

`design-system.md` requires `Modal`: *"open · closing. Focus trapped, Escape closes, focus returns to the
trigger."* Only Escape is implemented (lines 94-101). `modalRef` (line 89) is declared and assigned and
then never read — nothing autofocuses, nothing traps Tab, nothing returns focus on close. The `closing`
state is absent too.

The story's own Code Map claims *"Accessible modal container with focus trap and escape handling"* — a
claim the code does not deliver. `ConfirmDialog` (line 27) inherits the gap, and it is the element every
destructive confirmation under BR-7 will go through.

**Must-fix:** contradicts an explicit corpus MUST, and a control described as doing something it does not.

### M7 — `Markdown` is not a CommonMark renderer

`web/ui/src/components/Markdown.tsx:227-284`

`content.split('\n')` with four `startsWith` branches: `#`, `##`, `###`, and a single-line image regex.
Everything else becomes a `<p>`. No lists, emphasis, links, code blocks, blockquotes, tables, or any
multi-line construct — and no relative image resolution: `src` (line 268) is passed through raw.

`design-system.md` requires *"rendered CommonMark, read-only, with relative image resolution"*, and the
SPEC instructs *"Create them under `web/ui/` **as that file specifies**"*. The story's Code Map calls it a
*"CommonMark renderer"*. AD-9 makes this the element that renders the bytes a published Bundle serves, so
a lossy renderer here becomes a correctness problem in W3/W5, not a cosmetic one.

**Must-fix:** contradicts the SPEC's instruction and a design-system MUST; a component claiming a
capability it does not have.

### M8 — A first-run flag is persisted with no row in `inventory-db.md`

`apps/desktop/src-tauri/src/main.rs:19-30`

```rust
let flag_path = app_data_dir.join(".ran_before");
if !flag_path.exists() {
    let _ = std::fs::create_dir_all(&app_data_dir);
    let _ = std::fs::write(flag_path, b"1");
    return true;
}
```

This is persisted state. `inventory-db.md` row 8 enumerates exactly what `setting` holds — *"Vault
location, each hotkey binding, the Quality Budget pair, startup, open-editor-after-capture, the web
service address"* — and there is no first-run key; `grep -rn 'first.run|ran_before' .what/ .how/ .control/`
finds no row and no `SettingKey`. The state also sits outside `library.db`, so it escapes
`schema_version` and the store this component owns.

Secondary defect in the same block: both writes are `let _ =`, so if `create_dir_all` or `write` fails the
function still returns `true` and the Reviewer is shown first-run Settings on every launch forever, with
nothing reported.

**Must-fix:** corpus drift of the kind the brief names explicitly. Per the SPEC this is *reported*, not
absorbed — the corpus is read-only, so the owner decides whether this becomes a `setting` row or a new
inventory row.

### M9 — `web/ui` is not linted, not typechecked, and not tested by any gate

`apps/desktop/eslint.config.js` · `.github/workflows/desktop-ci.yml:43-55` · `web/ui/` (no eslint config)

`npx eslint . -f json` under `apps/desktop` reports **6 files**: `eslint.config.js`, `src/App.tsx`,
`src/main.tsx`, `src/test/setup.ts`, `src/test/shell.test.tsx`, `vite.config.ts`. The ten components in
`web/ui/src/components/` — the larger half of this story's TypeScript — are in none of them.

- `web/ui` has no eslint config of its own, and `apps/desktop`'s flat config does not reach outside its own directory.
- `apps/desktop/tsconfig.json:19` includes only `["src", "vite.config.ts"]`; `@snapdown/ui` resolves through `node_modules`, which `tsc` does not typecheck.
- `web/ui/package.json:8` *does* define `typecheck`, and `desktop-ci.yml:45` installs `web/ui` — but **no step ever runs it**.
- There is not one test for any `web/ui` component.

So M5, M6 and M7 were all reachable with every declared gate green.

**Must-fix:** a weakened guard — the verification the story reports as passing does not cover the code it
added.

### M10 — Vite 6, where the SPEC fixes Vite 7

`apps/desktop/package.json:34`

`"vite": "^6.0.0"`; installed is **6.4.3**. The SPEC constraint reads *"The stack is fixed by `DEC-001` and
MUST NOT be varied. Rust 1.96, Tauri v2, React 19 + Vite 7 + TypeScript 5"*, and the story's own
Boundaries repeat *"Vite 7"*. `^6.0.0` also cannot ever resolve to 7.

**Must-fix:** contradicts the SPEC and the story's own stated boundary.

### M11 — "Save Configuration" saves nothing and reports success

`apps/desktop/src/App.tsx:50-52, 82-87`

```tsx
<Button variant="primary" onClick={() => setShowToast(true)}>Save Configuration</Button>
...
<Toast message="Settings updated successfully" ... />
```

Nothing is read, nothing is written, and no IPC command exists — the `greet` command was removed in triage
and `src-tauri` now registers no `invoke_handler` at all. The only window this wave ships offers a primary
action that claims a persisted save and then asserts success. The "Vault Path" `TextField` (line 75) is
likewise inert, and `Active Route: {activeRoute}` (line 47) prints a hardcoded string.

The real Settings screen is W1-S3, so a placeholder shell is in scope — a placeholder that *lies* is not.
Either drop the control or label it as non-functional.

**Must-fix:** the brief's own wording — a control that claims to do something it does not do.

---

## FOLLOW-UP

| # | Where | What |
| --- | --- | --- |
| F1 | `tauri.conf.json:26` | `"csp": null` disables the webview's only egress control. No reachable path this wave (nothing loads remote content), so not must-fix — but AD-6 *binds: all* and every later wave inherits this config. **Fix before W3 renders Bundle Markdown with arbitrary `<img src>`.** |
| F2 | `Cargo.toml`, `desktop-ci.yml:18` | The SPEC fixes Rust 1.96; there is no `rust-toolchain.toml` and no `rust-version`, and CI uses `dtolnay/rust-toolchain@stable`. The pin is unenforced and CI will drift silently. |
| F3 | `desktop-ci.yml` | No `npm run build` and no `cargo tauri build` step. This is the specific gap that let M1 through; adding it is the cheapest guard in the change set. |
| F4 | `src-tauri/gen/schemas/*.json` | ~4,600 lines of Tauri-generated schema committed. Tauri's own template gitignores `gen/schemas`; `.gitignore` covers `target` and `dist` but not this. |
| F5 | `main.rs:15` | `window.emit("navigate", "/settings")` has no listener in the webview. Harmless while there is one screen; it becomes a dead tray action once there are two. |
| F6 | `TextField.tsx:75` | `currentLength` derives from `value` only, so a `showCharCount` field driven by `defaultValue` (as `App.tsx:78` does) silently counts 0. `TextArea` has no character count at all, though `design-system.md` gives it "the same" states as `TextField`. |
| F7 | `Modal.tsx:126` | No `aria-labelledby` linking the dialog to its own `<h2>`; `closing` state absent (see M6). |
| F8 | `tokens.css:16` | `--color-marker-ring` is not in `design-system.md`'s token table. The ring requirement is in that file's prose, so the token is justified — but the table is now incomplete. Reported, not patched: the corpus is read-only. |
| F9 | `Modal.tsx:112`, `MarkerBadge.tsx:198,201` | Literal `rgba(0, 0, 0, 0.5)` scrim, literal `borderRadius: '50%'`, literal `2px` ring. Against *"No spacing, colour, radius, or font size is written as a literal"*. A scrim token is the missing piece. |
| F10 | `snapdown-store/src/lib.rs:8-11` | `store_crate_initializes` asserts `id.len() == 36` — a placeholder asserting a literal, duplicating `util/id.rs`'s own test. Not one of the story's named tests, so not must-fix, but it should not be mistaken for coverage. |
| F11 | `domain/setting.rs:57` | `SettingKey::Custom(String)` opens an unbounded key space against `inventory-db.md` row 8's closed enumeration. Nothing constructs it yet. |
| F12 | Story Code Map | Cites `apps/desktop/Cargo.toml`, which does not exist (the manifest is `apps/desktop/src-tauri/Cargo.toml`). |
| F13 | `tauri.conf.json:29-31` | `bundle.active: true`, `targets: "all"` against the non-goal *"An installer, code signing, or auto-update."* Costs CI time and produces artifacts nobody wants yet. |
| F14 | both workflows | `branches: [main, master, "kodesh87/*"]` hardcodes one contributor's namespace in a public repo's CI. |

## What is clean, and worth saying

- `cargo fmt`, `cargo clippy -D warnings`, and `cargo test --workspace` all genuinely pass.
- No secret, token, credential, or non-synthetic fixture reaches a tracked file.
- No excluded dependency: no Next.js, no Express, no Go, no `crates/snapdown-mcp`, no `web/api`. Tauri v2,
  React 19, TypeScript 5 are all correct — Vite is the one variance (M10).
- No network code anywhere; AD-6's substance is respected in the code (F1 is config-level only).
- No new Logical Component was introduced, so `components.yaml` needs nothing — `LC-005`, `LC-009`,
  `LC-015`, `LC-025`, `LC-026` were all already registered.
- The port traits (`ports/mod.rs`) match the spine's Design Paradigm, and `QualityBudget`'s named constants
  with OQ-3-shaped ranges (`domain/setting.rs:5-11`) are exactly what the SPEC asked for, one story early.
- `validate.py`'s 12 findings match the story's report, and V25 on `desktop-app` is the coordinator's at
  wave close, per the SPEC — not a finding against this story.
