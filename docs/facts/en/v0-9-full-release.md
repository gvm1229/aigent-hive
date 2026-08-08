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
  - "repo:docs/plans/active/release-0.9.0.md#sha256:bf7fa4f0f5d2639358490df5e7978e9756cfe633e82ef84251ba4dc179101a05"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:27816088abbcfca7233e0e006f8b1e6cdec7aa55"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` remains absent. The latest public prerelease is `0.9.0-test.5`, with
`latest=0.8.0` unchanged. Candidate `31254605322` passed five native targets, npm packaging, and
attestation, but both publication attempts stopped before creating `0.9.0-test.6` artifacts.
Codex CLI `0.146.1` plugin activation is now a required `REL9-011` gate: current JSON
adapter/parser qualification, isolated marketplace→plugin→user-setup validation, rollback and
foreign-byte preservation, fresh-session discovery, and fixed numbered-prerelease acceptance.
One channel-bound `release-publish.yml` replaces separate token fallback workflows, and Copier and
Rust emit the same default Discord message-field configuration.
