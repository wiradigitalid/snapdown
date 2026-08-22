---
type: srs
component: sharing
status: draft
created: "2026-08-22"
updated: "2026-08-22"
satisfies: [FR-23, FR-24, FR-25, FR-26, NFR-10, NFR-11, NFR-12, NFR-13, NFR-14, NFR-15]
reviewed:
  date: "2026-08-22"
  sha: 1a67115
  lenses: [structure, prose, edge-case-hunter]
---

# SRS — sharing

## Decision Summary · [G3]

This component puts one named Bundle somewhere an agent on another host can read it, and takes it back
again. The Reviewer selects a Bundle, confirms what publishing means, and Snapdown uploads its Markdown
and images to a web service the Reviewer runs. The result is an unlisted URL: a slug long enough not to
be guessed, with nothing behind it that lists, searches, or enumerates. Unpublishing removes the
content and the URL stops resolving.

Three decisions carry it. Publishing is **an act on a named Bundle**, confirmed each time — never a
sync, never automatic, never a default, because a Capture can contain anything that was on the screen.
The slug is **unrelated to every Library id**, so one leaked URL is not a way to find the next. And an
unpublish that fails **keeps the Bundle marked published**, because telling the Reviewer something is
private when it may not be is the worst outcome this component can produce.

`mode: guarded`, `risk_accepted: low`. This is the only component where a mistake reaches the public
internet and cannot be undone: an unpublish cannot recall what was already fetched.

## Why · [G3]

Because the Reviewer's agents are not all in one place. `agent-access` reaches the one in the next
window and cannot reach the one on a server; nothing about a loopback interface can be stretched to
cover it. This component exists for that gap, and it is deliberately the only path out of the machine —
which is what makes AD-6 checkable at all.

## Actor Register · [G3]

| Actor | Who they are | What they may do |
| --- | --- | --- |
| Reviewer | The person operating Snapdown. The only writer | Publish a named Bundle after confirming, see whether a Bundle is published and where, copy its URL, and unpublish it |
| Remote coding agent | An AI coding agent on another host, holding a Publication URL | Fetch the Publication's Markdown and the images it references. Nothing else, and nothing that writes |

The Reviewer is also the second reader — they open a Publication URL in a browser to check what was
published. That is not a third actor; it is the same person using the same read.

## UC Catalogue · [G3]

| id | Use case | Actor | Satisfies | critical |
| --- | --- | --- | --- | --- |
| UC-20 | I put a review somewhere the agent on my server can reach it | Reviewer | FR-23 | yes |
| UC-21 | I read a review that was put up for me, from a machine that has nothing else on it | Remote coding agent | FR-24 | no |
| UC-22 | I take a published review back off the internet | Reviewer | FR-25 | no |
| UC-23 | I check whether a review of mine is still readable from outside | Reviewer | FR-26 | no |

One of four is `critical`: publishing puts images that may contain personal data on the public
internet and cannot be undone in the sense that matters. UC-22 is not marked — it reduces exposure
rather than creating it — and neither is UC-21, which is the reading of something already published.

## Constraints · [G3]

| Constraint | Source |
| --- | --- |
| Nothing leaves the machine except a publish the Reviewer confirmed on a named Bundle | AD-6, BR-18, NFR-11 |
| A Publication's slug is generated independently of every Library id, from a cryptographically secure source, with at least 128 bits of entropy. No Library id appears in a published URL or document | AD-8, NFR-10 |
| A slug is never reused for a different Bundle, including after an unpublish | BR-22 |
| Publishing a Bundle that is already published replaces its content at the same URL | BR-21 |
| A failed publish leaves nothing readable on the service and leaves the Bundle unpublished locally | BR-19 |
| A failed unpublish leaves the Bundle marked published | BR-20 |
| An unknown slug, a revoked slug, and one that never existed are refused identically | BR-24, NFR-15 |
| No route lists, searches, or enumerates Publications, on any surface | NFR-15 |
| Every surface here is read-only apart from the desktop's own credential-gated publish routes | AD-5, BR-15 |
| Only reduced images are transmitted; no unreduced capture exists to transmit | AD-4, BR-25, NFR-12 |
| The published Markdown is the Bundle's exact stored bytes | AD-9 |
| Every failure crossing a process boundary uses the envelope in `cross-cutting.md` | AD-7 |
| The web service is one executable, one configuration file, no database server, and its whole state is one directory | NFR-14 |
| The publish credential lives in the Windows credential store on the desktop and in the environment on the host. It is never in a config file, a log, or this repository | cross-cutting.md § Secrets |
| Deleting a published Bundle unpublishes it in the same action | BR-23 |
| The web service does not log the content it serves | cross-cutting.md § Logging |

## Non-Goals · [G3]

- **Syncing.** Nothing is uploaded that the Reviewer did not publish on a Bundle they named.
- **Exposing the Library.** Only a published Bundle leaves, and only as a copy. `web-api` never reads
  `library.db`.
- **Accounts, sign-in, or per-viewer permissions** on the service. The slug is the control in r2.
- **A read token.** Designed for by AD-8, not promised. PRD open question 3.
- **An index, gallery, search, or landing page.** NFR-15 forbids the first three; the fourth has no
  reason to exist.
- **A hosted service.** The Reviewer runs it. No third party is involved anywhere in the product.
- **Publishing more than one Bundle in one action.**
- **Composing or editing a Bundle.** `bundle` owns both, and BR-11 forbids the second entirely.
- **Server-side Marker rendering.** The Bundle's images already carry them, per AD-4.

## Prerequisite · [G3]

- `bundle` must exist and hold at least one Bundle. CAP-8 declares `depends_on: [CAP-4]`.
- A host running `web-api`, reachable over HTTPS — **OQ-13**, unresolved.
- A domain or subdomain for Publication URLs — **OQ-14**, unresolved.
- A publish credential configured on both sides.

Both open prerequisites are in `.control/questions/external.md` and both are **go-live only**. Neither
holds G3, G4, or any wave: the service and the client can be built and tested against a local instance.

## Success Signal · [G3]

The Reviewer publishes a Bundle, copies the URL, pastes it into an agent on a remote host, and that
agent reads the review and the images on the first try — within 60 seconds of the click. Unpublishing
makes the URL stop resolving, indistinguishably from a slug that never existed. At no point does
anything else leave the machine, and at no point is the Reviewer told a Bundle is private when it is
still being served.

## Assumptions, Risks, and To Be Confirmed · [G3]

### Assumptions

- An agent on a remote host can fetch an HTTPS URL and the images it references — OQ-7.
- An unguessable slug, optionally plus a read token later, is access control the Reviewer accepts —
  OQ-8.
- A coding agent handed Markdown with relative image paths can open them — OQ-1.

### Risks

- **The URL is a bearer credential.** Anywhere the Reviewer pastes it becomes somewhere the Bundle can
  be read from — an agent's transcript, a log, a shared terminal. This is the accepted weakness behind
  OQ-8, and the reason a read token is designed for rather than dismissed.
- **Publishing is not undoable.** An unpublish removes the content; it cannot recall a fetch. FR-23's
  confirmation has to say so, and a confirmation that implies otherwise is a defect rather than
  wording.
- **The dishonest-success failure.** An unpublish that half-succeeded, or that reported success
  without reaching the service, leaves the Reviewer believing something is private. BR-20 and the
  reconcile route (endpoint 14) exist for this and are the least optional part of the design.
- **Partial upload.** A publish that leaves a slug serving Markdown but not its images is worse than a
  failed publish. FR-23 requires all-or-nothing, which means content is staged and made reachable, not
  written into a live slug.
- **A host nobody is watching.** The service holds copies of images that may contain personal data on
  a machine outside this product's design. Its retention is one directory (NFR-14), and what happens
  to that directory is the Reviewer's operational concern, recorded in the devops repository rather
  than here.

### To Be Confirmed

- OQ-13 — which host runs `web-api`. `.control/questions/external.md`.
- OQ-14 — which domain serves Publication URLs. `.control/questions/external.md`.
- What happens to a Publication when its Bundle's source Findings are deleted. Currently nothing, and
  that is either correct or a surprise. PRD open question 4.

## Gate Checklist · [G3]

| Question | Answer |
| --- | --- |
| ★ Is every use case title a sentence a user would say? | Yes. UC-21 is written in the remote agent's voice, which is what that actor would say |
| ★ Any `FR` with no use case? | No. FR-23, FR-24, FR-25, FR-26 all have one |
| ★ Do the inventories and this catalogue describe one system? | Yes. Tables 6, 10–12, endpoints 9–14, screens 11, 14–15 |
| Actor list: is one missing, or are two the same person? | Two. The Reviewer opening a URL in a browser is the same actor performing UC-21's read, not a third |
| Does every `AD-N` here name a concrete failure that crosses components? | AD-5, AD-6, AD-7, AD-8, AD-9 all do. AD-8 is the one that exists only because of this component and binds `bundle` too |
| Which business rule am I not sure is right? | BR-21 — republishing at the same URL. It is convenient and it means a URL already handed out silently starts serving different content |
| Is there a term I have to guess the meaning of? | No |

## Design Reference · [G3]

Paired with `.how/sharing/SDD-sharing.md`.

Binding invariants: **AD-4** (no re-encoding), **AD-5** (read-only outside the desktop process),
**AD-6** (nothing leaves the machine without a confirmed publish), **AD-7** (one error envelope),
**AD-8** (a slug unrelated to every Library id), **AD-9** (one Bundle, one Markdown). No applied
`DEC-` binds this component yet.

---

## Slots

`02-rules/rules-sharing.md` — written at G4, `mode: guarded`.
`03-domain/domain-model.md` — written at G3, present.
`04-usecases/` — at most three full flows at `guarded`, written at G4.
`05-scenarios/` — not written below `mode: deep`.

## Open Items

- OQ-1 — whether an agent can open relative image paths. `.control/questions/assumptions.md`.
- OQ-7 — whether a remote agent can fetch an HTTPS URL and its images.
  `.control/questions/assumptions.md`.
- OQ-8 — whether an unlisted slug is acceptable access control.
  `.control/questions/assumptions.md`.
- OQ-13 — the host for `web-api`. `.control/questions/external.md`.
- OQ-14 — the domain for Publication URLs. `.control/questions/external.md`.
