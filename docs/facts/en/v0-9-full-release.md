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
  - "repo:docs/plans/active/release-0.9.0.md#sha256:0eb164fc7c9a028804b50c78f78cd8c673d6525817afffb6e1d202e531ff1445"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:1b7ea99554fcf2e475cc77dcb1a3452a7805315f"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` is absent; accepted `test.15` keeps `latest=0.8.0`. Protected-main candidate
`31482918509` passed five native archives, six npm packages, direct installers, attestations, and
the public authorization request. The release accepts macOS ad-hoc and Windows unsigned evidence;
paid platform certificates are not gates. External 2-of-3 Ed25519 authorization runs on an isolated
Mac, not the Windows acceptance host. Publication approval remains pending. Because that candidate
predates the mandatory knowledge-autocapture correction, it is historical qualification only
and cannot be published. A fresh Windows Codex write-and-recall acceptance and replacement
stable candidate are now additional `v0.9.0` gates.
