---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Test and Stable Release"
summary: "v0.9.0 uses minimal trust from GitHub attestations, SHA-256, npm OIDC provenance, and same-byte publication."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:903c4fd819d0d09afdbc379ac874a22d592274b495aab6de82ee15166381bcbb"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:f8c457200b2d02aafd77e71981e82af120aa2b91e3a23e877c2011fed38eabef"
  - "repo:docs/guides/release-update.md#sha256:785e83d497c4f39ea683ac280adf8e071b27fda02b19c4c086573782a70bcadb"
  - "repo:docs/plans/active/release-0.9.0-stable-publication.md#sha256:f3f18d72aae7c914ebf43bfb5a9497a4af62a41dc1c690e35eeeb26db8d8d7e8"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:085ecd5d61f590106f651f929c33c21ac4b87d296f4a603f430f605dba6d1805"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:1b7ea99554fcf2e475cc77dcb1a3452a7805315f"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` is not published. Required trust is a protected-main exact tag, a same-candidate
GitHub Release, SHA-256 sidecars, GitHub artifact attestations, and npm Trusted Publishing OIDC
provenance. The single human gate is the GitHub stable environment. macOS ad-hoc and Windows
unsigned status are disclosed. External release-trust ceremonies and platform-certificate gates
are removal targets; transactional backup, rollback, and recovery remain. The replacement candidate
also requires the usage-guard default, failure-only CodexBar, and projection purge corrections.
