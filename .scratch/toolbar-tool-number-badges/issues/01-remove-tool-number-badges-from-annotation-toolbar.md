# 01: Remove the number badges from the annotation toolbar buttons

**What to build:** The Marker, Shape, Callout, Blur, Arrow, and Text buttons in the Editor's
annotation toolbar currently each show a small numeric badge ("1" through "6") in the corner. These
badges imply a keyboard shortcut that does not exist anywhere in the app — no key handler switches
the active tool on digit press, only mouse clicks do. Remove the badges so the toolbar stops
promising a feature that isn't there. The Crop button, which already has no badge, is the reference
for how the other six should look and space out once theirs are gone.

If the badge-rendering path in the shared icon-button component ends up with no remaining caller
once these six are cleared, remove that dead code path too rather than leaving it unused.

**Blocked by:** None (can start immediately)

**Status:** done

**The premise was verified in the code before any edit**, because a stale ticket premise is this
repository's signature failure. It held: the six `ordinal-hint` values sat at `appwindow.slint:1800`,
`:1810`, `:1820`, `:1830`, `:1840`, `:1850`, Crop had none, and no digit key reached the tool
selector. `appwindow.slint` has exactly three `FocusScope`s - the window's at `:358`, the canvas's at
`:1975`, the preview's at `:3370` - and between them they handle `Escape`, `Return` and `Delete` and
nothing else. The badges promised a shortcut that was never written.

- [x] None of the seven annotation toolbar buttons render a numeric badge. The six `ordinal-hint`
      lines are gone from `appwindow.slint`, so all seven buttons now match Crop.
- [x] Button spacing and alignment are unaffected in both themes, **by construction rather than by
      inspection**. The badge was an absolutely-positioned `Text` inside each button's own
      `Rectangle` (`x: parent.width - self.width - 4px; y: 2px`), outside every layout; each button
      sets `width: 48px; height: 44px` explicitly at its call site, and the strip is a
      `HorizontalLayout { spacing: 2px; alignment: center; }`. Removing an absolutely-positioned
      child of a fixed-size box cannot move that box or its siblings, and nothing here is a colour
      question, so the two themes cannot differ.

      Worth noting in passing: the badge drew in `Theme.text-dim`, which is one of `BUG-54`'s six
      sub-AA pairings. This deletes one place that unreadable token appeared. It does **not** fix
      `BUG-54` - the token still fails everywhere else it is used.
- [x] No test asserted the badges, so nothing needed updating: `ordinal-hint` appeared in exactly
      nine places in the source tree, the six call sites and three lines of the component, and
      nowhere in `apps/desktop/tests/`. No keyboard shortcut behaviour was added.
- [x] `cargo fmt --all -- --check` exit 0 - `cargo clippy --workspace --all-targets -- -D warnings`
      exit 0 - `cargo test --workspace --no-fail-fast` exit 0, 41 `test result: ok` lines and no
      failures. Exit codes were captured into variables rather than read from a pipe or a trailing
      `echo`, per `AGENTS.md`.

## The dead path, and why no test guards it

The ticket's second paragraph applied: with the six call sites cleared, `IconButton`'s badge path had
no remaining caller, so both halves of it went - the `in-out property <string> ordinal-hint` and the
`if root.ordinal-hint != "" : Text { ... }` block. 18 lines deleted across the two files, nothing
added.

**A ratchet test was considered and deliberately not written.** Removing the property is already a
*compile-time* guard and a stronger one: a caller that sets `ordinal-hint` on an `IconButton` now
fails to build. It is also the honest guard. The reason these badges are wrong is that no digit
shortcut exists - if one is ever implemented, the badges become correct again, and a test forbidding
them forever would encode a rule that may legitimately reverse. The compiler guard disappears in the
same edit that would restore the property, so it cannot go stale the way a test asserting an absence
would.

## Nothing is owed to the corpus

Checked rather than assumed. `.what/`, `.how/`, `.control/` and `.constitution/` say "numbered badge"
in eight places and **every one of them is about a Marker** - the badge burned onto a captured image,
which is a different thing that this ticket does not touch. No document anywhere described the
toolbar's tool-number badges, so there is no document to bring into line.
