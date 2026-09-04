---
title: Agent Handoff
initiative: agent-handoff
created: "2026-08-22"
updated: "2026-09-04"
---

# PRD: Agent Handoff

## Revision History

| Date | What changed | Why | Releases affected |
|---|---|---|---|
| 2026-08-22 | Initial version | Copying Markdown reaches an agent on this machine and nothing else; two more paths are needed before a Bundle is usable where the agents actually run | r2 |
| 2026-09-04 | §4.1 (CAP-7, FR-19..FR-22, UJ-5), the Access Key throughout, and every cross-reference to them are withdrawn. §4.2 (CAP-8) is unchanged | `DEC-016` — the owner instructed that the Agent bridge/MCP feature be removed, because the copy-Markdown-and-paste workflow already covers what it was for. There is no replacement running channel for an agent on this machine; the Reviewer pastes Markdown by hand, the way `capture-to-markdown`'s Copy Markdown and Bundle composition already produce it | r2 |

## 0. Document Purpose

This PRD is for the Product Owner and for the downstream blueprint and component work. It covers
what happens after a Bundle exists and needs to reach an agent on another host: publishing it to an
unlisted URL over HTTPS.

It does not cover capturing, noting, marking, composing, or handing a Bundle to an agent on the same
machine — that handoff is the Markdown a Reviewer copies with *Copy Markdown* or gets automatically on
a successful Assemble & Save / Review & Update Save, and pastes themselves; it belongs to
`capture-to-markdown`'s `bundle` component, not to this PRD. `DEC-016` withdrew the running channel
this PRD used to promise for that case.

Vocabulary is anchored in `.control/product-glossary.md` and used verbatim. Where something was
inferred rather than confirmed it carries an inline `[ASSUMPTION]` tag and appears again in §9.

## 1. Vision

The Reviewer has a Bundle, and an agent on a server somewhere else needs to read it. The Reviewer
publishes the Bundle to an unlisted URL and pastes the URL. The agent fetches Markdown and images
over HTTPS the way it would fetch any other document. Publishing is an act the Reviewer performs on
one named Bundle — never a sync, never a background upload, and never the default — because a
Capture can contain anything that was on the screen.

An agent in the next window on the same machine reads the same Bundle a different way: the Reviewer
copies its Markdown and pastes it in directly. That path needs nothing from this PRD — it is
`capture-to-markdown`'s own promise, not a second format of this one.

## 2. Target User

### 2.1 Jobs To Be Done

- Reach an agent that is not running on the machine where I took the screenshots.
- Know, for any Bundle, whether it is currently readable from outside this machine.
- Never have a screenshot leave this machine because a background process decided to send it.

Two jobs stood here until 2026-09-04 — *"give an agent a whole review without pasting five images
into a conversation"* and *"let an agent re-read a review later in the same session without me
pasting it again"* — and a third, *"grant that access deliberately... and take it back in one
action"*, which was this JTBD's own answer to the second. `DEC-016` withdrew the running channel they
described. The first is now served without this PRD at all — Copy Markdown already does it — and the
other two have no answer left to give: there is no standing access to grant, hold, or revoke.

### 2.2 Non-Users (v1)

- An agent that wants to write. Nothing here accepts a change from an agent — not a Note, not a
  Marker, not a Bundle, and certainly not a Capture.
- Anyone browsing. There is no index, no search, and no discovery of published Bundles.
- A second Reviewer. Publishing shares a document, not the Library.

### 2.3 Key User Journeys

A **UJ-5. The Reviewer hands a Bundle to the agent in the next window** journey stood here until
2026-09-04, pasting a copied Access Key so the agent could call an MCP Bridge and read the Library
itself. `DEC-016` withdrew the channel it described. Handing a Bundle to an agent on the same machine
is now `capture-to-markdown`'s Copy Markdown, pasted directly — a journey belonging to that PRD, not
this one.

- **UJ-6. The Reviewer hands a Bundle to an agent on a server.**
  - **Persona + context:** same Reviewer. The agent runs on a remote host and has no access to this
    machine's filesystem.
  - **Entry state:** Bundle composed and unpublished. The Snapdown web service is running on a host
    the Reviewer controls, and Snapdown knows its address and publish credential.
  - **Path:** selects the Bundle → clicks *Publish* → is shown what publishing means and confirms →
    Snapdown uploads the Markdown and its images → the Bundle now shows as published with an unlisted
    URL → clicks *Copy URL* → pastes the URL into the remote agent's conversation.
  - **Climax:** the remote agent fetches the URL, reads the Markdown, and follows the image links —
    the same review, in a place it could reach.
  - **Resolution:** the Bundle shows when it was published and where. Clicking *Unpublish* makes the
    URL stop resolving.
  - **Edge case:** the web service is unreachable. Publishing fails with the reason, the Bundle stays
    unpublished, and nothing partial is left on the server.

## 3. Glossary

Every domain noun this document uses is defined once in `.control/product-glossary.md` and used
verbatim: **Bundle**, **Capture**, **Editor**, **Finding**, **Handoff**, **Library**, **Note**,
**Publication**, **Reviewer**, **Vault**. **Access Key**, **Local API**, and **MCP Bridge** were
glossary entries this document used too, until `DEC-016` withdrew what they named.

No synonym for any of them appears in this PRD.

## 4. Features

A **§4.1 Reading a Bundle from this machine** feature stood here until 2026-09-04 — CAP-7, serving
BG-2, realising UJ-5 through FR-19 (issue and copy an Access Key), FR-20 (serve the Library over a
Local API), FR-21 (read a Bundle through MCP), and FR-22 (revoke access). `DEC-016` withdrew the
capability and all four FRs along with `agent-access`, the Product Component that carried them; none
of the five ids is reused. The job the section served — handing a Bundle to an agent on the same
machine without pasting images by hand — is now done by Copy Markdown, which belongs to
`capture-to-markdown` and needed no running channel to begin with.

### 4.2 Reading a Bundle from somewhere else

**Capability:** CAP-8 — serves BG-4.

**Description:** The Reviewer publishes one named Bundle to a web service they control. Publishing
copies the Bundle's Markdown and images to that service and returns an unlisted URL — a slug long
enough not to be guessed, with no index and no listing behind it. The remote agent fetches the URL.
Unpublishing makes it stop resolving. Realizes UJ-6.
`[ASSUMPTION: an unguessable slug, optionally plus a read token, is access control the Reviewer accepts.]`
`[ASSUMPTION: an agent on a remote host can fetch an HTTPS URL and the images it references.]`

**Functional Requirements:**

#### FR-23: Publish a Bundle

The Reviewer can publish a selected Bundle to their web service and receive an unlisted URL for it.
Realizes UJ-6.

**Proof of done:** Publishing a Bundle yields a URL that, fetched from another machine, returns that
Bundle's Markdown with working image links.

**Consequences (testable):**
- Publishing requires an explicit confirmation that states the Bundle's name and what publishing
  exposes.
- Publishing is per Bundle and per act. A single Finding is published by composing a Bundle that
  holds only it — the granularity exists, at the cost of one composition step.
- Publishing happens only on a Bundle the Reviewer named; nothing is ever published automatically or
  in the background.
- A failed publish leaves nothing readable on the service and leaves the Bundle unpublished locally.
- Publishing an already published Bundle replaces its content at the same URL rather than creating a
  second one.
- The images published are the reduced images, never an unreduced original.
- Publishing is refused, with the reason, when no web service is configured.

#### FR-24: Serve a published Bundle

The web service serves a published Bundle at its unlisted URL as Markdown, with its images fetchable
from that document, to any client that has the URL. Realizes UJ-6.

**Proof of done:** A plain HTTP client given the URL retrieves the Markdown, and every image reference
in it retrieves an image.

**Consequences (testable):**
- The document is retrievable as raw Markdown, so an agent does not have to parse a web page.
- Image references resolve relative to the document's own URL.
- An unknown slug is refused indistinguishably from a revoked one.
- No path on the service lists, indexes, searches, or enumerates published Bundles.
- The service serves a human-readable rendering at the same URL for a browser, and raw Markdown for a
  client that asks for it.

**Out of Scope:**
- Accounts, sign-in, or per-viewer permissions on the service.
- Comments, edits, or any write from a viewer.

#### FR-25: Unpublish a Bundle

The Reviewer can unpublish a Bundle, after which its URL stops resolving and its content is removed
from the service.

**Proof of done:** After unpublishing, fetching the previously working URL from another machine
returns nothing, and the Bundle is still intact in the Library.

**Consequences (testable):**
- Unpublishing removes the Markdown and the images from the service, not just the mapping.
- The URL is never reused for a different Bundle.
- Unpublishing a Bundle that is not published is harmless and says so.
- Deleting a published Bundle unpublishes it as part of the same action.
- An unpublish that cannot reach the service is reported and the Bundle stays marked published, so
  the Reviewer is never told something is private when it is not.

#### FR-26: See and copy a Bundle's Publication

The Reviewer can see, for every Bundle, whether it is published, when it was published, and at what
URL — and can copy that URL in one action. Realizes UJ-6.

**Proof of done:** The Bundle list distinguishes published from unpublished Bundles at a glance, and
copying the URL of a published one puts a working URL on the clipboard.

**Consequences (testable):**
- Publication state is visible in the Bundle list without opening the Bundle.
- The publish timestamp and the URL are both shown.
- Copying tells the Reviewer it succeeded.
- A Bundle whose last unpublish failed is shown as still published, with the failure named.

## 5. Non-Goals (Explicit)

- Snapdown is not a sync client. Nothing is uploaded that the Reviewer did not publish, on a Bundle
  they named, in an action they confirmed.
- The Library is never exposed beyond this machine. Only a published Bundle leaves it, and only as a
  copy.
- Nothing here is writable by an agent. There is no MCP operation and no web endpoint that changes a
  Note, a Marker, a Bundle, or takes a Capture.
- The web service is not a product. It has no accounts, no billing, no gallery, no search, and no
  landing page.
- Snapdown does not host anything itself. The web service runs where the Reviewer puts it.
- No third-party service is required. Publishing goes to a host the Reviewer controls, not to an
  account they signed up for.

## 6. MVP Scope

### 6.1 In Scope

- Publish and unpublish one named Bundle, with confirmation and honest failure.
- A web service serving a published Bundle as raw Markdown and as a plain rendering, plus its images.
- Publication state and URL visible per Bundle in the Editor.

### 6.2 Out of Scope for MVP

- Publishing more than one Bundle in one action.
- Publishing a Finding directly, without composing it into a Bundle first. BR-14 keeps unbundled
  Findings invisible on every agent-facing surface, and a one-Finding Bundle covers the case.
  `[NOTE FOR PM]` If composing a one-Finding Bundle turns out to be friction the Reviewer feels on
  every single-screenshot handoff, the answer is a one-click "publish this finding" that composes
  behind the scenes — not a second publish path. Filed as OQ-16.
- An expiry on a Publication. An expiry on an Access Key stood here too, until `DEC-016` withdrew the
  key itself on 2026-09-04.
- Per-Bundle read tokens on top of the unlisted slug. The slug is the control in r2; the token is
  designed for but not promised.
- Exposing unbundled Findings.
- Any push from Snapdown to an agent. Publishing is pull.
- Server-side rendering of Markers. The Bundle's images already carry them.
- A managed hosted service the Reviewer does not run themselves.

## 7. Success Metrics

**Primary**

- **SM-7**: Bundles readable by an agent on another host, where previously the answer was none —
  target: every published Bundle is fetchable by the remote agent on the first try. Validates FR-23,
  FR-24.

**Secondary**

- **SM-8**: Time from *Publish* to the remote agent having read the Bundle — target under 60 seconds.
  Validates FR-23, FR-24, FR-26.
- **SM-9**: Published Bundles still live that the Reviewer thought were unpublished — target zero,
  which is why FR-25 refuses to lie about a failed unpublish. Validates FR-25, FR-26.

**Counter-metrics (do not optimize)**

- **SM-C3**: Number of published Bundles. A rising count is not success; publishing is the exception
  and the local path is the norm. Counterbalances SM-7.

`SM-6` and `SM-C4` stood here too, measuring how much of a Handoff avoided pasting Bundle content and
how long an Access Key lived, until `DEC-016` withdrew the key and the channel `SM-6` measured on
2026-09-04. Neither id is reused.

## 8. Open Questions

1. Does the remote agent need the images at all, or is the Markdown enough for most reviews? If
   images are rarely fetched, the publish payload could shrink considerably.
2. Should a Publication carry an optional read token from the start, rather than being designed for
   and added later?
3. What happens to a Publication when the Bundle's Findings are deleted from the Library? Currently
   nothing — the Publication is a copy — and that is either the right answer or a surprise.

A fourth question stood here — whether the Local API and the MCP Bridge should be one process rather
than two — until `DEC-016` withdrew both on 2026-09-04, and the PRD open question 2 it pointed at in
`SRS-agent-access.md` went with them.

## 9. Assumptions Index

- §4.2 — an unguessable slug, optionally plus a read token, is access control the Reviewer accepts.
  Filed as OQ-8.
- §4.2 — an agent on a remote host can fetch an HTTPS URL and the images it references. Filed as
  OQ-7.
- Carried from the brief and still load-bearing here: a coding agent can open relative image paths
  (OQ-1).

A §4.1 assumption stood here too — the Reviewer prefers pasting a key per session over the agent
holding standing access, filed as OQ-6 — until `DEC-016` closed OQ-6 on 2026-09-04: the friction it
asked about no longer exists, because the thing that caused it is gone.

## Cross-Cutting NFRs

- **NFR-10** — serves BG-4. A Publication's URL slug carries at least 128 bits of entropy from a
  cryptographically secure source. Enforced by an assertion on slug generation.
- **NFR-11** — serves BG-4. No Finding, Note, or Bundle leaves this machine except through a publish
  the Reviewer confirmed on a named Bundle. Enforced by a test that exercises capture, composition,
  and idle operation with outbound network calls failing, and asserts none was attempted.
- **NFR-12** — serves BG-3. Only reduced images are ever transmitted or published; an unreduced
  capture never leaves the machine. Enforced by an assertion on the publish payload.
- **NFR-13** — serves BG-5. An unpublish takes effect on the next request, with no cache or grace
  period. Enforced by a test that calls immediately after. (Read *"a revoke and an unpublish"* until
  `DEC-016` withdrew the revoke it named, on 2026-09-04.)
- **NFR-14** — serves BG-6. The web service runs as one executable with one configuration file, needs
  no database server, and its whole state is one directory. Enforced by an integration test that
  starts it from a clean directory and serves a published Bundle.
- **NFR-15** — serves BG-4. The web service exposes no route that lists, searches, or enumerates
  Publications, and returns the same refusal for an unknown slug as for a revoked one. Enforced by a
  route-inventory test and a response-equality test.

`NFR-9`, binding the Local API to loopback and a valid Access Key, stood here too until `DEC-016`
withdrew both on 2026-09-04; not reused.

## Constraints and Guardrails

### Safety

- Publishing is irreversible in the sense that matters: once a Bundle has been fetched, unpublishing
  cannot recall it. The confirmation in FR-23 must say so rather than implying publishing is
  undoable.
- Unpublishing must never report success it did not achieve. NFR-13 and FR-25 exist because the
  failure mode "the Reviewer believes it is private" is the worst outcome in this initiative.

### Privacy

- A Capture may contain personal data. Every requirement here is shaped by that: nothing leaves
  automatically, nothing is listed, and nothing is retained after an unpublish.
- The publish credential is a secret. It may not appear in a log, a Bundle, a published document, a
  crash report, or this repository. The Access Key was the other secret named here, until `DEC-016`
  withdrew it on 2026-09-04.
- The web service must not log the content it serves, and must not retain a request log that pairs a
  slug with an address for longer than it needs to operate.

### Cost

Beyond the brief: the web service needs a host and a domain, and both are unresolved — OQ-13 and
OQ-14 in `.control/questions/external.md`. Neither blocks a design gate; both block go-live of CAP-8.

### Beyond the brief

Everything else that binds here is already a product-wide constraint in
`.what/_product-brief/brief.md` — in particular that no publish may happen that the Reviewer did not
perform on a named Bundle. Not restated. The brief's other constraint named here — that an agent MUST
NOT reach the Library without the Reviewer handing it a key — described the running channel `DEC-016`
withdrew on 2026-09-04; there is no channel left for it to bind.
