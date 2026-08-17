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
  - "repo:.github/workflows/release.yml#sha256:53f1b3c4284326ae392594d1135ad600ddbd8035ec2caf16ed9ea0e52dc2efd4"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:c5dba7810327a88235025ea62ba2b77387a072c8e76b044b661ddb911aa26220"
  - "repo:docs/plans/active/release-compatibility-qualification-0.9.5.md#sha256:fde4ef5a0a738761093a40f84f93d0a6eb4e6a5e64449f606b71ef6619cb20da"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:0fd5ea87fa377dc584dcfa6ad93ae9ee74eb4e97"
status: active
---

# Release Compatibility Qualification

The `0.9.5` local qualification adds a digest-bound project-base coverage report and a compiled
CLI and signed release-update matrix for prior project states. The candidate workflow retains that report as an artifact.
The public test and stable candidate must use matching artifact and coverage-report digests, or
promotion stops.
