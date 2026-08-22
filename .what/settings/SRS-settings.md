---
type: srs
component: settings
status: draft
created: "2026-08-22"
updated: "2026-08-22"
satisfies: [FR-5, FR-16, FR-17, FR-18, NFR-6, NFR-7]
reviewed:
  date: "2026-08-22"
  sha: 1a67115
  lenses: [structure, prose, edge-case-hunter]
---

# SRS — settings

## Decision Summary · [G3]

Four choices, set once, that stop Snapdown becoming the thing being managed: where image files are
kept, which keys set it off, whether it is running when the Reviewer signs in, and how much picture
quality a screenshot is worth. A fifth, whether the Editor opens after a Capture, exists only because
that default may be wrong.

Nothing here is a preference panel for its own sake. Each setting is present because leaving it fixed
would break something the Reviewer cannot work around — a hotkey that clashes with their editor, a
folder on the wrong drive, a Snapdown that is not running when they need it. The one behaviour that
matters more than the values is honesty: a hotkey that cannot be registered is reported at the moment
of binding and again at startup, never left silently broken.

This is the only component at `mode: catalog`, so **its G4 is skipped by design**. `risk_accepted` is
`medium`: no money, no personal data, no irreversible action, and the worst outcome is a hotkey that
does not work, which is visible immediately.

## Why · [G3]

Because the other four components read these values and none of them should own them. The Vault
location is read by `finding` and `bundle`; the Quality Budget by `finding`; the web service address
by `sharing`; the hotkey bindings by `finding`. Put any one of them inside its reader and the others
have to reach across a seam to get it.

## Actor Register · [G3]

| Actor | Who they are | What they may do |
| --- | --- | --- |
| Reviewer | The person operating Snapdown. The only human actor and the only writer | Choose the Vault folder and move existing files, bind and clear hotkeys, turn run-at-sign-in on and off, set the Quality Budget, choose whether the Editor opens after a Capture, and set the web service address |

## UC Catalogue · [G3]

| id | Use case | Actor | Satisfies | critical |
| --- | --- | --- | --- | --- |
| UC-13 | I decide how much picture quality a screenshot is worth | Reviewer | FR-5 | no |
| UC-14 | I decide where my screenshots are kept | Reviewer | FR-16 | no |
| UC-15 | I change the keys that set Snapdown off, because one of them clashes | Reviewer | FR-17 | no |
| UC-16 | I have Snapdown ready the moment I sign in | Reviewer | FR-18 | no |

None is `critical`. Moving the Vault comes closest, and BR-29 makes it all-or-nothing rather than
irreversible: no file is lost, they are either all in the new place or all in the old one.

At `mode: catalog` this catalogue plus the three inventories plus C4 is the whole record a builder
gets for this component, and that is a finished state — not a placeholder waiting for flows.

## Constraints · [G3]

| Constraint | Source |
| --- | --- |
| A hotkey that cannot be registered is reported at binding time and again at startup, never swallowed | BR-26, FR-17 |
| No two Snapdown actions share one combination | BR-27 |
| Capture works before anything is configured — a default Vault location applies until one is chosen | BR-28 |
| Changing the Vault moves every existing file or none | BR-29, AD-2 |
| A Quality Budget change applies only to later Captures; no stored image is re-encoded | BR-9 |
| Hotkey and startup registration succeed without administrator rights | NFR-7 |
| The setting for run-at-sign-in reflects the actual OS registration, not a remembered intention | FR-18 |
| Secrets are not settings. The publish credential and the Access Key live in the Windows credential store | cross-cutting.md § Secrets |
| No network call originates here | AD-6 |

## Non-Goals · [G3]

- **Doing anything with the values.** Capturing, reducing, composing, and publishing all belong to
  their own components. This one stores and validates.
- **Holding secrets.** The publish credential and the Access Key are in the OS credential store;
  `agent-access` and `sharing` own them.
- **A second Vault, or switching between Vaults.** One at a time — OQ-11.
- **Per-project or per-workspace settings.** One Library, one set.
- **Theming, layout, or any appearance option.** Not a setting because leaving it fixed breaks
  nothing.
- **Import or export of settings.**

## Prerequisite · [G3]

- Windows 11, for the sign-in registration and the credential store.
- A writable location for `library.db` and a default Vault, so BR-28 holds on first run.
- Nothing external.

## Success Signal · [G3]

On first run the Reviewer picks a folder, changes the Capture hotkey because the default clashes,
turns on run-at-sign-in, and closes Settings — and the next hotkey press dims the screen. After
signing out and back in, Snapdown is in the tray with its hotkeys registered, no window opened, within
3 seconds (NFR-6), with no administrator prompt at any point (NFR-7).

## Assumptions, Risks, and To Be Confirmed · [G3]

### Assumptions

- Windows global hotkeys register from a user-level process without administrator rights — OQ-5.
- One Vault at a time is enough — OQ-11.
- The shipped Quality Budget default is usable without being changed — OQ-3.

### Risks

- **Detecting a hotkey conflict.** Windows does not always report a registration failure the way
  BR-26 needs. If the conflict cannot be detected at binding time, the promise in FR-17 has to become
  "reported at the next registration attempt", which is a weaker promise and a change to the `FR`.
- **Startup registration drifting from reality.** FR-18 requires the setting to reflect the OS, not a
  remembered intention. Reading it back on every open is the cheap answer; caching it is the tempting
  one.
- **Moving the Vault.** BR-29's all-or-nothing across a whole folder of files, on Windows, with files
  possibly held open. The safe answer may be to refuse the move rather than attempt it, and that is a
  design decision this component's `mode: catalog` does not document.

### To Be Confirmed

- The shipped default long edge and quality — OQ-3.

## Gate Checklist · [G3]

At `mode: catalog` only the starred questions are asked.

| Question | Answer |
| --- | --- |
| ★ Is every use case title a sentence a user would say? | Yes, all four in the Reviewer's own voice |
| ★ Any `FR` with no use case? | No. FR-5, FR-16, FR-17, and FR-18 each have one |
| ★ Do the inventories and this catalogue describe one system? | Yes. Tables 8–9, screen 12, and no endpoint |

## Design Reference · [G3]

Paired with `.how/settings/SDD-settings.md`, which stays a **skeleton** at `mode: catalog` — that is
the finished state for this component, not an omission.

Binding invariants: **AD-6** (nothing leaves the machine). At `mode: catalog` the quoted `Inherited
Constraints` section is not written, and the invariant binds regardless.

---

## Slots

All empty, and that is a finished state at `mode: catalog`. `03-domain/domain-model.md` is the one
exception — it is G3 output and exists at every mode.

## Open Items

- OQ-3 — the shipped Quality Budget default. `.control/questions/assumptions.md`.
- OQ-5 — hotkeys without administrator rights. `.control/questions/assumptions.md`.
- OQ-11 — one Vault at a time. `.control/questions/assumptions.md`.
