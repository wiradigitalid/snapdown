---
type: inventory
kind: db
scope: _platform
status: draft
created: "2026-08-22"
updated: "2026-08-22"
derived_from: plan
verified: ""
---

# Inventory — tables

Two stores, one table list. The `library.db` rows live on the Reviewer's machine inside
`desktop-app`; the `publication.db` rows live on the host beside `web-api`. Both are embedded SQLite,
neither is a container, and a row's owning component is what decides which store it is in.

`No` is stable. A new table takes the next number; a removed one keeps its number with
`status: removed` and the number is never reused.

## Rows

| No | Table | Owning component | What it holds | Key columns | Status |
| --- | --- | --- | --- | --- | --- |
| 1 | `finding` | `finding` | One observation. The row exists only while its image file does (AD-2) | `id` UUIDv7 pk · `image_path` relative to the Vault · `image_width` · `image_height` · `captured_at` · `source_monitor` · `region` | draft |
| 2 | `note` | `finding` | The prose body of one Finding's Note. The numbered lines are not here — they belong to `marker` (AD-1) | `id` pk · `finding_id` fk unique · `body` · `updated_at` | draft |
| 3 | `marker` | `finding` | One numbered Marker and the Note line that is the same thing as it. `ordinal` is the badge number and the line number at once | `id` pk · `finding_id` fk · `ordinal` unique per finding, from 1, no gaps · `x` · `y` normalised 0–1 (AD-3) · `comment` | draft |
| 13 | `visual_annotation` | `finding` | Visual markup elements (Shape, Callout, Blur, Arrow, Text) on canvas overlay. Does not bind to Markdown note lines. | `id` pk · `finding_id` fk · `kind` (shape, callout, blur, arrow, text) · `properties_json` (coords, dimensions, style, font, text, tail) · `created_at` | draft |
| 4 | `bundle` | `bundle` | One composed Bundle, including the composed Markdown itself so that every handoff path serves the same authored document rather than each surface composing its own (AD-9, DEC-012) | `id` pk · `name` · `markdown` · `markdown_path` relative to the Vault · `composed_at` | draft |
| 5 | `bundle_item` | `bundle` | The membership of one Finding in one Bundle, and the path of the Marker-burned image copy written for it | `id` pk · `bundle_id` fk · `finding_id` fk · `position` · `image_path` · unique on (`bundle_id`, `finding_id`) | draft |
| 6 | `publication` | `sharing` | Where a Bundle is published and whether it is still live. `slug` is generated independently of every id here (AD-8) | `id` pk · `bundle_id` fk unique · `slug` unique · `base_url` · `published_at` · `unpublished_at` nullable · `last_error` nullable | draft |
| 7 | `access_key` | `agent-access` | The one Access Key that may be valid, stored as a hash. The key itself lives in the Windows credential store, never here | `id` pk · `key_hash` · `issued_at` · `revoked_at` nullable | draft |
| 8 | `setting` | `settings` | One persisted preference per key: Vault location, each hotkey binding, the Quality Budget pair, startup, open-editor-after-capture, the web service address | `key` pk · `value` · `updated_at` | draft |
| 9 | `schema_version` | `settings` | The migration level of `library.db`, so a newer binary knows what it is opening | `version` pk · `applied_at` | draft |
| 10 | `published_bundle` | `sharing` | `web-api`'s own record of one served Publication: its slug, its Markdown, and where its blobs are. Holds no Library id (AD-8) | `slug` pk · `markdown` · `blob_dir` · `created_at` · `deleted_at` nullable | draft |
| 11 | `published_blob` | `sharing` | One image belonging to one served Publication | `id` pk · `slug` fk · `filename` · `content_type` · `byte_size` | draft |
| 12 | `web_schema_version` | `sharing` | The migration level of `publication.db` | `version` pk · `applied_at` | draft |

Rows 1–9 are `library.db`. Rows 10–12 are `publication.db`. Nothing in `publication.db` references
anything in `library.db`: a Publication is a copy of a document, which is what makes FR-25's deletion
complete.

No row is owned by `_platform`. Every table above exists because one component's `FR` promises it,
which is why `platform_owns` is empty and `cross-cutting.md` has no platform data section.

## Findings

None — `derived_from: plan`, and there is no code to derive from yet. When there is, this file is
re-derived by `.constitution/method/scripts/inventory.py` against
`.constitution/project/inventory-readers.py`, and any difference from the plan above is reported here
rather than patched into agreement.
