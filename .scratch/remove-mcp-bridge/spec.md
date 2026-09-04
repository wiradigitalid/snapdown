# Spec: Remove the Agent Bridge (MCP executable, Access Key ceremony, Settings tab)

**Status:** ready-for-agent
**Source:** `DEC-016` (status: `accepted`, supersedes `DEC-002`) — this spec is the "separate
`/to-spec` → `/to-tickets` → `/implement-spec` pass" its own Cost section names as the removal work
it only unblocks. This is removal work, not a new promise: no `FR` or `UC` is invented here.
**Corpus ids this rests on:** `DEC-016` · `DEC-002` (superseded) · `OQ-6` (closed by `DEC-016`) ·
`BUG-59` (names the retired surface). The corpus is an input, never a gate: where a document and the
code disagree, the code wins and the document is corrected afterwards — and per `DEC-016`'s own Cost
section, updating the corpus (`agent-access` component, its SRS/SDD, the spine's Binds lists, the PRD,
the two structure maps, and the `BUG-59`/`OQ-6` registry rows) is explicitly a *separate* pass, not
this one.

---

## Problem Statement

The owner instructed, directly in chat on 2026-09-04: *"Agent bridge hapus aja dari settings dan
fitur, MCP tidak relevan lagi harusnya, karena main sistem copy paste markdown saja sudah selesai
semuanya"* — remove the Agent Bridge feature from Settings and from the product, because the
copy-Markdown-and-paste workflow already covers what the bridge was for. `DEC-016` recorded that
instruction as a decision and superseded `DEC-002` (the decision that built the bridge). What remains
is dead weight in the tree: an executable nothing launches, a domain/store layer nothing calls, and a
Settings tab that has told every Reviewer for weeks that the feature is "not available yet" — when the
truth, after `DEC-016`, is that it will never arrive, because the product no longer wants it.

## Solution

Delete every part of the running-channel design that `DEC-016` retires, in full, so the tree matches
the decision:

1. **The `mcp-bridge` executable** — `crates/snapdown-bridge` (the stdio MCP server and its client),
   and its membership in the workspace.
2. **The Access Key ceremony's code**, even though it was never wired to a live server. Investigation
   found this is *not* limited to the bridge crate: `crates/snapdown-core` carries the `AccessKey`
   domain type and the `AccessKeyStore` port, and `crates/snapdown-store` carries
   `SqliteAccessKeyStore` plus its own test file. None of it is a dependency of `snapdown-bridge`
   (that crate depends on `serde`, `serde_json`, `ureq`, `base64` only) or of `apps/desktop` — it is
   scaffolding for the Local API server that `components.yaml`'s own comment already says "has no
   implementation at all today." `DEC-016` calls the ceremony out by name as removed in full, so this
   dead scaffolding goes too.
3. **The Settings UI.** Investigation found an actual "Agent Bridge" tab in the shipping Slint
   Settings screen (`apps/desktop/ui/components/settings.slint`, `apps/desktop/ui/appwindow.slint`,
   wired from `apps/desktop/src/main.rs`) that the dispatch brief's own investigation missed because
   it searched for the strings `AccessKey`/`access-key`/`agent-access`/`AgentBridge` and the tab is
   spelled "Agent Bridge" with a space. It shows a static `bridge-status` message telling the
   Reviewer the feature is "Not available yet." This tab, its backing property, and the Rust code
   that sets the message are removed. The Settings tab strip goes from four tabs to three; the two
   blocks that followed the removed tab (About, Built With) shift from `tab == 3` to `tab == 2`.
4. **The one-executable CI guard's expectations.** `.github/workflows/desktop-ci.yml`'s
   `desktop-build` job names `snapdown-bridge.exe` as a second legitimate workspace binary in its
   `$known` array and comment. After this removal that binary is never produced, so the guard and its
   comment are updated to expect one executable only.

## User Stories

This is removal work undoing a specific, already-recorded product decision, not a new capability —
per `to-spec`'s own instruction not to invent a requirement the corpus does not already state, no
new `FR`/`UC` is written and no user story list is produced. The single "story" this spec serves is
the one `DEC-016` already recorded in its own words: a Reviewer who wants to hand a Bundle to an agent
does it by copying the assembled Markdown, never by pointing an agent at a running channel into the
Library.

## Implementation Decisions

- **Delete `crates/snapdown-bridge` in full** (`src/lib.rs`, `src/main.rs`, `src/client.rs`,
  `src/mcp.rs`, `tests/test_bridge_mcp.rs`, `Cargo.toml`) and remove its line from the root
  `Cargo.toml` workspace `members` list. `Cargo.lock` regenerates on the next `cargo` invocation;
  commit the regenerated file.
- **Delete the Access Key domain/port/store code**: `crates/snapdown-core/src/domain/access_key.rs`,
  `crates/snapdown-core/src/ports/access_key_store.rs`,
  `crates/snapdown-store/src/sqlite/access_key_store.rs`,
  `crates/snapdown-store/tests/test_sqlite_access_keys.rs` — plus every `pub mod` / `pub use` line
  in each crate's `lib.rs`, `domain/mod.rs`, `ports/mod.rs`, and `sqlite/mod.rs` that names them.
- **Migration history judgment call, made here rather than left implicit:** the SQLite migration
  that creates the `access_key` table (`crates/snapdown-store/src/sqlite/migrations.rs`, version 4)
  is removed from the `MIGRATIONS` array rather than kept as inert history. The migration system
  applies strictly by `version > current_version` and does not require contiguous version numbers,
  so this is safe on both paths: a fresh install simply never creates the table, and an existing
  installed library (already past version 4) is untouched — it keeps an orphaned, never-queried
  table, which is harmless. Every test asserting `get_schema_version() == 9` still holds, since 9
  remains the highest version in the array. This was chosen over leaving the entry in place because
  the table never backed a reachable feature (no Local API ever existed to use it) and DEC-016 asks
  for the ceremony removed "in full," not merely disconnected.
- **Remove the Settings UI**: in `settings.slint`, delete the `bridge-status` property, the "Agent
  Bridge" entry in the tab-label list, and the `if root.tab == 2` block that renders it; renumber the
  two following `if root.tab == 3` blocks (About, Built With) to `tab == 2`. In `appwindow.slint`,
  delete the `bridge-status` property and its pass-through into `SdSettings`, and trim the stale
  "Local MCP Bridge" mention out of the Assemble & Review comment (the other two absent channels it
  names, Copy Markdown and Publish, are untouched — this spec only touches the Bridge). In
  `apps/desktop/src/main.rs`, delete the `window.set_bridge_status(...)` call and its comment.
- **Update the CI guard**: `.github/workflows/desktop-ci.yml`'s `desktop-build` job — drop
  `"snapdown-bridge.exe"` from the `$known` array and rewrite the comment above it so it no longer
  describes a second legitimate workspace binary that no longer exists.
- **No change to `apps/web-service` (Go)** — investigation found no reference to the bridge, the
  access key, or the agent-bridge Settings tab there.

## Testing Decisions

This is a removal, so the proof is subtractive: the workspace still builds and every existing test
still passes with the removed surfaces gone, and nothing remains that references what was deleted.

- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo test --workspace --no-fail-fast` from the repo root must all exit 0 after the removal — the
  same three commands `AGENTS.md` names as this repo's verification.
- No new test is written to prove an absence beyond what the compiler and the existing suite already
  prove: a dangling `use` of a deleted module is a compile error, not a silent gap, and the CI guard's
  own `$unexpected` check (already covered by `apps/desktop/tests/test_executable_identity.rs` and the
  workflow itself) is what would catch a stray `snapdown-bridge.exe` reappearing.
- Prior art for "deletion proven by the build," not a bespoke assertion: this mirrors how `web/ui`'s
  removal under `OQ-27` was verified (nothing in the active workspace referenced the deleted package),
  per `AGENTS.md`'s own account of that removal.

## Out of Scope

- **Updating the corpus.** `DEC-016`'s own Cost section names retiring the `agent-access` Product
  Component, its `SRS`/`SDD`, the spine's `AD-1`/`AD-5`/`AD-7`/`AD-9`/`AD-10` Binds lists, the PRD,
  and the two structure maps as a separate pass. `.what/`, `.how/`, `.control/decisions/DEC-016*.md`,
  and `.control/registry/components.yaml` are not touched by this spec's tickets.
- **Closing `BUG-59`** (`.control/registry/defects.yaml`) and any other `agent-access`-component
  defect/registry row. Registry upkeep belongs to the same separate corpus pass.
- **The "Local MCP Bridge" column in the Assemble & Review UX asset's three-channel design**, beyond
  removing the one stale mention of it found in `appwindow.slint`'s comment. Whether that column's
  place in the design is itself revisited is a UX question, not this removal's.
- **Any change to `apps/web-service`** — nothing there references the removed surfaces.

## Further Notes

- Investigation surfaced two things the dispatch brief's own summary got wrong or missed, corrected
  here so a future reader does not repeat the search that missed them: (1) the Access Key domain/port
  code in `snapdown-core`/`snapdown-store` is real, compiled, tested code today, not merely a bridge
  concern; (2) the Slint Settings screen does have an Agent Bridge tab — it was missed because it is
  spelled with a space ("Agent Bridge"), not the identifier forms the brief searched for.
- `crates/snapdown-bridge/tests/test_bridge_mcp.rs` and `crates/snapdown-store/tests/test_sqlite_access_keys.rs`
  are deleted wholesale along with the code they test — there is nothing left for them to test.
