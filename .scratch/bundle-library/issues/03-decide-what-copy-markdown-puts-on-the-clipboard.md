# 03: Decide what Copy Markdown puts on the clipboard

**Type:** grilling
**Status:** open
**Blocked by:** None (can start immediately)

## Question

`Copy Markdown` is the Library's primary hand-off action — the way a Reviewer gets a Bundle into a
coding agent or a chat. Decide exactly what lands on the clipboard.

- **The text itself.** The stored `Bundle.markdown` verbatim, or something rewritten on the way out?
- **The image problem.** Images cannot ride a text clipboard. The pasted Markdown will carry image
  links that only resolve for a reader with filesystem access to this machine's Vault. Decide
  whether the links are left as stored, rewritten to absolute paths so a local agent can open them,
  or something else — and what the Reviewer is told, if anything, about images not coming along.
- **Whether the Reviewer is warned at all**, or whether a plain "Copied" toast is enough.
- **Relationship to `Open file location`.** These two are siblings: one hands over text, the other
  hands over the file. Decide whether the answer here makes one of them redundant, or whether they
  serve genuinely different moments.

## Owner's answer, 2026-08-31 — narrows this ticket

**Copy Markdown puts the whole document on the clipboard with image links rewritten to absolute
paths.** The images stay where they are in the Vault; the absolute links let a local agent open them
from wherever it is working.

Tested and it holds, with one correction and one caveat:

- **It does not conflict with `NFR-8`.** That requirement governs the *stored* Markdown file, which
  keeps its folder-relative links. The clipboard is a different artifact with a different reader, so
  a different path convention is legitimate. Implement it as one composer with a base-path
  parameter, not a second copy of the serializer.
- **Windows absolute paths are not valid in Markdown as-is.** Backslashes are escape characters and
  spaces terminate a link, and the Vault may well sit in a path containing spaces:

  ```
  ![Finding 1](C:\Users\<user>\Snapdown Vault\bundles\b1\finding_1_burned.png)   broken twice over
  ![Finding 1](<C:/Users/<user>/Snapdown Vault/bundles/b1/finding_1_burned.png>) works
  ```

  Forward slashes plus `<>` wrapping, or percent-encoding, or a `file:///` URI. Whichever is chosen
  must be proven against a Vault path containing a space.
- **Caveat worth a decision:** an absolute path exposes the operator's OS username. Pasted into a
  shared or public chat, that goes with it.

**An environment variable was considered and rejected for this.** The proposal was to have Snapdown
export `SNAPDOWN_VAULT_PATH` globally and have the copied Markdown reference it instead of a real
path, keeping the operator's username out of pasted text. It fails on its first requirement: **no
CommonMark renderer expands variables**, so `$SNAPDOWN_VAULT_PATH/bundles/...` renders as literal
text and the link is simply broken. It also inherits three problems — already-running processes never
see a newly set variable, the Vault can move (`vault_migration.rs`) leaving the variable silently
stale, and writing to the user's environment is a system-level side effect that outlives the app.

Decisive point: for the clipboard, images **never** render for a human reader anyway — no chat client
loads local files. The only consumer of these links is a local agent, and an agent can open an
absolute path directly, with no expansion step. The variable adds a layer only an agent could
resolve, for a problem only an agent has, which the absolute path already solves. The idea is worth
taking up separately as a way for an agent to *find* the Vault — that is `agent-access` territory,
recorded in the map's Out of scope.

**What is still open:** which of the three encodings above to emit; whether a plain "Copied" toast
suffices or the Reviewer is told images are not on the clipboard; and whether this makes
`Open file location` redundant or leaves it serving a genuinely different moment.

**The prerequisite has landed.** The stored Markdown's image links were resolving one folder too
deep and loading nothing; fixed 2026-08-31 as `BUG-86`, so the stored document is now genuinely
portable and links read `./finding_N_burned.png`. Decide this ticket against that behaviour.

**Implementation note, not a decision:** the app already links `clipboard-win`, but every existing
use is `raw::set_bitmap_with` for images (`main.rs:2917-2922`). Copying text would be the first text
clipboard call in the codebase — small, and Windows-only like the rest, with a `#[cfg(not(windows))]`
stub alongside.

## Added 2026-08-31, from ticket 08's session — `AD-9` was never checked here, and it bites

The owner's answer above checks the rewrite against `NFR-8` and clears it. It does not check it
against `AD-9`, which is the requirement that actually governs a clipboard:

> *"A Bundle's Markdown MUST be composed once, by the core, and stored. **Every handoff path MUST
> serve those exact bytes.** No surface may re-render, re-order, decorate, or summarise a Bundle on
> the way out; a surface that needs a different shape is asking for a change to the composer."*
> — `AD-9`, quoted at `.how/agent-access/SDD-agent-access.md:81`

`Copy Markdown` **is** a handoff path, and rewriting every image link to an absolute path is a
surface producing different bytes on the way out. `NFR-8` is about the shape of the links; `AD-9` is
about the bytes being identical on every path. Clearing one does not clear the other.

Two things soften it but neither dissolves it:

- The answer already prescribes *"one composer with a base-path parameter, not a second copy of the
  serializer"* — which is `AD-9`'s own stated remedy, *"a surface that needs a different shape is
  asking for a change to the composer."* The rewrite goes through the composer, not around it.
- `SDD-agent-access.md:81` records that the MCP path returns `bundle.markdown` **verbatim** and that
  *"the golden-file test in `bundle` covers this path"*. That test exists —
  `crates/snapdown-store/tests/test_golden_markdown.rs:137` asserts the serializer's output matches a
  golden reference **byte-for-byte**, citing `AD-9` and `INV-EXPORT-001`. A base-path parameter means
  a second golden, and the invariant stops being "one set of bytes" and becomes "one composer, two
  renderings".

So the open list at the end of this ticket gains a fourth item, and it is the one that needs settling
first because it decides whether the other three are even reachable: **does emitting different bytes
to the clipboard than to disk contradict `AD-9`, or does going through the composer satisfy it?** If
it contradicts, a `DEC-` is mandatory — `AGENTS.md` makes contradicting an `AD-N` the one case where
recording is not optional. Settle it against `AD-9`'s own text, not against a summary of it.

Note this is the **same** question `BR-11` raises for the Review & Update window, in the opposite
direction: there the composer re-runs and re-stores, here it renders a second form on the way out.
See ticket 08's second-session section — worth answering both in one sitting so the two answers cannot
drift.
