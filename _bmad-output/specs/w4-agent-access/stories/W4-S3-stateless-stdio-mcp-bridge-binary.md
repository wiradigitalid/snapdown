---
id: W4-S3
title: Stateless stdio MCP bridge binary (snapdown-bridge)
wave: W4
status: done
created: 2026-08-23
dependencies: [W4-S2]
files:
  - crates/snapdown-bridge/Cargo.toml
  - crates/snapdown-bridge/src/main.rs
  - crates/snapdown-bridge/src/mcp.rs
  - crates/snapdown-bridge/src/client.rs
  - Cargo.toml
  - crates/snapdown-bridge/tests/test_bridge_mcp.rs
---

# W4-S3: Stateless stdio MCP bridge binary (snapdown-bridge)

## User Story
As an AI coding agent running on the reviewer's machine, I want an MCP (Model Context Protocol) stdio bridge executable that translates JSON-RPC tool calls into Local API HTTP requests without persisting credentials to disk.

## Acceptance Criteria
- [ ] Create workspace binary crate `crates/snapdown-bridge`.
- [ ] Implement stdio JSON-RPC parser supporting standard MCP handshake/capabilities and tools:
  - `mcp:set_access_key` (stores key in memory for lifetime of process).
  - `mcp:list_bundles` (calls `/v1/bundles`).
  - `mcp:read_bundle` (calls `/v1/bundles/:id`).
  - `mcp:read_bundle_image` (calls `/v1/bundles/:id/images/:filename` and returns base64 image block).
- [ ] Bridge handles connection refused / desktop not running immediately with `unavailable` (never hangs/infinite loop).
- [ ] Preserves upstream error envelopes as MCP errors verbatim.
- [ ] Zero persistence: holds no disk cache, logs no secret keys.
- [ ] Comprehensive unit and stdio-driven integration tests.
