# Waiting on an Outside Party

**Loaded when:** an answer can only come from outside this session.

This file **does not hold a gate** unless its row is also pointed to from `blocking.md`.

## Open

| id | Question | Waiting on | Since |
|---|---|---|---|
| OQ-13 | Which host runs the Snapdown web service, and how is it reached over HTTPS? Blocks go-live of the publishing surface only — never a design gate | Repo owner | 2026-08-22 |
| OQ-14 | Which domain or subdomain serves published bundle URLs? Needed before the first real publish, not before the code | Repo owner | 2026-08-22 |
| OQ-15 | A code-signing certificate for the Windows installer, so the first run is not a SmartScreen warning. Go-live only; unsigned builds are fine for the owner's own machine | Repo owner | 2026-08-22 |

Every row here is a **go-live** prerequisite for the publishing surface. None of them blocks G1, G2,
G3, G4, or the desktop waves, and none MUST be treated as blocking without also appearing in
`blocking.md`.
