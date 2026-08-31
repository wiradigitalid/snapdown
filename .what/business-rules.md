---
type: rules
scope: global
status: draft
created: "2026-08-22"
updated: "2026-08-31"
---

# Business Rules — Snapdown

Rules binding more than one Product Component. A rule that binds only one lives in that component's
`02-rules/rules-<pc>.md`, from `mode: outline` up.

Every rule here is checkable and states no mechanism. Where a rule exists because an `AD-N` forbids
the alternative, the `AD-N` is named as its source — the invariant is the reason, this is the
behaviour a reviewer checks.

## Rules

| id | Rule | Binds | Source | Status |
| --- | --- | --- | --- | --- |
| BR-1 | A Marker's number is the number of its line in the Note. There is never a Marker without a line, or a numbered line without a Marker. | `finding`, `bundle`, `agent-access`, `sharing` | AD-1 · FR-8 | active |
| BR-2 | Marker numbers run from 1 upward with no gaps. Removing one renumbers every Marker after it, and its line with it. | `finding`, `bundle` | FR-8 · UC-5 | active |
| BR-3 | A Marker's comment may be empty. Its numbered line still exists. | `finding`, `bundle` | FR-8 | active |
| BR-4 | A Note may be empty. A Finding with no words is still a Finding. | `finding`, `bundle` | FR-2 · FR-7 | active |
| BR-5 | A change to what is on disk lands completely or not at all. Deleting a Finding, a Bundle, or a BundleItem deletes its files; saving a change to a Bundle writes both its stored document and its file. Either way, if any part fails then nothing changes, the Reviewer is told which file refused, and an unsaved edit survives so it can be tried again. | `finding`, `bundle` | AD-2 · FR-13 · FR-14 · FR-40 | active |
| BR-6 | Every destructive action is confirmed exactly once, and the confirmation states what will go and how many of them. | `finding`, `bundle`, `sharing` | FR-13 · FR-14 · FR-23 · FR-41 · FR-42 · UC-7 · UC-12 · UC-30 · UC-31 | active |
| BR-7 | Nothing is soft-deleted. There is no bin, no archive, and no state in which a deleted thing is still readable. | `finding`, `bundle`, `sharing` | BG-5 · AD-2 | active |
| BR-8 | An image is reduced once, when it is captured. No later step re-encodes or re-scales it. | `finding`, `bundle`, `sharing` | AD-4 · FR-4 | active |
| BR-9 | A change to the Quality Budget applies only to Captures taken after it. No stored image is ever re-encoded. | `finding`, `settings` | FR-5 · UC-13 | active |
| BR-10 | A Bundle is a snapshot. Editing a Finding, its Note, or its Markers after composition changes nothing in a Bundle that already holds it. | `bundle`, `finding`, `agent-access`, `sharing` | AD-9 · FR-10 | active |
| BR-11 | A Bundle's stored document is changed only by the composer writing it again over the Bundle's own copy. No surface edits a Bundle's document directly, and no change to a Bundle ever reads or writes a Finding. | `bundle`, `agent-access`, `sharing` | AD-9 · FR-40 | active |
| BR-12 | The store permits one Finding to belong to several Bundles, and each such Bundle keeps its own image copy. No surface offers it: a Finding that a Bundle already holds leaves the filmstrip, and the filmstrip is the only place assembly selects from. | `bundle`, `finding` | FR-10 · FR-13 | active |
| BR-13 | Composition refuses, naming the Finding, if any selected Finding's image file is missing. It never writes a Bundle with a broken image reference. | `bundle`, `finding` | AD-2 · FR-10 · UC-9 | active |
| BR-14 | Only a Bundle is ever readable by an agent. An unbundled Finding is invisible on every agent-facing surface. | `agent-access`, `sharing`, `bundle` | FR-20 · FR-24 | active |
| BR-15 | Every agent-facing surface is read-only. None of them creates, changes, or deletes anything. | `agent-access`, `sharing` | AD-5 | active |
| BR-16 | Exactly one Access Key is valid at a time. Issuing a new one revokes the previous one immediately. | `agent-access` | FR-19 · FR-22 | active |
| BR-17 | A refusal is always distinguishable from an empty result. "No Access Key" and "no Bundles" are never the same answer. | `agent-access`, `sharing` | AD-7 · FR-20 | active |
| BR-18 | Nothing leaves the machine unless the Reviewer confirmed a publish on a named Bundle. | all | AD-6 · NFR-11 | active |
| BR-19 | A publish that fails leaves nothing readable on the service, and leaves the Bundle unpublished locally. | `sharing` | FR-23 | active |
| BR-20 | An unpublish that fails leaves the Bundle marked published. The Reviewer is never told something is private when it may not be. | `sharing` | FR-25 · FR-26 | active |
| BR-21 | Publishing a Bundle that is already published replaces its content at the same URL. A second URL is never created for one Bundle. | `sharing` | FR-23 · AD-8 | active |
| BR-22 | A Publication slug is never reused for a different Bundle, including after an unpublish. | `sharing` | AD-8 · FR-25 | active |
| BR-23 | Deleting a published Bundle unpublishes it as part of the same action. | `bundle`, `sharing` | FR-14 · FR-25 | active |
| BR-24 | An unknown slug, a revoked slug, and a slug that never existed are refused identically. | `sharing` | NFR-15 | active |
| BR-25 | Only reduced images are ever transmitted. An unreduced capture never leaves the machine, because none is kept. | `sharing`, `finding` | AD-4 · NFR-12 | active |
| BR-26 | A Snapdown action bound to a hotkey that is unavailable is reported at the moment of binding, and again at startup if registration fails. It is never left silently broken. | `settings`, `finding` | FR-17 · NFR-7 · UC-15 | active |
| BR-27 | No two Snapdown actions share one hotkey combination. | `settings` | FR-17 | active |
| BR-28 | Capture works before anything is configured. A default Vault location is used until the Reviewer chooses one. | `settings`, `finding` | FR-16 · UC-14 | active |
| BR-29 | Changing the Vault location either moves every existing file or moves none. | `settings`, `finding`, `bundle` | AD-2 · FR-16 | active |
| BR-30 | Timestamps are UTC everywhere they are stored or transmitted. Local time exists only in what a person is shown. | all | cross-cutting.md § Timestamps | active |
| BR-103 | The Quality Budget always holds exactly one of five named states — `Auto`, `Sharp`, `Balanced`, `Small`, `Custom` — and `Custom` holds if and only if the Reviewer set a resolved value directly. There is no unnamed state, and the transition into `Custom` is visible in the interaction that causes it. | `settings`, `finding` | DEC-004 · FR-5 | active |
| BR-104 | Under `Auto`, the long edge and encoder quality applied to a Capture are a function of that Capture's region. They are never read back from a Setting, and two Captures of different sizes are never reduced by the same resolved pair. | `finding`, `settings` | DEC-004 · FR-5 · NFR-18 | active |
| BR-105 | Every stored Finding carries the long edge and encoder quality that were actually applied to it. A change to how a budget resolves never rewrites what an existing Finding says about itself. | `finding` | NFR-18 · BR-9 | active |
| BR-106 | Snapdown is one installed executable with two personas. The tray, the executable, and the window title never disagree about the product's name, and no second desktop executable is produced by a build. | `settings`, all | DEC-003 · FR-27 | active |
| BR-107 | No colour is defined for only one Windows theme, and no literal colour exists outside the token stylesheet. Every text element meets WCAG AA contrast against its own background in both themes. | all | NFR-16 · NFR-17 | active |
| BR-108 | A control that reports state owned by the operating system shows that it does not yet know, rather than showing an assumed value, until that state has been read. | `settings` | FR-18 · NFR-16 | active |
| BR-109 | Every primary surface of the Editor is reachable from every other primary surface, including a surface whose component is frozen and gaining no new behaviour. | `settings`, `finding`, `bundle`, `sharing`, `agent-access` | FR-28 · DEC-005 | active |
| BR-122 | A Bundle whose source Findings still exist can give them back; one whose source Findings are gone cannot. Which of the two holds is read from whether those Findings exist, never from a stored flag on the Bundle. | `bundle`, `finding` | FR-14 · FR-41 · BR-12 | active |

## Amended

A narrowed rule keeps its id and its old wording is recorded here, for the same reason a retired one
is never deleted: documents cite it, and a reader who finds a citation that no longer matches the rule
cannot tell whether the rule moved or the citation was wrong.

**BR-5, widened 2026-08-31.** It used to read: *"Deleting a Finding, a Bundle, or a BundleItem
deletes its files. If a file cannot be removed, nothing is removed and the Reviewer is told which file
refused."*

It covered destruction and said nothing about writing, because until `FR-40` nothing wrote over a
Bundle. Saving an edited Bundle now writes in two places — the `markdown` column in `library.db` and
the `bundle.md` file in the Vault — and `wdi-review` raised the gap: neither this rule nor any other
said what happens when the second write fails while the first has already landed.

Put to the owner on 2026-08-31 with two options, and they chose **one rule rather than two**: a partly
saved Bundle is not a state the product may leave behind. The rule is therefore widened rather than
joined by a sibling, which is also why its id does not change — the invariant was always
*all-or-nothing on disk*, and deletion was simply the only direction that existed when it was written.

The clause about the unsaved edit surviving is not decoration. Without it, all-or-nothing means the
Reviewer loses their typing to a full disk, which is the one outcome neither option was arguing for.

**BR-12, narrowed 2026-08-31, and the code is what narrowed it.** It used to read: *"One Finding may
belong to several Bundles. Each Bundle keeps its own image copy."*

Read as a promise about the product, that is false, and it had been false for as long as the filmstrip
has existed. `apps/desktop/src/main.rs:269-283` collects every Finding id that any Bundle holds and
filters those Findings out of the strip, with the intent stated in its own comment — *"The strip is the
queue of Findings not yet handed over, so anything a Bundle already holds leaves it."* The filmstrip is
the only place assembly selects from, so a second Bundle can never be offered a Finding the first one
took.

What survives is the half that is true, and it is true of the **store**: `bundle_item` is unique on
`(bundle_id, finding_id)`, not on `finding_id`, so two Bundles holding one Finding is legal data. The
new wording says exactly that and no more — the shape is permitted, no surface reaches it.

This was found while chasing a `wdi-review` finding which claimed that discarding one Bundle's
originals would silently seal another Bundle sharing a Finding. **That finding is void:** the situation
it describes cannot be reached. `BR-122` cites `BR-12` and was checked — it stays true either way,
because it reads the sealed state from whether the Findings exist, never from how many Bundles held
them.

**BR-11, narrowed 2026-08-31.** It used to read: *"A Bundle is never edited in place. A change means
composing a new Bundle."*

`FR-40` promises the Reviewer can correct a composed Bundle's title, its Bundle notes, and the note
text on the Findings inside it. The old wording forbade that outright, and it cited `AD-9` as its
source while saying more than `AD-9` says. `AD-9`'s clauses all govern the way **out** of a Bundle —
*"every handoff path MUST serve those exact bytes… no surface may re-render… on the way out"* — and
its closing clause directs a surface that needs a different shape to **change the composer**. Running
the composer again over the Bundle's own stored copy is that remedy, not a breach of it. `AD-9` is
therefore untouched and is **not** amended.

What the old wording was protecting is kept, and kept by construction rather than by prohibition: a
Bundle must not drift from what was handed over. `FR-40` writes only the Bundle's own copy and never
reads a Finding, which is why the three rules below stay true and **MUST NOT** be tidied into line
with this change:

- **BR-10** — a Bundle is a snapshot; editing a Finding afterwards changes nothing in a Bundle holding
  it. Still true, and it now carries more weight, not less: it is the rule that makes `FR-40` safe.
- **BR-65** (`bundle`-local) — opening a Bundle shows what was composed, not a live view of the
  Findings. Still true, for the same reason.
- **BR-59** (`bundle`-local) — composing does not remove the Findings it used. Still true; `FR-41` is a
  separate, later, explicit act and not part of composing.

**`sharing` and `agent-access` gain nothing and lose nothing.** `DEC-005` (applied) forbids new work on
both, and BR-11 binds them. Their obligation is unchanged in substance and in wording: neither may
edit a Bundle's document, and both still serve what the composer wrote. The narrowing removes a
prohibition that only ever bit `bundle`, because `bundle` is the only component that runs the
composer. Nothing here is new work on a frozen component.

**One clause was deliberately left out of the new wording.** The old rule was doing two jobs: saying
who may change a Bundle, and restating `AD-9`'s byte-identity promise on the way out. Only the first
is written above. The second is `AD-9`'s own and is currently in question from the other direction —
`FR-12` was amended on 2026-08-31 to permit the clipboard to render a Bundle's image links as absolute
paths, which is a different set of bytes on a handoff path. Restating byte-identity here would have
frozen that question by accident. It is owed a `DEC-` and is named in the report that accompanied this
amendment.

## Retired

None yet. A retired rule keeps its id, states what replaced it, and is never deleted — documents
still cite it.
