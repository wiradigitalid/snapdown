---
type: flow
component: finding
realizes: [UC-1, UC-2]
created: "2026-08-23"
updated: "2026-08-23"
---

# Flow — hotkey to stored Finding

The most-executed path in the product, and the only one with two timing requirements across it.

```mermaid
sequenceDiagram
    participant W as Windows
    participant H as LC-009 hotkey-registrar
    participant O as LC-001 capture-overlay
    participant N as LC-029 capture-note-field
    participant C as LC-002 region-capturer
    participant I as LC-003 image-reducer
    participant V as LC-005 vault-blobs
    participant S as LC-004 finding-store

    W->>H: hotkey pressed
    H->>O: capture requested
    Note over O: NFR-1 — visible within 200 ms,<br/>across three monitors
    O->>O: dim every monitor, crosshair on the active one
    O->>O: Reviewer drags; live W x H readout
    O->>N: region released with area
    N->>N: Reviewer types, presses Enter
    N->>C: region + note
    C->>O: dismiss
    Note over O: NFR-2 — dismissed and focus returned<br/>within 500 ms. Everything below is after this.
    C->>I: raw pixels + region
    I->>I: resolve the pair for THIS region (BR-104)
    I->>V: write the reduced image
    V-->>I: relative path
    I->>S: Finding + Note + resolved pair (NFR-18)
    S-->>O: toast with the running count
```

## The line that matters

`NFR-2`'s 500 ms budget ends at **dismiss**, not at **stored**. Everything below that line in the
diagram runs with no overlay on screen, and that ordering is the requirement rather than an
optimisation: reduction of a 4K capture is not free, and doing it before dismissal would put it
inside the budget.

Three consequences follow, and each is easy to get wrong:

- **A failure after dismissal has no surface to report into.** Hence the toast (`CF-1`, Failure).
- **The Reviewer is already typing in another window.** Nothing may steal focus back.
- **`UC-2` — five captures in a row — means five reductions may be in flight.** They must not
  serialise behind each other, and none may block the next hotkey press.

## Where `AD-4` sits

"An image is reduced exactly once, at capture, and no original is kept." The raw pixels exist only
between `C` and `I` in the diagram. Nothing writes them to disk, which is why there is no unreduced
capture to leak (`BR-25`) and no way to re-encode later (`BR-9`).

It also means the reduction is **irreversible and unrepeatable**. `SCN-03`'s upgrade case is a direct
consequence: tune the derivation and old Findings simply differ, because the input that produced them
is gone.

## As-built

`[MISSING]` — `I` resolves nothing. `crates/snapdown-core/src/domain/setting.rs` holds
`DEFAULT_MAX_LONG_EDGE_PX = 1600` and `DEFAULT_ENCODER_QUALITY = 75`, read as constants, so the
`resolve the pair for THIS region` step does not exist.

`[MISSING]` — the resolved pair is not passed to `S` and there are no columns to hold it
(`05-model/data-model.md`).

`[PARTIAL]` — `NFR-1` and `NFR-2` are stated with timed tests in the wave plan; whether those tests
run against three real monitors, or against a mock, was not established. Filed for `wdi-question`.
