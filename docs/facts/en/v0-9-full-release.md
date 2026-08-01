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
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:d6bcfc0ec1e77c1f76cb8e24e7686457311099026f8af7cd191ad568d351e1ec"
links: [release-verification, test-distribution, version-policy]
reviewed_revision: "git:5e09d0ff23e841381c22bac24e707dbc6402dae4"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Default test identity: package `0.9.0-test`, npm dist-tag `test`, and a GitHub
prerelease. Additional test versions use `0.9.0-test.N` only when needed. A test
publication never changes `latest` or triggers stable publication. Test and stable
artifacts have identical features, defaults, and diagnostics, including an explicit
consumer developer-report preview and export with no automatic upload. Stable
`0.9.0|latest` follows test acceptance as a separate protected-`main` publication.
Apple and Windows signing, external TUF authorization, protected approvals, public
install, and upgrade evidence remain mandatory for stable publication.
