---
type: sdd
component: settings
status: draft
created: "2026-08-22"
updated: "2026-08-23"
realizes: [UC-13, UC-14, UC-15, UC-16, UC-24, UC-25, UC-26]
binds: [AD-2, AD-6, AD-10, AD-11]
reviewed:
  date: '2026-08-23'
  sha: '7c9a6b1'
  lenses: [structure, prose, edge-case-hunter]
---

# SDD — settings

**`mode: deep`, raised from `catalog` on 2026-08-23. `risk_accepted: medium`.**

This document is an **as-built record with a design delta**, not a design. The code has been running
since W1. Every claim about what exists names the file that proves it; every claim about what does not
yet exist carries an evidence label and is dispositioned rather than left in the prose.

Until 2026-08-23 this component sat at `mode: catalog`, which **skips G4 by design**. That is why its
four choices had no flow, no state machine, no failure behaviour, and no screen specification — and
why the owner's Settings complaints had nowhere in the corpus to be answered. Raising the depth is
what makes them answerable; it does not make the earlier state a mistake.

## Decision Summary · [outline]

`settings` is the component that owns everything the Reviewer sets once and everything that frames
what they see. Four persisted choices, and — since `CAP-9` — the Editor's window frame.

**Built as** four Logical Components inside `desktop-app`, plus the shell:

- a **store** over the `setting` table, the only writer of a Setting anywhere (`BR-110`)
- two **gateways** to Windows: the autostart registrar and the hotkey registrar
- a **screen**, and now a **shell** that draws the window's two personas

The two most expensive choices, and what reversing each costs:

**Setting is one generic entity, not an entity per choice.** Adding a Setting is a row, not a schema
change. Reversing it means a migration per choice and a domain model that grows with the preference
list — the cost is paid forever, which is why this is not a close call.

**The hotkey registrar lives here, not in `finding`.** `finding` owns the capture the hotkey triggers;
this component owns the binding. The registrar raises a capture-requested event that `finding`
listens for. Reversing it would put a Setting's writer inside a component that `BR-110` says may only
read — so the reversal is not a refactor, it is a rule change.

**The delta this pass adds**, and it is not small: `LC-028 editor-shell`. `FR-27` and `FR-28` are
promises about the *frame*, and at the time this SDD was raised to `deep` the frame was inline JSX at
the top of the (pre-`DEC-007`) webview's `App.tsx` — owned by no component, tested by nothing, named
in no inventory. That is how both promises came to be unmet without any document being wrong.
`LC-028` now has an owner and, since `DEC-007`, a file: the `AppWindow` component in
`apps/desktop/ui/appwindow.slint`.

## Structure · [outline]

| LC | Type | Area | Depends on |
|---|---|---|---|
| `LC-025` `settings-store` | store | settings-store | — |
| `LC-009` `hotkey-registrar` | gateway | settings-store | `LC-025` |
| `LC-026` `startup-registrar` | gateway | settings-store | — |
| `LC-015` `settings-screen` | ui-screen | settings-store | `LC-025`, `LC-026`, `LC-009` |
| `LC-028` `editor-shell` | ui-screen | editor-ui | — |

Dependency direction is one-way into the store, and the shell depends on nothing. `LC-028` holding no
dependency is deliberate: a frame that reads state is a frame that can fail to draw, and `FR-28`
requires navigation to survive any surface's failure.

`LC-025` is the only writer. The other three read it or the operating system.

## Inherited Constraints · [guarded]

Quoted verbatim from `.how/_platform/ARCHITECTURE-SPINE.md`. A design that must deviate goes to
`wdi-decision`; it does not argue here.

**AD-2 — A record and its files live or die together**
> Any operation that creates or removes a Finding, a Bundle, or a BundleItem MUST create or remove
> that record's files in the same unit of work, and MUST leave the prior state intact if any part of
> it fails. A record MUST NOT be committed before its files exist, and files MUST NOT be removed
> before the record is.

Reaches this component through the Vault move (`UC-14`, `SCN-01`), and the code satisfies it by
**ordering** rather than by compensation: every file is copied and verified before any source is
removed, and the location Setting is written only after the move returns. No file ever exists in
neither place, which is what AD-2 actually needs. Verified against the pre-`DEC-007` webview's
`vault_migration.rs:138`; the Slint rebuild's equivalent is `migrate_vault`/`migrate_vault_dir` in
`apps/desktop/src/main.rs`.

**AD-6 — Nothing leaves the machine except a confirmed publish of a named Bundle**

Reaches this component because it holds the web service address. Holding an address is not reaching
it; nothing here initiates a network call.

**AD-10 — Colour has exactly one authority, and every colour exists in both themes**
> Every colour MUST be defined once, in the token stylesheet, and MUST be defined for both the light
> and the dark theme. A component MUST NOT contain a colour literal. A meaning background MUST be
> used only through its paired foreground token, so the pair is proven once rather than at each use.
> A token that is deliberately theme-invariant — the Marker badge, the capture overlay's scrim — MUST
> say so where it is defined and MUST still be defined in the token file.

**The shipped code violated this**, and `W6-S1` fixed it at `420ecce`, in the pre-`DEC-007` webview.
`HotkeySection.tsx` carried `#dcfce7`, `#166534`, `#f1f5f9`, `#64748b`, `#fef3c7`, `#fde047`,
`#854d0e`, `#eff6ff` — all light-theme values on a surface that renders under either theme. It used
the `--color-success-*` and `--color-neutral-*` pairs, each proven against its own background once.

Recorded in the past tense rather than removed: `AD-10` reads as a preference until you know it was
written against eight literals in one file. The Slint rebuild carries the same rule under
`design-system-guide.md`, enforced over `apps/desktop/ui/theme.slint` rather than `tokens.css`.

**AD-11 — One process owns the Library, and the Editor is a persona of it**
> Exactly one desktop process MUST own the Library. The tray, the global hotkeys, the capture
> overlay, and the Editor window MUST all live in it. A second desktop executable MUST NOT be
> produced by a build.

The build satisfies both sentences today. `target/release/` held a stale `desktop.exe` beside
`Snapdown.exe` until it was deleted by hand on 2026-08-23, and nothing prevented it recurring at the
time; `apps/desktop/tests/test_executable_identity.rs` now asserts exactly one binary named
`Snapdown` at build time, resolving `BR-121`. The product name is set in `apps/desktop/Cargo.toml`'s
`[[bin]]` table since `DEC-007` (`tauri.conf.json` before that).

## Failure Behaviour · [guarded]

The boundary list is this component's rows in the two platform inventories: `inventory-screen.md`
rows 0 and 12, and the two Windows gateways. `settings` owns no endpoint in `inventory-api.md`.

| Boundary | Other side is slow | Other side is absent | Other side is lying |
|---|---|---|---|
| **`LC-025` → `library.db`** | The screen shows every group inert and no assumed values. No timeout: a local SQLite read that is slow is a disk problem, and failing fast would hide it | Reported with the file's path, and **nothing is created over it** (`BR-118`). A store recreated beside a corrupt one is silent data loss | A store that opens and returns a schema version the code does not know is refused, named, and not migrated |
| **`LC-026` → Windows autostart** | The toggle stays in `Unknown` and stays inert. No spinner replaces it, because `Unknown` **is** the render | The toggle shows `Unreadable` with a Retry. It does not fall back to the stored value — the stored value is not the answer to this question | Registration reported as succeeding while the entry is absent surfaces at the next sign-in, not now. The product cannot detect it and does not claim to |
| **`LC-009` → Windows hotkeys** | Binding is attempted with no timeout; `RegisterHotKey` is synchronous | Refused at binding, naming the combination. The previous binding stays registered | A combination that registers and never fires is `Unregistered`'s mirror image and is **not detected**. `[MISSING]` — no health check exists, and a periodic one was rejected as a background task the product does not otherwise have |
| **`LC-015` screen → `LC-025`** | Groups render their frames; controls stay inert; layout does not shift when values land | The failing group alone shows its error and a Retry. The other four keep working — a partial failure is scoped to where it happened | Out of scope: same process, same memory |
| **`LC-028` shell → anything** | Nothing. The shell depends on nothing and always draws | Same | Same |

"Returns an error" appears nowhere above, and one row admits the product cannot detect a failure at
all. That admission is the useful part of this table.

## Integrations · [guarded]

One: **Windows**, in `03-integrations/windows-shell.md`. The owner outside the team is Microsoft, and
what happens when they change it without telling anyone is the reason the file exists.

## ABCE · [deep]

Boundary → Control → Entity → Behaviour. Not in the SRS, and not written below `deep`.

### Boundary

| Object | What crosses it |
|---|---|
| `SettingsScreen` | The Reviewer's intent, in and out |
| `EditorShell` | Navigation intent; nothing else |
| `AutoStartBackend` | A registration request and a truth question, to Windows |
| `HotkeyBackend` | A registration request and a key event, to and from Windows |
| `SettingStore` | Every read and the only writes |

`AutoStartBackend` and `HotkeyBackend` are ports with a Windows adapter each, which is what makes
`SCN-02`'s four runs testable without a real registry — `MockAutoStartBackend` existed at
`apps/desktop/src-tauri/src/startup/mod.rs` in the pre-`DEC-007` webview; the Slint rebuild's
equivalent is `apps/desktop/src/startup.rs`.

### Control

| Object | Decides |
|---|---|
| `VaultRelocation` | Validate, count, move, roll back, and only then write the location (`SCN-01`) |
| `QualityBudgetResolver` | Which named state holds, and what pair it resolves to for a given region (`BR-104`) |
| `HotkeyBinder` | Conflict against Snapdown's own actions, then against Windows, then register and unregister in one act |
| `StartupReconciler` | The `unset` / `off` / `on` distinction of `BR-112`, and the `Unknown` state |

`QualityBudgetResolver` is the one Control object with no code behind it. `[MISSING]` — `Auto` does
not exist; `crates/snapdown-core/src/domain/setting.rs` holds `DEFAULT_MAX_LONG_EDGE_PX = 1600` and
`DEFAULT_ENCODER_QUALITY = 75` as constants. Dispositioned as planned work.

### Entity

One: `Setting`. Generic by decision, per the domain model. No entity per choice.

### Behaviour

Every behaviour of this component is one of: read a Setting (many callers, no writes), write a Setting
(one caller), ask Windows a question, or tell Windows to do something. Nothing here is long-running,
nothing is scheduled, and nothing runs in the background — which is why the health check under
Failure Behaviour was rejected rather than deferred.

## Slots · [deep]

| Slot | Holds |
|---|---|
| `02-contracts/` | The command surface this component exposes to its Slint UI |
| `04-components/` | One file per `LC` above `LC-025` |
| `05-model/data-model.md` | The `setting` table and its dictionary |
| `06-flows/` | The Vault relocation and the startup reconciliation |

## Evidence labels outstanding

| Label | Claim | Disposition |
|---|---|---|
| ~~`[MISSING]`~~ **resolved** | No lint rule or contrast assertion enforced `AD-10`; eight literals in `HotkeySection.tsx` alone | **Done — `W6-S1` at `420ecce`, in the pre-`DEC-007` webview.** The Slint rebuild carries its own lint rule and contrast assertion over `apps/desktop/ui/theme.slint`, per `design-system-guide.md` |
| ~~`[MISSING]`~~ **resolved** | No build assertion prevents a second desktop executable (`BR-121`) | **Done.** `apps/desktop/tests/test_executable_identity.rs` asserts exactly one binary named `Snapdown` |
| `[MISSING]` | `Auto` does not exist; the budget is two constants | Planned work — `FR-5`, `DEC-004` |
| `[MISSING]` | The startup control has no `Unknown` state | Still true in the Slint rebuild: `apps/desktop/ui/components/settings.slint` declares `run-at-startup` as a plain `bool`, the same shape `App.tsx`'s `useState<boolean>(true)` had. Planned work — `FR-18`, `BR-108` |
| `[MISSING]` | No registration health check; a hotkey that registers and never fires is undetected | **Not planned.** Rejected: it needs a background task this product does not otherwise have |
| `[MISSING]` | The Vault move swallows both `fs::remove_file` results (`vault_migration.rs:141`, `:180`). A source file that will not delete leaves an **unreported duplicate** of an image that may hold personal data, and the move still reports success | Planned work — the highest-value item in this table |

No `[MISSING]` above was deleted. Each sentence is the only surviving evidence that somebody once
believed the thing existed.

## Open Items

- `OQ-3` — is `Auto`'s output legible at its smallest? Restated by `DEC-004`, still unmeasured.
- `OQ-18` — are four named budgets distinguishable enough to be chosen between?
- `OQ-19` — does hiding rather than destroying the Editor window keep memory unnoticed?
