---
schema_version: 1
pair_id: release-compatibility-qualification
topic_slug: release-compatibility-qualification
language: en
counterpart: ../ko/release-compatibility-qualification.md
title: "Release Compatibility Qualification"
summary: "The published 0.9.5 release keeps compatibility qualification reproducible through a full-history native runtime workflow."
tags: [compatibility, migration, release, testing]
aliases: ["Compatibility matrix gate"]
sources:
  - "repo:.github/workflows/release-runtime.yml#sha256:e02b4cfeaf85ed248bd09113bb208e6e3c72a083cd510309c5d3f718c90d3fa8"
  - "repo:docs/plans/active/release-0.9.5-stable-publication.md#sha256:7681da81e7ae900184cbaaaffd51763f547e417d959b329f1e1e42f167867475"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:a49b4c9520a9099f41da1a70ea543eaf445e1053"
status: active
---

# Release Compatibility Qualification

`0.9.5` is published as npm `latest` and GitHub Release `v0.9.5`. macOS arm64 public installation
passed isolated user setup, Codex install, validation, and stable update check. Native runtime uses
`fetch-depth: 0` because release-version parity reads the prior patch tag. Windows x64 public stable
installation and the repaired five-target native runtime run remain separate closure evidence.
