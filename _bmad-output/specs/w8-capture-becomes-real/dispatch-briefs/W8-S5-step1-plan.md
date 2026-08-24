# W8-S5 · Step 1 — PLAN ONLY

**Halt after planning.** Do not implement. This step ends when the story spec file exists with
frontmatter `status: ready-for-dev`.

## Method position

WDI Method, **G5 Release**, wave **W8**, `wdi-build` Phase 3 Step 1. Read `AGENTS.md` first.

Run `bmad-build-auto` under folder+id dispatch:

- `spec_folder`: `_bmad-output/specs/w8-capture-becomes-real/`
- `story_id`: `W8-S5`

`W8-S1` through `W8-S4` have landed. Capture, reduction, the marker burn and the capture Note are
all real. **This story is the one that finds out what that broke.**

## What this story is

**The expensive half of the wave**, and the reason it is a story rather than a chore folded into the
other four.

Every test in this repository asserts **dimensions**, never pixels. That is precisely why a 17-byte
fake PNG survived five waves and three audits. Now that real bytes arrive, a large number of
assertions become wrong or meaningless.

**The dangerous outcome is not that they fail. It is that they KEEP PASSING while proving nothing.**

## `test_golden_markdown.rs` is the sharpest case

`AD-9` — *"One Bundle, one Markdown, byte-identical on every path"* — is enforced by a golden file.
Read `.how/_platform/ARCHITECTURE-SPINE.md:150-157` for the rule itself, and
`.how/bundle/SDD-bundle.md:143-147` for why `MarkdownWriter` being pure is what makes the golden test
possible at all.

The golden was generated against fabricated output. **A golden test over fabricated bytes proves
byte-identity of a fabrication.** It has to be regenerated from real output, and then shown to fail
when the bytes change — a golden nobody has watched go red is a file, not a test.

## Read AD-9 before you cite it

`AD-9` is about **Markdown** bytes. It says nothing about image bytes.

One of the test names below — `a_bundle_copies_the_same_bytes_as_the_finding_it_came_from` — is
about an **image** copy, and no `AD-N` currently backs it. **Find what does back it in `.what/` or
`.how/`, and if nothing does, report that as a corpus gap rather than citing `AD-9` for something it
does not say.** A test justified by a misquoted decision is how `BUG-3` and `BUG-10` sat unfixed for
a day.

## Every file that asserts on image bytes or dimensions today

`waves.yaml` names them, and all of them need reading:

```
core/src/domain/{finding,image,markdown,setting}.rs
core/tests/test_markdown_serializer.rs
store/src/image/{burner,pipeline}.rs
store/src/sqlite/{finding_store,migrations,settings_store}.rs
store/src/vault/sweeper.rs
store/tests/{test_bundle_deletion,test_golden_markdown,test_image_reduction,test_orphan_sweeper}.rs
```

## Report, do not quietly rewrite

**Where a test turns out to have been asserting nothing all along, REPORT IT as a finding.** That is
information about how this repository got here, and it is worth more than a tidy diff. A silently
corrected test erases the evidence of its own failure mode.

Two concrete precedents to recognise, both from `W7`:

- `a_failed_open_leaves_no_wal_or_shm_file_beside_the_database` reads correctly, asserts a plausible
  thing, and **still passes with its own defect reinstated** — SQLite removes those files on a clean
  close. It is accurate to its name and insensitive to the defect. Belt-and-braces, not a guard.
- Every corrupt-database fixture used garbage bytes, which SQLite rejects at `Connection::open`
  before a single pragma runs. The test never reached the code it claimed to cover. A valid header
  with corrupt pages is what reaches it.

**Prove a fixture reaches the code you think it does.**

## Mutation is the acceptance criterion, not a nicety

For each converted test: break the thing it claims to cover, watch it go red, put it back.

**Use `--no-fail-fast`.** Cargo stops at the first failing binary otherwise, later tests never run,
and a live test reads as dead. That produced a false result in `W8-S2` and cost a round trip.

## The tests

`waves.yaml` records four, carried through verbatim:

```
cargo::the_golden_bundle_markdown_is_regenerated_from_real_image_output
cargo::a_bundle_copies_the_same_bytes_as_the_finding_it_came_from
cargo::changing_one_pixel_of_a_source_image_changes_the_bundle_copy
cargo::no_image_test_asserts_only_a_signature_and_a_dimension
```

The last one is a **meta-test** and it needs thought rather than a literal reading. It cannot mean
"grep the test sources for a signature comparison" — that would assert a copy of the input, which is
the exact failure this story exists to clean up. Say in the plan what it actually asserts, and if the
honest answer is that it cannot be expressed as a cargo test, **say that and propose what replaces
it** rather than writing something that passes.

**Fixtures synthesised programmatically**, never recorded from a real screen. The repository is
public and the brief forbids a fixture derived from real capture output.

## Scope boundary — `W8-S6` owns the wiring

While writing this brief the coordinator found `BUG-19`: `bundle.rs:41-48` records a burned image
path and **nothing ever writes that file**; `MarkerBurner` is called from nowhere. That is `W8-S6`,
a separate story, and it is **not yours**.

It matters to you only in one way: **do not write a test here that would pass only once `W8-S6`
lands, and do not write one that silently accommodates the gap.** If an assertion you need depends
on that file existing, say so in the plan and leave it to `W8-S6`.

## Rules that bind every worker on this repo

- **Debugging is conditional, never a phase.** Unknown cause → `wdi-systematic-debugging` first; a
  third failed attempt is an escalation.
- **The corpus is not yours to change.** MUST NOT edit `.what/`, `.how/`, or an `applied` `DEC-`. A
  deviation is **reported** and becomes a `DEC-`.
- **Verification is run, not assumed.** Every command in `AGENTS.md` § Code, and read the four ways a
  verification run lies recorded there.
- **Write UTF-8, no BOM, and watch for a lone cp1252 byte.** Four story files in this wave have
  arrived with one or the other.
- **No scratch files in the commit. Do not push.**

## Done means

`_bmad-output/specs/w8-capture-becomes-real/stories/W8-S5-*.md` exists, carries an
`<intent-contract>`, and its frontmatter reads `status: ready-for-dev`.

Report `worker_done` with `--outcome succeeded` and the spec path, or `--outcome failed` with the
blocking reason.
