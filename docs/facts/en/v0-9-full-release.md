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
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:4a57fee408818d98f6b0bba20f8487743e9965d816f45b50a4d0d0d1a3915ea0"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:4c8f3d48f1157625f9e766b5525c4feb28b8f95eff5cabdee576082e1ff2bd15"
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
