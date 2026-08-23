---
id: W2-S4
title: Findings Editor list, detail view, and inline Note editing
wave: W2
status: done
created: 2026-08-23
dependencies:
  - W2-S1
  - W2-S2
  - W2-S3
files:
  - packages/shared-ui/src/components/FindingsEditor.tsx
  - apps/desktop/src/components/FindingsView.tsx
  - apps/desktop/src-tauri/src/commands/finding.rs
  - apps/desktop/src/services/finding.ts
---

# W2-S4: Findings Editor list, detail view, and inline Note editing

## User Story
As a tester reviewing visual findings, I want an editor to browse the list of captured findings, view finding details alongside the screenshot, and add/edit notes inline, so that I can organize and annotate test results quickly.

## Acceptance Criteria
- [ ] Implement `FindingsEditor.tsx` / `FindingsView.tsx` with sidebar finding list and main detail pane.
- [ ] Support viewing image thumbnail, metadata (timestamp, dimensions, status), and associated notes.
- [ ] Implement inline note creation and editing with automatic or explicit save.
- [ ] Expose Tauri IPC commands (`list_findings`, `get_finding_detail`, `save_note`).
- [ ] Full unit and component test coverage (`npm run test`, `cargo test`).
