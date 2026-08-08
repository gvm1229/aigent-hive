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
  - "repo:docs/plans/active/release-0.9.0.md#sha256:04fd1386880e69f61277c8618eb810901cfbd05ff95d24fab6493172feb25a54"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:50a3420f5b8c8aadae4dde3d74cf44c76f19bf88"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` remains absent; `0.9.0-test.5` remains the latest public prerelease and
`latest=0.8.0` is unchanged. Candidate `31254605322` passed five native targets, npm packaging,
and attestation, but publication stopped before `0.9.0-test.6` artifacts. Codex CLI `0.147.0`
reproduced physical-path reporting in an isolated macOS user root. Hive now canonicalizes the
no-follow-validated root before activation and setup validation. The local install/setup flow and
rollback and foreign-byte regressions pass. `REL9-011` remains open pending a fixed numbered
prerelease's Windows clean-install and fresh-session acceptance.
