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

- **Access Key** — the secret string a Reviewer copies out of Snapdown and pastes into an agent
  conversation to grant that agent read access to the Library over the Local API. Exactly one Access
  Key is valid at a time; issuing a new one revokes the previous one. It is not a password and there
  is no account behind it.
- **Bundle** — a named group of Findings, composed once into one Markdown document with its own copy
  of the images it references. A Bundle has zero or one Publication. A Finding may belong to zero,
  one, or several Bundles. A Bundle is recomposed rather than edited.
- **BundleItem** — the membership of one Finding in one Bundle, holding the Finding's position in
  that Bundle and the copy of the image that was written for it. Exactly one BundleItem per Finding
  per Bundle; deleting the Bundle deletes all of them.
- **Capture** — the act of selecting a screen region and storing the resulting image. One Capture
  produces exactly one Finding.
- **Capture Overlay** — the full-screen, semi-transparent surface, one per monitor, on which the
  Reviewer drags out the region to capture. It exists only between the hotkey press and the moment
  the Finding is saved or the Capture is cancelled.
- **Editor** — the Snapdown Editor: the desktop window that lists the Library, shows each Finding
  with its Note and Markers, and is where Bundles are composed. There is exactly one Editor window.
- **Finding** — one observation: a captured image, the Note that describes it, and the Markers drawn
  on it. The atomic unit of the product; nothing smaller is handed to an agent. A Finding has exactly
  one image file in the Vault.
- **Handoff** — the act of giving a Bundle to an agent. Three shapes carry the same content: copying
  the Markdown, reading it over MCP with the Access Key, or fetching a Publication over HTTPS.
- **Library** — the whole set of Findings and Bundles held on this machine, together with their
  metadata. One Library per installation.
- **Local API** — the loopback-only HTTP interface over the Library, reachable at `127.0.0.1` and
  refusing every request that does not carry the current Access Key. The MCP Bridge is its only
  intended client.
- **Marker** — a numbered badge placed by the Reviewer on a Finding's image. Marker `n` is bound to
  line `n` of that Finding's Note; the two share one sequence and are never kept in sync as two
  things. A Finding has zero or more Markers, numbered from 1 with no gaps.
- **MCP Bridge** — the separate executable that speaks the Model Context Protocol to an agent and the
  Local API to Snapdown. It holds no data of its own.
- **Note** — the Reviewer's prose about one Finding: a free-text body, plus one numbered line per
  Marker. Exactly one Note per Finding; it may be empty.
- **Publication** — the record of a Bundle having been published: the unlisted URL it was served at,
  when it happened, and whether it is still live. Unpublishing ends a Publication without deleting
  the Bundle.
- **Quality Budget** — the Reviewer's setting pair that governs image reduction: a maximum long edge
  in pixels, and an encoder quality. Applied to every Capture on the way into the Vault, never
  afterwards.
- **Reviewer** — the person operating Snapdown: the one who captures, writes Notes, composes Bundles,
  and decides what is handed off. The only human actor.
- **Setting** — one persisted preference of the installation: the Vault location, a hotkey binding,
  the Quality Budget pair, whether Snapdown starts with Windows, whether the Editor opens after a
  Capture, and where the web service lives. One value per key, one set per Library.
- **Vault** — the folder on disk holding Finding and Bundle image files. Its location is a setting.
  One Vault per Library.
