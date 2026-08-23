# Code Review: W5-S2 (Go web service with embedded SQLite, staged publish, and public routes)

## Metadata
- **Story**: W5-S2
- **Author**: Amelia (Worker)
- **Reviewer**: Reviewer A (Orchestrator)
- **Verdict**: `ACCEPTED`
- **Date**: 2026-08-23

## Verification Checkpoints
1. **Container `web-api` (LC-023 / LC-024)**: Pure Go service implemented in `apps/web-service` with `chi` router and pure Go SQLite (`modernc.org/sqlite`).
2. **Schema & Tables**: Implements tables `published_bundle`, `published_blob`, `web_schema_version` matching `inventory-db.md` (tables 10–12). No Library IDs stored (AD-8, BR-101).
3. **Staged All-or-Nothing Publishing**: `PUT /publish/{slug}` implements staged verification before publishing to live (BR-89).
4. **Verbatim Markdown & Relative Images**: `GET /b/{slug}/raw.md` and content negotiation serve exact markdown bytes (AD-9, BR-91, BR-93); relative images resolve at `/b/{slug}/images/{filename}` (BR-92) with path traversal guards.
5. **NFR-15 Invariant**: Unknown, deleted, or revoked slugs return identical HTTP 404 response payload (BR-24).
6. **Tests**: Go test suite in `apps/web-service/internal/server_test.go` and workspace tests pass.

## Decision
Verdict: `ACCEPTED`
