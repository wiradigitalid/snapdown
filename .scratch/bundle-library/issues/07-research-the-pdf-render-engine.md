# 07: Research the PDF render engine

**Type:** research
**Status:** resolved
**Blocked by:** None (can start immediately)

## Question

Export PDF is entirely greenfield in this repo — no crate, no code, no requirement, no prior
discussion. Choosing the render engine is the expensive-to-reverse decision underneath it, so
establish the facts before anything is committed.

**Leading candidate: `typst` as an embedded Rust library.** Verify or kill it on:

- **Embeddability.** Can it genuinely be used as a library from a Rust desktop binary, or is it
  realistically a CLI? How stable is that API across releases?
- **Binary size.** How much does it add to a shipped Windows desktop binary? This is the figure most
  likely to disqualify it.
- **Licence.** Compatible with how Snapdown ships?
- **Text layer.** Confirm output carries real selectable text, not a rasterised page. This is the
  one property that decides whether a machine can read the result at all.
- **Images.** Behaviour with large PNGs — memory, scaling quality, and whether a tall image can be
  kept off a page break.
- **Fonts.** Does it need fonts shipped alongside, and what happens on a machine without them?

**Alternatives to weigh against it:** `genpdf` and `printpdf` (lighter, but flowing text and
pagination become our problem), and the browser-engine route (`headless_chrome`, `wkhtmltopdf`) or
external binaries (`pandoc`, `weasyprint`) — note for these last two whether the dependency they drag
in is acceptable for a desktop app that should stay small and offline.

## Constraints already settled — do not reopen

- **PDF is an artifact for humans.** The agent hand-off path is Copy Markdown; agents prefer
  Markdown to PDF. Optimise for the reader, and only ensure a machine is not actively obstructed.
- **A4 only.** Letter and other sizes are deferred; adding one later changes nothing structural.
- **Document shape:** single column, ~2–2.5cm margins; a title block (Bundle name, date, Finding
  count) rather than a full cover page; one section per Finding as heading → image → notes → marker
  notes; page numbers and the Bundle name in the footer; **an image must never be split across a
  page break** — push it to the next page instead.
- Output must have a **real text layer** and populated PDF metadata (title, creation date).

## Deliverable

A recommendation with the numbers behind it — chosen engine, the binary-size cost, the licence, and
the one or two things that would make the decision wrong later.

## Answer

Resolved 2026-08-31. **Recommendation: `typst`, embedded via `typst-as-lib`** — with one number that
must be accepted before it is committed to.

No vendor publishes binary-size figures for any of these crates, so the researcher **built and
measured them** (Windows x86_64 MSVC, `--release`, thin LTO, `codegen-units = 1`, stripped):

| Build | Binary | Delta | Crates |
|---|---|---|---|
| Empty `fn main` baseline | 0.12 MiB | — | 0 |
| `typst` + `typst-pdf` + `typst-as-lib` | **35.6 MiB** | **+35.5 MiB** | 324 |
| `printpdf 0.12.7` (html, images) | 11.8 MiB | +11.7 MiB | 145 |

The +35.5 MiB carries **no fonts** — `typst-assets`' `fonts` feature is non-default, so a chosen set
is extra (Libertinus Serif regular/bold/italic ≈ 0.9 MB, DejaVu Sans Mono ≈ 0.33 MB).

**Licences are all permissive**, no copyleft: typst crates Apache-2.0, `typst-as-lib` MIT, `krilla`
and `pdf-writer` MIT-OR-Apache-2.0. Bundled fonts are SIL OFL 1.1 — bundling with software is
explicitly permitted, carry the licence text.

**Text layer verified by extraction, not assumed.** A compiled document with an embedded PNG was
re-opened and its text pulled back out intact, with a real embedded image object. Typst has also
emitted **tagged, accessible PDFs by default** since 0.14, with opt-in PDF/UA-1 and PDF/A — the
strongest machine-extractability story of anything surveyed.

**Embeddability is real**, not a CLI in disguise: `typst-as-lib` implements the `World` trait, a
static resolver feeds PNG bytes in by name, and nothing touches the filesystem. Typst's own
maintainers state on their forum that a stable high-level API is *not planned* and that
`typst-as-lib` is the right thing to use today.

### Five sharp edges, all reproduced

1. **35.5 MiB.** The one figure that could disqualify it. See the open question below.
2. **No stable library API, by design.** Every minor release has broken something (0.13 generic
   `compile`, 0.14 removed `Default for Library`, 0.15 reworked `typst-kit` and pushed MSRV to
   1.92). Pin exact versions and budget a small port per upgrade.
3. **A tall image with `block(breakable: false)` overflows the page rather than scaling down**
   (typst #6074, #2073). For a screenshot tool this is the failure mode that matters — tall images
   must be clamped to text height ourselves via `image(fit: "contain")` inside a height-bounded box.
   It does **not** degrade gracefully.
4. **Fonts must be shipped, and their absence fails silently.** A build with no fonts compiled with
   exit 0 and produced a 2,299-byte PDF containing no text. Embed a font set; never depend on system
   fonts, or the same document renders differently per machine.
5. **Markdown → typst markup is an escaping problem shaped like injection.** `#`, `$`, `@`, `*`,
   `_`, `<`, backslash in user text will be read as typst syntax. Needs a real escaper with tests,
   not a `format!`.

Also noted: typst re-encodes rather than passes PNGs through (#5278) — a 142 KB PNG produced a
178 KB single-page PDF, so a 30-screenshot Bundle will not be small.

### Runners-up

- **`printpdf` 0.12.7 — the near miss.** Not the low-level crate its reputation suggests: it now
  ships an HTML layout engine with break-aware pagination, MIT throughout, and at 11.7 MiB it is a
  third of typst's size. It lost on one reproduced fact: **`from_html` silently dropped an `<img>`**
  — zero image objects, zero warnings, across raw bytes, base64, and a data URI. Silent image loss
  disqualifies it for a screenshot product. **Worth re-testing on a future release — if images land,
  the size argument flips the recommendation.**
- **`krilla`** — what typst itself renders through, but does no layout or page-breaking. Choosing it
  means writing the paginator, which is the expensive part typst already contains.
- **WebView2 `PrintToPdfAsync`** — ~0 bytes added and Chromium-grade output on Windows 11, but means
  hosting a hidden WebView2 control purely as a PDF backend via `unsafe` COM, plus a fallback where
  the runtime is missing, and output then varies with the user's Edge version.
- **Dead or disqualified:** `genpdf` (unmaintained since 2021; only live fork is EUPL-1.2 copyleft),
  `wkhtmltopdf` (archived 2023), `pandoc` (GPL, ~40 MB, delegates to LaTeX anyway),
  `headless_chrome`/`chromiumoxide` (need Chrome on the user's machine — breaks offline),
  WeasyPrint (Python + native stack, unbundlable).
- **`markdown2pdf`** — the only maintained direct Markdown→PDF Rust crate, but built on `printpdf`
  and lexer-driven rather than a typesetter. If printpdf becomes attractive on size, test its image
  support specifically; it uses a different API than the broken `from_html` path.

**Markdown → typst markup → PDF is the normal path.** No maintained Rust crate does Markdown → PDF
well end to end.

### Follow-up round — the size objection does not survive measurement

Snapdown's current release binary is **24.4 MiB**, so typst would take it to ~60 MiB, a 2.5×
increase. That prompted a second measurement round, and it changed the picture.

**File size is not runtime cost.** A binary was built with typst fully linked but only invoked when
`argv[1] == "export"`, then measured both ways:

| Condition | exe | peak Working Set | WS-Private | Private Commit |
|---|---|---|---|---|
| baseline (empty `fn main`) | 0.1 MiB | 3.7 MiB | 0.4 MiB | 0.6 MiB |
| **typst linked, never called** | **36.6 MiB** | **3.9 MiB** | **0.4 MiB** | **0.8 MiB** |
| export, 1 page + 1 image | 36.6 MiB | 18.5 MiB | 2.3 MiB | 3.2 MiB |
| export, 12 pages + 12 images | 36.6 MiB | 20.0 MiB | 9.1 MiB | 10.1 MiB |

Adding 36 MiB to the exe costs **~0.2 MiB of RAM at idle**. Windows memory-maps the PE and
demand-pages it, so code never executed is never resident — note the last row, where a 36.6 MiB
binary peaks at 20 MiB, *less than its own file size*. The ~16 MiB is transient, only while an
export runs.

**So the cost is download and installer size only, not runtime.**

**Image embedding, verified by decoding the output** (not by checking a header — the pitfall
`AGENTS.md` names explicitly):

- **typst passes.** 12 pages, 12 images, all decoded back at full 1200×1600 RGB. Identical images
  are deduplicated automatically — 12 placements became one object, 191 KB total. Text layer still
  extractable, and `block(breakable: false)` held image and caption together on one page.
- **printpdf fails, silently.** `from_html` produced **zero `/Image` objects and zero warnings**,
  across all three input forms (`Base64OrRaw::Raw`, `Base64OrRaw::B64`, inline `data:` URI). This is
  now confirmed by decode, twice, and it settles the comparison: printpdf's size advantage cannot be
  claimed for a screenshot product whose images do not arrive.
- **`markdown2pdf` remains untested** — it sits on printpdf but uses a lower-level API than the
  broken `from_html` path, so its image support may be fine. Not assumed either way.

### ⚠️ The sidecar recommendation below was RETRACTED in the fourth round — see "Packaging: stay
in-process" further down. It is kept here because the reasoning is what the retraction argues
against.

### Recommended packaging: a separate exporter process

Not for size — the installer carries the same ~36 MiB wherever typst lives, and the runtime argument
is now dead. **For panic isolation.** `AGENTS.md` already records this as an expensive class of bug:
`AD-11` puts the tray, hotkeys, overlay and Editor in one process, and `DEC-003` accepted in writing
that *"a panic in the editor's Tauri commands kills the tray, the hotkeys, and the overlay with it."*
Adding 324 third-party crates (ICU, resvg, hayagriva, rayon) to the process holding the tray and the
global hotkeys widens the crash surface exactly where it costs most.

Measured cost of the sidecar: **17 ms** to spawn the 36.6 MiB binary, compile, embed an image and
write the PDF (27 ms cold, 17-19 ms after). Imperceptible for an explicit action. Implementation is a
second `[[bin]]` in the same Cargo workspace, invoked via `Command`, JSON in on stdin, PDF path out
on stdout. Side benefits: the main exe carries none of typst's dependencies, its build time does not
grow by ~5 minutes, and the supply chains stay separate.

**If this is recorded as a `DEC-`, state the reason as resilience, not lightness.** The runtime-weight
argument is measurably false and would mislead the next reader.

### Third round — the Markdown → typst escaper, settled by experiment

Two strategies were compiled for real, then had their text extracted back and compared against the
input:

| Strategy | Round-trips | Wrong | Failed to compile |
|---|---|---|---|
| A — escape characters in markup mode | **25/25** | 0 | 0 |
| B — code-mode string literal `#"..."` | **25/25** | 0 | 0 |
| C — control, no escaping at all | 6/25 | 11 | 8 |

**The harness was proven red before being trusted** — the no-escape control failed 19 of 25, and the
dangerous half were the ones that compiled cleanly and corrupted the text silently:

```
'issue #42 and #43'  ->  'issue 42 and 43'    exit 0
'back\slash\here'    ->  'backslashhere'      exit 0
'a * b * c'          ->  'a b c'              exit 0
'use `code` inline'  ->  'use code inline'    exit 0
```

**Recommendation: strategy B.** Both pass today, so the choice is about which survives typst's
churn. A is a blocklist of 18 characters that must stay complete as typst's syntax grows — and typst
has no stable API, with breaking changes every minor. B's escape set is **closed by definition**: a
string literal can only ever contain two special characters.

```rust
'\\' => "\\\\",   '"' => "\\\"",
'\n' => "\\n",    '\r' => "\\r",    '\t' => "\\t",
```

**B stays composable**, which was the real worry. Hostile text `cost #5 * 2 = $10 _really_` was
injected into a heading, bold, emph, bullet, enum and table cell: all six survived intact while the
surrounding markup kept working — heading stayed a heading, bullets stayed bullets, table cells
stayed cells — and a long paragraph still line-broke normally, so a string literal is not an atomic
run. The pattern is therefore: **structure** (heading/bold/list/table) is emitted as typst markup
from the `pulldown-cmark` parse; **leaf text** always goes through `#"..."`.

Twelve edge cases all round-trip (CRLF, empty, lone quote, lone backslash, trailing backslash,
`########`, `$$$$`, Indonesian text with em-dash). **One defect: tabs vanish silently** —
`col1\tcol2` came back `col1col2`. Convert tabs to spaces before insertion, or it becomes another
silent data loss.

### Packaging: stay in-process — the sidecar recommendation is withdrawn

The sidecar's only surviving justification was panic isolation, since the size and RAM arguments had
already been measured away. That justification was then tested directly and did not hold.

**Verified independently in this repo:** `Cargo.toml` contains no `[profile]` block and no `panic`
setting anywhere, so the default `unwind` applies. `catch_unwind` therefore works, and an in-process
guard gives the isolation the sidecar was for:

```
catch_unwind around an ordinary panic  -> caught = true
guard around a normal typst export     -> ok = Ok(Ok(15163))
```

The remaining worry was stack overflow, which `catch_unwind` cannot catch — typst's layout is
recursive. Pathological nesting was tried up to 100,000 levels:

```
depth 100 / 1000 / 5000 / 20000 / 100000  ->  clean error, exit_code=0
```

Typst enforces its own depth limit and returns an error rather than crashing. No input reached a
stack overflow.

So all three sidecar arguments are now closed: **no size saving** (+0.2 MiB idle), **no RAM saving**
(same measurement), and **panic isolation is available in-process**. Staying in-process also means
the architecture does **not** contradict `AD-11`, so the mandatory `DEC-` raised in the third round
is no longer required.

Two secondary sidecar benefits do survive but are not load-bearing: the main binary would not carry
typst's 324-crate tree, and cold build time would not grow by ~5 minutes. Neither justifies a process
boundary on its own.

**Honest residual risk:** `catch_unwind` does not catch `abort` or OOM, so a sufficiently extreme
document could still take the process down. The realistic path there is closed by typst's own depth
limit.

### Packaging is DEFERRED to the Export PDF effort — do not settle it here

This ticket's job was to choose a render engine. It did. Packaging — in-process versus a separate
exporter crate — was analysed across three further rounds and the recommendation reversed **twice**
(sidecar → in-process → sidecar). That oscillation is the finding: it is an architectural judgement
call, not a fact question another measurement round will settle, and Export PDF is not being built
yet.

What is established and worth keeping, so the next reader does not redo it:

- **`AD-11`'s actual text**, verified at `.how/_platform/ARCHITECTURE-SPINE.md:173`: *"Exactly one
  desktop process MUST own the Library. […] A second desktop executable MUST NOT be produced by a
  build. The `mcp-bridge` is not an exception: it holds no Library state, writes nothing, and
  reaches the Library only through the Local API."*
- **What it guards** (its *Prevents* section): two writers to one SQLite file and one Vault, **and**
  the product disagreeing with its own window about its name — which has happened here, a stale
  `desktop.exe` beside `Snapdown.exe` that led the owner to report four defects that did not exist.
- **The precedent is real.** `snapdown-bridge.exe` already ships as a second executable under the
  stated justification, and an exporter fits that justification at least as cleanly: it holds no
  Library state and reads nothing from it, since its input is handed to it by the parent.
- **The mechanical test would not object.** `apps/desktop/tests/test_executable_identity.rs` only
  asserts that `apps/desktop/Cargo.toml` declares exactly one `[[bin]]` named `Snapdown`; an
  exporter as its own crate passes it untouched. Passing that test is **not** the same as satisfying
  `AD-11`'s intent — the identity-confusion half is untested and unaddressed.
- **In-process is viable**: no `panic = "abort"` anywhere in `Cargo.toml` (verified), so
  `catch_unwind` works, and typst returns clean errors at 100,000 nesting levels rather than
  overflowing the stack.
- **A separate crate's real advantages**: hard isolation against `abort`/OOM/stack overflow, which
  `catch_unwind` cannot catch; and typst's 324-crate tree and 5m34s cold build stay out of the
  desktop crate.
- **The call shape would be trivial either way** — argv in, exit code out, file paths rather than
  content (Windows caps a command line at 32,767 characters, so Markdown cannot ride argv).

Whoever takes Export PDF decides this, and if a separate crate is chosen it is worth a `DEC-` —
written as a statement of reasoning, in the shape `DEC-008` already uses (*"It contradicts no
`AD-N`: `AD-11` requires…"*), not as a violation.

### Image height — solved in two stages, measured from the PDF itself

Placement was read back out of the PDF's transformation matrices rather than trusted from the code
that wrote it. Page 566.9 × 793.7 pt, text box 481.9 × 708.7 pt.

**The overflow is real, and silent.** With a naive `image(width: 100%)`, an 800×9000 screenshot is
drawn **5421 pt tall on a 793 pt page** — about seven pages of image stacked on one, most of it off
the canvas and simply gone. No error, no warning. That is the red control which makes the rest of
this measurement mean something.

**Stage one, clamp.** Dimensions are read in Rust straight from the PNG's IHDR header, so typst is
handed exact point values and never guesses — **no `image` crate needed**. Aspect ratio held exactly
in every case, and crucially `normal` and `wide` came out **byte-identical to the naive version**:
images that already fit are not touched.

**But clamping alone is not enough.** An 800×9000 screenshot fits at **53.5 pt — 11% of text
width** — technically correct and practically unreadable. A full-page web screenshot is ordinary
input for this product, so this is not an edge case. Fit-to-width starts breaching the height limit
once **aspect > 1.25**; a phone screenshot (9:16, aspect 1.78) lands at 56% width, still fine.

**Stage two, slice across pages.** Above aspect 3, the image is drawn at full text width and cut per
page using typst's own `box(clip: true)` + `place(dy:)` — **no pixels touched**, so still no image
dependency. An 11.25-aspect image became 9 full-width pages, verified:

```
distinct embedded image streams: 1  ->  referenced 9 times
source png 239,012 bytes  |  pdf 349,498 bytes
even step between slices: [-602.4]   all offsets distinct: True
```

Embedded **once** and referenced nine times — a nine-page PDF costs 349 KB from a 239 KB source, not
nine copies. Slice step is exactly 602.4 pt with no variation, matching the clip window, so slices
butt together with no gap and no overlap.

| Aspect (height/width) | Treatment |
|---|---|
| ≤ 1.25 | Full text width, unchanged |
| 1.25 – 3.0 | Scaled so height ≤ 85% of text height, aspect preserved |
| > 3.0 | Full text width, sliced across pages |

The 85% leaves room for a caption so image and caption still fit together inside one
`block(breakable: false)`.

**Both thresholds are choices, not measurements.** 3.0 is where remaining width falls to about a
third of text width; 85% was not calibrated against real caption heights. All testing used synthetic
images and one font. **Recalibrate against real Snapdown screenshots** before shipping.

### Still open

- Whether ~60 MiB of **download/installer** size is acceptable — the runtime objection is measured
  and closed; the distribution one is not.
- Whether to test `markdown2pdf`'s lower-level image path. Low value now: it sits on the same
  printpdf whose image pipeline is confirmed broken, and its lexer-driven layout is weaker than a
  typesetter's for image-heavy documents.
