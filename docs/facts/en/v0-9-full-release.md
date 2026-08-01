---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Test and Stable Release"
summary: "The 0.9.0 release uses an independent bare test channel before a separately authorized stable publication."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:f7f0428ef5d7e194a08d97d64ec13b02f50edf0b6598c6bcf19396942a9d1782"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:fc1e23854bf6cbc09a2dc7704d8185ae247212a0"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Default test identity: package `0.9.0-test`, npm dist-tag `test`, and a GitHub
prerelease. Additional test versions use `0.9.0-test.N` only when needed. A test
publication never changes `latest` or triggers stable publication. Test and stable
artifacts have identical features, defaults, and diagnostics, including an explicit
consumer developer-report preview and export with no automatic upload. This parity
includes the `markdown|notion` backend and optional Discord guard notification. Stable
`0.9.0|latest` follows test acceptance as a separate protected-`main` publication.
Apple and Windows signing, external TUF authorization, protected approvals, public
install, and upgrade evidence remain mandatory for stable publication.
