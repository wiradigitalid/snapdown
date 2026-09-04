# 03: Remove the "Agent Bridge" tab from the Settings screen

**What to build:** a Reviewer opening Settings sees three tabs — General & Quality, Hotkeys, About —
never a fourth "Agent Bridge" tab. That tab currently shows a static message saying the feature is
"Not available yet"; after `DEC-016` that is no longer true (it is never coming), so the honest
product surface is for the tab not to exist at all, matching the owner's own instruction to remove the
Agent Bridge "dari settings dan fitur" (from Settings and from the product).

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] `apps/desktop/ui/components/settings.slint`: the `bridge-status` in-property is deleted; the
      `{ label: "Agent Bridge" }` entry is removed from the tab list (three entries remain: "General &
      Quality", "Hotkeys", "About"); the `if root.tab == 2 : SdCard { ... AGENT BRIDGE ... }` block is
      deleted; the two blocks that followed it (`if root.tab == 3` for "SNAPDOWN"/About and
      `if root.tab == 3` for "BUILT WITH") are renumbered to `tab == 2`.
- [ ] `apps/desktop/ui/appwindow.slint`: the `bridge-status` property and its pass-through into
      `SdSettings { ... bridge-status: root.bridge-status; ... }` are deleted. The Assemble & Review
      comment's stale mention of "Local MCP Bridge" as one of the UX asset's three absent channels is
      trimmed to reflect that this channel is retired rather than merely unbuilt — without touching
      what the comment says about the other two channels (Copy Markdown, Publish), which are unrelated
      to this removal.
- [ ] `apps/desktop/src/main.rs`: the `window.set_bridge_status(...)` call and its preceding comment
      (citing `BUG-59`) are deleted.
- [ ] No remaining reference to `bridge-status`, `set_bridge_status`, or an "Agent Bridge" tab anywhere
      under `apps/desktop/`.
- [ ] `cargo build --workspace`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
      -- -D warnings`, and `cargo test --workspace --no-fail-fast` all succeed from the repo root,
      including `apps/desktop/tests/*` (Slint compiles as part of the crate build, so a dangling
      `tab == 3` reference or an unset property would fail the build, not just look wrong).
