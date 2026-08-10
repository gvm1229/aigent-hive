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
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:6f3521bbf939c70b51f3ebcb31c3019e174b558f19b2658e0d9cfb563bed02e0"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:27ef86823b582e9435e08fe3adfd45203c5b0328751c5b3b130465662f48ebd0"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` remains absent. Candidate `31294665865` and OIDC publication `31295045199`
released `0.9.0-test.6` from `f356500`; all six npm packages have `test=0.9.0-test.6` and
`latest=0.8.0`, with an annotated GitHub prerelease. Hive canonicalizes the no-follow-validated
user root before Codex activation and setup validation. Stable qualification now also requires the
product-only 22-Skill catalog and one global/project-aware product usage guard. `REL9-011` remains
open pending clean-install and fresh-session acceptance on the maintainer's actual Windows 11
machine; macOS install or cross-compilation is not substitute evidence.
