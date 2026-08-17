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
  - "repo:docs/plans/active/release-0.9.5-stable-publication.md#sha256:ea8a5d1573a3f10cb03925e57b706528c91ea2dffefba1ef2a0072387b37210f"
  - "repo:docs/plans/active/release-compatibility-qualification-0.9.5.md#sha256:fde4ef5a0a738761093a40f84f93d0a6eb4e6a5e64449f606b71ef6619cb20da"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:0fd5ea87fa377dc584dcfa6ad93ae9ee74eb4e97"
status: active
---

# Release Compatibility Qualification

The `0.9.5-test.3` candidate and public release bind source `224170e`, exact npm test packages,
and the Windows archive digest. The direct installer and npm `0.9.4` upgrade passed in an isolated
Windows path. User-projection/bare-update and public `0.9.2` project-upgrade acceptance remain
blocked by unauthenticated existing ownership; stable promotion must not bypass that boundary.
