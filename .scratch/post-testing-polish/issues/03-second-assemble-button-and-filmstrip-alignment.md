# 03: A second Assemble entry point near the canvas, and the filmstrip's misaligned Assemble area

**What to build:** A second Assemble button at the canvas's top-right, firing the exact same
`assemble-bundle-clicked` callback and reading the exact same ticked selection `prepare_bundle` already
reads (`main.rs:3423`) — a second door to the identical act, not a new one. In the same change, fix the
filmstrip's own Assemble button area so it lines up with the filmstrip's frame (a plain visual defect,
no behaviour change) — call this out as a separate, clearly-labelled acceptance line so a reviewer can
tell "moved" from "fixed" apart.

**Decision already made, do not re-open:** the spec names an open question — should the new button (a)
just be a closer door to the same selection-gated act, refusing exactly like the existing doors when
nothing is ticked, or (b) also tick the active Finding first if nothing is ticked (new behaviour beyond
`FR-10`). **Build (a).** The spec says "Build (a) unless the owner asks for (b) when the ticket opens" —
under this mandate there is no owner to ask, so (a) is what ships. Record this in the ticket the way
`BUG-104` recorded its own reversal of `ticket 19`'s reasoning, quoting rather than dropping it: ticket
19 kept Assemble out of the ribbon because "it acts on the filmstrip's ticked selection, not on the
canvas beside it" — this spec deliberately reverses that for reachability, on the owner's own request,
while keeping the underlying behaviour (selection-gated, same refusal message) unchanged.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

Realizes `FR-10` (a second entry point to it). See `.scratch/post-testing-polish/spec.md`
Implementation Decisions § "A second Assemble entry point" for the full design.

## Seam

Component/wiring test (`test_annotation_wiring.rs`'s shape): the new button is instantiated and its
callback bound to `assemble-bundle-clicked`. Behavioural: same refusal message as the existing doors
when nothing is ticked (assert the same string/function, not a second copy of the message).

## Acceptance

- [ ] New Assemble button exists near the top of the canvas, fires `assemble-bundle-clicked`
- [ ] Behaves exactly like the filmstrip-footer door: same ticked-selection rule, same refusal message
      when nothing is ticked — reading option (a), not (b)
- [ ] Filmstrip's Assemble button area now visually aligns with the filmstrip's own frame (visual fix
      only, no behaviour change)
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] **Look at:** confirm the new button's placement actually reads as closer to the canvas in the
      real, built UI (not a mock); confirm the filmstrip alignment fix looks right beside real content
