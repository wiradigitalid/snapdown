---
id: SPEC-W4
wave: W4
title: Agent access complete — the key, the Local API, the MCP bridge
status: draft
created: 2026-08-23
companions:
  - .control/registry/index.yaml
  - .control/registry/components.yaml
  - .control/product-glossary.md
  - .what/agent-access/SRS-agent-access.md
  - .how/agent-access/SDD-agent-access.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/c4-l2-containers.md
  - .how/_platform/inventory-api.md
  - .how/_platform/inventory-db.md
  - .how/_platform/inventory-screen.md
  - .how/_platform/cross-cutting.md
  - .control/decisions/DEC-002-mcp-bridge-and-key.md
sources:
  - .what/_prd/agent-handoff/prd.md
  - .control/registry/requirements.yaml
  - .control/registry/usecases.yaml
  - .control/registry/waves.yaml
---

# Wave W4 Specification Contract: Agent Access

## Why

This wave implements the door an external AI coding agent reads Bundles through, and the secret key that opens it. It satisfies **CAP-7** (FR-19, FR-20, FR-21, FR-22, UC-17, UC-18, UC-19), binding **AD-4** (no image re-encoding), **AD-5** (strictly read-only external interfaces), **AD-7** (cross-cutting error envelope), and **AD-9** (verbatim stored Markdown serving).

The architecture enforces a clean separation across two processes:
1. **Desktop App Local API**: A read-only HTTP server listening strictly on loopback (`127.0.0.1`) gating `/v1/bundles`, `/v1/bundles/{id}`, and `/v1/bundles/{id}/images/{filename}` via constant-time hash comparison of an `AccessKey` stored in `library.db` and the Windows Credential Store (`keyring`/`winreg`).
2. **Stateless MCP Bridge**: A dedicated `snapdown-bridge` (or `mcp-bridge`) binary speaking stdio Model Context Protocol (JSON-RPC) to agent hosts, accepting `set_access_key` in memory only, and forwarding read operations (`list_bundles`, `read_bundle`, `read_bundle_image`) to the Local API.

## Stories Breakdown

1. **W4-S1: SQLite Schema Migration v4 (`access_key`), `AccessKey` domain entity, `AccessKeyStore` & `AccessKeyManager`**
   - Schema migration v4 creating `access_key` (`id`, `key_hash`, `issued_at`, `revoked_at`).
   - Domain entity `AccessKey` & port `AccessKeyStore` in `snapdown-core`.
   - `SqliteAccessKeyStore` and `AccessKeyManager` with constant-time hash verification and Windows Credential Store integration (with fallback stub for tests/headless environments).

2. **W4-S2: Local API HTTP Server (`127.0.0.1`) with Constant-Time Auth and Route Handlers**
   - Lightweight loopback HTTP server (`tiny_http`) bound to `127.0.0.1` in Tauri/service background.
   - Routes: `GET /v1/health`, `GET /v1/bundles`, `GET /v1/bundles/:id`, `GET /v1/bundles/:id/images/:filename`.
   - Gated by constant-time `AccessKey` verification; returns standard error envelopes (`key_required`, `key_invalid`, `not_found`, `bad_request`, `unavailable`).
   - Traversal protection for bundle image file paths.

3. **W4-S3: Stateless stdio MCP Bridge (`snapdown-bridge` crate)**
   - Standalone CLI crate `crates/snapdown-bridge` implementing JSON-RPC / Model Context Protocol over stdio.
   - Tools: `mcp:set_access_key`, `mcp:list_bundles`, `mcp:read_bundle`, `mcp:read_bundle_image`.
   - Communicates with Local API over HTTP, holding key strictly in-memory (no persistence).
   - Preserves error envelope verbatim and preserves refusal vs empty result distinction.

4. **W4-S4: Settings — Agent Access UI Screen (`AgentAccessView.tsx` / `/settings/agent-access`) & IPC Commands**
   - UI screen at `/settings/agent-access` in `@snapdown/ui` and `apps/desktop`.
   - State indicators (Active Key timestamp, Validated Status, Revoked Status).
   - Actions: Generate Access Key, Copy / Re-copy to Clipboard, Revoke Key.
   - Tauri IPC commands: `generate_access_key`, `get_access_key_status`, `copy_access_key`, `revoke_access_key`.

5. **W4-S5: Integration & Verification Test Suite**
   - End-to-end integration tests: Refusal (`key_required`, `key_invalid`) vs empty bundle list distinction.
   - Timing attack resistance (constant-time verification).
   - MCP Bridge stdio tool calling suite via mock/real Local API.
   - Loopback-only binding isolation and Golden Markdown / image payload verification.

## Verification Suite

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix web/ui run typecheck
npm --prefix web/ui run lint
npm --prefix web/ui run test
npm --prefix apps/desktop run typecheck
npm --prefix apps/desktop run lint
npm --prefix apps/desktop run test
npm --prefix apps/desktop run build
```
