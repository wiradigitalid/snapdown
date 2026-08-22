---
type: sdd
component: sharing
status: draft
created: "2026-08-22"
updated: "2026-08-22"
realizes: [UC-20, UC-21, UC-22, UC-23]
binds: [AD-4, AD-5, AD-6, AD-7, AD-8, AD-9]
reviewed:
  date: "2026-08-22"
  sha: 9bdda00
  lenses: [structure, prose, edge-case-hunter]
---

# SDD — sharing

## Decision Summary · [outline]

This component spans three containers and two languages, which is the whole reason it is the riskiest
one. On the desktop, a publish client reads a Bundle and pushes it over HTTPS. On the host, a Go
service holds served Publications in an embedded SQLite file and a blob directory and serves them at an
unlisted slug. In the reader's browser, a React page renders one Publication for a person while an
agent takes the raw Markdown from the same URL.

Three choices cost the most to reverse.

**A publish is staged, then made reachable, in that order.** The client uploads the Markdown and every
image under a staging identity; only when all of it has landed does the service make the slug resolve.
The obvious alternative — write files under the live slug as they arrive — makes FR-23's "nothing
partial is left readable" impossible to promise, and the failure it produces is a Publication serving
Markdown whose images 404, which reads to an agent as a broken review rather than as a failed publish.

**The slug belongs to the Bundle, not to the act.** It is generated once, on first publish, stored on
the `publication` row, and reused for every republish (BR-21). A URL already handed out keeps working.
The cost is stated plainly: republishing silently changes what that URL serves, which is the one rule
in this component the SRS records as uncomfortable.

**Local Publication state is a claim, and the service is the authority.** `publication.last_error`
exists so that an unconfirmed unpublish leaves the Bundle marked published, and endpoint 14 exists so
that claim can be checked rather than believed. Everything in this component is arranged so that the
Reviewer is never told something is private when it may not be — which is why an unpublish failure is
*sticky* rather than retried into silence.

## Structure · [outline]

Six Logical Components across three containers. Registered in `.control/registry/components.yaml`.

| LC | type | Container | Responsibility |
| --- | --- | --- | --- |
| LC-020 `publish-client` | gateway | `desktop-app` | Publish, unpublish, and reconcile against the service. Owns the staging protocol and the publish credential's use |
| LC-021 `publication-store` | store | `desktop-app` | The `publication` row: slug, base URL, timestamps, and the sticky last error |
| LC-022 `publish-dialog` | ui-composite | `desktop-app` | The confirmation that names the Bundle and says what publishing exposes, plus the state and URL shown per Bundle |
| LC-023 `publication-router` | gateway | `web-api` | The six web routes: three public reads, three credential-gated writes. Owns content negotiation and the identical-refusal rule |
| LC-024 `served-publication-store` | store | `web-api` | `published_bundle` and `published_blob`, plus the blob directory. The whole state NFR-14 keeps in one directory |
| LC-027 `bundle-reader` | ui-screen | `web-ui` | Renders one Publication for a person. Fetches the same Markdown an agent gets |

```mermaid
graph TD
    LC022["LC-022 publish-dialog"] --> LC020["LC-020 publish-client"]
    LC020 --> LC021["LC-021 publication-store"]
    LC020 --> LC013(["LC-013 bundle-store<br/>bundle, read-only"])
    LC020 --> LC025(["LC-025 settings-store<br/>settings, read-only"])
    LC020 -->|"HTTPS, publish credential"| LC023["LC-023 publication-router"]
    LC023 --> LC024["LC-024 served-publication-store"]
    RA(["Remote coding agent"]) -->|"HTTPS GET, raw Markdown"| LC023
    LC027["LC-027 bundle-reader"] -->|"HTTPS GET"| LC023
    P(["Person in a browser"]) --> LC027
```

`LC-023` and `LC-024` are the Go half and depend on nothing in the Rust tree. The only thing crossing
the language seam is the publish request, and rows 12–14 of `inventory-api.md` are the entire
agreement. That is `DEC-001`'s stated cost, tracked as RISK-10.

Crossings out of this component on the desktop: read-only calls into `LC-013 bundle-store` and
`LC-025 settings-store`. One call *into* this component: `bundle` calls `LC-020` to unpublish when a
published Bundle is deleted (BR-23).

## Inherited Constraints · [guarded]

Quoted verbatim from `.how/_platform/ARCHITECTURE-SPINE.md` under their original ids.

| AD | Quoted rule | How it lands here |
| --- | --- | --- |
| AD-4 | "The capture adapter MUST apply the Quality Budget before the image reaches the Vault, and MUST NOT retain the unreduced pixels. No later stage — composition, publishing, or serving — may re-encode or re-scale a stored image. A Bundle's image is a copy of the Finding's image with Markers drawn on it, at the same dimensions." | `LC-020` uploads the Bundle's stored image bytes unchanged, and neither `LC-020` nor `LC-023` has an image library in its dependency tree. No thumbnailing, no format conversion, no `srcset` |
| AD-5 | "The Local API, the MCP Bridge, `web-api`, and `web-ui` MUST expose no operation that creates, changes, or deletes anything in the Library. Write authority lives in the desktop process and reaches it only from the Reviewer's own actions. A new route or tool on any of those surfaces that is not a read is a violation, not a feature." | `LC-023`'s three write routes write only `web-api`'s own store, never the Library — the two share no storage and `web-api` cannot reach `library.db`. `LC-027` issues `GET` only. A route-inventory test asserts the public routes are reads |
| AD-6 | "No component may open an outbound network connection carrying Finding, Note, Marker, or Bundle content, except the publish client, executing a publish the Reviewer confirmed on a named Bundle. There is no telemetry, no analytics, and no crash reporter that carries content." | `LC-020` is the exception the rule names and the only networked LC in the desktop app. It refuses to run without a confirmation token minted by `LC-022` for one named Bundle, so there is no code path that publishes without a Reviewer's act. NFR-11's test asserts no other outbound call is attempted anywhere |
| AD-7 | "Every failure crossing a process boundary MUST be returned in the envelope defined in `cross-cutting.md`, carrying a code from that file's catalogue. A refusal MUST be distinguishable from an empty result by its code, never only by its body being empty." | `LC-023` has one error writer used by every handler, including the `not_found` that NFR-15 requires to be identical across three different causes. `LC-020` maps an envelope onto a message `LC-022` shows verbatim |
| AD-8 | "A Publication's slug MUST be generated independently of the Bundle's id and of every other Library id, from a cryptographically secure source. No Library id may appear in a published URL, in a published document, or in anything `web-api` serves." | The slug is 160 bits from the OS CSPRNG, base32 without padding, generated in `LC-021` on first publish only. `LC-024` stores no Library id at all — not the Bundle's, not a Finding's — which is what makes the constraint checkable by reading its schema |
| AD-9 | "A Bundle's Markdown MUST be composed once, by the core, and stored. Every handoff path MUST serve those exact bytes. No surface may re-render, re-order, decorate, or summarise a Bundle on the way out; a surface that needs a different shape is asking for a change to the composer." | `LC-020` uploads `bundle.markdown` verbatim; `LC-023` stores and serves those bytes; `LC-027` renders them client-side without rewriting the source. The golden-file test in `bundle` covers the published path |

## Failure Behaviour · [guarded]

Every boundary this component has. Derived from `.how/_platform/inventory-api.md` rows 9–14 and
`inventory-screen.md` rows 11, 14, and 15, plus the two store boundaries.

| Boundary | Slow | Absent | Lying | What the user sees | What is logged |
| --- | --- | --- | --- | --- | --- |
| `LC-020` → `LC-023`, publish (endpoint 12) | Progress is shown per file. No response in 60 s per request is treated as absent, and the staged upload is abandoned — the slug never resolves, so nothing partial is readable | The host is unreachable, DNS fails, or TLS fails. The publish fails, the Bundle stays unpublished locally, and nothing was staged that could resolve (BR-19) | Answers 200 for an upload it did not store. Detected by verifying the staged manifest before asking the service to make the slug live: the client sends every filename and byte count, and the service confirms what it holds. A mismatch abandons the publish | A dialog naming what failed and that nothing was published. The Bundle is exactly as it was | `event=publish_failed`, the slug, the stage reached, the reason. Never the Markdown, never image bytes |
| `LC-020` → `LC-023`, unpublish (endpoint 13) | No response in 30 s is treated as absent, and the outcome is the sticky one below | Unreachable. The Publication stays marked **published** with `last_error` set. This is the one place the product deliberately keeps the more alarming state (BR-20) | Answers 204 without removing anything. Guarded by immediately calling endpoint 14 and requiring `not_found`; anything else keeps the Publication marked published | The Bundle still shown as published, with the failure named and a retry offered. Never a false "it is private now" | `event=unpublish_failed`, the slug, the reason. This one is at error level |
| `LC-020` → `LC-023`, reconcile (endpoint 14) | Over 15 s the panel shows the state as unverified rather than guessing | Unreachable. The Bundle shows its last known state, explicitly labelled as last known rather than current | Reports a slug as absent while still serving it — a caching layer, a stale replica. Nothing local can detect it. Mitigated only by the service having no cache in front of it, which is a deployment constraint recorded in the devops repository | Publication state shown as unverified, with when it was last confirmed | `event=reconcile_failed`, the slug, the reason |
| `LC-020` → the publish credential | Over 2 s the publish is refused rather than blocking | The credential store has no publish credential, or none is configured. Publishing is refused before anything is read from the Bundle, naming what is missing (FR-23) | Returns a credential the service rejects. The service answers 401, mapped to `publish_failed` with the reason "the service rejected the credential", so the Reviewer is not told the network failed | A dialog saying what is missing or rejected, with an action that opens Settings | `event=publish_credential_missing` or `..._rejected`. Never the credential |
| `LC-020` → `LC-013 bundle-store` | Local SQLite; over 5 s means a failing disk, and the publish is refused | The Bundle is gone — deleted between the click and the read. The publish is abandoned | Returns a Bundle whose image files are missing from the Vault. Checked before staging: a missing image refuses the publish naming the file, rather than publishing a review with broken images | A dialog naming what is missing. Nothing was published | `event=publish_source_incomplete`, the Bundle id, the filenames |
| `LC-021 publication-store` | Not applicable | The `publication` row is missing while the service still serves the slug — an orphaned Publication. Found by the reconcile the Reviewer can run, and it is why endpoint 14 takes a slug rather than a Bundle id | Holds a slug the service has never heard of, from a publish that failed after the row was written. The row is written **after** the service confirms, so this is unreachable by design rather than by check | Nothing, unless a reconcile surfaces it | `event=publication_orphaned`, the slug |
| Remote agent → `LC-023` (endpoints 9, 10, 11) | The service is one binary reading local files; slow means the host is saturated. No timeout of its own — the agent's client owns that | The slug is unknown, was never issued, or was unpublished. All three answer identically: `not_found`, same status, same body (NFR-15, BR-24) | Not applicable in this direction — the agent is a reader with nothing to lie about. A crafted image filename is the hostile input, refused by resolving against the Publication's own directory | Nothing on the desktop. The agent gets `not_found` and stops | `event=publication_not_found`, the slug prefix only, at debug level. A full log of every attempted slug is itself an enumeration surface |
| `LC-027` → `LC-023` (screens 14, 15) | The page shows a loading state, then a plain message. It never blocks on a spinner without a way out | `not_found`. The page renders screen 15, the refused state, which is identical for every cause | Not applicable | A page saying the review is not available. No hint about whether it ever existed | Nothing server-side beyond the row above. `web-ui` logs nothing anywhere |
| `LC-023` → `LC-024` | Embedded SQLite plus a directory. Slow means a failing disk; over 5 s answers `unavailable` | The directory or the SQLite file is missing — a mounted volume gone. Every route answers `unavailable`, including the write routes, so a publish fails rather than half-succeeding | Reports a blob written that is not on disk. The write path re-stats every file before confirming the manifest, which is the same check the client relies on | The publishing Reviewer sees `publish_failed`; a reader sees `unavailable`, which is honestly different from `not_found` | `event=store_unavailable`, the operation. `web-api` never logs the content it serves |

Two entries are the ones worth arguing about, and both are deliberate:

- **A failed unpublish keeps the Bundle marked published.** Every instinct says clear the flag and
  retry in the background. That is the failure mode the whole component is arranged against.
- **`not_found` is identical for unknown, never-issued, and unpublished.** It means the Reviewer cannot
  tell from the outside whether their unpublish worked — which is what endpoint 14 and the publish
  credential are for. The reader-facing surface stays uninformative on purpose.

## Design Notes

- **The staging identity is derived from the slug, not separate from it.** A staging prefix plus the
  slug keeps one namespace, so an abandoned stage is collectable by name without a second index.
- **An abandoned stage is collected on the service's own schedule**, not by the client. A client that
  crashed mid-publish cannot be relied on to clean up, and NFR-14's one directory is what makes the
  sweep trivial.
- **`publication.last_error` is sticky and is cleared only by a confirmed outcome** — a successful
  unpublish, or a successful republish. Clearing it on a retry that also failed is how "the Reviewer
  believes it is private" gets reintroduced.
- **`LC-023` serves raw Markdown by content negotiation and at an explicit path.** Endpoint 10 exists
  because negotiation is exactly the thing a minimal HTTP client gets wrong, and an agent that cannot
  read the review is the failure this component exists to prevent.
- **`web-api` has no route that takes a Library id, and no column that stores one.** That is stronger
  than a rule about URLs, and it is checkable by reading the schema.
- **`web-api` writes no request log pairing a slug with an address beyond what it needs to operate.**
  A slug is a bearer credential, so an access log is a credential store.
- **`web-ui` is served by `web-api` at the same origin**, so a Publication needs no cross-origin
  configuration and `web-api` needs no CORS policy — one fewer thing to get wrong on a surface where
  getting it wrong exposes data.

---

## Slots

`01-ux/` — not written below `mode: deep`. Screens 11, 14, and 15 in `inventory-screen.md`.
`02-contracts/` — `[deep]` only. The six routes are rows 9–14 of `inventory-api.md`; their five lanes
are unwritten at `guarded`, and `Failure Behaviour` above is what stands in. This is the component
where that gap costs the most, and raising it to `deep` is the obvious first move if publishing
proves fragile.
`03-integrations/` — `[guarded]` and not applicable. `web-api` is ours, not a third party. There is no
third party anywhere in this product.
`04-components/`, `05-model/`, `06-flows/` — `[deep]` only. UC-20 would earn a `06-flows/` sequence
diagram at `deep`: it is irreversible state crossing a process boundary.

## Open Items

- OQ-7 — whether a remote agent can fetch an HTTPS URL and its images.
  `.control/questions/assumptions.md`.
- OQ-8 — whether an unlisted slug is access control the owner accepts.
  `.control/questions/assumptions.md`.
- OQ-13 — which host runs `web-api`. `.control/questions/external.md`. Go-live only.
- OQ-14 — which domain serves Publication URLs. `.control/questions/external.md`. Go-live only.
- RISK-8 — a dishonest unpublish. `.control/registry/risks.yaml`.
- RISK-9 — a leaked Publication URL. `.control/registry/risks.yaml`.
- RISK-10 — the publish seam nothing typechecks. `.control/registry/risks.yaml`.
- PRD open question 4 — what happens to a Publication when its Bundle's source Findings are deleted.
  Currently nothing, because a Publication is a copy.
