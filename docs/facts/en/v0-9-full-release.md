---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Test and Stable Release"
summary: "The 0.9.0 stable release remains blocked: test.12 proved preserving uninstall and Windows clean reinstall; new Codex-session discovery and an outbound Discord test with its environment value remain."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:6f3521bbf939c70b51f3ebcb31c3019e174b558f19b2658e0d9cfb563bed02e0"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:52abb3afc4d52549b1d3b701ad91bed84a28d95f0331f5768aa76a2dfab4572a"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` remains absent. Candidate `31391084832` and OIDC publication `31392103115`
released `0.9.0-test.12` from `c98add0`; all six npm packages have `test=0.9.0-test.12` and
`latest=0.8.0`, as a GitHub prerelease. Actual Windows user-root preserving uninstall,
clean reinstall, `dry-run → apply → validate`, and install validation passed. Saved preferences and
knowledge digests remained intact; 22 active product Skills, no retired name, 20% usage guard,
persisted Discord configuration, and no home temporary answer. Webhook environment value unavailable.

`REL9-011` remains open for automatic CLI discovery in a newly opened Codex session on Windows 11
and an outbound Discord test when its configured environment value is available.
