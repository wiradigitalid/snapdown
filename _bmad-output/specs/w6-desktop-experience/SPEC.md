---
id: SPEC-w6-desktop-experience
companions:
  - .control/registry/index.yaml
  - .control/registry/components.yaml
  - .control/registry/defects.yaml
  - .control/product-glossary.md
  - .what/business-rules.md
  - .what/settings/SRS-settings.md
  - .what/settings/02-rules/rules-settings.md
  - .what/settings/03-domain/domain-model.md
  - .what/settings/03-domain/state-machines.md
  - .what/settings/04-usecases/EXPERIENCE.md
  - .what/settings/05-scenarios/SCN-01-the-vault-move-that-fails.md
  - .what/settings/05-scenarios/SCN-02-the-first-run-and-the-startup-default.md
  - .what/finding/03-domain/state-machines.md
  - .what/finding/04-usecases/EXPERIENCE.md
  - .what/finding/05-scenarios/SCN-03-the-quality-budget-that-resolves-differently.md
  - .what/finding/05-scenarios/SCN-04-the-note-line-deleted-without-its-marker.md
  - .what/bundle/03-domain/state-machines.md
  - .what/bundle/04-usecases/EXPERIENCE.md
  - .what/bundle/05-scenarios/SCN-05-a-finding-deleted-out-from-under-a-bundle.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/design-system.md
  - .how/_platform/cross-cutting.md
  - .how/_platform/inventory-screen.md
  - .how/_platform/c4-l3-desktop-app.md
  - .how/settings/SDD-settings.md
  - .how/settings/01-ux/DESIGN.md
  - .how/settings/02-contracts/contract-inventory.md
  - .how/settings/03-integrations/windows-shell.md
  - .how/settings/04-components/LC-028-editor-shell.md
  - .how/settings/05-model/data-model.md
  - .how/settings/06-flows/flow-startup-reconciliation.md
  - .how/finding/SDD-finding.md
  - .how/finding/01-ux/DESIGN.md
  - .how/finding/02-contracts/contract-inventory.md
  - .how/finding/04-components/LC-003-image-reducer.md
  - .how/finding/05-model/data-model.md
  - .how/finding/06-flows/flow-capture.md
  - .how/bundle/SDD-bundle.md
  - .how/bundle/01-ux/DESIGN.md
  - .how/bundle/02-contracts/contract-inventory.md
  - .how/bundle/04-components/LC-013-bundle-store.md
  - .how/bundle/05-model/data-model.md
  - .constitution/project/codebase-stack-guide.md
  - .control/decisions/DEC-003-one-process-two-windows.md
  - .control/decisions/DEC-004-quality-budget-presets.md
  - .control/decisions/DEC-005-desktop-first-ordering.md
sources:
  - .what/_product-brief/brief.md
  - .what/_prd/capture-to-markdown/prd.md
  - .control/registry/requirements.yaml
  - .control/registry/usecases.yaml
  - .control/registry/waves.yaml
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete,
> preservation-validated contract for what to build, test, and validate. Source documents listed in
> frontmatter are for traceability only.

# W6 — The desktop experience rework

## Why

**Five waves shipped every capability the PRDs promised, and the first sustained use of the result
produced a list of experience defects rather than a list of missing features.**

The owner could not tell which application they had opened. They could not find the Editor. They
could not read the labels on Findings and Bundles against their own background. They were asked to
set a maximum long edge in pixels and an encoder quality — two numbers the team's own PRD admits have
never been measured. Snapdown was not running when they signed in.

Not one of those is a missing promise. Every one is a promise the surface failed to deliver, and the
root cause is a single absence: **no `wdi-ux` output had ever been written for this product.** No
document anywhere said what a screen owes. `BG-7` and `CAP-9` now carry that, and this wave is where
it is paid for.

Two of the ten stories are defects rather than new work — code disagreeing with a requirement that has
been `active` since G2. Both were found by taking `bundle` and `settings` to `mode: deep` and reading
the code against the documents.

**A third defect was found after this SPEC was written, and it outranks the rest of the wave.**
`BUG-4`: `capture.rs` opens the overlay window at `index.html?overlay=true`, and nothing in the
frontend reads `window.location.search`. There is one html entry point and no second bundle, so the
overlay window mounts the Editor shell instead of `CaptureOverlay`. **The capture path does not
work** — no dim, no crosshair, no region drag, no note field, no Finding. `FR-1`, `FR-2`, `UC-1` and
`UC-2` are unmet in the shipped build. It is folded into `W6-S2`, because deciding which root
component a window renders is a frame concern and `W6-S2` is where the frame gains an owner.

**A fourth, found in the same pass and equally critical.** `BUG-5`: the Editor never renders a
Finding's image. `FindingsEditor.tsx` shows metadata, a Note field, and the marker *count* as text;
`MarkerLayer` is exported and mounted nowhere. Markers cannot be placed, and `AD-1` — Markers and
Note lines are one sequence, the invariant the whole product is built on — has no user interface.
Folded into `W6-S7`.

`BUG-4` and `BUG-5` share one shape, and it is the finding underneath both: **every component in
this product has passing unit tests, and nothing tests composition.** `CaptureOverlay` and
`MarkerLayer` are both correct and both unmounted. A green suite proved the parts and never asked
whether they were assembled.

**A second habit, found the same way and equally systemic: `let _ =` on a `Result` that an invariant
depends on.** Three instances — `vault_migration.rs:141` and `:180` (`W6-S10`), and three paths in
`bundle.rs` (`BUG-9`, folded into `W6-S9`). One of them matters more than the rest: `delete_bundle`
swallows the unpublish, so a failed unpublish leaves the published copy **live on the public
internet** while the local record is deleted and the Reviewer believes it is gone. `BR-20` was
written to forbid exactly that outcome.

`let _ =` reads as deliberate, clippy does not flag it at default levels, and no test exercises a
failing filesystem — every store test uses a writable temp directory.

Who is affected: the Reviewer, on every screen. Nothing here is visible to an agent. `DEC-005` freezes
`sharing` and `agent-access`; their surfaces stay reachable (`BR-120`) and gain no behaviour.

## Capabilities

- **CAP-9** — The surface itself: name it, reach it, fit it, read it
  - **intent:** A Reviewer who has never seen Snapdown can say what the application is called and
    which part of it they are in, reach every part of it from wherever they are, and see everything a
    screen offers without hunting for it — in either Windows theme.
  - **success:** On a clean Windows 11 machine in the dark theme, at the window's minimum size of
    1024×720: the title bar reads `Snapdown Editor`; all four primary surfaces are listed and
    reachable from each of them; all four Settings groups are visible without scrolling; and an
    automated contrast assertion over every screen in both themes passes with no failures.

- **CAP-2** — Automatic image reduction (amended by `DEC-004`)
  - **intent:** The Reviewer chooses a Quality Budget by naming what they want of it, and never has
    to judge a number they have no way to judge.
  - **success:** A Reviewer who never opens **Advanced** captures a 312×118 tooltip and a 3840×2160
    screen; both are stored legibly and small; the two are reduced by **different** resolved
    parameters; and Settings can state in one word which budget produced each.

- **CAP-6** — Keep the tool out of the way (amended)
  - **intent:** Snapdown is running when the Reviewer signs in, without their having asked, and its
    controls never claim a state they have not yet read.
  - **success:** After a fresh install and a sign-out/sign-in, Snapdown is in the tray with its
    hotkeys registered. After the Reviewer turns it off and signs in again, it is **not**. The
    startup control renders a distinct not-yet-known state until Windows has answered.

## Constraints

- **AD-10 — colour has one authority.** Every colour is defined once, in
  `web/ui/src/styles/tokens.css`, and defined for both themes. **No component may contain a colour
  literal**, and a lint rule enforces it. A meaning background is used only through its paired
  foreground token. Three token groups are theme-invariant on purpose and must declare it where they
  are defined: `--color-marker*`, the capture overlay's scrim and region ring, `--canvas-checker`.
- **AD-11 — one process, one executable.** A build produces exactly one desktop executable. The
  executable name, the tray tooltip, and the window title derive from one source and can never
  disagree. `snapdown-bridge` is not an exception.
- **AD-4 — reduced once, at capture, no original kept.** The resolution in `LC-003` runs inside that
  window; nothing may write raw pixels to disk to make derivation easier to test.
- **`NFR-2`'s budget ends at dismiss, not at stored.** Reduction runs after the overlay is gone.
  Adding work to `LC-003` must not move it before the dismissal.
- **`BR-9` — no stored image is ever re-encoded.** Adding `Auto` does not rewrite existing Findings,
  which is exactly why `NFR-18` requires the resolved pair to be stored with each Finding.
- **`BR-117` — `Custom` has exactly one way in:** the Reviewer editing an Advanced value, visibly, in
  the same interaction. `Auto` resolving an unusual pair does not become `Custom`.
- **`BR-114` — a registration failure is a fact about the OS at a moment, not a value of the
  Setting.** It never overwrites what the Reviewer chose.
- **`BR-120` — a frozen component's surface stays listed.** `sharing` and `agent-access` are frozen by
  `DEC-005` and must remain reachable.
- **The corpus is not this wave's to change.** A worker MUST NOT edit `.what/`, `.how/`, or an
  `applied` `DEC-`. A deviation from an SDD or an `AD-N` is reported and becomes a `DEC-`.
- **Verification is run, not assumed.** Commands and their directories are in
  `.constitution/project/codebase-stack-guide.md`. A green `korpus.yml` is not proof the code compiles.

## Non-goals

- **No new capability for `sharing` or `agent-access`.** `DEC-005`. Their surfaces are touched only
  where `BR-120` requires them to stay reachable.
- **No annotation tools.** No arrows, callouts, highlights, blur, or effects. A PRD Non-Goal, and
  independently the conclusion Cobalt Capture reached for the same machine audience. Numbered Markers
  are the whole annotation vocabulary.
- **No editable Bundle Markdown.** A Bundle is recomposed, never patched. The preview gets no cursor.
- **No sub-navigation inside Settings.** All four groups on one surface. A sub-nav would satisfy
  `FR-29`'s letter by hiding three groups behind a click.
- **Agent access is not a Settings group.** It is a primary surface of its own (`FR-28`,
  `inventory-screen.md` row 13). Drawing it inside Settings puts one thing in two places.
- **No background task.** The hotkey health check was rejected, not deferred: it would be the only
  one in the product, and `NFR-6`'s idle budget is written for a product that has none.
- **No re-encoding of existing Findings**, and no migration that rewrites stored images.
- **No recovery of `bundle_item` rows already lost to `BUG-1`.** They are gone; the fix stops further
  loss.

## Success signal

At the end of this wave, on a clean Windows 11 machine, in **both** themes, at 1024×720:

1. `cargo test --workspace`, `npm --prefix apps/desktop run test`, and
   `npm --prefix web/ui run test` are green, including the 45 named tests in `waves.yaml`.
2. A grep for a hex literal under `apps/desktop/src` and `web/ui/src` outside `tokens.css` returns
   nothing, and the lint rule fails the build if one is added.
3. `target/release/` holds exactly one desktop executable, named `Snapdown.exe`.
4. A Reviewer who has never seen Snapdown reaches Findings, Bundles, Agent access and Settings from
   any one of them, without being told how.
5. All four Settings groups are visible without scrolling, and no group holds space it does not use.
6. Capturing a tooltip and a full screen on `Auto` produces two different resolved pairs, both stored
   with their Findings, and Settings names the budget for each.
7. Deleting a Finding that belongs to a Bundle leaves that Bundle's item list and Markdown intact.
8. **Pressing the capture hotkey shows the capture overlay and produces a Finding.** It does not
   today (`BUG-4`), and no other item on this list matters if this one fails.
9. **Opening that Finding shows its image, and clicking the image places a numbered Marker bound to
   a numbered Note line.** It does not today (`BUG-5`). Items 8 and 9 together are the product.
10. **A build produces a binary that loads its own frontend.** It does not today (`BUG-11`): without
    the Tauri CLI a release binary requests `devUrl` and shows `ERR_CONNECTION_REFUSED`. **Items 1
    through 9 cannot be checked in the product until this one is true** — every one of them is a
    claim about a running application, and there is currently no reproducible way to produce one.

## Order

Resequenced 2026-08-23, **risk first**:

`S1 → S2 → S7 → S9 → S10 → S3 → S4 → S5 → S6 → S8`

| Position | Story | Carries |
|---|---|---|
| 1 | `W6-S1` | The colour foundation every later story writes against. **Done** |
| 2 | `W6-S2` | `BUG-4` — the capture path does not work |
| 3 | `W6-S7` | `BUG-5` — the Editor never renders a Finding's image · `BUG-6` — the orphan report is unreachable |
| 4 | `W6-S11` | `BUG-11` — **the application cannot be built reproducibly.** Until this lands, nothing in this wave can be verified in the product |
| 5 | `W6-S9` | `BUG-1` — deleting a Finding guts every Bundle holding it · `BUG-9` — a deleted Bundle can stay live on the internet |
| 6 | `W6-S10` | The Vault move reports success while leaving an unreported duplicate |
| 7–11 | `S3` `S4` `S5` `S6` `S8` | Everything the owner originally reported. Unpleasant rather than wrong |

The original order put those four at positions 7, 9 and 10. Nobody chose that — it fell out of
linearising the DAG over shared `touches` to satisfy `V11`, and a sequencing artifact is a bad reason
to leave `BUG-1` corrupting the record of what was handed over for six more stories.

Worth being plain about what this reorders **behind** the defects: every complaint the owner actually
made — the unreadable panels, the wasted third of the Settings window, the two numbers nobody can
judge, the startup toggle. Those are real and they are the reason this wave exists. They are also
survivable in a way that a capture hotkey doing nothing is not.

## Assumptions

- `OQ-18` — four named budgets are distinguishable enough that a Reviewer picks between them rather
  than leaving `Auto` forever. If wrong, `DEC-004`'s reversal trigger fires and Advanced/Custom go.
- `OQ-19` — hiding rather than destroying the Editor window keeps webview memory unnoticed.
- `OQ-20` — Snagit and Cobalt Capture are the right experience benchmark for a product whose reader
  is a machine.
- `OQ-3` — restated, not closed: whether `Auto`'s output is legible at its smallest is unmeasured,
  and story W6-S4 cannot settle it. It asserts that the derivation **varies**; whether it varies to
  the right values needs a Reviewer looking at the output.

## Open questions

- **The exact `Auto` curve is not specified anywhere in `.what/` or `.how/`, and this SPEC does not
  invent one.** `LC-003-image-reducer.md` states three requirements it must satisfy — it varies with
  the region, quality is high where no downscale applies, quality is lower where a hard downscale
  applies — and explicitly leaves the curve as an implementation choice. W6-S4's builder chooses it
  and records the choice; a curve that fails any of the three is wrong, and one that satisfies all
  three is acceptable until `OQ-3` is measured.
- `OQ-21` — the story lists for W2–W5 were never written into `waves.yaml`, so six shipped use cases
  are scheduled to no story. Out of scope for this wave; recorded so it is not lost.

## Wrapper-only content

The narrative rationale in `DEC-003`, `DEC-004`, and `DEC-005` — the alternatives tables, the
reversal triggers, the Cost sections — is deliberately **not** lifted into this contract. It explains
why the decisions are what they are; the decisions themselves are in Constraints and Non-goals above.
The `DEC-` files stay in `companions:` for a reader who needs the reasoning.
