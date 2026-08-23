---
id: W4-S2
title: Local API HTTP server with constant-time auth and bundle routes
wave: W4
status: planned
created: 2026-08-23
dependencies: [W4-S1]
files:
  - crates/snapdown-core/src/domain/error.rs
  - apps/desktop/src-tauri/src/server/mod.rs
  - apps/desktop/src-tauri/src/server/auth.rs
  - apps/desktop/src-tauri/src/server/handlers.rs
  - apps/desktop/src-tauri/src/server/error.rs
  - apps/desktop/src-tauri/tests/test_local_api_server.rs
---

# W4-S2: Local API HTTP server with constant-time auth and bundle routes

## User Story
As an external coding agent needing to read composed bundles, I want a local HTTP API bound to loopback (127.0.0.1) that verifies the Access Key and serves bundle metadata, markdown, and images in read-only mode with standard error envelopes.

## Acceptance Criteria
- [ ] Implement loopback HTTP server (`tiny_http` or lightweight TCP/HTTP handler) binding `127.0.0.1` on an available port.
- [ ] Implement `GET /v1/health` unauthenticated returning service liveness and required custom Snapdown header without revealing library contents.
- [ ] Implement `GET /v1/bundles` returning list of bundles (`id`, `name`, `finding_count`, `composed_at`). Returns `key_required` (401) on missing key, `key_invalid` (401) on revoked/wrong key, or 200 with JSON list.
- [ ] Implement `GET /v1/bundles/:id` returning exact stored markdown content and referenced image filenames.
- [ ] Implement `GET /v1/bundles/:id/images/:filename` streaming image bytes with path traversal guards.
- [ ] Error envelope formatting matching `cross-cutting.md` (`error.code`, `error.message`, `error.request_id`, `error.detail`).
- [ ] Integration tests verifying all endpoints, header guards, traversal prevention, and status codes.
