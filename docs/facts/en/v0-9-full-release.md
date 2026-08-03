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
  - "repo:docs/plans/active/release-0.9.0.md#sha256:06ea57eb932d8de296f9a910aceffe217733c9c243a7acc67d1676b58c2430d6"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:a5b7ebfb6ad70159fe33c4f94902e649eff0c504"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Default test identity: `0.9.0-test`, npm `test`, GitHub prerelease; `0.9.0-test.N` only when
needed. Candidate `30771098518` from `6761f0b` passed five native targets and npm umbrella.
PR #16 registered the workflow on `main`. Run `30789141992` stopped before npm publication:
`dist/...` resolved as a Git remote; npm, tag, and GitHub Release mutations remain zero.
Commit `3782475` changes both paths to `./$archive`; regression and full pre-push pass.
Retry `30808850724` awaits `release-publication` approval. `deployment: false` retains approval
and secrets without a Deployment record. Test never changes `latest` or triggers stable
publication. Test/stable parity: report preview/export, `markdown|notion`, optional Discord guard.
