---
id: W7-S3
title: 'W7-S3: Never hand an agent an error with no message in it'
type: 'bug'
wave: W7
status: ready-for-dev
created: '2026-08-24'
review_loop_iteration: 0
followup_review_recommended: false
dependencies:
  - W7-S2
files:
  - crates/snapdown-bridge/src/client.rs
  - crates/snapdown-bridge/tests/test_bridge_mcp.rs
context:
  - _bmad-output/specs/w7-failure-paths/SPEC.md
  - _bmad-output/specs/w7-failure-paths/stories.yaml
  - _bmad-output/specs/w7-failure-paths/dispatch-briefs/W7-S3-step1-plan.md
  - .what/business-rules.md
  - .what/agent-access/SRS-agent-access.md
  - .what/agent-access/02-rules/rules-agent-access.md
  - .what/agent-access/04-usecases/UC-17-let-the-agent-in-front-of-me-read-my-reviews.md
  - .what/agent-access/04-usecases/UC-18-read-a-review-from-inside-my-agent.md
  - .control/registry/defects.yaml
  - .control/registry/waves.yaml
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/cross-cutting.md
  - .how/agent-access/SDD-agent-access.md
  - .control/decisions/DEC-003-one-process-two-windows.md
  - .control/decisions/DEC-005-desktop-first-ordering.md
  - .constitution/project/codebase-stack-guide.md
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:**
In `crates/snapdown-bridge/src/client.rs:155-163`, `parse_error_response` ignores the `Result` of reading the response body using `let _ =`:

```rust
fn parse_error_response(_code: u16, resp: ureq::Response) -> String {
    let mut body = String::new();
    let _ = resp.into_reader().read_to_string(&mut body);

    if let Ok(env) = serde_json::from_str::<ApiErrorEnvelope>(&body) {
        format!("{}: {}", env.error.code, env.error.message)
    } else {
        body
    }
}
```

When reading the error response body fails (e.g. dropped network connection mid-response or non-UTF-8 stream bytes), `body` remains empty (`""`), `serde_json::from_str` fails on the empty string, and the fallback branch returns `body` (`""`). The caller wraps this empty string in `Err("")`, which surfaces through the MCP protocol as an empty error string (`{"isError": true, "content": [{"type": "text", "text": ""}]}`).

This directly contradicts:
- `AD-7` (*One error envelope across every process boundary*): *"A refusal MUST be distinguishable from an empty result by its code, never only by its body being empty."*
- `BR-17`: *"A refusal is always distinguishable from an empty result. 'No Access Key' and 'no Bundles' are never the same answer."*

An error whose message is the empty string is a refusal that says nothing — the agent learns only that a tool call failed with no actionable reason or status.

**Severity & Real-World Reality:**
This is honestly low severity (`BUG-10`). Because it sits on the error path of an HTTP call that has already returned an error status code, reaching it requires a second failure (such as a broken connection mid-stream or invalid UTF-8 body bytes). It does not cause data loss. However, it is in Wave W7 because it represents the third instance of a single harmful habit (`let _ =` on a `Result` that an invariant depends on), alongside `BUG-9` and `W6-S10`.

**Deliberate Scope Boundaries:**
- `main.rs:21-22` swallows stdout write and flush in the stdio loop (`let _ = writeln!(stdout, ...)` and `let _ = stdout.flush()`). As documented in `BUG-10` and `SPEC.md`, this is defensible because there is no other channel to report a stdout failure over. It is intentionally left untouched and must NOT be widened into.
- `DEC-005` freezes `agent-access` and permits this fix by its own terms (*"This decision does not forbid a fix. It forbids new work"*). No new FRs, use cases, MCP tools, or routes are introduced.

**Approach & Architectural Shape:**
1. **Refactor `parse_error_response` with a Reader Seam:**
   - Decompose `parse_error_response` to accept any reader implementing `std::io::Read`:
     ```rust
     pub(crate) fn parse_error_response_reader<R: std::io::Read>(code: u16, mut reader: R) -> String {
         let mut body = String::new();
         if let Err(e) = reader.read_to_string(&mut body) {
             return format!("HTTP {code}: (failed to read error response: {e})");
         }

         if let Ok(env) = serde_json::from_str::<ApiErrorEnvelope>(&body) {
             format!("{}: {}", env.error.code, env.error.message)
         } else if body.trim().is_empty() {
             format!("HTTP {code}: (empty error response)")
         } else {
             body
         }
     }

     fn parse_error_response(code: u16, resp: ureq::Response) -> String {
         parse_error_response_reader(code, resp.into_reader())
     }
     ```
2. **Guarantee Honest Error Formatting on All Paths:**
   - When `read_to_string` fails, return a message explicitly stating the HTTP status `code` and that the error response body could not be read.
   - When `read_to_string` succeeds but the body parses into `ApiErrorEnvelope`, format as `"{code}: {message}"` per `AD-7`.
   - When `read_to_string` succeeds but is not JSON and has non-empty text, return the raw `body`.
   - When `read_to_string` succeeds on an empty or whitespace-only response body, return `format!("HTTP {code}: (empty error response)")` rather than `""`.
   - Under no circumstances does `parse_error_response` or `parse_error_response_reader` return an empty string (`""`).
3. **Implement Required Unit & Integration Tests:**
   - In `crates/snapdown-bridge/src/client.rs` (or `tests/test_bridge_mcp.rs`), implement tests exercising:
     - `cargo::a_failed_error_body_read_never_yields_an_empty_message` (using a mock `Read` returning an `io::Error` or broken connection).
     - `cargo::the_status_code_survives_a_failed_error_body_read` (asserting that status code, e.g. 500, 502, 401, is contained in the returned error string).
     - `cargo::a_readable_error_envelope_is_still_parsed_as_before` (regression guard confirming `ApiErrorEnvelope` is properly formatted as `code: message`).

## Boundaries & Constraints

**Always:**
- `parse_error_response` MUST NEVER return an empty string (`""`) on any execution path (`AD-7`, `BR-17`, `BUG-10`).
- On a failed body read, the returned error message MUST include the HTTP status code and state that reading the error body failed.
- Valid `ApiErrorEnvelope` JSON bodies MUST continue to parse into `"{code}: {message}"` format (`AD-7`).
- All 3 named tests from `waves.yaml` MUST be implemented and pass in `cargo test --workspace`.

**Block If:**
- Upstream requirements in `.what/` or `.how/` contradict error formatting.

**Never:**
- Never use `let _ =` on `read_to_string` or error-body reading.
- Never touch or modify `main.rs:21-22` (the stdio loop stdout writes are explicitly left alone).
- Never add new tools, use cases, or requirements to `agent-access` (frozen by `DEC-005`).
- Never modify files in `.what/`, `.how/`, or `.constitution/`.

## I/O & Edge-Case Matrix

| Scenario | Input (Code + Reader/Body) | Expected Output Message | Invariants / Guarantees |
|---|---|---|---|
| Valid JSON Error Envelope | `code: 401`, body: `{"error":{"code":"key_required","message":"Key required","detail":null,"request_id":""}}` | `"key_required: Key required"` | Preserves `AD-7` cross-cutting error envelope format |
| Failed Reader (I/O Error) | `code: 502`, reader fails with `io::ErrorKind::ConnectionReset` | `"HTTP 502: (failed to read error response: connection reset)"` (non-empty) | Non-empty message; includes status code `502` |
| Failed Reader (Non-UTF8) | `code: 500`, reader yields invalid UTF-8 bytes (`&[0xFF, 0xFE]`) | `"HTTP 500: (failed to read error response: ...)"` (non-empty) | Non-empty message; includes status code `500` |
| Empty Body (Success Read) | `code: 404`, body: `""` | `"HTTP 404: (empty error response)"` | Non-empty message; includes status code `404` |
| Raw Plaintext Body | `code: 503`, body: `"Service Unavailable"` | `"Service Unavailable"` | Preserves raw server error text if not JSON |

</intent-contract>

## Code Map

- `crates/snapdown-bridge/src/client.rs` -- Refactor `parse_error_response` to delegate to `parse_error_response_reader`, eliminate `let _ =`, ensure honest error formatting with HTTP status code on read failures, and add unit tests.
- `crates/snapdown-bridge/tests/test_bridge_mcp.rs` -- Integration tests proving end-to-end MCP tool error delivery retains honest messages for both readable envelopes and failing body streams.

## Tasks & Acceptance

**Execution:**
- `crates/snapdown-bridge/src/client.rs` -- Define `parse_error_response_reader<R: std::io::Read>(code: u16, mut reader: R) -> String` handling `Err` on `read_to_string` and empty bodies.
- `crates/snapdown-bridge/src/client.rs` -- Update `parse_error_response` to call `parse_error_response_reader(code, resp.into_reader())`.
- `crates/snapdown-bridge/src/client.rs` (or `crates/snapdown-bridge/tests/test_bridge_mcp.rs`) -- Implement test `cargo::a_failed_error_body_read_never_yields_an_empty_message` proving a failing body read returns a non-empty string.
- `crates/snapdown-bridge/src/client.rs` (or `crates/snapdown-bridge/tests/test_bridge_mcp.rs`) -- Implement test `cargo::the_status_code_survives_a_failed_error_body_read` proving HTTP status code is present in error output on read failure.
- `crates/snapdown-bridge/src/client.rs` (or `crates/snapdown-bridge/tests/test_bridge_mcp.rs`) -- Implement test `cargo::a_readable_error_envelope_is_still_parsed_as_before` proving standard JSON error envelopes parse correctly.

**Acceptance Criteria:**
- Given an HTTP error response whose body reader fails (e.g. I/O error or non-UTF-8 bytes), `parse_error_response` returns a non-empty error message containing the HTTP status code and an explanation that reading the body failed.
- Given an HTTP error response with an empty body, `parse_error_response` returns a non-empty error message containing the HTTP status code.
- Given a valid `ApiErrorEnvelope` response (e.g. `{"error":{"code":"key_required","message":"Key required",...}}`), `parse_error_response` returns `"key_required: Key required"`.
- Given `cargo test -p snapdown-bridge` and `cargo test --workspace`, all tests pass with 0 failures and clippy reports 0 warnings.

## Spec Change Log

<!-- Append-only. Populated during review loops. -->

## Design Notes

**Why factor out `parse_error_response_reader`:**
`ureq::Response` can only be constructed via HTTP network responses or internal ureq builders. Factoring the core logic to accept generic `R: std::io::Read` allows deterministic, fast in-memory unit tests with custom failing reader implementations (simulating connection drops and non-UTF-8 payloads) without requiring live socket listeners or flaky network simulation.

## Verification

**Commands:**
- `cargo test -p snapdown-bridge` -- expected: All snapdown-bridge unit and integration tests pass.
- `cargo test --workspace` -- expected: Full workspace test suite passes.
- `cargo clippy --workspace --all-targets -- -D warnings` -- expected: Zero warnings.
- `cargo fmt --all -- --check` -- expected: Code formatting is clean.
