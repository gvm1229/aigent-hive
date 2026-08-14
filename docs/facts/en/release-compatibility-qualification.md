---
schema_version: 1
pair_id: release-compatibility-qualification
topic_slug: release-compatibility-qualification
language: en
counterpart: ../ko/release-compatibility-qualification.md
title: "Release Compatibility Qualification"
summary: "The 0.9.5 plan makes every declared compatibility source an executable matrix requirement before stable promotion."
tags: [compatibility, migration, release, testing]
aliases: ["Compatibility matrix gate"]
sources:
  - "repo:.github/workflows/release.yml#sha256:3f92bd0fb18bf5519493389af3edcaf44a6d81e0c1c9fdc0382cf4a6931c7f6c"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:0aa8c272002f64443b8204e80f5744c02474e4621ca807d28cfe36ff3bdb49f6"
  - "repo:docs/plans/active/release-compatibility-qualification-0.9.5.md#sha256:edb53b6054f7d0f17a09e7331ff15e15aa02d630a1adbf483b954b400d83f247"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:e44b9498ca36ebeb0b477a3d4e5c06a4e71561ef"
status: active
---

# Release Compatibility Qualification

The `0.9.5` local qualification adds a digest-bound project-base coverage report and a compiled
CLI matrix for prior project states. The future candidate workflow retains that report as an
artifact. Public test and stable publication remain deferred by the maintainer; when resumed,
their artifact and coverage-report digests must match or promotion stops.
