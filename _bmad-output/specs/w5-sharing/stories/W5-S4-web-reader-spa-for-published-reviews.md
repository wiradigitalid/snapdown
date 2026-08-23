---
id: W5-S4
title: Web reader SPA for published reviews (Screen 14 and 15)
wave: W5
status: planned
created: 2026-08-23
dependencies: [W5-S2]
files:
  - web/ui/src/screens/PublishedBundleReader.tsx
  - web/ui/src/screens/PublicationNotFound.tsx
  - web/ui/src/index.ts
  - apps/web-service/web/index.html
---

# W5-S4: Web reader SPA for published reviews (Screen 14 and 15)

## User Story
As a human reader opening a Publication URL in a web browser, I want a clean, responsive single-page reader rendering the bundle's Markdown and embedded images without revealing any library metadata.

## Acceptance Criteria
- [ ] Implement `PublishedBundleReader.tsx` (Screen 14) and `PublicationNotFound.tsx` (Screen 15) in `@snapdown/ui`.
- [ ] Embed or serve the SPA assets via `web-api` at `/b/{slug}`.
- [ ] Markdown rendering with responsive layouts, token-based typography, and image loading placeholders.
- [ ] Vitest component test suite for both rendered and 404 refused states.
