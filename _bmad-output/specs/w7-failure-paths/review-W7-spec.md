# W7 · SPEC review — findings

- **Artifact:** `_bmad-output/specs/w7-failure-paths/SPEC.md`, `stories.yaml`, and the three story files
- **Lenses:** `structure` · `prose` · `edge-case-hunter`
- **Method position:** WDI Method, `wdi-review`; a wave `SPEC` always carries `edge-case-hunter`
- **Date:** 2026-08-24 · **Reviewer:** dispatched worker, read-only
- **Verified against:** `defects.yaml` (`BUG-3`, `BUG-10`, `BUG-12`), `DEC-005`, `waves.yaml` W7,
  `requirements.yaml`, `BR-17`, `BR-118`, `AD-7`, `AD-11`, `cross-cutting.md`,
  `SDD-settings.md` § Failure Behaviour, and the code at `HEAD` (`7c67dfb`)

**Nothing was edited.** Every finding below names the file, what is wrong, and what would fix it.

---

## Summary

Three high, ten medium, nine low. The wave's reasoning is careful and most of its quotations are
verbatim-accurate — but its first story describes code that no longer exists, and its two strongest
constraints (`BR-118`, "terminates gracefully without panicking") are both broken by the code path
the story says it is fixing.

The single most important finding is `H1`: **`W7-S1` is already implemented at `HEAD`.**

---

## High

### H1 · `W7-S1`'s defect was fixed four commits ago; the SPEC, the story, the register and `waves.yaml` all describe code that no longer exists

*Files:* `SPEC.md` § Why and § Capabilities/CAP-6 · `stories.yaml` `W7-S1` · the whole `W7-S1` story ·
`.control/registry/defects.yaml` `BUG-12` · `.control/registry/waves.yaml` `W7-S1`

Commit `aa30434` — *feat(W6-S5): Run at Windows startup* — is an ancestor of `HEAD` and it already
contains, in `apps/desktop/src-tauri/src/lib.rs`:

| The story plans | Already at `HEAD` |
|---|---|
| `StartupError::DatabaseOpen { path, source }` | `lib.rs:51-59` |
| `StoresBundle` | `lib.rs:61-67` |
| `init_app_stores(&Path) -> Result<StoresBundle, StartupError>` | `lib.rs:69-102` |
| `format_startup_error_message` | `lib.rs:105-120` |
| `show_native_message_dialog` via `MessageBoxW` | `lib.rs:122-152` |
| `report_startup_error` writing `startup-error.log` | `lib.rs:154-163` |
| The fallible setup hook returning `Err` | `lib.rs:226-233` |

`grep` for `.expect(` in that file returns **one** hit — `lib.rs:347`, the Tauri `run` call that
`BUG-12` deliberately did not register. **The five `.expect()` store opens at `lib.rs:109-119` are
gone.** Two of the four named tests already exist in
`apps/desktop/src-tauri/tests/test_startup.rs` under different names —
`an_unreadable_library_db_is_reported_with_its_path_and_not_recreated` (which covers three of the
four acceptance criteria, including the byte-identity check) and
`a_corrupt_library_db_does_not_panic_the_setup_hook`.

Everything in the wave rests on this. The SPEC's ordering argument ("`BUG-12` … is first because it
is the only one of the three that can make the whole product disappear"), the wave's `size: M`, the
`W7-S2 → depends_on: [W7-S1]` chain, and the story's entire *Approach* section are all written
against a defect that a previous wave closed as a side effect.

Note what this means about the story's provenance: the acceptance criterion asserting the exact
sentence *"Snapdown will not recreate or overwrite this file to prevent data loss."* is not a
specification — it is a transcription of `lib.rs:112`, which is the literal-assert failure the SPEC
itself forbids two sections earlier.

**What would fix it.** Re-scope `W7-S1` to the work that is genuinely open — renaming/adding the
four `waves.yaml` test names, adding `a_readable_store_still_starts_normally` (the only one with no
coverage today), and `H2`/`H3` below. Move `BUG-12` to `resolved` in `defects.yaml` naming `W6-S5`
and `aa30434`, with the residue re-registered as its own row. Re-weigh the wave's ordering and size
once `W7-S1` is a test story rather than a fix story.

### H2 · The setup hook does not terminate gracefully; it panics one line later, through the very `.expect()` `BUG-12` excused

*Files:* `W7-S1` story § Approach step 3 and § Intent · `.control/registry/defects.yaml` `BUG-12` note

`W7-S1` step 3 states the design returns `Err` from the setup hook *"so the Tauri runtime terminates
gracefully **without panicking** or creating any windows."* It does not. `lib.rs:346-347`:

```rust
.run(tauri::generate_context!())
.expect("error while running tauri application");
```

A setup hook returning `Err` makes `Builder::run` return `Err`, and that `.expect()` turns it into a
panic. The dialog is shown first, so the Reviewer is not left in the dark — but the process still
exits by panic, not gracefully, and the story's assertion is false as written.

The sharper half: `BUG-12` excluded `lib.rs:347` from the sweep on the reasoning *"If that fails
there is nothing left to report with."* That was true when every store open panicked before reaching
it. Under `W7-S1`'s own design it is now the **routine** exit path for a store failure, and the
exclusion's premise no longer holds.

**What would fix it.** Either delete the claim and say the process exits by panic after reporting,
or replace `.expect()` at `lib.rs:347` with a matched exit and record in `BUG-12`'s note that
`W7-S1` invalidated the exclusion.

### H3 · `BR-118` has a hole the story's own edge-case row walks into: `journal_mode = WAL` is set before `quick_check`, so a corrupt store *is* written to and files *are* created beside it

*Files:* `SPEC.md` § Constraints (`BR-118`) · `W7-S1` § I/O & Edge-Case Matrix row 2 and the third
acceptance criterion

`crates/snapdown-store/src/sqlite/settings_store.rs:24-38` (and the same shape in the four sibling
stores) opens the connection, then:

```rust
conn.pragma_update(None, "journal_mode", "WAL")?;   // <- writes to the file
conn.pragma_update(None, "foreign_keys", "ON")?;
conn.pragma_update(None, "busy_timeout", 5000)?;
// only now:
let integrity_res: String = ... "PRAGMA quick_check;" ...
```

Switching an existing database to WAL **mutates page 1 of the file** and creates `library.db-wal`
and `library.db-shm` next to it. So for the story's own matrix row 2 — *"valid SQLite header but
corrupt B-Tree pages"* — the acceptance criterion *"the byte content of `library.db` on disk is
identical to its pre-launch state, and no secondary or replacement database file is created"* is
false, and `BR-118`'s *"nothing is created over it"* and the SDD's *"a store recreated beside a
corrupt one is silent data loss"* are both grazed.

The existing test cannot detect this, and neither will the four named ones as written, because the
only fixture anyone has used is garbage bytes — SQLite rejects those at `Connection::open` before
any pragma runs. **The matrix names the dangerous case and the fixture avoids it.**

**What would fix it.** Add the valid-header/corrupt-pages fixture explicitly to
`a_corrupt_store_is_never_recreated_beside_itself`, and assert the absence of `-wal`/`-shm` as well
as byte identity. Then either run `quick_check` before any writing pragma, or open read-only for the
check first. Naming this in the SPEC's `BR-118` constraint would be better still — it is the exact
trade the constraint says must not be made.

---

## Medium

### M1 · The `DEC-005` reading is honest on the permission and silent on the vehicle

*Files:* `SPEC.md` § Why, last paragraph, and § Constraints

The quoted sentence is verbatim and correctly used — `DEC-005` § Why ends *"This decision does not
forbid a fix. It forbids new work."* The SPEC's reading of the permission is **not** stretched.

What it omits is the Cost section the brief points at: *"A defect in the frozen components has an
awkward home. It is a fix, not a wave, and the method has no third thing. It lands as a defect row
and a **patch release**, and that path is thinner than a wave's."* `DEC-005` therefore permits the
fix while saying the vehicle is not a wave — and `W7` is a wave.

`waves.yaml` W7 § scope *does* answer this: *"A wave is used here rather than a fastpath because
`BUG-3` contradicts `NFR-15`, and fastpath work that turns out to touch a requirement MUST stop and
be raised to a wave anyway."* That is a good answer. It is missing from the SPEC, which quotes only
the permissive half — so a builder reading the canonical contract alone sees a decision quoted in
support of a vehicle that decision names as wrong.

**What would fix it.** Carry `waves.yaml`'s two-sentence justification into `SPEC.md` § Why.

### M2 · The Open Question may be `DEC-005`'s reversal trigger firing, and the SPEC does not say so

*Files:* `SPEC.md` § Open Questions · `.control/decisions/DEC-005` § Reversal trigger

`DEC-005`'s second reversal trigger: *"A defect is found in `sharing` or `agent-access` that cannot
be fixed as a patch — **one that needs a new promise**. That is a re-plan, and it reopens the
ordering rather than bending it."*

The SPEC's Open Question says exactly that no promise covers output encoding and that the missing
promise is *"reported upstream to `wdi-product`"*. Whether `BUG-3` "needs" a new promise or merely
"lacks" one is a genuine judgement call — the fix is unambiguous without it, which is the SPEC's
argument and it is a fair one. But the decision names this shape as a re-plan trigger, and the SPEC
does not engage with that at all.

**What would fix it.** One sentence in the Open Question distinguishing the two: the *fix* is a
patch and needs no promise; the *promise* is a gap `wdi-product` will close later. If the author
does not believe that distinction holds, the trigger has fired and this is a re-plan.

### M3 · § Why and § Open Questions contradict each other about `BUG-3`

*Files:* `SPEC.md` § Why, first paragraph vs § Open Questions

§ Why: *"None of the three is a missing feature; **each is a promise the product already made and
does not keep**."*
§ Open Questions: *"No requirement covers output encoding on the published page … what is missing is
the promise it restores."*

Both cannot be true of `BUG-3`. The Why sentence is the load-bearing one for `DEC-005` compliance
(a fix restores an existing promise; new work makes one), so the contradiction sits directly under
`M1` and `M2`.

**What would fix it.** Qualify § Why: two of the three restore a written promise; `BUG-3` restores a
promise nobody wrote down, which is why it carries an Open Question.

### M4 · `W7-S2` is missing three template sections its two siblings have, and closes its contract block in the wrong place

*File:* `W7-S2` story

Against `.claude/skills/bmad-build-auto/spec-template.md`:

| Section | S1 | S2 | S3 |
|---|---|---|---|
| `## Boundaries & Constraints` | ✓ | **missing** | ✓ |
| `## I/O & Edge-Case Matrix` | ✓ | **missing** | ✓ |
| `## Code Map` | ✓ | **missing** | ✓ |
| `## Review Triage Log` | **missing** | **missing** | **missing** |
| `</intent-contract>` position | after the matrix (l.110) | **at end of file** (l.95) | after the matrix (l.133) |

S2 folds its boundaries into an ad-hoc *"Boundaries & Non-Goals (under DEC-005 freeze)"* bullet list
inside § Intent, with no Always/Never/Block If. And because `</intent-contract>` sits at the very
end, S2's Tasks, Acceptance Criteria, Design Notes and Verification are all *inside* the contract
block while S1's and S3's are outside — any reader or script that extracts the contract gets three
different shapes from one wave.

The omission bites hardest here: **S2 is the security story and it is the one story with no
edge-case matrix**, in a SPEC reviewed under `edge-case-hunter`.

**What would fix it.** Add the three sections to S2 with a real matrix (see `M6`, `L8`), move
`</intent-contract>` to after it, and add `## Review Triage Log` to all three.

### M5 · Acceptance criteria assert literals instead of behaviour — the mistake the SPEC forbids on the same page

*Files:* `W7-S2` § Acceptance Criteria (criteria 2 and 3) · `W7-S1` § Acceptance Criteria (criterion 2)

The SPEC § Constraints: *"A test that asserts a literal instead of the behaviour it claims to cover
is a defect, not a style choice. This repository has landed that mistake three times."* `W7-S2`'s
own Approach repeats it: *"Assert behavior, not hardcoded escaped literals."*

Then:

- S2 criterion 3: *"the title element contains the escaped slug `Snapdown Review - test&lt;slug&gt;&amp;42`"* —
  a hardcoded copy of `html.EscapeString`'s output. If the implementation switches to
  `html/template` (which the same story offers as an alternative and which emits `&#34;`/`&#39;`
  for quotes), the assertion breaks on a correct change and passes on an incorrect one that happens
  to reuse the same escaper.
- S2 criterion 2: *"the `</pre>` tag inside the note is escaped as `&lt;/pre&gt;`"* — same shape.
- S1 criterion 2 asserts the exact sentence *"Snapdown will not recreate or overwrite this file to
  prevent data loss."*, which is `lib.rs:112` copied verbatim (see `H1`).

**What would fix it.** State the behaviour: *the response body contains no element node the Note
introduced* — parse with `golang.org/x/net/html` or assert that `<script`, `</pre`, `<b` do not
appear as raw tags in the body while the Note's visible text does. For S1, assert that the message
contains the path and states non-destruction, not one frozen sentence.

### M6 · `W7-S2`'s slug fixture cannot run on Windows, and CI will not notice

*Files:* `W7-S2` § Acceptance Criteria criterion 3 · § Verification

The criterion requires publishing a bundle whose slug is `test<slug>&42`.
`apps/web-service/internal/store/store.go:103-128` does
`filepath.Join(s.dataDir, "blobs", slug)` and `os.MkdirAll` on it. `<` and `>` are **illegal in
Windows filenames**, so `Publish` fails and the fixture cannot be built.

`.github/workflows/desktop-ci.yml` runs `web-service` on `ubuntu-latest`, where it passes. This
repository's primary platform is Windows, and the story's own verification command
(`cd apps/web-service && go test -v ./...`) is the one that will fail. That is a green CI over a red
developer machine — a shape this repository's AGENTS.md already records twice.

Adjacent, and unnamed anywhere: a slug reaching `filepath.Join` unfiltered is a path-traversal seam
(`../../x`), and this AC is the first thing in the corpus to publish a non-CSPRNG slug.

**What would fix it.** Use a slug that is hostile to HTML but legal as a path segment — `a&b"c'd` —
or drive the escaping unit directly instead of through `Publish`. If the traversal seam is out of
scope, say so in a non-goal rather than leaving it discovered-and-unwritten.

### M7 · The dependency chain is invented; three disjoint stories are serialised, and the public XSS fix is last in line behind a story that is already done

*File:* `stories.yaml` / the story frontmatter — `W7-S2 depends_on: [W7-S1]`, `W7-S3 depends_on: [W7-S2]`

The three stories touch entirely disjoint trees: `apps/desktop/src-tauri` (Rust),
`apps/web-service` (Go), `crates/snapdown-bridge` (Rust). No file, symbol, schema, or test fixture
is shared. Nothing in the SPEC or `waves.yaml` claims a technical dependency; `waves.yaml`'s `why`
fields give only a severity ordering ("first because…", "last because it is genuinely the
smallest").

`depends_on` encodes a build-blocking relationship, not a priority. As written it forbids parallel
dispatch and — given `H1` — gates a public, unauthenticated HTML-injection fix behind a story whose
code already landed.

**What would fix it.** `depends_on: []` on all three, and express the ordering as ordering.

### M8 · `W7-S3` says it does not change the error shape; its remedy has the bridge invent an error, which `cross-cutting.md` forbids in as many words

*Files:* `SPEC.md` § Constraints (`AD-7`) · `W7-S3` § Approach step 2 and § Boundaries

`.how/_platform/cross-cutting.md` § Error envelope, second rule after `AD-7`: *"**The MCP Bridge does
not invent its own errors.** It maps a Local API envelope onto an MCP tool error, preserving `code`
and `message` verbatim."* And `error.code` is *"from the catalogue below"*, *always present*.

The prescribed replacements — `"HTTP 502: (failed to read error response: connection reset)"`,
`"HTTP 404: (empty error response)"` — are errors the bridge invents, free-form, carrying no code
from the catalogue. `W7-S3` asserts the opposite: *"`AD-7` binds the SHAPE and this story does not
change it."*

The catalogue has the right answers already: `internal` (500, *"something the producer did not
anticipate"*) for an unreadable body, `unavailable` (503) for a dropped connection.

Note this is a defect in the *claim*, not in the direction — a non-empty invented message is
unambiguously better than `""`, and `AD-7`'s own rule (*"A refusal MUST be distinguishable from an
empty result by its code, never only by its body being empty"*) is what makes the code the point.

**What would fix it.** Have `parse_error_response_reader` synthesise a catalogue code, e.g.
`format!("internal: HTTP {code}, the error body could not be read ({e})")`, and change the story's
boundary line from "does not change the shape" to "synthesises the one code the catalogue provides
for an unanticipated producer failure". If the author prefers the free-form string, that is a
deliberate deviation from `cross-cutting.md` and belongs in `.control/questions/`.

### M9 · A `MessageBoxW` with a `NULL` owner can open behind other windows — the exact "nothing happens" the wave exists to end

*Files:* `SPEC.md` CAP-6 § success · `W7-S1` § Approach step 2, § Design Notes

CAP-6's success criterion is *"a visible report"*. `lib.rs:122-147` calls `MessageBoxW(null_mut(),
…, MB_OK | MB_ICONERROR)`. With no owner window and none of `MB_SETFOREGROUND`, `MB_TOPMOST`, or
`MB_SYSTEMMODAL`, Windows may not bring the box to the foreground when the process has no foreground
activation — a double-clicked exe that fails at setup, with the Explorer window still focused, is
precisely that case. The Reviewer then sees a taskbar flash and nothing else, which is the defect.

Two more properties nobody has written down: the call **blocks the setup hook indefinitely** until
someone clicks OK, and `tauri-plugin-single-instance` is in the dependency list, so a second launch
while the box is up behaves in a way nothing specifies.

No unit test can catch any of this, which is why it belongs in the contract rather than in the code
review.

**What would fix it.** Add `MB_SETFOREGROUND | MB_TOPMOST` and state it as a constraint in the SPEC
next to *"`AD-11` leaves no surviving surface to report into"*. The remaining "is it actually
visible" question is a UI-verification item, not a `cargo test` item — say so in § Verification.

### M10 · `waves.yaml` names four `go::` tests whose names are not legal Go test identifiers, and no story says how they map

*Files:* `.control/registry/waves.yaml` `W7-S2` § tests · `W7-S2` § Approach step 2

The four names — `go::a_note_containing_markup_is_escaped_in_the_rendered_page` and siblings — are
reproduced **verbatim and correctly** in the story (see the clean-lens note below). But Go requires
`func TestXxx(t *testing.T)` with an exported name; `a_note_…` cannot be a test function. The
repository's one existing Go test is `TestWebServiceLifecycleAndPublicRoutes` — a different
convention again.

The obvious mapping is `t.Run("a_note_containing_markup_is_escaped_in_the_rendered_page", …)`
subtests under one parent, which makes `go test -run` match the registry name. Nothing says so, so
the builder will guess, and a guess here is a silent rename against a registry the gate checks.

**What would fix it.** One line in `W7-S2` § Approach naming the subtest mapping.

---

## Low

**L1 · Stale line references.** `SPEC.md`, `stories.yaml`, `W7-S2` and `BUG-3` all cite
`server.go:145`; the `fmt.Sprintf` is at `apps/web-service/internal/server/server.go:148-152`
(line 145 is a closing brace). `client.rs:155-163` is exact. `lib.rs:109-119` is gone entirely
(`H1`).

**L2 · The unwrap non-goal rests on a sweep a later wave invalidated.** § Non-goals says *"`BUG-12`'s
own register entry already swept and deliberately did not register three groups"*. A fresh sweep of
non-test `src/` finds **32** `unwrap`/`expect`: the 26 `Header::from_bytes`, the 2 in `mcp.rs`, the
1 at `lib.rs:347` — and **three the register does not name**, at
`crates/snapdown-core/src/domain/setting.rs:266-268` (`self.named.fixed_pair().unwrap()`),
introduced by W6-S4 after the 2026-08-23 sweep. They are infallible by construction (the match arm
guarantees `Some`), so the non-goal's *conclusion* holds — but its *premise* is stale, and it tells
the next sweeper a fourth group is already accounted for when it is not. **The other four non-goals
are real boundaries**, each anchored in a rule (`BR-118`), a decision (`DEC-005`), a missing artifact
(`inventory-screen` row 14), or a recorded gap (`OQ-21`). Fix: date the sweep, or add the fourth
group to `BUG-12`'s note.

**L3 · `startup-error.log` is a product behaviour with no promise, no test, and a swallowed write.**
`report_startup_error` (`lib.rs:156-162`) writes `<app_data_dir>/startup-error.log`. The string
appears **nowhere** in `.what/`, `.how/`, `.control/`, or `.constitution/`. `W7-S1` § Approach
mandates it — *"as a persistent audit trace"* — in a wave whose `waves.yaml` scope says *"none of
them adds behaviour"*, and none of the four named tests covers it. The write itself is
`let _ = std::fs::write(…)`, the exact habit `BUG-9`, `BUG-10` and `W6-S10` registered, inside the
wave whose theme is *a failure the code declines to report*. Defensible (there is nowhere left to
report a log failure), but it should be said rather than swallowed. Fix: name the file in
`SDD-settings.md`, or drop it from the story's contract; either way record the `let _ =` reasoning
the way `BUG-10`'s note records `main.rs:21-22`.

**L4 · `CAP-6`'s "startup" is not this startup.** The SPEC hangs CAP-6 *"(the startup half)"* on this
work, but `requirements.yaml` CAP-6 is *"Keep the tool out of the way — folder, hotkeys, startup"*
and its startup element is `FR-18`, *Run at Windows startup*. This work maps to `BR-118` and the
SDD Failure Behaviour row and to no FR under CAP-6 — which is correct for a defect fix
(`satisfies: []`), but the parenthetical reads as an FR mapping it does not have. Fix: *"(the
`BR-118` half)"*, or name the rule in the intent line.

**L5 · "The only standard, dependable mechanism" is an overclaim, and the crate-vs-raw-FFI choice is
left unconstrained.** `W7-S1` § Design Notes. `tauri-plugin-dialog` exists, and so does
`AllocConsole`; `MessageBoxW` is the *best* choice, not the only one, and the real reasons — no new
dependency, and it works before any Tauri surface exists — are stronger than the one given.
Answering the brief's question 6 directly: **the story adds no dependency and does not need to.**
`apps/desktop/src-tauri/Cargo.toml` gains nothing; the code at `HEAD` declares
`#[link(name = "user32")] extern "system" { fn MessageBoxW(…) }` by hand. That is a hand-written
`unsafe` FFI signature with no crate audit behind it, and neither the SPEC nor the story says which
of the two routes is wanted. `AD-11` justifies *that a native dialog is needed* — it says nothing
about how it is obtained, and the story's *"MUST be displayed … (`AD-11`)"* attributes to `AD-11` a
rule `AD-11` does not contain. Fix: say "raw `user32` FFI, deliberately, to avoid a dependency", and
attribute the MUST to CAP-6's success criterion rather than to `AD-11`.

**L6 · The SDD row `W7-S1` anchors on has a third column nobody addressed.**
`.how/settings/SDD-settings.md` § Failure Behaviour, `LC-025 → library.db`, *other side is lying*:
*"A store that opens and returns a schema version the code does not know is refused, named, and not
migrated."* `run_migrations` (`migrations.rs:148-175`) reads `MAX(version)` and applies every
migration greater than it — a database written by a **future** build (version > 7) opens, is not
refused, is not named, and is used against a schema the code does not know. `W7-S1` takes its
authority from this row and implements only its *absent* column. Whether it belongs in W7 is a scope
call; silence is not one. Fix: a named non-goal, or a defect row.

**L7 · The register's fix includes an affordance the SPEC drops silently.** `BUG-12` § fix: *"show a
window — or a native dialog — naming the file that could not be opened **and offering to open its
folder**."* Neither the SPEC nor `W7-S1` offers the folder, and no non-goal covers the omission. It
is a reasonable cut (`MB_OK` has no second button), but per the brief's rule that every SPEC claim
match its register entry, the cut should be visible. Fix: one non-goal line.

**L8 · The CAP-8 success sentence over-reaches past the HTML route.** *"reaches the browser as text
rather than as markup"* is true of `GET /b/{slug}` after the fix and not of `GET /b/{slug}/raw.md`,
which the story correctly keeps verbatim and which serves the same hostile bytes under
`text/markdown` with no `X-Content-Type-Options: nosniff` on any route. Blob serving is safe —
`store.go:201-205` forces `image/png` or `image/webp`, so no `.svg`/`.html` blob can be served as
markup. Fix: scope the sentence to the rendered page, and either add `nosniff` or name it a
non-goal.

**L9 · `CAP-7`'s title is truncated.** SPEC: *"Let an agent on this machine read a Bundle"*;
`requirements.yaml`: *"…, after the Reviewer hands it a key"*. The dropped clause is the access
control. `CAP-8`'s title is quoted exactly; `CAP-6`'s is trimmed with a parenthetical (see `L4`).

---

## Where the lenses found nothing — verified clean

These were checked directly and are correct. A clean lens is a result.

- **Test names match `waves.yaml` verbatim — all eleven.** S1's four, S2's four, S3's three, each
  character-for-character against the registry. No silent rename. (S2's `go::` prefix raises `M10`,
  which is a mapping question, not a rename.)
- **The Open Question is independently verified and its claim is true.**
  `.control/registry/requirements.yaml` holds exactly six `sharing` NFRs — `NFR-10` slug entropy,
  `NFR-11` nothing leaves the machine, `NFR-12` reduced images only, `NFR-13` revoke takes effect
  immediately, `NFR-14` one executable/one config, `NFR-15` no enumeration + identical refusal —
  and **none covers output encoding**. Nor do the four `sharing` FRs: `FR-23`–`FR-26` promise
  publish, retrieve, unpublish, and copy-the-URL; `FR-24`'s proof is that *"a plain HTTP client …
  retrieves the Markdown"*, which says nothing about how a page renders it. `NFR-8`'s CommonMark
  clause belongs to `bundle` and is about the Markdown file. The SPEC's reading of `NFR-15` — that
  it is about enumeration and identical refusals, not rendering — matches its text exactly. **No
  requirement covers this; the SPEC is right and right for the right reason.**
- **Does the SPEC invent a promise `.what/`/`.how/` does not hold?** No. Every Capability success
  criterion traces to something written: CAP-6 → `BR-118` + `SDD-settings.md` `LC-025`; CAP-8 →
  `NFR-15` (preserved, not extended); CAP-7 → `AD-7` + `BR-17`. The `satisfies: []` on all three
  stories is honest and `waves.yaml` explains it. The one thing the corpus does not describe is
  `startup-error.log` (`L3`) — and that is existing code, not something this SPEC invented.
- **Every quotation is verbatim and correctly attributed.** `DEC-005` (*"This decision does not
  forbid a fix. It forbids new work."*), `DEC-003`'s Cost sentence, `BR-118`, `BR-17`, `AD-7`'s
  rule, `AD-11`'s rule, and the `SDD-settings.md` `LC-025` row all check out word for word. The
  quoting is unusually disciplined for a document of this length.
- **`BUG-3` and `BUG-10`: every SPEC claim matches the register.** Severity, component, `contradicts`
  list, evidence, and the scope carve-outs (`main.rs:21-22` for `BUG-10`; the three swept groups for
  `BUG-12`) are reproduced accurately. `BUG-12` is the exception, and that is `H1`.
- **`BUG-10`'s evidence is exact against the code.** `crates/snapdown-bridge/src/client.rs:155-163`
  is the function, unchanged, `let _ =` and all, and the caller at line 145 does pass `code` — so
  the story's `parse_error_response_reader(code, …)` refactor needs no signature change upstream.
  The story's severity framing is honest, and its `main.rs:21-22` carve-out is correctly reasoned.
- **`W7-S3`'s edge-case matrix is genuinely complete for its unit.** Valid envelope, failed read,
  non-UTF-8, empty body, raw plaintext — five rows, and the non-UTF-8 row is right about Rust's
  behaviour (`read_to_string` restores the buffer on `Err`, so `body` really does stay empty). The
  only gap is the code-vs-message question at `M8`.
- **The frozen-component discipline holds.** No new route, no new FR, no new use case, no UX pass,
  no depth change. `W7-S2` explicitly pins `NFR-15` with a test rather than assuming it, which is
  the right instinct and is the strongest single line in the three stories.
- **Prose.** Clear, concrete, and consistent in register across SPEC and stories; the house habit of
  bold capitals for the load-bearing sentence is used consistently, not decoratively. Terminology
  is stable (Reviewer, Note, Bundle, Publication, Finding). The only prose defects worth raising are
  the internal contradiction at `M3` and the overclaim at `L5`.

---

## What the author should do first

1. **`H1`** — settle whether `W7-S1` is a fix story or a test story. Everything else in the wave is
   downstream of that answer.
2. **`H3`** and **`H2`** — two real defects in the code `W7-S1` is about, neither of which the
   current contract would catch.
3. **`M4`**, **`M5`**, **`M6`**, **`M10`** — `W7-S2` is the security story and it is the least
   finished of the three.
4. **`M1`**/**`M2`**/**`M3`** — the `DEC-005` argument is nearly right; it needs the vehicle
   justification carried over from `waves.yaml` and one contradiction removed.
