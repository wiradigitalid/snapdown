# Product Glossary

**Loaded when:** writing any document in the corpus.

The SSOT for **product** vocabulary — what this product talks about. Every term is defined **once**
here, then used as-is across the corpus.

**Method** vocabulary lives in `.constitution/method/method-glossary.md` and MUST NOT be redefined
here. The split test: does this term still hold if used in another product? Yes →
`method-glossary.md`, no → here.

## Rules

- A new term appearing in any document MUST be added here **in the same pass**.
- A definition MUST name its relationship to other terms and its cardinality where relevant.
- One term MUST NOT have two entries.
- This file is born **empty** and filled from the product. Its first entries are born with the brief at G1.

## Entries

<!-- Alphabetical. Format: **Term** — definition. Relationship. Cardinality where relevant. -->

- **Access Key** — stood here, naming the secret string a Reviewer copied out of Snapdown and pasted
  into an agent conversation to grant that agent read access to the Library over the **Local API**,
  until `DEC-016` withdrew it on 2026-09-04. Not reused.
- **Advanced** — the disclosure in Settings holding the Quality Budget's resolved numbers, the maximum
  long edge and the encoder quality, for direct entry. Editing either moves the Quality Budget to
  `Custom`. Source: `DEC-004`.
- **Auto** — the shipped Quality Budget. It derives a maximum long edge and an encoder quality from
  each captured region, so a small region and a full screen are not reduced by the same rule. Source:
  `DEC-004`.
- **Balanced** — the fixed Quality Budget that does not vary with the capture. One of three named
  alongside **Sharp** and **Small**. Source: `DEC-004`.
- **Bundle** — a named group of Findings, composed once into one Markdown document with its own copy
  of the images it references. A Bundle has zero or one Publication. A Finding may belong to zero,
  one, or several Bundles. Its title and its notes can be corrected afterwards; the set of Findings
  in it cannot change, so a different selection means a new Bundle. Source: `FR-40`, `BR-11`.
  Until 2026-08-31 this entry ended *"A Bundle is recomposed rather than edited"*.
- **BundleItem** — the membership of one Finding in one Bundle, holding the Finding's position in
  that Bundle and the copy of the image that was written for it. Exactly one BundleItem per Finding
  per Bundle; deleting the Bundle deletes all of them.
- **Capture** — the act of selecting a screen region and storing the resulting image. One Capture
  produces exactly one Finding.
- **Capture Overlay** — the full-screen, semi-transparent surface, one per monitor, on which the
  Reviewer drags out the region to capture. It exists only between the hotkey press and the moment
  the Finding is saved or the Capture is cancelled.
- **Custom** — the Quality Budget state a Reviewer reaches by editing an **Advanced** value. It is a
  named state rather than a silent condition so that Settings can always answer *which budget am I on*
  in one word. Source: `DEC-004`.
- **Editor** — the Snapdown Editor: the desktop window that lists the Library, shows each Finding
  with its Note and Markers, and is where Bundles are composed. There is exactly one Editor window,
  and it is one of Snapdown's two **personas** rather than a second application. Source: `DEC-003`.
- **Export** — rendering a Bundle as a **PDF**, and nothing else. Snapdown has no Markdown export:
  the Markdown is the product's output format rather than one of several, so the path that hands it
  over is **Copy** (`FR-12`) and the path that opens its folder is **Open file location** (`FR-43`).
  Source: `FR-39`, `CAP-12`, and the Product Brief, which already put it this way at G1 — *"the output
  format is the point rather than an export option"*. Settled as vocabulary on 2026-08-31.
- **Finding** — one observation: a captured image, the Note that describes it, and the Markers drawn
  on it. The atomic unit of the product; nothing smaller is handed to an agent. A Finding has exactly
  one image file in the Vault.
- **Handoff** — the act of giving a Bundle to an agent. Two shapes carry the same content: the
  Reviewer copying the Markdown and pasting it themselves, or fetching a Publication over HTTPS. A
  third shape — reading it over MCP with an **Access Key** — stood here until `DEC-016` withdrew it
  on 2026-09-04.
- **Library** — the whole set of Findings and Bundles held on this machine, together with their
  metadata. One Library per installation.
- **Local API** — stood here, naming the loopback-only HTTP interface over the Library that the
  **MCP Bridge** called, until `DEC-016` withdrew it on 2026-09-04. Not reused.
- **Marker** — a numbered badge placed by the Reviewer on a Finding's image. Marker `n` is bound to
  line `n` of that Finding's Note; the two share one sequence and are never kept in sync as two
  things. A Finding has zero or more Markers, numbered from 1 with no gaps.
- **MCP Bridge** — stood here, naming the separate executable that spoke the Model Context Protocol
  to an agent and the Local API to Snapdown, until `DEC-016` withdrew it on 2026-09-04. Not reused.
- **Note** — the Reviewer's prose about one Finding: a free-text body, plus one numbered line per
  Marker. Exactly one Note per Finding; it may be empty.
- **Orphan report** — the Editor surface listing image files present in the Vault that no Finding or
  Bundle points at, with the option to delete them. Source:
  `.what/_prd/capture-to-markdown/prd.md` § FR-15; `.how/_platform/inventory-screen.md` row 7.
- **Persona** — one of the two faces of the single `Snapdown.exe` process. **Snapdown** is the tray
  icon that owns the global hotkeys and the Capture Overlay and has no window; **Snapdown Editor** is
  the workspace window. They are personas and not processes: there is one executable, one Library
  writer, and one lifecycle. Source: `DEC-003`. Two personas, one process, always.
- **Publication** — the record of a Bundle having been published: the unlisted URL it was served at,
  when it happened, and whether it is still live. Unpublishing ends a Publication without deleting
  the Bundle.
- **Quality Budget** — the named intent that governs image reduction, chosen by the Reviewer from
  five states: `Auto` (the shipped default), `Sharp`, `Balanced`, `Small`, and `Custom`. It resolves
  to a maximum long edge in pixels and an encoder quality, which `Auto` derives from each captured
  region rather than holding as constants. Applied to every Capture on the way into the Vault, never
  afterwards; the resolved values are stored with the Finding they produced. One Quality Budget per
  Library. Source: `DEC-004`; `.what/_prd/capture-to-markdown/prd.md` § FR-5.
  This entry previously named the setting *pair* — a long edge and an encoder quality — which was the
  shape before `DEC-004`. The pair still exists, one level down, and is reachable through **Advanced**.
- **Reviewer** — the person operating Snapdown: the one who captures, writes Notes, composes Bundles,
  and decides what is handed off. The only human actor.
- **Setting** — one persisted preference of the installation: the Vault location, a hotkey binding,
  the Quality Budget pair, whether Snapdown starts with Windows, whether the Editor opens after a
  Capture, and where the web service lives. One value per key, one set per Library.
- **Sealed** — the condition of a Bundle whose source Findings have been discarded. A sealed Bundle
  stays readable on its own copies of the images and can still be corrected, exported and deleted; it
  can no longer give its Findings back to the Library. It is read from whether those Findings exist,
  never from a stored flag. Source: `FR-41`, `BR-122`; the word is ticket 02's on the Bundle
  Library map — *"Findings go, Bundle stays and seals"* — not a coinage of this pass.
- **Sharp** — the fixed Quality Budget that keeps small text crisp, at a larger file size. Source:
  `DEC-004`.
- **Small** — the fixed Quality Budget producing the smallest file that stays readable. Source:
  `DEC-004`.
- **Vault** — the folder on disk holding Finding and Bundle image files. Its location is a setting.
  One Vault per Library.
