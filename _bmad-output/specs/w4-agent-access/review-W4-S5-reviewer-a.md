# Code Review: W4-S5 (Integration verification suite for agent access and refusal envelopes)

## Metadata
- **Story**: W4-S5
- **Author**: Amelia (Worker)
- **Reviewer**: Reviewer A (Orchestrator)
- **Verdict**: `ACCEPTED`
- **Date**: 2026-08-23

## Verification Checkpoints
1. **End-to-End Key & API Flow**: Integrates `SqliteAccessKeyStore`, `LocalApiServer`, and `VaultBlobStore` verifying unauthenticated `/v1/health` answers with required `X-Snapdown-Service: local-api` header without revealing Library state (BR-79).
2. **Refusal Envelope Assertions**: Verifies that requests without AccessKey return 401 `key_required` JSON error envelopes, distinguishing refusal from an empty library result (AD-7, BR-77, RISK-6).
3. **Authorized Reads**: Verifies bundle listing, verbatim Markdown serving (`/v1/bundles/:id`), and raw byte image serving (`/v1/bundles/:id/images/:filename`) match stored payload contracts (AD-4, AD-9, BR-82, BR-83).
4. **Security & Path Traversal**: Verifies path traversal attempts (`..`, escaped paths) are rejected with 400 `bad_request` (BR-84).
5. **Immediate Revocation**: Verifies that revoked keys result in immediate 401 `key_invalid` without latency or grace periods (NFR-13, BR-77).
6. **Full Test Pass**: Workspace tests passing across all crates.

## Decision
Verdict: `ACCEPTED`
