---
id: W4-S5
title: Integration verification suite for agent access and refusal envelopes
wave: W4
status: planned
created: 2026-08-23
dependencies: [W4-S1, W4-S2, W4-S3, W4-S4]
files:
  - tests/integration/test_agent_access_e2e.rs
  - apps/desktop/src-tauri/tests/test_agent_access_integration.rs
---

# W4-S5: Integration verification suite for agent access and refusal envelopes

## User Story
As an engineer maintaining Snapdown, I want a complete end-to-end integration test suite verifying that external agents can read bundles correctly when authorized, receive distinct refusal envelopes when unauthorized, and that the MCP bridge translates these behaviors faithfully without data leakage.

## Acceptance Criteria
- [ ] End-to-end flow: Issue key via AccessKeyManager -> Launch Local API -> Run Bridge CLI with stdio -> Call `set_access_key` -> `list_bundles` -> `read_bundle` -> `read_bundle_image`.
- [ ] Refusal verification: Assert distinct HTTP 401 `key_required` vs `key_invalid` responses (never empty 200).
- [ ] Immediate revocation test: Revoke key -> Next bridge tool call fails immediately with `key_invalid`.
- [ ] Security test: Attempt path traversal (e.g. `../../secret.png`) via image route and assert `bad_request` (400) or `not_found` (404).
- [ ] Automated regression pass across all workspace crates and web packages.
