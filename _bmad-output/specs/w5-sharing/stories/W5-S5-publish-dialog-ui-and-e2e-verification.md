---
id: W5-S5
title: Publish dialog UI and end-to-end publishing verification
wave: W5
status: planned
created: 2026-08-23
dependencies: [W5-S1, W5-S2, W5-S3, W5-S4]
files:
  - web/ui/src/screens/PublishDialog.tsx
  - apps/desktop/src/components/PublishDialog.tsx
  - apps/desktop/src/components/BundleView.tsx
  - apps/desktop/src/services/sharing.ts
  - apps/desktop/src/test/publish_dialog.test.tsx
  - tests/integration/test_sharing_e2e.rs
---

# W5-S5: Publish dialog UI and end-to-end publishing verification

## User Story
As a reviewer, I want a Publish dialog modal (`PublishDialog.tsx` / Screen 11) in the Desktop app with clear confirmation warnings, URL copying, and unpublish controls, backed by an end-to-end verification test suite across all containers.

## Acceptance Criteria
- [ ] Create `PublishDialog.tsx` in `@snapdown/ui` and `apps/desktop` adhering to BR-86 confirmation requirements.
- [ ] Render publication badge (Published / Unpublished / Error / Last Known) in `BundleView.tsx`.
- [ ] Copy URL feedback toast/action with URL formatting.
- [ ] End-to-end verification test across Desktop app, Go web-service, and Web UI reader.
