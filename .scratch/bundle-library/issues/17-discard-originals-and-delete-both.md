# 17: Discard originals, and the two-step Delete both

**What to build:** An unsealed Bundle's menu gains **Discard originals…** (`FR-41`, `UC-30`): the
source Findings behind the Bundle — with their notes, Markers and annotations, and their image files
— are removed through the existing whole-Finding deletion path, one Finding at a time (record, then
files), while the Bundle's row, items, document and image copies are not touched. Sealing is a
**consequence** of the Findings being gone, not a write: the next time the menu opens it reads the
live state and shows Delete… only. The confirmation carries the artboards' copy ("This Bundle keeps
its own copies and stays readable, but it can no longer be disassembled.") and **additionally names
any other Bundle that shares one of those Findings and will therefore be sealed too** — that
consequence is real (`BR-12`, `BR-122`) and this dialog is the only place to say it. **Delete both**
— the Bundle and its originals, "Nothing comes back to the filmstrip" — is offered as a **second-step
choice inside the Disassemble confirmation**, never as a menu row: the most destructive act in the
product is never one click away. `BR-59` stays true: composing still never removes a Finding; this is
a separate, later, explicit act.

**Blocked by:** 16

**Status:** ready-for-agent

- [ ] Discard originals appears only for an unsealed Bundle; after it runs, the same Bundle's menu
      shows Delete… only, without any stored flag having been written
- [ ] After Discard originals the Bundle's row, BundleItems, stored document and every image copy are
      byte-identical to before; the source Findings and their files are gone (`FR-41`'s proof)
- [ ] When another Bundle shares one of the Findings, the confirmation names that Bundle; when none
      does, it says nothing extra
- [ ] Delete both is reachable only from within the Disassemble confirmation, and removes the Bundle
      (row, then folder) and then each Finding (record, then files); nothing returns to the filmstrip
- [ ] A failure partway through Discard originals leaves the Findings not yet processed intact and the
      Bundle untouched, and the toast says how many were discarded and which refused
- [ ] Callback-reachability and component-wiring tests cover the new paths
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] **Look at:** compose two Bundles that share a Finding; Discard originals on one — the dialog must
      name the other, and afterwards both must show Delete… only, both must still open and read
      correctly in Review & Update, and the filmstrip must no longer hold that Finding. Then
      Disassemble a fresh Bundle and find Delete both as the second step — it must not be on the menu
