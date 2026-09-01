---
status: Accepted         # raised 2026-09-01; see § Provenance
ratified_by: a06a8f3     # the commit whose content ratifies this file
---

# conventions — codebase guide

**Loaded when:** writing or reviewing code.

## Reachability — a component ships with proof that something reaches it

Every component that a person can see or operate MUST ship with at least one test asserting that
**something mounts it** and that **each of its callbacks is bound**, in the same change that adds it.
A test proving the component behaves correctly does not satisfy this and MUST NOT be offered as if it
did — the two failures look identical from a green suite.

**In Slint that is two questions, and both MUST be asked:**

- Is the component **instantiated** in `apps/desktop/ui/*.slint`?
- Is each of its callbacks **bound** by an `.on_<callback>(` in `apps/desktop/src/`?

Either half missing means nobody can reach it. `apps/desktop/tests/test_annotation_wiring.rs` is the
worked example and new tests SHOULD copy its shape, including two details that are not decoration:
it collapses whitespace before matching, because `rustfmt` decides where a method chain breaks and a
guard a reformat turns red is a guard nobody keeps; and it strips `//` lines before asserting a
string is **absent**, because otherwise the comment explaining a removal makes the assertion fail and
the removal goes unexplained.

`V12` MUST NOT be relied on for this. It checks that an `LC` is *registered*, never that it is
*reached*, and the two are independent.

**Why the tax is worth paying.** One sweep on 2026-08-23 found four components built, unit-tested and
mounted nowhere — `CaptureOverlay` (`BUG-4`), `MarkerLayer` (`BUG-5`), `OrphanReportView` (`BUG-6`)
and `EmptyState` — leaving `FR-1`/`FR-2`, `FR-8` and `FR-15` unmet for four waves while every test
passed. `AGENTS.md` names it this repository's signature failure.

## Colour

Colour MUST be defined in `apps/desktop/ui/theme.slint` and nowhere else, for both themes (`AD-10`).
A literal outside it is a defect, not a style choice: a literal exists in exactly one theme, so it
paints correctly under one Windows setting and wrongly under the other.

The theme-invariant group — the overlay scrim, the selection ring, the loupe grid — is the one
exception and MUST stay in that same file, each with a comment saying why. Those three sit over a
*screenshot* rather than over chrome, and a light scrim on a dark capture is invisible.

Enforcement is a pair of Rust tests rather than a lint: `test_theme_contrast.rs` measures WCAG
contrast over every token in both themes, and `test_capture_interaction.rs` refuses a literal in the
overlay.

## Provenance

**Born empty on purpose, and filled on 2026-09-01 rather than at a wave's distillation.** The file's
own rule was that it MUST NOT be filled before code exists that ratifies it, which is why it stood
empty through eight waves. Both sections above are distilled from code that already runs:
`test_annotation_wiring.rs` for reachability, added by `a06a8f3` and carried in `ratified_by`, and
`theme.slint` plus its two guard tests for colour.

The reachability rule is the owner's answer to `OQ-23`, taken on 2026-09-01. It is **not** recorded as
a `DEC-`: a test convention is cheap to reverse, and this project records a decision only when the
answer to *why is it like this* cannot be read from the code. Extending `validate.py` was considered
and rejected as the wrong address — that file lives in `.constitution/method/` and is replaced in
full on every `wdi-method update`, so a check added there would not survive.
