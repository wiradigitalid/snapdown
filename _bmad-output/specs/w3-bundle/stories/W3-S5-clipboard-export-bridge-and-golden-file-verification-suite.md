---
id: W3-S5
title: Clipboard export bridge and golden-file verification suite
wave: W3
status: done
created: 2026-08-23
dependencies:
  - W3-S2
  - W3-S3
  - W3-S4
files:
  - apps/desktop/src-tauri/src/commands/bundle.rs
  - apps/desktop/src/services/bundle.ts
  - apps/desktop/src/components/BundleView.tsx
  - crates/snapdown-store/tests/test_golden_markdown.rs
---

# W3-S5: Clipboard export bridge and golden-file verification suite

## User Story
As a user sharing findings across tools (Slack, Jira, GitHub), I want to copy the rendered Markdown bundle with embedded image attachments or links directly to my clipboard, and have golden-file automated tests verifying byte-for-byte Markdown output consistency across runs.

## Acceptance Criteria
- [ ] Implement `copy_bundle_to_clipboard` in `apps/desktop/src-tauri/src/commands/bundle.rs` copying Markdown text to the OS clipboard.
- [ ] Add "Copy Markdown" button in `BundleView.tsx` with instant feedback indicator.
- [ ] Add golden-file regression test suite in `crates/snapdown-store/tests/test_golden_markdown.rs` verifying full bundle markdown against reference snapshots (`INV-EXPORT-001`).
- [ ] Full unit and automated test coverage across Rust (`cargo test`) and React (`npm run test`).
