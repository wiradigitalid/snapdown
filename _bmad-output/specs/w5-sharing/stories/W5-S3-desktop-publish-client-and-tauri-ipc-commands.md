---
id: W5-S3
title: Desktop publish client and Tauri IPC commands
wave: W5
status: done
created: 2026-08-23
dependencies: [W5-S1, W5-S2]
files:
  - apps/desktop/src-tauri/src/publish/mod.rs
  - apps/desktop/src-tauri/src/publish/client.rs
  - apps/desktop/src-tauri/src/commands/sharing.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tests/test_publish_client.rs
---

# W5-S3: Desktop publish client and Tauri IPC commands

## User Story
As a reviewer in the Desktop app, I want a staged publish client (`LC-020`) and Tauri IPC commands to publish a confirmed bundle, unpublish a published bundle, and reconcile publishing status with the web service.

## Acceptance Criteria
- [ ] Implement `PublishClient` in Rust (`apps/desktop/src-tauri/src/publish/client.rs`) executing the staged upload protocol against `web-api` over HTTPS/HTTP with publish credentials.
- [ ] Implement sticky error tracking on `publication.last_error` so unpublish failures keep the bundle marked as published until confirmed (BR-20, BR-96, BR-97).
- [ ] Automatically unpublish when a published bundle is deleted (BR-23).
- [ ] Tauri IPC commands: `publish_bundle`, `unpublish_bundle`, `get_publication_status`, `reconcile_publication`.
- [ ] Integration tests covering successful publish, retry/republish, sticky error handling, and unpublish confirmation.
