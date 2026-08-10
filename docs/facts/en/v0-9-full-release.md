---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Test and Stable Release"
summary: "test.15 is the current published acceptance build; Windows preserving-reinstall acceptance and the full develop CI pass, while production signing and external TUF authorization remain required."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:5c00353f7683e586ada9ccfec9e80dd7504d2f464d88309ea8d9786f916219d5"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:00aacb0a11b5595075096985ce3872bda492799b24ecbc726025e3b558a75080"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:a39b88112f5582a836e0c5848668407190d4a616"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` is absent. Candidate `31407585364` and OIDC publication `31409030152` released
`0.9.0-test.15` from `6f809a27`; all six packages retain `test=0.9.0-test.15` and `latest=0.8.0`.
Windows preserving reinstall, setup validation, new-session discovery, Discord delivery, and all
19 jobs of develop CI `31410354787` passed. Stable publication remains blocked on macOS/Windows
signing and external TUF authorization with rollback-floor verification, absent from both release workflows.
