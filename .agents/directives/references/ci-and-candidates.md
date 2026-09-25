# Ci And Candidates

## Risk-Tier CI and Candidate Economy

- Match CI to behavior as well as paths. Shipped Skill or directive changes affecting routing,
  consent, execution, or closure require affected behavioral and packaging checks even when all
  files are Markdown. Prose-only work uses documentation integration and starts no
  Rust, cross-platform, native package, runtime qualification, or candidate. Product work runs its
  affected lane, Linux conformance, and the smallest relevant macOS/Windows smoke; full platforms
  remain nightly or candidate evidence.
- CI for a superseded PR or branch commit must cancel when a newer commit for that same PR or ref
  starts. Never apply cancellation to release publication, accepted public tests, or protected
  stable candidate workflows.
- Cache only pinned dependencies per OS; never replace a test, lockfile contract, or artifact.
- At a completed user-authorized product milestone, automatically create, publish, and accept one
  next numbered test without another approval; intermediate commits never publish. Stable remains
  version-specific and explicit.
- Before a test candidate, `check-test-release-gate.py` must prove new shipped product bytes against
  `docs/public-test-product.json` and one or more checked non-release implementation plan IDs.
  Missing, stale, identical, source-only, or unplanned evidence refuses the candidate; fix the
  scope or evidence instead of asking the user to approve a release bypass.
- Plans, state, facts, docs, source-only Skills/directives, tests, CI, notices, and identical product
  trees never create or reset a test. Batch product work until milestone verification finishes.
- One always-run protected merge gate verifies risk-matched jobs; never require a conditionally
  skipped product job directly.
