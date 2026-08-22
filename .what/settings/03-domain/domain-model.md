---
type: model
component: settings
layer: conceptual
created: "2026-08-22"
updated: "2026-08-22"
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
| Quality Budget — maximum long edge | How far a capture is downscaled | `finding` |
| Quality Budget — encoder quality | How hard the reduced image is compressed | `finding` |
| Capture hotkey | Which combination starts a Capture | `finding` |
| Open Editor hotkey | Which combination opens the Editor | `finding` |
| Open Editor after a Capture | Whether a Capture opens the Editor. Off by default | `finding` |
| Run at Windows startup | Whether Snapdown is running after sign-in | `settings` itself |
| Web service address | Where a publish goes | `sharing` |

The Access Key and the publish credential are **not** Settings. They are secrets, they live in the
Windows credential store, and they belong to `agent-access` and `sharing` respectively.

## Relationships

- One Library has **exactly one** set of Settings. There is no per-project, per-Vault, or per-Bundle
  override.
- A Setting is read by other components and written only by this one. Nothing outside this component
  may change one.
- Run at Windows startup is the only Setting whose value is a claim about something outside the
  Library. Its truth lives in the operating system, and this component reads it back rather than
  remembering what it asked for.

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
7. Run at Windows startup reflects the actual operating-system registration, not a remembered
   intention. → FR-18
8. No Setting holds a secret. → cross-cutting.md § Secrets
