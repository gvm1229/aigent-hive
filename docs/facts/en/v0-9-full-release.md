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
  - "repo:docs/plans/active/release-0.9.0.md#sha256:79ade57874d13dbf8657de917f1d5b571ccfb457c2cbcd5dc81425425a8973dc"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:a78aed2efcf96d34ef020addc30ebdd70f035286"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Test prereleases use npm `test` and GitHub prereleases; stable `v0.9.0` remains absent. Candidate
`31090062784` and publication `31090917408` from `5341bdf` created `v0.9.0-test.3` with 22 assets.
All six packages have `test=0.9.0-test.3`, `latest=0.8.0`; the annotated prerelease tag is
`v0.9.0-test.3`. Its CLI label is `AIgent Hive v0.9.0-test #3 · developer test build (released
2026-08-06)`.
