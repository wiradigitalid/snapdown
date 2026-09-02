# 10: The composer reads its own document back into blocks

**What to build:** Today the composer turns blocks — a Bundle's title, its notes, and for each
Finding its image, note and Marker notes — into the one Markdown document a Bundle stores. Give it the
inverse: read a document it wrote back into exactly those blocks, so that reading then writing
reproduces the document **byte for byte**. Nothing appears on screen. This is the foundation the
owner approved for Review & Update (ticket 13/14) and Copy Markdown (ticket 12): a sealed Bundle has
no Findings left to rebuild from, and `BR-11` forbids a change to a Bundle reading a Finding at all,
so the stored document must be able to stand on its own. Include the two operations those tickets
need on top of the parse: **rebasing** every image link to an absolute path in the form ticket 03
settled (forward slashes, wrapped in `<>`), changing nothing else in the document; and **no-op
detection** — telling whether a set of edited blocks would serialise to a document identical to the
stored one. Pure core code, no I/O.

Prefactor, per `/to-tickets`: make the change easy, then make the easy change.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Parsing the existing golden Bundle document and serialising the result reproduces it byte for
      byte; the golden test is the first fixture and is not weakened
- [ ] Round-trip holds over generated documents with every field populated, every optional field
      empty (no Bundle notes, a Finding with no Markers, a Marker with an empty note), and text
      containing Markdown metacharacters (`#`, `*`, `_`, `<`, `>`, backslash, a line that looks like
      a heading)
- [ ] A document whose shape the composer never produces is rejected with an error that says what was
      unexpected, rather than parsed into something plausible
- [ ] Rebasing produces a document that differs from the stored one **only** in image link
      destinations — asserted by diffing the two, not by inspecting code — and those destinations are
      absolute, forward-slashed and `<>`-wrapped for a Vault path containing a space, one containing
      parentheses, and one containing an apostrophe (`AD-9` as narrowed by `DEC-012`)
- [ ] No-op detection returns "unchanged" for blocks parsed from a document and re-submitted untouched,
      and "changed" for a single-character edit in any of the four editable fields
- [ ] Each guard was seen red first: break the serialiser's round-trip in one place, watch the test
      fail, restore it
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] Nothing to look at in the running app; say so plainly in the hand-off
