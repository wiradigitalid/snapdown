---
type: cross-cutting
scope: _platform
status: draft
created: "2026-08-22"
updated: "2026-08-22"
---

# Cross-Cutting — Snapdown

What is defined once for the whole product. This file describes; it does not forbid. Anything that
forbids is an `AD-N` in `ARCHITECTURE-SPINE.md`.

Three process boundaries exist — the Local API, the MCP Bridge, and `web-api` — and AD-7 requires all
three to speak the shape below.

## Error envelope

```json
{
  "error": {
    "code": "key_required",
    "message": "An Access Key is required. Paste the key from Snapdown.",
    "detail": null,
    "request_id": "0192f3c1-8a7e-7c31-9b44-3d2f1a6c5e08"
  }
}
```

| Field | Type | Means | Always present |
| --- | --- | --- | --- |
| `error.code` | string, `snake_case`, from the catalogue below | What went wrong, in a form a caller can branch on | yes |
| `error.message` | string, English, one sentence | What a person reading the agent's output should understand. Never carries Finding, Note, or Bundle content | yes |
| `error.detail` | object or null | Structured extra facts for exactly one code — the field that failed validation, the filename that was refused. Never free-form prose | no |
| `error.request_id` | string, UUIDv7 | Correlates the response with the one log line the producing side wrote | yes |

Two rules follow from AD-7 and are worth stating where the shape is:

- A **refusal** always carries a code. It is never an empty success. `key_required` and an empty
  Bundle list are different answers, and an agent that cannot tell them apart reports to the Reviewer
  that their Library is empty.
- The MCP Bridge does not invent its own errors. It maps a Local API envelope onto an MCP tool error,
  preserving `code` and `message` verbatim.

## Error catalogue

| Code | HTTP | Means | Caller should |
| --- | --- | --- | --- |
| `key_required` | 401 | No Access Key was presented | Tell the Reviewer to paste the key from Snapdown. Do not retry |
| `key_invalid` | 401 | A key was presented but is not the currently valid one — wrong, or revoked | Tell the Reviewer the key is no longer valid and a new one is needed. Do not retry |
| `not_found` | 404 | The Bundle, image, or slug does not exist — or once did and no longer does. The two are deliberately indistinguishable (NFR-15) | Stop. Do not probe for a nearby id |
| `not_allowed` | 403 | The operation exists but is not permitted on this surface. Every write on a read-only surface lands here (AD-5) | Stop. This is a defect in the caller, not a permission to acquire |
| `bad_request` | 400 | The request is malformed — a filename that escapes its folder, a missing field. `detail` names it | Fix the request. Do not retry unchanged |
| `unavailable` | 503 | Snapdown is not running, or `web-api` cannot reach its store | Tell the Reviewer what is not running. Retry is reasonable |
| `publish_failed` | 502 | The publish or unpublish did not complete. Nothing partial was left behind | Surface the reason. The Bundle's local state is unchanged, and an unpublish failure keeps it marked published (FR-25) |
| `conflict` | 409 | The slug is already served by a different Bundle | Stop. Slugs are never reused (AD-8) |
| `internal` | 500 | Something the producer did not anticipate | Surface `request_id` and stop |

`web-api` returns `not_found` for an unknown slug, a revoked slug, and a slug that was never issued —
the same status, the same code, the same body. That equality is NFR-15's second half.

## Platform-owned

Nothing. `platform_owns` in `components.yaml` is empty and no inventory row is owned by `_platform`.

Every table, endpoint, and screen in this product exists because one Product Component's `FR`
promises it, so the test in `corpus-guide.md` — no single component's promise explains it, **and** more
than one component depends on it — fails on the first half every time. The two tables that look
platform-shaped, `schema_version` and `web_schema_version`, are owned by `settings` and `sharing`
respectively: they are the migration level of a store one component is responsible for opening, not a
shared concern.

## Other product-level agreements

### Timestamps

**Applies to:** all — `desktop-app`, `mcp-bridge`, `web-api`.
**Enforced by:** a serialisation test asserting every timestamp field matches RFC 3339 with a `Z`
offset, run over the Local API responses, the published Markdown, and the publish request body.

Every timestamp is stored and transmitted in UTC as RFC 3339 with an explicit `Z`. Local time exists
only in what a UI renders for a person. A Bundle's composed-at in its Markdown is UTC, which is why
the same Bundle read on the desktop and on the server says the same thing.

### Colour, theme, and contrast

**Applies to:** every surface that draws — `desktop-app`. (`web-ui` was listed here until
`DEC-015` withdrew that container; `web-api`'s published page carries no palette of its own.)
**Enforced by:** two Rust guards over the shipped palette — `apps/desktop/tests/test_theme_contrast.rs`
measures WCAG contrast over every token in both themes, and `apps/desktop/tests/test_capture_interaction.rs`
refuses a colour literal in the overlay.

**Corrected 2026-09-01.** This paragraph named *"a lint rule refusing a colour literal outside the token
stylesheet"* plus a contrast assertion over `prefers-color-scheme`, and gave the authority as the token
stylesheet inside `web/ui`. Both halves were browser-shaped and neither reached the product: `DEC-007`
moved the UI to Slint, which has no `prefers-color-scheme`, and `OQ-27` deleted `web/ui` entirely on
2026-09-01. The lint had never covered a Slint file, which is how nine colour literals accumulated
across them undetected until `DEC-009`'s study found them on 2026-08-27.

Colour has one authority — `apps/desktop/ui/theme.slint` — and every colour is defined for both the
Windows light and the Windows dark theme (AD-10, NFR-16, NFR-17). A component contains no colour
literal. Meaning colours come in pairs, a background and the foreground proven against it, so the
pair is checked once rather than at each use.

Three token groups are **theme-invariant on purpose**, and each says so where it is defined:
`--color-marker*`, the capture overlay's scrim and region ring, and the canvas transparency
checkerboard. Each is drawn over the Reviewer's own screen content, or over an exported image that
will be read on another machine under another theme. This machine's theme is the wrong reference for
them, and the exception is documented so a later pass does not "fix" the inconsistency.

A theme change while the application is running is honoured without a restart.

This agreement exists because the product shipped its opposite. `finding` and `bundle` paint light
panels unconditionally; the shell paints text from theme-following tokens. Under the dark theme they
meet and produce white on white. Neither component is wrong alone.

### State a control does not yet know

**Applies to:** `desktop-app`.
**Enforced by:** a test asserting that a control bound to operating-system state renders its unknown
state until the read resolves.

A control reporting state owned by the operating system — the Windows startup registration is the only
one today — renders a distinct *not yet known* state until that state has been read. It MUST NOT
render an assumed value first (BR-108, FR-18). The shipped build assumes enabled and repaints to
disabled, and the Reviewer watches the product change its mind about its own state.

This is an agreement rather than one control's detail because the next such control will be written
by someone who did not read `FR-18`.

### Identifiers

**Applies to:** all.
**Enforced by:** a generation helper in `snapdown-core` used by every writer, plus a test asserting
no id is produced anywhere else.

Every entity id is a UUIDv7 as a lowercase hyphenated string. Sortable by creation, opaque to the
reader, generated by the writer with no coordination. A Publication slug is **not** an id and is
generated separately, per AD-8.

### Bundle Markdown shape

**Applies to:** `bundle`, `agent-access`, `sharing`.
**Enforced by:** the single composer in `snapdown-core`, plus a golden-file test asserting its output
against a stored reference — `crates/snapdown-store/tests/test_golden_markdown.rs`.

One Bundle produces one Markdown document, composed once and stored (AD-9). Its shape:

**Corrected 2026-08-31.** The Enforced-by line used to read *"plus a golden-file test asserting the
clipboard, Local API, and published bytes are identical."* It was wrong on two counts. There is one
golden-file test and it pins the composer's output against a reference, not three surfaces against
each other — and two of the three surfaces have no code: the clipboard-Markdown path is unimplemented,
and the Local API does not exist (`BUG-59`). `DEC-012` also retired the identity claim itself: a path
MAY render the image links against its own base. **The shape below is the STORED document's shape**,
which keeps folder-relative links — that is `NFR-8`, and `DEC-012` does not touch it. A handoff path
rendering those links against a different base is serving this same document, not a second one.

```markdown
# {Bundle name}

_Snapdown · {N} findings · composed {RFC 3339 UTC}_

## 1. {first line of the Note, or "Finding 1" when the Note is empty}

![finding 1](images/{finding-image-filename})

{the Note's body}

1. {marker 1 comment}
2. {marker 2 comment}

## 2. …
```

CommonMark only. No HTML, no extensions, no front matter. Image references are relative to the
document's own folder, which is what makes the same bytes work on disk, over the Local API, and under
a Publication URL.

### Logging

**Applies to:** all.
**Enforced by:** a structured logger with a fixed field set, plus a test asserting no log line
carries a `body`, `comment`, `markdown`, or image byte field.

One event per line, structured, with `ts`, `level`, `event`, `request_id`, and event-specific fields.
No Finding, Note, Marker, or Bundle content in any field — not truncated, not hashed, not "just for
debugging". `web-api` does not log the content it serves and does not retain a log pairing a slug with
a client address longer than it needs to operate.

### Secrets

**Applies to:** `agent-access`, `sharing`, `settings`.
**Enforced by:** the Windows credential store on the desktop and process environment on the server,
plus a repository scan in CI.

The Access Key and the publish credential are the only two secrets in the product. On the desktop
both live in the Windows credential store; `library.db` holds only a hash of the Access Key. On the
host the publish credential arrives in the environment. Neither is written to a configuration file, a
log line, a Bundle, a published document, or this repository.

### Configuration

**Applies to:** `desktop-app`, `mcp-bridge`, `web-api`.
**Enforced by:** one loader per program that reads the file then applies environment overrides, and
fails to start on an unknown key rather than ignoring it.

One TOML file per program, with environment variables overriding it. `mcp-bridge` is the exception
worth naming: it takes the Local API address from configuration and the Access Key from the
`set_access_key` tool call only, holding it for the life of the process and never writing it down.
