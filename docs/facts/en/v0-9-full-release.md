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
  - "repo:docs/plans/active/release-0.9.0.md#sha256:979c5cc733a3cc7d2397fcae1ce689036558f9dd297751e3562c9b6498500d52"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:0a2fb65ae90b93fb111fd75acff42e917692b69e"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Default test is `0.9.0-test` on npm `test` and GitHub prerelease; `.N` is optional. Candidate
`30771098518` from `6761f0b` and maintainer recovery created `v0.9.0-test` with 22 assets. App-token
candidate `31042797141` from `dd0224a` and publication `31043631056` created `v0.9.0-test.1` with 22
assets. All six packages have `test=0.9.0-test.1`, `latest=0.8.0`. Stable `v0.9.0` and npm `0.9.0`
remain absent.
