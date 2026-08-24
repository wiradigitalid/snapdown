# W7-S3 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W7**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w7-failure-paths/`
- `story_id`: `W7-S3`

## `BUG-10` — the bridge can hand an agent an error with an empty message

`crates/snapdown-bridge/src/client.rs:155-163`:

```rust
fn parse_error_response(_code: u16, resp: ureq::Response) -> String {
    let mut body = String::new();
    let _ = resp.into_reader().read_to_string(&mut body);
    if let Ok(env) = serde_json::from_str::<ApiErrorEnvelope>(&body) { ... } else { body }
}
```

If the body read fails, `body` stays `""`, the envelope parse fails, and the function returns the
**empty string** as the error message. The caller wraps it in `Err()` and the MCP tool surfaces an
error with nothing in it.

## What it contradicts, and the fit here is exact

`AD-7` — *One error envelope across every process boundary*:

> Every failure crossing a process boundary MUST be returned in the envelope defined in
> `cross-cutting.md`, carrying a code from that file's catalogue. **A refusal MUST be distinguishable
> from an empty result by its code, never only by its body being empty.**

`BR-17`:

> A refusal is always distinguishable from an empty result. "No Access Key" and "no Bundles" are
> never the same answer.

An error whose message is the empty string is a refusal that says nothing — the agent learns only
that something failed. `AD-7`'s **Prevents** clause names the outcome this protects against: an agent
receiving an empty answer, and reporting to the Reviewer that their Library is empty.

**`AD-7` binds the SHAPE and this story does not change it.** The envelope and its code catalogue
stay as they are; what changes is the message inside.

## The fix

On a failed read, return a message that **says so** — the status code plus that the error body could
not be read. Never return an empty error string from any path through `parse_error_response`.

## Honestly low, and the plan should reflect that rather than inflate it

This is the error path of a call that has **already failed**, so reaching it needs a *second*
failure. The two likely readings are a **dropped connection mid-error-body** and a **non-UTF-8
body**. Neither is common and neither loses data. Write the tests against those two readings; do not
manufacture a dramatic scenario the code cannot actually reach.

It is in this wave because it is the **third instance of one habit** — `let _ =` on a `Result` an
invariant depends on, after `BUG-9` and `W6-S10` — and fixing two of three leaves the pattern alive.
That habit is now a pitfall in `AGENTS.md`.

## Scope — one thing was swept and deliberately left alone

`main.rs:21-22` swallows the stdout write and flush in the stdio loop. That is **defensible**: there
is nothing useful to do about a failure on the very channel you would report it over. It is recorded
in `BUG-10`'s register entry so the next sweep does not re-raise it. **Do not widen into it.**

`DEC-005` freezes `agent-access` and permits this by its own sentence — *"This decision does not
forbid a fix. It forbids new work."* No new FR, no use case, no depth above `guarded`.

## The tests that matter

`waves.yaml` records three, carried through verbatim:

```
cargo::a_failed_error_body_read_never_yields_an_empty_message
cargo::the_status_code_survives_a_failed_error_body_read
cargo::a_readable_error_envelope_is_still_parsed_as_before
```

**No test exercises a failing body read** — every bridge test uses a live loopback response, which is
why this survived. The failing-reader fixture is the substance of the story.

The third test is the regression guard: the existing envelope path must still parse exactly as it
did. Assert the behaviour, not a literal.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** A failing test or build whose cause is unknown → run
  `wdi-systematic-debugging` before proposing any fix. A third failed fix attempt is the signal to
  escalate.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`.
- **Verification is run, not assumed.** Every command in `AGENTS.md` § Code. Note that `cmd | tail`
  reports `tail`'s exit code, and `cmd; echo "EXIT=$?"` makes the harness report 0 whatever `cmd`
  did.
- **Write UTF-8, and no BOM.**
- **Never commit a captured screenshot.** No scratch files in the commit. **Do not push.**

## Done means

`_bmad-output/specs/w7-failure-paths/stories/W7-S3-*.md` exists, carries an `<intent-contract>`, and
its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
