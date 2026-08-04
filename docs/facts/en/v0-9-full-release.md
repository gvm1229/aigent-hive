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
  - "repo:docs/plans/active/release-0.9.0.md#sha256:48d88db3020fbad2f4e5fd3aa76ed0e3b663be58f1dafd66229f446e754831f0"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:0a2fb65ae90b93fb111fd75acff42e917692b69e"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Default test is `0.9.0-test` on npm `test` and GitHub prerelease; `.N` is optional. Candidate
`30771098518` from `6761f0b` passed five native targets and npm umbrella. Reviewer-free bootstrap
run `30890841117` published all six test packages and verified `test=0.9.0-test`, `latest=0.8.0`.
Its final tag/Release step lacked workflow-tag permission. Authenticated maintainer recovery created
annotated `v0.9.0-test` at that candidate and the prerelease with 22 assets. Stable `v0.9.0`, npm
`0.9.0`, and `latest` remain unchanged. Future fully automatic finalization needs separately
authorized repo-scoped contents/workflows-write GitHub credential; no current credential is copied.
