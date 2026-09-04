# 15: An edited Bundle says so

**What to build:** Ticket 09's option B. A Bundle gains a **last-edited time**. Every existing Bundle
is backfilled with its composed time, so an untouched Bundle reads as never edited and that is true.
The time moves **only when Save actually changed the stored document or the title** — ticket 14's
no-op Save leaves it alone, which is what keeps the always-clickable Save free of visible side
effects. In the Library row the meta line appends ` · edited <relative time>` **only when** the
last-edited time differs from the composed time; Review & Update's header provenance line does the
same. **The list order does not change** — newest-composed first — so fixing a typo in an old Bundle
does not move it.

**Blocked by:** 11 (the row), 14 (Save)

**Status:** done — feat/merge commits per `.control/memlog/autopilot-2026-09-04.md` iteration 1; PR #38 merged 2026-09-03. Status line was stale until this correction.

- [ ] A database written before this change opens, migrates, and every existing Bundle's last-edited
      time equals its composed time — the pre-migration database is the fixture
- [ ] A Save that changed the title or document moves the last-edited time; a Save that changed nothing
      leaves it exactly as it was — both asserted at the store seam
- [ ] A row whose two times are equal shows only `composed <when>`; a row whose times differ shows
      `composed <when> · edited <when>`; the header in Review & Update matches
- [ ] After editing the oldest Bundle, the Library order is unchanged
- [ ] The migration guard seen red first: run the new store against the old schema without the
      migration and watch it fail
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] **Look at:** with existing Bundles, open the Library — no row should say "edited". Edit the
      oldest one's note and Save — its row gains "edited just now" and stays at the bottom. Save again
      with no change — the edited time does not move
