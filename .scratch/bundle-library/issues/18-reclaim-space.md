# 18: Reclaim space

**What to build:** A second full-window overlay, **Reclaim space** (`FR-42`, `UC-31`), reached from the
Library's header and from a new entry in Settings' Vault area beside the existing Vault path controls.
It lists every **unsealed** Bundle — name, count of original captures, relative composed time, and the
disk its **original** Findings' image files occupy, **measured from the files**, not estimated — with a
header total ("373.1 MB reclaimable") and a checkbox per row. The footer reads `N of M selected · X MB
will be freed`, **Cancel**, and **Discard originals**, which runs ticket 17's act for each ticked
Bundle behind one confirmation that counts Bundles and captures; the shared-Finding notice from
ticket 17 appears here too when it applies. The screen's explanatory line is the artboards':
"Discarding a Bundle's originals keeps the Bundle readable and shareable. It can no longer be
disassembled." Its empty state: "Nothing to reclaim — No Bundle is holding original captures. Every
Bundle here either had its originals discarded already, or has none left to discard."

**Blocked by:** 17

**Status:** done — feat/merge commits per `.control/memlog/autopilot-2026-09-04.md` iteration 1; PR #38 merged 2026-09-03. Status line was stale until this correction.

- [ ] Reachable from the Library header and from Settings' Vault area; both open the same screen
- [ ] Lists exactly the unsealed Bundles; each row's size equals the sum of its original Findings'
      image file sizes on disk, asserted against a fixture with known file sizes; the header total is
      their sum
- [ ] Ticking rows updates the footer's count and freed total; the total after discarding equals the
      previous total minus the sum of what was discarded (`FR-42`'s proof)
- [ ] The bulk confirmation counts Bundles and captures and, where applicable, names Bundles that will
      be sealed as a side effect
- [ ] After discarding, the screen re-reads and the discarded Bundles are gone from it; opening the
      Library shows them sealed
- [ ] The empty state renders when no Bundle is unsealed
- [ ] Contrast, colour-literal, design-system, callback-reachability and component-wiring tests green
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] **Look at:** with several Bundles, open Reclaim space from Settings and compare a row's size
      against the Vault folder's properties in Explorer; tick two, discard, and watch the total drop by
      their sum. Then open it again from the Library header and confirm the same list
