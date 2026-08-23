---
type: rules
scope: component
component: settings
status: draft
created: "2026-08-23"
updated: "2026-08-23"
---

# Business Rules — settings

Rules binding **only** this component. A rule that turns out to bind a second one is promoted to
`.what/business-rules.md`, never copied.

Ids continue the global sequence. This file is born late: `settings` sat at `mode: catalog` until
2026-08-23, and `catalog` skips G4, so it never had one.

Eight of this component's rules are already cross-component and are **not** repeated below: `BR-26`
(a hotkey that cannot be registered is reported, never swallowed), `BR-27` (no two actions share a
combination), `BR-28` (capture works before anything is configured), `BR-29` (a Vault move is
all-or-nothing), `BR-103` and `BR-104` (the Quality Budget's named states and Auto's derivation),
`BR-106` (one executable, two personas, one name), `BR-108` (no assumed value for state the OS owns).

## Rules

| id | Rule | Binds | Source | Status |
| --- | --- | --- | --- | --- |
| BR-110 | A Setting is written by this component alone. Every other component reads and never writes. | `settings` | AD-11 · domain model § Relationships | active |
| BR-111 | A Setting with no chosen value reads as its shipped default. No caller ever handles an "unset" answer. | `settings` | BR-28 | active |
| BR-112 | *Run at Windows startup* ships **on**, applied once on a first run where nothing was configured. It is never re-applied over a Reviewer who turned it off. | `settings` | FR-18 · UC-16 | active |
| BR-113 | A hotkey Setting may be empty. Empty disables that action and is a valid stored value, distinct from a value that failed to register. | `settings` | FR-17 · UC-15 | active |
| BR-114 | A registration failure is a fact about the operating system at a moment in time, not a value of the Setting. It never overwrites what the Reviewer chose. | `settings` | BR-26 | active |
| BR-115 | A Vault location is validated by *writing* to it, not by inspecting its permissions. | `settings` | FR-16 · UC-14 | active |
| BR-116 | The Quality Budget's named state and its resolved pair are one Setting with one write. They can never be observed disagreeing. | `settings` | BR-103 · DEC-004 | active |
| BR-117 | `Custom` is entered only by a Reviewer editing a resolved value directly, and the transition is visible in the same interaction. `Auto` resolving an unusual pair does not become `Custom`. | `settings` | BR-103 · DEC-004 · UC-13 | active |
| BR-118 | The settings store is opened, never created over. A store that cannot be read is reported with its path, and no fresh one is started beside it. | `settings` | AD-2 | active |
| BR-119 | No Setting holds a secret. | `settings` | cross-cutting § Secrets | active |
| BR-120 | Every primary surface of the Editor is listed in the shell, including one whose component is frozen. Reachability is not conditional on a component being under active work. | `settings` | BR-109 · FR-28 · UC-25 | active |
| BR-121 | The product name is read from one source at build time. The executable name, the tray tooltip, and the window title are derived from it and never written independently. | `settings` | BR-106 · FR-27 · UC-24 | active |

## Two rules that look local and are not

**"A hotkey change takes effect without a restart."** `finding` owns the capture action the hotkey
triggers, and the re-registration crosses both. It is `BR-26`'s territory.

**"A Quality Budget change applies only to later Captures."** `BR-9`, cross-component, because
`finding` is what must not re-encode. Written here it would be a second copy that drifts.

## BR-112, and why a default earns a rule

A default that re-asserts itself is a bug wearing a default's clothes, and the distinction is
invisible in code reading `if (!configured) enable()` — because *configured* is doing work nobody
wrote down. The rule names the two states apart: **nothing was ever configured** is not the same as
**the Reviewer turned it off**, and only the first takes the default.

Recording that requires a tri-state — unset, on, off — which looks like a contradiction of `BR-111`
and is not. Unset still *reads* as the default to every caller; it is additionally *recorded* as
unset, and only `BR-112` ever looks at that.
