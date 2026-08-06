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
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:9172a8fa815052211dac6f561775f47852f4fe86bd629cb02004bbf5e0e30acb"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:eb18df03b5a407f3fb3a405a9af0dd146ff653d92dbc5ba6528f08198efedc7c"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:6980e8b38c08a9ebe483a4ffa7937f70999d63a5"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Test prereleases use npm `test` and GitHub prereleases; stable `v0.9.0` remains absent. Candidate
`31082481203` from `6980e8b` and publication `31083602464` created `v0.9.0-test.2` with 22 assets.
All six packages have `test=0.9.0-test.2`, `latest=0.8.0`; an isolated install prints the exact
developer-test build label and `2026-08-06`. Trusted publishing `31083140684` returned two scoped-package
`404`s without mutation, then the existing registry-auth fallback published the release.
