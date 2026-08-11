---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Test and Stable Release"
summary: "The earlier candidate is historical only; v0.9.0 publication now requires the knowledge-autocapture correction and a replacement stable candidate."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:505cc48a16b2ccc7ca7fe39fdaf47d7b851a19810cb75c784fdfe5a6717c5823"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:d2b7908ed7cdbcba844c47f406d91eb74b5d9bceb5aae562908e14b101800927"
  - "repo:docs/guides/signed-update-and-release.md#sha256:41b38d004edd0a2305919b183b706d65705c3f0b8b3998ac63308f529ae7a549"
  - "repo:docs/plans/active/release-0.9.0-stable-publication.md#sha256:55725c877b0bbcd94e3197f86c38df08c99e3161faf72ec6c21f86e182e74cf4"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:c40863c90f3b8947dfe52bfe43ef1f52ae5f1ed72150f6fcc2921e10bcfaa39f"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:1b7ea99554fcf2e475cc77dcb1a3452a7805315f"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` is absent; `test.16` keeps `latest=0.8.0`. Protected-main candidate `31482918509`
is historical only because it predates mandatory knowledge autocapture. A fresh Windows Codex
write-and-recall acceptance, replacement stable candidate, isolated Mac 2-of-3 Ed25519 authorization,
and publication approval remain gates. `test.16` embeds the historical `2026-08-01` date; immutable
published bytes stay unchanged, and the next normal release must use and verify the actual UTC date;
that correction alone does not warrant a separate test release.
