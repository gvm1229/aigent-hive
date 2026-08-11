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
  - "repo:.github/workflows/release-publish.yml#sha256:505cc48a16b2ccc7ca7fe39fdaf47d7b851a19810cb75c784fdfe5a6717c5823"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:f8c457200b2d02aafd77e71981e82af120aa2b91e3a23e877c2011fed38eabef"
  - "repo:docs/guides/signed-update-and-release.md#sha256:41b38d004edd0a2305919b183b706d65705c3f0b8b3998ac63308f529ae7a549"
  - "repo:docs/plans/active/release-0.9.0-stable-publication.md#sha256:3da4edef672d721a8fc0b9a83100f5cc6076f35111c50551348e5ef357016cb7"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:c40863c90f3b8947dfe52bfe43ef1f52ae5f1ed72150f6fcc2921e10bcfaa39f"
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
