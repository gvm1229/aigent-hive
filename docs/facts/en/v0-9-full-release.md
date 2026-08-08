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
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:9b55b1293372b76fd080e98f49d1307c26c5bdbc9c39100364f59ce2719d50a5"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:a7a53138de9f3be5e4627e8ac7781cb1ed7bbd968712d6d1bc040502186aca9d"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:a78aed2efcf96d34ef020addc30ebdd70f035286"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Test prereleases use npm `test` and GitHub prereleases; stable `v0.9.0` remains absent. The latest
public prerelease is `0.9.0-test.5`: all six packages have `test=0.9.0-test.5`, while
`latest=0.8.0` remains unchanged. Candidate `31254605322` built `0.9.0-test.6` from commit
`9e08a48` for all five native targets, the npm umbrella, and
GitHub attestations. Its two publication attempts, `31255061771` through Trusted Publishing and
`31255167232` through the bootstrap fallback, both stopped at the first scoped package with npm
`404`; no `0.9.0-test.6` npm version, tag, or GitHub prerelease was created.
