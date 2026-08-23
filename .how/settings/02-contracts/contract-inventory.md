---
type: contract-inventory
component: settings
created: "2026-08-23"
updated: "2026-08-23"
---

# Contract inventory — settings

The Tauri command surface this component exposes to its own webview. It is **not** in
`.how/_platform/inventory-api.md`, and that is correct: that inventory holds the Local API's HTTP
endpoints, which cross a process boundary. These cross a language boundary inside one process, which
`AD-5` and `AD-7` do not reach.

Numbers are stable. A new command takes the next `CS-` and a removed one keeps its number.

Derived by reading `apps/desktop/src-tauri/src/commands/{settings,hotkey,startup}.rs` on 2026-08-23.

| id | Command | Reads / writes | Realizes | State |
|---|---|---|---|---|
| CS-1 | `get_settings` | reads | UC-13, UC-14 | built |
| CS-2 | `set_vault_path` | writes | UC-14 | built |
| CS-3 | `set_quality_budget` | writes | UC-13 | built, **changing** |
| CS-4 | `get_latest_finding_size` | reads | UC-13 | built |
| CS-5 | `pick_vault_folder` | reads (native picker) | UC-14 | built |
| CS-6 | `open_vault_folder` | reads (shell) | UC-14 | built |
| CS-7 | `get_hotkeys` | reads | UC-15 | built |
| CS-8 | `set_hotkey` | writes | UC-15 | built |
| CS-9 | `clear_hotkey` | writes | UC-15 | built |
| CS-10 | `get_startup_status` | reads OS | UC-16 | built |
| CS-11 | `set_startup_status` | writes OS | UC-16 | built |
| CS-12 | `get_quality_budget_presets` | reads | UC-13 | **planned** — `DEC-004` |

## The five lanes, per contract

Every contract answers all five. `none` with a reason is an answer.

### CS-3 `set_quality_budget` — the one that changes

| Lane | Answer |
|---|---|
| **Shape** | Today: `(max_long_edge: u32, encoder_quality: u8) -> QualityBudget`. Under `DEC-004`: `(budget: NamedBudget, advanced: Option<ResolvedPair>) -> QualityBudgetState`, where passing `advanced` moves the state to `Custom` |
| **Refusal** | A value outside its range is refused at entry, naming the range (`FR-5`). The state does **not** move to `Custom` on a refused value (`BR-117`) |
| **Idempotence** | Yes. Setting the budget already in effect is a no-op that still returns the current state |
| **Ordering** | The named state and the resolved pair are one write (`BR-116`). They are never observable disagreeing |
| **Failure** | A store write that fails leaves the previous budget in effect and reports it in the group, not as a toast |

The shape change is why this row reads **changing** rather than **built**: the command exists, and it
is the wrong shape for the promise `FR-5` now makes.

### CS-10 / CS-11 — the OS pair

| Lane | Answer |
|---|---|
| **Shape** | `get_startup_status() -> StartupStatusDto { enabled: bool }` |
| **Refusal** | `set_startup_status` returns the registration state **after** the attempt, not the state requested. A refused enable returns `enabled: false` |
| **Idempotence** | Yes on both, because both end by re-reading the OS |
| **Ordering** | None. There is no ordering between them worth stating: each is one synchronous call |
| **Failure** | `[MISSING]` — the DTO has no way to say *unreadable*. `bool` cannot carry the `Unknown` state that `BR-108` and `state-machines.md` § 1 require, so the webview must invent one. It currently invents `true` |

That last row is the whole defect, and it is visible in the contract before it is visible in the UI: a
`bool` where the domain has three states forces every caller to guess the third.

### CS-2 `set_vault_path`

| Lane | Answer |
|---|---|
| **Shape** | `(new_path: String, migrate: bool) -> String`, returning the path in effect afterwards |
| **Refusal** | A folder that cannot be written to is refused before anything moves (`BR-115`), by writing to it rather than by inspecting it |
| **Idempotence** | Yes. The path already in effect is a no-op and attempts no move |
| **Ordering** | Copy every file, verify every copy, remove the sources, then write the Setting. The Setting is last (`AD-2`) |
| **Failure** | Nothing moved, sources intact, Setting unchanged (`SCN-01`). Two swallowed `fs::remove_file` results are the known gap |

### CS-1, CS-4, CS-5, CS-6, CS-7, CS-12 — reads

| Lane | Answer |
|---|---|
| **Shape** | Each returns its DTO; none takes a mutating argument |
| **Refusal** | `none` — a read refuses only when the store cannot be opened, which is `LC-025`'s boundary, not each command's |
| **Idempotence** | Trivially, being reads. `CS-5` and `CS-6` are the exceptions worth naming: both open something in the shell and are idempotent in effect but not in *observable* result, because a second call opens a second window |
| **Ordering** | `none` |
| **Failure** | The failing group shows its own error and a Retry. The other four keep working |

### CS-8 / CS-9 — the hotkey pair

| Lane | Answer |
|---|---|
| **Shape** | `set_hotkey(action, shortcut)`, `clear_hotkey(action)` |
| **Refusal** | Against Snapdown's own actions first (`BR-27`), then against Windows. The two refusals are worded differently on purpose — one the Reviewer can resolve, one they may not be able to |
| **Idempotence** | Yes. Re-binding the combination already bound re-registers it, which is harmless and is the recovery path from `Unregistered` |
| **Ordering** | Register the new binding, then unregister the old. Never the reverse — unregistering first leaves a window with no hotkey at all if the new one fails |
| **Failure** | The previous binding stays registered and the chip never shows a combination that is not in effect |
