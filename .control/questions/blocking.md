# Blocking Questions

**Loaded when:** every gate.

Only this file holds a gate. Written through `wdi-question`, never by hand directly; the row shape
lives in `.constitution/method/document/templates/questions.md`.

Target **<=3 rows per Product Component**. A question rises to here only through three tests, one is
enough:

1. It touches money, personal data, or a legal obligation.
2. It changes the wording of an `FR`'s promise.
3. Answering it wrong forces a rewrite of more than one component.

Failing all three → `assumptions.md`. A question MUST NOT be filed here "to be safe".

## Open

| id | Question | Blocks | Owner | Before |
|---|---|---|---|---|
| — | — | — | — | — |

**Empty as of 2026-08-22, after G1, G2, and G3.** Fifteen questions are open across
`assumptions.md` and `external.md` and not one of them passes the three tests above: none touches
money or a legal obligation; the personal data in a Capture is governed by decided constraints
rather than by an open question; none would change the wording of an `FR`'s promise; and the most
expensive one to get wrong — OQ-5, whether Windows hotkeys register without administrator rights —
would force a rewrite of `finding` alone. The three go-live prerequisites in `external.md` block the
publishing surface shipping, and by that file's own rule they hold no design gate.
