---
type: inventory
kind: api
scope: _platform
status: draft
created: "2026-08-22"
updated: "2026-08-22"
derived_from: plan
verified: ""
---

# Inventory — endpoints

Three surfaces, one list. Not one of them accepts a write from outside the desktop process (AD-5).

- **Local API** — `desktop-app`, bound to `127.0.0.1` only, every request carrying the Access Key
  (NFR-9). Prefix `/v1`.
- **MCP** — `mcp-bridge`, Model Context Protocol tools over stdio. Not HTTP; listed here because it
  is the surface an agent actually calls, and leaving it out would make the API record incomplete.
- **Web** — `web-api` on the Reviewer's host. Public routes serving one Publication, plus two
  credential-gated routes the desktop publish client uses.

`No` is stable. A new row takes the next number; a removed one keeps its number with
`status: removed`.

## Rows

| No | Method | Path | Owning component | Description | Status |
| --- | --- | --- | --- | --- | --- |
| 1 | GET | `/v1/health` | `agent-access` | Local API liveness. The only route that does not require the Access Key, and it returns nothing about the Library | draft |
| 2 | GET | `/v1/bundles` | `agent-access` | List Bundles: id, name, Finding count, composed-at. Refuses with `key_required` when no valid key is presented, which is distinguishable from an empty list (AD-7) | draft |
| 3 | GET | `/v1/bundles/{id}` | `agent-access` | One Bundle's stored Markdown — the same authored document *Copy Markdown* serves, with the stored folder-relative image links rather than a rebased set (AD-9, DEC-012) — plus the list of image filenames it references | draft |
| 4 | GET | `/v1/bundles/{id}/images/{filename}` | `agent-access` | One image belonging to that Bundle. Refuses any filename that escapes the Bundle's own folder | draft |
| 5 | TOOL | `mcp:list_bundles` | `agent-access` | MCP tool over row 2 | draft |
| 6 | TOOL | `mcp:read_bundle` | `agent-access` | MCP tool over row 3. Returns the Markdown as text | draft |
| 7 | TOOL | `mcp:read_bundle_image` | `agent-access` | MCP tool over row 4. Returns the image as an MCP image content block | draft |
| 8 | TOOL | `mcp:set_access_key` | `agent-access` | Accepts the Access Key the Reviewer pasted, for the lifetime of this bridge process only. The bridge persists nothing (AD-5) | draft |
| 9 | GET | `/b/{slug}` | `sharing` | A Publication. Raw Markdown when the client asks for `text/markdown` or `text/plain`; an HTML document `web-api` renders itself when a browser asks for HTML (`DEC-015`). Same bytes of Markdown either way | draft |
| 10 | GET | `/b/{slug}/raw.md` | `sharing` | The same Markdown, unambiguously, for a client that will not negotiate content types | draft |
| 11 | GET | `/b/{slug}/images/{filename}` | `sharing` | One image of that Publication. Resolves relative to row 9's document, which is what makes the Markdown's relative paths work | draft |
| 12 | PUT | `/publish/{slug}` | `sharing` | Create or replace one Publication: its Markdown and its images, in one request that either completes or leaves nothing (FR-23). Requires the publish credential | draft |
| 13 | DELETE | `/publish/{slug}` | `sharing` | Remove a Publication's Markdown and images, after which row 9 refuses identically to an unknown slug (NFR-15). Requires the publish credential | draft |
| 14 | GET | `/publish/{slug}` | `sharing` | Whether that slug is currently served, so the desktop can reconcile a Publication whose last unpublish failed (FR-25, FR-26). Requires the publish credential | draft |

There is no route that lists, searches, or enumerates Publications, on any surface. Its absence is
what NFR-15 checks, so a row appearing here later is a violation rather than an addition.

There is no write route on rows 1–11. Rows 12–14 are the desktop's own publish client talking to its
own service with a credential the Reviewer configured, which is the one outbound path AD-6 permits.

No row is owned by `_platform`.

## Findings

None — `derived_from: plan`, and there is no code to derive from yet.
