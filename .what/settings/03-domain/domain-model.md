---
type: model
component: settings
layer: conceptual
created: "2026-08-22"
updated: "2026-08-23"
---

# Model — settings

Conceptual. No column types, no storage shape.

## Entities

| Entity | What it is | Identified by |
| --- | --- | --- |
| Setting | One persisted choice of this installation, held under a name the rest of the product asks for by | Its name |

There is one entity and it is deliberately generic. The alternative — an entity per choice — would put
the list of choices in the domain model, where adding one becomes a schema change rather than a
setting.

The named choices that exist, and who reads each:

| Setting | What it decides | Read by |
| --- | --- | --- |
| Vault location | Where Finding and Bundle files are kept | `finding`, `bundle` |
| Quality Budget | Which named intent governs reduction: `Auto`, `Sharp`, `Balanced`, `Small`, `Custom` | `finding` |
| Quality Budget — maximum long edge | The resolved downscale limit. Under `Auto` it is derived per Capture and is not a stored choice | `finding` |
| Quality Budget — encoder quality | The resolved compression. Under `Auto` it is derived per Capture and is not a stored choice | `finding` |
| Capture hotkey | Which combination starts a Capture | `finding` |
| Open Editor hotkey | Which combination opens the Editor | `finding` |
| Open Editor after a Capture | Whether a Capture opens the Editor. Off by default | `finding` |
| Run at Windows startup | Whether Snapdown is running after sign-in | `settings` itself |
| Web service address | Where a publish goes | `sharing` |

The publish credential is **not** a Setting. It is a secret, it lives in the Windows credential store,
and it belongs to `sharing`. The Access Key was a second such secret, belonging to `agent-access`,
until `DEC-016` withdrew both on 2026-09-04.

## Relationships

- One Library has **exactly one** set of Settings. There is no per-project, per-Vault, or per-Bundle
  override.
- A Setting is read by other components and written only by this one. Nothing outside this component
  may change one.
- Run at Windows startup is the only Setting whose value is a claim about something outside the
  Library. Its truth lives in the operating system, and this component reads it back rather than
  remembering what it asked for.
- The Quality Budget is **one choice with two derived companions**, not three independent Settings.
  The named intent is what the Reviewer sets; the long edge and the encoder quality are what it
  resolves to. Under `Auto` they are not stored at all — they are computed per Capture, which is why
  NFR-18 requires the resolved pair to be written onto the Finding rather than read back from here.
  Under `Custom` the Reviewer sets the two directly and the named intent follows them. Source:
  `DEC-004`.

## State Lifecycle

A Setting has no status. It has a value, which is either the shipped default or one the Reviewer chose.

Two conditions that look like states and are not:

- **Unset.** A Setting with no chosen value reads as its default. There is no third answer, which is
  what makes BR-28 hold — capture works before anything is configured.
- **A hotkey that failed to register.** That is a fact about the operating system at this moment, not a
  value of the Setting. It is reported (BR-26) and it does not change what the Reviewer chose.

## Invariants

1. Every Setting has a value at all times: the Reviewer's choice, or the shipped default. → BR-28
2. No two hotkey Settings hold the same combination. → BR-27
3. A hotkey Setting may be empty, which disables that action rather than leaving a broken binding.
   → FR-17
4. The Vault location is a folder that can be written to. A location that cannot be is refused at the
   moment of choosing, not at the next Capture. → FR-16
5. Changing the Vault location moves every existing file or moves none. → BR-29, AD-2
6. A Quality Budget change applies only to Captures taken after it. No stored image is re-encoded.
   → BR-9
9. The Quality Budget always holds exactly one of five named states, and `Custom` holds if and only if
   the Reviewer has set a resolved value directly. There is no unnamed state. → BR-103, BR-116, DEC-004
10. Under `Auto`, the resolved long edge and encoder quality are a function of the captured region and
    are never read back from a Setting. → BR-104, DEC-004, NFR-18
7. Run at Windows startup reflects the actual operating-system registration, not a remembered
   intention. → FR-18
8. No Setting holds a secret. → cross-cutting.md § Secrets
