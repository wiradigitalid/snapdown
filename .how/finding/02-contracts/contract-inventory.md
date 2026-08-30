---
type: contract-inventory
component: finding
created: "2026-08-23"
updated: "2026-08-23"
---

# Contract inventory — finding

The command surface this component exposes to its UI. Derived by reading
`apps/desktop/src-tauri/src/commands/{finding,capture}.rs` on 2026-08-23 — the pre-`DEC-007` Tauri
webview, archived at `archive/desktop-tauri`.

**Not re-derived since `DEC-007`.** The desktop app moved to native Slint; the command names below
may still name the right operations, but their exact shape has not been checked against
`apps/desktop/src`'s current Rust functions. Treat this table as the last-known contract, not a
current one, until it is re-derived.

Numbers are stable. A new command takes the next `CF-`.

| id | Command | Reads / writes | Realizes | State |
|---|---|---|---|---|
| CF-1 | `capture_screen_region` | writes | UC-1 | built, **changing** |
| CF-2 | `trigger_overlay` | — | UC-1 | built |
| CF-3 | `dismiss_overlay` | — | UC-1 | built |
| CF-4 | `list_findings` | reads | UC-3, UC-6 | built |
| CF-5 | `get_finding_detail` | reads | UC-3, UC-4 | built |
| CF-6 | `save_note` | writes | UC-4 | built |
| CF-7 | `add_marker` | writes | UC-5 | built |
| CF-8 | `update_marker` | writes | UC-5 | built |
| CF-9 | `delete_marker` | writes | UC-5 | built |
| CF-10 | `delete_finding` | writes | UC-7 | built |
| CF-11 | `scan_orphans` | reads | UC-8 | built |
| CF-12 | `clean_orphans` | writes | UC-8 | built |

## The five lanes

### CF-1 `capture_screen_region` — the one on the clock

| Lane | Answer |
|---|---|
| **Shape** | Today: a region plus a Note, returning the stored Finding. Under `DEC-004` it must additionally return the **resolved pair and budget name** applied, because `NFR-18` stores them |
| **Refusal** | A zero-area region is refused before anything is captured; the overlay returns to `Armed` rather than ending (`state-machines.md` § 1). A Vault that refuses the write is reported by toast, because the overlay is already gone |
| **Idempotence** | **No, and deliberately not.** Two identical captures are two Findings. A Reviewer capturing the same region twice meant to |
| **Ordering** | Dismiss the overlay, **then** reduce and store. `NFR-2` gives 500 ms to dismiss and reduction must not block it. The reverse ordering is the one that fails the requirement |
| **Failure** | The Finding is not created. No partial row, no orphaned file (`AD-2`) |

### CF-7 / CF-8 / CF-9 — the Marker sequence

| Lane | Answer |
|---|---|
| **Shape** | Add takes a normalised position (`AD-3`); update takes a position; delete takes an ordinal |
| **Refusal** | A position outside 0.0–1.0 is refused. An ordinal that does not exist is refused, not ignored |
| **Idempotence** | Update, yes. Add, no — a second add is a second Marker |
| **Ordering** | Each of the three MUST write the Marker **and** its Note line in one operation (`AD-1`). Delete renumbers the remainder contiguously in the same operation (`BR-5`). Three commands, one collection, and this is the lane the invariant actually lives in |
| **Failure** | Neither the Marker nor the line is written. A half-write is what `AD-1` forbids |

### CF-10 `delete_finding`

| Lane | Answer |
|---|---|
| **Shape** | An id, confirmed once by the caller |
| **Refusal** | `none`. A Finding inside a Bundle is still deletable — the Bundle holds its own copy (`BR-13`) |
| **Idempotence** | Yes. Deleting a Finding that is gone succeeds silently |
| **Ordering** | Record and image file in one unit of work; prior state intact on any failure (`AD-2`) |
| **Failure** | Nothing removed. Not the row, not the file |

### CF-11 / CF-12 — orphans

| Lane | Answer |
|---|---|
| **Shape** | Scan returns files nothing points at and records pointing at nothing. Clean takes what to remove |
| **Refusal** | Clean refuses a path that escapes the Vault root. `VaultBlobStore` already carries that guard |
| **Idempotence** | Both, yes |
| **Ordering** | `none` — scan is read-only, and clean acts on what the Reviewer confirmed from a scan they saw |
| **Failure** | A file that will not delete is **reported**, not swallowed. This is the lane `settings`' Vault move got wrong at the pre-`DEC-007` webview's `vault_migration.rs:141`, and it is worth naming the contrast |

### CF-2 / CF-3 / CF-4 / CF-5 / CF-6

| Lane | Answer |
|---|---|
| **Shape** | Overlay control takes nothing; the reads return their DTOs; `save_note` takes an id and a body |
| **Refusal** | `trigger_overlay` while an overlay is already up is **ignored**, not refused. Two overlays never stack |
| **Idempotence** | All five |
| **Ordering** | `save_note` is debounced by the caller and last-write-wins. There is one writer — the Reviewer — so there is no conflict to resolve |
| **Failure** | The read shows its own error in place. `save_note` failing leaves the previous body and says so; it does not lose what was typed |
