# 16: Disassemble a Bundle, or Delete a sealed one

**What to build:** The row menu (overflow button and right-click, the filmstrip's gesture) gains its
destructive group, and **which verb appears is read live from whether the Bundle's source Findings
still exist — never from a stored flag** (`BR-122`). A Bundle every one of whose Findings still exists
is *unsealed* and offers **Disassemble…**; one missing any of them is *sealed* and offers **Delete…**
only. Both confirmations carry the artboards' copy: the Bundle named in quotes, a count of what goes,
what comes back ("Its 3 Findings become available to assemble again, with their notes and markers
intact" / "Its original captures were discarded earlier, so nothing comes back"), and "This cannot be
undone."; the cancel verb keeps ("Keep it"), the confirm verb is the act. Both acts do the same thing
to the Bundle: remove its row (BundleItems cascade) **and only then** its folder — record first, then
files, per `AD-2`; a crash between the two leaves orphan files the Vault sweeper already owns, never a
listed Bundle whose images are gone. Disassemble writes no Finding: the Findings reappear in the
filmstrip because it filters out whatever a Bundle holds, and the Bundle no longer holds them. The
Library refreshes; a toast says what happened. **Discard originals… is not rendered yet** (ticket 17),
and neither is any Delete-both path.

**Blocked by:** 11

**Status:** ready-for-agent

- [ ] The menu shows Disassemble… for a Bundle whose Findings all exist and Delete… for one missing
      any, determined by a live read — asserted with a fixture where a Finding is deleted between two
      menu openings
- [ ] Both confirmations show the settled copy with the real Bundle name and counts
- [ ] Disassemble removes the row and BundleItems, then the folder; afterwards the Findings are back in
      the filmstrip with notes and Markers intact and their own image files untouched
- [ ] Delete on a sealed Bundle removes row, items and folder; nothing appears in the filmstrip
- [ ] Write ordering asserted at the store seam: with a failure injected after the row delete and
      before the folder removal, the row is gone and the files remain; the reverse state is never
      produced. Seen red first by swapping the order
- [ ] A row-delete failure leaves everything intact and the toast names what refused
- [ ] Callback-reachability and component-wiring tests cover the new callbacks and the confirmation
      component
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] **Look at:** disassemble a Bundle and watch its Findings return to the filmstrip; check the Vault
      folder is gone and the Findings' images are not. Delete a sealed Bundle and confirm nothing
      returns. Read both dialogs as a first-time user — each must say what goes and what comes back
