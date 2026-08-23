---
type: contract-inventory
component: bundle
created: "2026-08-23"
updated: "2026-08-23"
---

# Contract inventory — bundle

Derived by reading `apps/desktop/src-tauri/src/commands/bundle.rs` on 2026-08-23.

| id | Command | Reads / writes | Realizes | State |
|---|---|---|---|---|
| CB-1 | `create_bundle` | writes | UC-9 | built |
| CB-2 | `list_bundles` | reads | UC-10 | built |
| CB-3 | `get_bundle_detail` | reads | UC-10, UC-11 | built |
| CB-4 | `delete_bundle` | writes | UC-12 | built |
| CB-5 | `copy_bundle_to_clipboard` | reads | UC-11 | built |

## The five lanes

### CB-1 `create_bundle`

| Lane | Answer |
|---|---|
| **Shape** | A name and an ordered selection of Finding ids, returning the Bundle |
| **Refusal** | A taken name is refused while typing, naming the existing Bundle. A selection containing a Finding whose image is missing is refused, naming it (`BR-13`). A selection of zero is refused (`BR-60`); a selection of one is not special-cased (`BR-61`) |
| **Idempotence** | **No.** Two calls with the same selection make two Bundles, and the second name would be refused first anyway |
| **Ordering** | Image copies, then the Markdown file, then the rows. Nothing is committed before its files exist (`AD-2`). Position is fixed here and never changes (`BR-58`) |
| **Failure** | Nothing written — no half-Bundle, no orphaned image copies. `[PARTIAL]` — the rollback's coverage of the image copies was not verified |

### CB-4 `delete_bundle`

| Lane | Answer |
|---|---|
| **Shape** | An id, confirmed once |
| **Refusal** | `none` |
| **Idempotence** | Yes |
| **Ordering** | Unpublish first if published (`BR-23`), then rows, image copies, and the Markdown file in one unit of work. Unpublishing after deleting would leave a live URL for a Bundle that no longer exists |
| **Failure** | Nothing removed. The source Findings are never touched, and the confirmation says so beforehand |

### CB-5 `copy_bundle_to_clipboard`

| Lane | Answer |
|---|---|
| **Shape** | An id. The bytes go to the clipboard, not to the caller |
| **Refusal** | `none` |
| **Idempotence** | Yes |
| **Ordering** | `none`. It reads `bundle.markdown` — one column, one set of bytes (`AD-9`) |
| **Failure** | **Reported, always.** A silent clipboard failure loses the primary handoff path in the whole product and the Reviewer would have no way to know |

### CB-2 / CB-3

| Lane | Answer |
|---|---|
| **Shape** | A list DTO, and a detail DTO carrying the Markdown and the items in position order |
| **Refusal** | `none` — a store that cannot open is `LC-013`'s boundary |
| **Idempotence** | Yes |
| **Ordering** | `none` |
| **Failure** | The surface shows its error in place, with a Retry. An item whose image copy is missing is flagged; the Bundle still opens |

`CB-3`'s item list is where `BUG-1` surfaces to a Reviewer: it returns what `bundle_item` holds, and
after a source Finding is deleted that is one row short of the Markdown it sits beside.
