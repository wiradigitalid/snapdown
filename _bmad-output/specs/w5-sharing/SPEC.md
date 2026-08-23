---
id: SPEC-W5
wave: W5
title: Sharing complete — publish, the web service, the reader
status: draft
created: 2026-08-23
companions:
  - .control/registry/index.yaml
  - .control/registry/components.yaml
  - .control/product-glossary.md
  - .what/sharing/SRS-sharing.md
  - .how/sharing/SDD-sharing.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/c4-l2-containers.md
  - .how/_platform/inventory-api.md
  - .how/_platform/inventory-db.md
  - .how/_platform/inventory-screen.md
  - .how/_platform/cross-cutting.md
  - .what/sharing/02-rules/rules-sharing.md
sources:
  - .what/_prd/agent-handoff/prd.md
  - .control/registry/requirements.yaml
  - .control/registry/usecases.yaml
  - .control/registry/waves.yaml
---

# Wave W5 Specification Contract: Sharing

## Why

Wave W5 delivers the publishing capability of Snapdown. It allows the Reviewer to publish a confirmed Bundle to an unlisted URL on an independently run `web-api` server so remote coding agents or human reviewers can read the review without direct access to the local machine (CAP-8: FR-23, FR-24, FR-25, FR-26, UC-20, UC-21, UC-22, UC-23).

It spans three containers:
1. **Desktop App (`desktop-app`)**: `publish-client` (LC-020), `publication-store` (LC-021), `publish-dialog` (LC-022 / Screen 11).
2. **Web Service (`web-api`)**: Go HTTP service using `net/http` and `chi` with embedded SQLite (`published_bundle`, `published_blob`) and a blob directory (LC-023 `publication-router`, LC-024 `served-publication-store`).
3. **Web Reader (`web-ui`)**: React + Vite SPA rendered in reader's browser (LC-027 `bundle-reader` / Screens 14 & 15).

## Stories Breakdown

1. **W5-S1: SQLite Schema Migration v5 (`publication`), Publication domain entity & store in Desktop App**
   - Migration v5 adding table `publication` (`id`, `bundle_id`, `slug`, `base_url`, `published_at`, `unpublished_at`, `last_error`).
   - Domain entity `Publication` with CSPRNG base32 160-bit slug generator (AD-8).
   - `SqlitePublicationStore` in `snapdown-store`.

2. **W5-S2: Go Web Service (`apps/web-service` / `web-api`) with embedded SQLite and Staging Protocol**
   - Go service with embedded SQLite schema (`published_bundle`, `published_blob`, `web_schema_version`).
   - Credential-gated write routes:
     - `PUT /publish/{slug}`: Staged all-or-nothing publish (Markdown + images).
     - `DELETE /publish/{slug}`: Unpublish and wipe assets.
     - `GET /publish/{slug}`: Reconcile query.
   - Public read routes:
     - `GET /b/{slug}`: Content-negotiated (HTML for browsers, raw markdown for text/markdown).
     - `GET /b/{slug}/raw.md`: Explicit raw markdown path.
     - `GET /b/{slug}/images/{filename}`: Relative image resolution with path traversal guard.
   - Identical 404 refusal for unknown, revoked, and never-issued slugs (NFR-15).

3. **W5-S3: Desktop Publish Client (`LC-020`) & Tauri IPC Handlers**
   - Staged upload client in Rust communicating with `web-api` using publish credential.
   - Sticky error recording on unpublish failure (`publication.last_error`).
   - Automatic unpublish trigger on bundle deletion (BR-23).
   - Tauri IPC commands: `publish_bundle`, `unpublish_bundle`, `get_publication_status`, `reconcile_publication`.

4. **W5-S4: Web UI Reader SPA (`apps/web-reader` / `@snapdown/ui` bundle-reader screen)**
   - Single-page React reader (Screens 14 & 15).
   - Clean client-side rendering of published Markdown and embedded images.
   - Refused state screen matching Screen 15.

5. **W5-S5: Publish UI Modal / Dialog in Desktop App & End-to-End Golden Verification**
   - Publish dialog (`PublishModal.tsx` / Screen 11) with clear confirmation wording (BR-86).
   - Bundle list publication badges and last-known status indicators (BR-98, BR-99).
   - End-to-end integration tests between Desktop publish client, Go web-service, and Web UI reader.
