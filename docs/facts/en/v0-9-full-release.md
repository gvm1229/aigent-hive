---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Test and Stable Release"
summary: "The 0.9.0 stable release is blocked until current Codex plugin activation and setup pass a fixed numbered prerelease."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:ee293a5b839fb7af3b7f4ebefc9be662f9ab595242e37cf31e6b143c6c69cb20"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:378c59f5e55241f4d50037965f8a0fe865255f15dd0ce814462048d6a2c3d770"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:f356500621c21702abb8c21746cf138078a9d9fc"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` remains absent. Candidate `31294665865` and OIDC publication `31295045199`
released `0.9.0-test.6` from `f356500`; all six npm packages have `test=0.9.0-test.6` and
`latest=0.8.0`, with an annotated GitHub prerelease. Hive canonicalizes the no-follow-validated
user root before Codex activation and setup validation. `REL9-011` remains open pending
Windows clean-install and fresh-session acceptance of this fixed prerelease.
