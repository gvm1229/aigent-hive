---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Test and Stable Release"
summary: "The 0.9.0 test prerelease uses a protected independent channel before a separately authorized stable publication."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:f534f4713c0a95b9a5e7ad63eed1470cd4cfd720adb37ecaea85f0a5dfad5009"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:eb57729c43e676c42fcb133b60b0efc5d17f4400805758447728fad2b4de8027"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:a78aed2efcf96d34ef020addc30ebdd70f035286"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Test prereleases use npm `test` and GitHub prereleases; stable `v0.9.0` remains absent. The latest
public prerelease is `0.9.0-test.5`, while `latest=0.8.0` remains unchanged. Candidate
`31254605322` built `0.9.0-test.6` from `9e08a48` for all native targets, the npm umbrella, and
attestations, but both publication attempts stopped at the first scoped package with npm `404`.
One channel-bound `release-publish.yml` now replaces the separate token fallback workflows.
