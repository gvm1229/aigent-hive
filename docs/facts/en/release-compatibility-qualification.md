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
  - "repo:.github/workflows/release-runtime.yml#sha256:06e8657e24d89fd2b28d87208fc15eb76d4b60357c1dd9d3c9c7c315b563d350"
  - "repo:docs/plans/active/release-0.9.5-stable-publication.md#sha256:e00beda4bab8467a5fa667fdc1f2799403d216398e29597f67f937bf94d46e95"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:a49b4c9520a9099f41da1a70ea543eaf445e1053"
status: active
---

# Release Compatibility Qualification

`0.9.5` is published as npm `latest` and GitHub Release `v0.9.5`. macOS arm64 public installation
passed isolated user setup, Codex install, validation, and stable update check. Native runtime uses
`fetch-depth: 0` because release-version parity reads the prior patch tag. Windows x64 public stable
installation and the repaired five-target native runtime run remain separate closure evidence.
