# 11: The Library opens and lists every Bundle

**What to build:** Clicking the Editor's Library icon — which today prints a line and does nothing —
opens the **Library**: a full-window overlay over the Editor, the same pattern Review & Assemble uses,
so the Editor underneath keeps its canvas, selection and scroll. It lists every Bundle newest-composed
first, one row each: a thumbnail of the Bundle's first image, its name, and a mono meta line reading
`N Findings · composed <relative time>`. The header names the screen, states the Bundle count, and
carries "All Bundles" and "Newest first" as **static readouts** of what the list is — not controls,
because there is no search, filter or sort in this release and a control that cannot be operated is
the same mistake as the toolbar's fake shortcut badges. Three states beyond the list: **empty** names
the next action ("Tick Findings in the strip and press Assemble. The Bundle you compose lands here."),
**loading** shows skeleton rows in the list's own shape so nothing jumps, and **cannot be read** says
what refused with **Try again** and **Open file location** beside it. Closed from the header's X and
by Escape. Rows carry no actions yet — that is ticket 12 — but the row's hover state and its overflow
button's placement are laid out now from the artboards so later tickets add to the row without moving
it. Build from the artboards in the design folder beside this ticket; their README traces every value
to a running-app component.

**Blocked by:** None (can start immediately)

**Status:** done — feat/merge commits per `.control/memlog/autopilot-2026-09-04.md` iteration 1; PR #38 merged 2026-09-03. Status line was stale until this correction.

- [ ] The Library icon opens the overlay and the stub that printed a line is gone; the known-stubs
      ratchet test no longer lists it and passes
- [ ] Every Bundle in the store appears once, newest-composed first, with thumbnail (the BundleItem at
      position 1), name, and `N Findings · composed <relative time>`; the header count equals the
      number of rows
- [ ] Empty, loading and cannot-be-read states render as the artboards show; the failure text names
      what refused; Try again re-reads the store
- [ ] Thumbnails sit on the canvas ground and read the same in both themes; every other element takes
      its colour from the theme and passes the contrast gate in both themes; the colour-literal test
      stays green
- [ ] Closing (X, Escape) returns to an Editor whose selection and scroll are exactly as left
- [ ] A reachability test — in the shape of the annotation-wiring test — asserts the Library component
      is instantiated and each of its callbacks is bound in Rust
- [ ] Built from the shared header, button and text components with the artboards' values; the
      design-system test passes
- [ ] `FR-28`: from the Library the Reviewer reaches the Editor (close), and from the Editor the
      Library (icon), without being told how
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] **Look at:** open the Library with several Bundles, in both themes; then with an empty store;
      then with the database held open by another process. Check thumbnails match the first image of
      each Bundle and the newest is on top
