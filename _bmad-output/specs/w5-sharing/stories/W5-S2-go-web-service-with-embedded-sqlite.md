---
id: W5-S2
title: Go web service with embedded SQLite, staged publish, and public routes
wave: W5
status: planned
created: 2026-08-23
dependencies: []
files:
  - apps/web-service/go.mod
  - apps/web-service/main.go
  - apps/web-service/internal/store/store.go
  - apps/web-service/internal/server/server.go
  - apps/web-service/internal/server/handlers.go
  - apps/web-service/internal/server_test.go
---

# W5-S2: Go web service with embedded SQLite, staged publish, and public routes

## User Story
As an external agent or web browser, I want an independent lightweight Go web service (`web-api`) with embedded SQLite and blob storage that provides credential-gated staged publishing and unpublishing alongside public read routes serving verbatim Markdown, relative images, and identical 404 refusals.

## Acceptance Criteria
- [ ] Create `apps/web-service` Go module using standard library and `chi` router with embedded SQLite (`modernc.org/sqlite` or `mattn/go-sqlite3`).
- [ ] Implement embedded SQLite tables `published_bundle`, `published_blob`, `web_schema_version`.
- [ ] Credential-gated routes:
  - `PUT /publish/{slug}`: Staged all-or-nothing publish.
  - `DELETE /publish/{slug}`: Unpublish and wipe assets.
  - `GET /publish/{slug}`: Reconcile query.
- [ ] Public routes:
  - `GET /b/{slug}`: Content-negotiated (HTML for web browser, raw Markdown for `text/markdown`).
  - `GET /b/{slug}/raw.md`: Unambiguous raw markdown.
  - `GET /b/{slug}/images/{filename}`: Relative image serving with path traversal refusal.
- [ ] Strict NFR-15 conformance: Identical 404 response for unknown, revoked, and never-issued slugs.
- [ ] Unit & HTTP integration tests in Go (`go test ./...`).
