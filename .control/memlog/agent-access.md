---
topic: Snapdown — agent-access component depth
artifact: .how/agent-access/SDD-agent-access.md
updated: 2026-08-31T16:54
---

- (change) G4 design 2026-08-31, scoped correction pass on SDD-agent-access.md for DEC-012. The AD-9 row in Inherited Constraints quoted the retired Rule verbatim, so it moved with the spine. Its 'How it lands here' cell also claimed 'The golden-file test in bundle covers this path' - marked [MISSING] and corrected: LC-017 has no implementation (BUG-59, 'The Local API does not exist, so the MCP Bridge cannot reach the product at all'), so there is no path for a test to cover, and crates/snapdown-store/tests/test_golden_markdown.rs pins the composer against a stored reference rather than any surface. Whoever fixes BUG-59 inherits that guard as unwritten, not as inherited. Added DEC-012's guidance that this reader gets the stored folder-relative links with NO rebasing, because an agent on the same machine can be told the Vault path once instead of inside every link. DEC-005 freezes this component and explicitly permits this: 'This decision does not forbid a fix. It forbids new work.'
