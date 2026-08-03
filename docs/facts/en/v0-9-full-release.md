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
  - "repo:docs/plans/active/release-0.9.0.md#sha256:d5f13f0eb3579b6b60930279662ac42f15e602fed67c72462a1566d88ae44152"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:7346af3c9d9bdc6f84e7532f84654da3ef6c1b53"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Default test identity: `0.9.0-test`, npm `test`, and GitHub prerelease;
`0.9.0-test.N` only when needed. Candidate `30771098518` from `6761f0b` passed five
native targets and the npm umbrella. PR #16 registered the test workflow on `main`.
Run `30789141992` from `develop` awaits `release-publication` approval.
`deployment: false` preserves approval and secrets without a Deployment record. PR #17
extends that policy to stable. Before approval, npm, tag, and GitHub Release mutations are
zero. Test never changes `latest` or triggers stable publication. Test/stable parity:
report preview/export, `markdown|notion`, and optional Discord guard notification. Stable
follows test acceptance through protected `main` publication.
