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
  - "repo:docs/archive/plans/releases/0.9.5/release-0.9.5-stable-publication.md#sha256:70ed823701fa0ae8be728d97b8705846f0eaa50e6e8758425d439bfee4d1334c"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:a520f8e5113c7fc02711eb5e5d8021605f7ee551"
status: active
---

# Release Compatibility Qualification

`0.9.5` is npm `latest` and GitHub Release `v0.9.5`. The macOS arm64 public installation passed.
The Windows x64 public npm binary identity was verified. Isolated Codex `0.148.0` setup, install,
validation, and stable update check passed. Full-history native runtime run
`32118217691` passed all five targets.
