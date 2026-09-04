# 13: Open a Bundle locked in Review & Update

**What to build:** Clicking a Library row opens the Bundle in **Review & Update** — the same window
used to assemble, opened **locked**: a faithful view of the Bundle exactly as composed, built entirely
from the composer's read of the stored document (ticket 10), never from Findings, so a sealed Bundle
opens the same way as an unsealed one. The header carries a static provenance line (`N Findings ·
composed <when>`) and a badge reading **As composed**. There is no Edit/Preview pair: locked *is* the
preview. **No affordance of any kind appears on any element** — no chips, no field borders, no
hover targets on images; ticket 05 established that the compose window shows none on images either,
so a lock chip here would have had to invent a control in order to disable it. Footer: primary
**Edit** (which does nothing yet beyond being the button ticket 14 wires — do **not** render it
enabled until 14 lands; render it disabled with no tooltip, or omit it, and say which in the hand-off)
and Close. Closing returns to the Library.

**Blocked by:** 10 (the blocks come from the parse), 11 (the row click)

**Status:** done — feat/merge commits per `.control/memlog/autopilot-2026-09-04.md` iteration 1; PR #38 merged 2026-09-03. Status line was stale until this correction.

- [ ] Clicking a row opens the window locked; the title, Bundle notes, every Finding's image, note and
      Marker notes shown are exactly the stored document's, verified against a Bundle whose stored
      document was hand-edited to differ from what its Findings would produce today
- [ ] A sealed Bundle (its Findings deleted) opens and renders identically to how it did before
      sealing; nothing in this path calls the Finding store, asserted by the reachability test's
      binding check and by the sealed fixture
- [ ] Header shows the provenance line and the **As composed** badge; no Edit/Preview control exists
- [ ] No editable field, chip, border or hover affordance is rendered in locked mode; the source-reading
      test asserts the locked layout declares none
- [ ] Close returns to the Library with its scroll intact
- [ ] Contrast gate and colour-literal test green in both themes; design-system test green
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] **Look at:** open a Bundle from the Library and compare it side by side with its Markdown file;
      then discard that Bundle's originals by deleting its Findings from the filmstrip and open it again
      — it must look the same. Confirm nothing on screen invites a click except Close
