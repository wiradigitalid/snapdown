# RTR-W4: Agent Access (Wave W4 Retrospective)

## Wave Information
- **Wave**: W4
- **Title**: Agent access complete — the key, the Local API, the MCP bridge
- **Status**: Completed
- **Date Closed**: 2026-08-23
- **Delivered Stories**: W4-S1, W4-S2, W4-S3, W4-S4, W4-S5

## Scope Delivered
1. **W4-S1**: SQLite Schema Migration v4 (`access_key`), `AccessKey` domain model with constant-time SHA-256 validation, `AccessKeyStore` port trait, and `SqliteAccessKeyStore` repository enforcing single-active-key invariant.
2. **W4-S2**: Local API loopback HTTP server (`127.0.0.1`) with unauthenticated `/v1/health` (revealing zero library state), authenticated read-only `/v1/bundles`, `/v1/bundles/:id` (serving verbatim Markdown), and `/v1/bundles/:id/images/:filename` with path traversal guards and standard JSON error envelopes.
3. **W4-S3**: `snapdown-bridge` dedicated workspace binary crate implementing Model Context Protocol (JSON-RPC 2.0) over stdio with tools `mcp:set_access_key`, `mcp:list_bundles`, `mcp:read_bundle`, `mcp:read_bundle_image`, zero disk persistence, and fast failure when Local API is offline.
4. **W4-S4**: Settings Agent Access panel (`AgentAccessView.tsx`) in `@snapdown/ui` and Desktop app with Tauri IPC commands to generate/rotate/revoke access keys without cleartext disk persistence.
5. **W4-S5**: Comprehensive end-to-end integration test suite verifying refusal envelopes vs empty results, timing attack resistance, path traversal prevention, and immediate revocation.

## Review Panel Outcomes
- All story review files (`review-W4-S1-reviewer-a.md` through `review-W4-S5-reviewer-a.md`) recorded verdict `ACCEPTED`.
- Zero compiler warnings, 100% test pass across Rust workspace and TypeScript packages (`@snapdown/ui`, `apps/desktop`).
