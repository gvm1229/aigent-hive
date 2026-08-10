---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Test and Stable Release"
summary: "test.12 completed Windows acceptance; stable 0.9.0 still requires automated default-profile installation acceptance and measured test-suite rationalization."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:9a4853952266b6a234ecf88bda90eebbf16148b8e529aa7739c9321c45866b91"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:629c28cdace1188fa43c8129dc38d293c4f9806f752a529171b67c492fd96d2e"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:3e960b5185f637d7606eb01126d2543519138608"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` remains absent. Candidate `31391084832` and OIDC publication `31392103115`
released `0.9.0-test.12` from `c98add0`; all six npm packages have `test=0.9.0-test.12` and
`latest=0.8.0`. Windows preserving uninstall, clean reinstall, `dry-run → apply → validate`,
install validation, new-session `hive` discovery, and Discord delivery are accepted.

Pre-main internal gates: automated clean install and preserving reinstall using the product-owned
expedited default profile without contributor preferences or setup dialogue; measured test-suite
inventory and consolidation with replacement coverage. Disposable consumer fixtures belong under
the ignored `tests/work/` boundary.
