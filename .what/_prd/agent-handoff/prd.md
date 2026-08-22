---
title: Agent Handoff
initiative: agent-handoff
created: "2026-08-22"
updated: "2026-08-22"
---

# PRD: Agent Handoff

## Revision History

| Date | What changed | Why | Releases affected |
|---|---|---|---|
| 2026-08-22 | Initial version | Copying Markdown reaches an agent on this machine and nothing else; two more paths are needed before a Bundle is usable where the agents actually run | r2 |

## 0. Document Purpose

This PRD is for the Product Owner and for the downstream blueprint and component work. It covers
everything that happens after a Bundle exists: how an agent on this machine reads one without the
Reviewer pasting anything but a key, and how an agent on another host reads one over HTTPS.

It does not cover capturing, noting, marking, or composing. Those are the `capture-to-markdown`
initiative, and a reader looking for them will not find them here. This PRD assumes a Bundle exists
and treats it as read-only input.

Vocabulary is anchored in `.control/product-glossary.md` and used verbatim. Where something was
inferred rather than confirmed it carries an inline `[ASSUMPTION]` tag and appears again in §9.

## 1. Vision

The Reviewer has a Bundle. Their agents are in two places: one running in a terminal on the same
machine, one running on a server somewhere else. The Bundle has to reach both, and neither path may
require the Reviewer to prepare it twice.

For the agent on this machine, the Bundle is read where it already lives. The Reviewer copies an
Access Key, pastes it into the conversation, and the agent can read that Bundle and no more. Nothing
was standing open before the key was pasted, and revoking the key closes it again.

For the agent somewhere else, the Reviewer publishes the Bundle to an unlisted URL and pastes the
URL. The agent fetches Markdown and images over HTTPS the way it would fetch any other document.
Publishing is an act the Reviewer performs on one named Bundle — never a sync, never a background
upload, and never the default — because a Capture can contain anything that was on the screen.

Both paths carry the same Bundle. Neither of them is a second format.

## 2. Target User

### 2.1 Jobs To Be Done

- Give an agent a whole review without pasting five images into a conversation.
- Let an agent re-read a review later in the same session without me pasting it again.
- Reach an agent that is not running on the machine where I took the screenshots.
- Grant that access deliberately, for as long as I want it, and take it back in one action.
- Know, for any Bundle, whether it is currently readable from outside this machine.
- Never have a screenshot leave this machine because a background process decided to send it.

### 2.2 Non-Users (v1)

- An agent that wants to write. Nothing here accepts a change from an agent — not a Note, not a
  Marker, not a Bundle, and certainly not a Capture.
- Anyone browsing. There is no index, no search, and no discovery of published Bundles.
- A second Reviewer. Publishing shares a document, not the Library.

### 2.3 Key User Journeys

- **UJ-5. The Reviewer hands a Bundle to the agent in the next window.**
  - **Persona + context:** the primary user, Bundle composed, a coding agent open in a terminal on
    the same machine with the repository the review is about.
  - **Entry state:** Snapdown running. The MCP Bridge is configured in the agent's MCP settings but
    holds no Access Key, so it can read nothing.
  - **Path:** in the Editor, clicks *Copy access key* on the Bundle → pastes it into the agent
    conversation with a sentence like "read the Snapdown bundle with this key" → the agent calls the
    Bridge, lists what the key unlocks, and reads the Bundle's Markdown and images.
  - **Climax:** the agent starts working from five Findings it read itself, each Note bound to its own
    image, without a single image having been pasted into the conversation.
  - **Resolution:** the key stays valid until revoked, so the agent can re-read the Bundle later in
    the session. Clicking *Revoke* ends it immediately.
  - **Edge case:** the agent calls the Bridge before a key is pasted. It gets a refusal that says a
    key is required, not an empty list that looks like an empty Library.

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
verbatim: **Access Key**, **Bundle**, **Capture**, **Editor**, **Finding**, **Handoff**, **Library**,
**Local API**, **MCP Bridge**, **Note**, **Publication**, **Reviewer**, **Vault**.

No synonym for any of them appears in this PRD.

## 4. Features

### 4.1 Reading a Bundle from this machine

**Capability:** CAP-7 — serves BG-2.

**Description:** Snapdown exposes a Local API over the Library, bound to loopback and closed until an
Access Key exists. A separate MCP Bridge executable speaks the Model Context Protocol to an agent and
the Local API to Snapdown, so an agent that only knows how to launch a stdio MCP server can still
read a Library held by a running desktop application. The Reviewer grants access by copying a key and
pasting it; nothing is readable before that and nothing stays readable after a revoke. Realizes UJ-5.
`[ASSUMPTION: the Reviewer prefers pasting a key per session over the agent holding standing access.]`

**Functional Requirements:**

#### FR-19: Issue and copy an Access Key

The Reviewer can generate an Access Key and put it on the clipboard in one action, ready to paste into
an agent conversation. Realizes UJ-5.

**Proof of done:** Clicking *Copy access key* puts a key on the clipboard, and an agent given that key
can read the Library while an agent given no key cannot.

**Consequences (testable):**
- Exactly one Access Key is valid at a time; issuing a new one invalidates the previous one.
- The key is shown once at issue and can be re-copied while it is valid, so the Reviewer is not forced
  to reissue to recover it.
- The Editor shows whether a key is currently valid and when it was issued.
- The key is never written into a Bundle, a Note, a log line, or a published document.

#### FR-20: Serve the Library over the Local API

The system serves the Library — the Bundle list, a Bundle's Markdown, and a Bundle's images — over an
interface reachable only from this machine and only with a valid Access Key. Realizes UJ-5.

**Proof of done:** With a valid Access Key, a request from this machine returns a Bundle's Markdown;
the same request without the key, or from another machine, is refused.

**Consequences (testable):**
- The interface binds to loopback only and is not reachable from any other host.
- A request with no key, a wrong key, or a revoked key is refused with a reason.
- The refusal for a missing key is distinguishable from an empty Library.
- Only Bundles are readable. An unbundled Finding is not exposed.
- Nothing on the interface accepts a write, a delete, or a capture.

**Out of Scope:**
- Exposing the Library to another machine on the LAN. That is what publishing is for.

#### FR-21: Read a Bundle through MCP

An agent configured with the MCP Bridge can list the Bundles a valid Access Key unlocks, read one
Bundle's Markdown, and fetch that Bundle's images. Realizes UJ-5.

**Proof of done:** A coding agent with the Bridge configured and the key pasted can name the Findings
in a Bundle and describe what is in their images, without the Reviewer pasting any content.

**Consequences (testable):**
- The Bridge starts and responds even when Snapdown is not running, saying so rather than hanging.
- The Bridge holds no copy of the Library and no key of its own between runs.
- Listing Bundles returns each Bundle's name, its Finding count, and when it was composed.
- Reading a Bundle returns the same Markdown that *Copy Markdown* produces.
- Image references an agent receives resolve to images it can actually fetch through the Bridge.
- The Bridge exposes no operation that changes anything.

#### FR-22: Revoke access

The Reviewer can revoke the current Access Key in one action, after which nothing can be read until a
new key is issued.

**Proof of done:** After a revoke, an agent that had been reading Bundles a moment earlier is refused
on its next call.

**Consequences (testable):**
- Revoking takes effect on the next request; no cached grant survives it.
- Revoking with no key present is harmless and says so.
- Revoking does not change, delete, or unpublish anything in the Library.
- The Editor shows plainly that no key is valid.

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

- One Access Key at a time: issue, copy, show state, revoke.
- A loopback-only Local API serving Bundle list, Bundle Markdown, and Bundle images, key-gated.
- An MCP Bridge executable: list Bundles, read one Bundle, fetch its images. Read-only.
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
- An expiry on an Access Key or a Publication. `[NOTE FOR PM]` A time-limited key is the obvious next
  step and was left out only because "revoke" already closes the hole; revisit for r3.
- Per-Bundle read tokens on top of the unlisted slug. The slug is the control in r2; the token is
  designed for but not promised.
- Exposing unbundled Findings over either path.
- Any push from Snapdown to an agent. Both paths are pull.
- Server-side rendering of Markers. The Bundle's images already carry them.
- A managed hosted service the Reviewer does not run themselves.

## 7. Success Metrics

**Primary**

- **SM-6**: Share of Handoffs that reach the agent without the Reviewer pasting Bundle content —
  target the majority, once both paths exist. Validates FR-19, FR-21, FR-23.
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
- **SM-C4**: Access Key lifetime. Longer is more convenient and is exactly the wrong thing to
  optimise — a key that never gets revoked is standing access, which is what FR-22 exists to prevent.
  Counterbalances SM-6.

## 8. Open Questions

1. Does the remote agent need the images at all, or is the Markdown enough for most reviews? If
   images are rarely fetched, the publish payload could shrink considerably.
2. Should the Local API and the MCP Bridge be one process rather than two, for MCP clients that can
   speak HTTP directly? Two is the shape that works everywhere; one is simpler where it works.
3. Should a Publication carry an optional read token from the start, rather than being designed for
   and added later?
4. What happens to a Publication when the Bundle's Findings are deleted from the Library? Currently
   nothing — the Publication is a copy — and that is either the right answer or a surprise.

## 9. Assumptions Index

- §4.1 — the Reviewer prefers pasting a key per session over the agent holding standing access.
  Filed as OQ-6.
- §4.2 — an unguessable slug, optionally plus a read token, is access control the Reviewer accepts.
  Filed as OQ-8.
- §4.2 — an agent on a remote host can fetch an HTTPS URL and the images it references. Filed as
  OQ-7.
- Carried from the brief and still load-bearing here: a coding agent can open relative image paths
  (OQ-1).

## Cross-Cutting NFRs

- **NFR-9** — serves BG-4. The Local API binds to `127.0.0.1` only, and every request without a
  currently valid Access Key is refused. Key comparison is constant-time. Enforced by tests that
  attempt the interface from a non-loopback address and with absent, wrong, and revoked keys.
- **NFR-10** — serves BG-4. A Publication's URL slug carries at least 128 bits of entropy from a
  cryptographically secure source. Enforced by an assertion on slug generation.
- **NFR-11** — serves BG-4. No Finding, Note, or Bundle leaves this machine except through a publish
  the Reviewer confirmed on a named Bundle. Enforced by a test that exercises capture, composition,
  and idle operation with outbound network calls failing, and asserts none was attempted.
- **NFR-12** — serves BG-3. Only reduced images are ever transmitted or published; an unreduced
  capture never leaves the machine. Enforced by an assertion on the publish payload.
- **NFR-13** — serves BG-5. A revoke and an unpublish each take effect on the next request, with no
  cache or grace period. Enforced by tests that call immediately after each.
- **NFR-14** — serves BG-6. The web service runs as one executable with one configuration file, needs
  no database server, and its whole state is one directory. Enforced by an integration test that
  starts it from a clean directory and serves a published Bundle.
- **NFR-15** — serves BG-4. The web service exposes no route that lists, searches, or enumerates
  Publications, and returns the same refusal for an unknown slug as for a revoked one. Enforced by a
  route-inventory test and a response-equality test.

## Constraints and Guardrails

### Safety

- Publishing is irreversible in the sense that matters: once a Bundle has been fetched, unpublishing
  cannot recall it. The confirmation in FR-23 must say so rather than implying publishing is
  undoable.
- Unpublishing must never report success it did not achieve. NFR-13 and FR-25 exist because the
  failure mode "the Reviewer believes it is private" is the worst outcome in this initiative.

### Privacy

- A Capture may contain personal data. Every requirement here is shaped by that: nothing leaves
  automatically, nothing is exposed before a key exists, nothing is listed, and nothing is retained
  after an unpublish.
- The Access Key and the publish credential are secrets. Neither may appear in a log, a Bundle, a
  published document, a crash report, or this repository.
- The web service must not log the content it serves, and must not retain a request log that pairs a
  slug with an address for longer than it needs to operate.

### Cost

Beyond the brief: the web service needs a host and a domain, and both are unresolved — OQ-13 and
OQ-14 in `.control/questions/external.md`. Neither blocks a design gate; both block go-live of CAP-8.

### Beyond the brief

Everything else that binds here is already a product-wide constraint in
`.what/_product-brief/brief.md` — in particular that an agent MUST NOT reach the Library without the
Reviewer handing it a key, and that no publish may happen that the Reviewer did not perform on a
named Bundle. Neither is restated.
