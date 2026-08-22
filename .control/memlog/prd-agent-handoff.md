---
topic: Agent Handoff — MCP and web publishing
artifact: .what/_prd/agent-handoff/prd.md
updated: 2026-08-22T23:03
---

- (event) headless run, intent create. Second initiative: everything after a Bundle exists
- (decision) CAP-7..CAP-8, FR-19..FR-26, NFR-9..NFR-15, UJ-5..UJ-6 continue the global sequence
- (decision) two executables — a loopback Local API in the desktop app and a stdio MCP Bridge — because that is the only shape satisfying all four access criteria
- (decision) the Bridge holds no key between runs; the paste ceremony is what makes revoke meaningful
- (decision) exactly one Access Key valid at a time; issuing a new one revokes the old
- (decision) only Bundles are exposed. Unbundled Findings stay invisible to every agent path
- (decision) publishing is per named Bundle, confirmed, never automatic. Unlisted slug is the access control in r2; a read token is designed for and not promised
- (decision) raw Markdown is the primary representation of a published Bundle; the browser rendering is the courtesy
- (decision) a failed unpublish MUST keep showing the Bundle as published — telling the Reviewer something is private when it is not is the worst outcome here
- (assumption) the Reviewer prefers a per-session key over standing agent access — OQ-6
- (assumption) unlisted slug plus optional read token is access control the owner accepts — OQ-8
- (assumption) a remote agent can fetch an HTTPS URL and the images it references — OQ-7
- (gap) no host and no domain for the web service — OQ-13 and OQ-14, go-live only, no design gate held
- (change) stack answered mid-run by the owner: web service in Go with net/http plus chi, web UI React on Vite. Next.js and Express are excluded by the owner. Lands as AD-N at G3
- (event) PRD and addendum written; review lenses structure+prose applied at write time
- (decision) owner confirmed mid-run: publishing is selective per Bundle and never automatic. A single screenshot is published as a one-Finding Bundle; publishing a Finding directly stays out, because BR-14 keeps unbundled Findings invisible on every agent surface. Filed as OQ-16
