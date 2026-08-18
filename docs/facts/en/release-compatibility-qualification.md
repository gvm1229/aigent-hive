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
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:5d1ded97d4dfa1fcc3bbac149ededed530ce4d384eb8b87b360c441fbbce8deb"
  - "repo:docs/plans/active/release-0.9.5-stable-publication.md#sha256:7a4fa8bee33908d623acc312c9e8c9431d578bca08306e0711b514347a1e733b"
  - "repo:docs/plans/active/release-compatibility-qualification-0.9.5.md#sha256:722d961e65b3ed28b344ce2fc27edb1f08453f738cb5c33b11e920ae15c53429"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:eaf0126b6abd73427dc01cd02d21eeda9a7736e5"
status: active
---

# Release Compatibility Qualification

The corrected updater passed public Windows `test.12` to `test.13` and macOS `test.14` to `test.15`
direct-install acceptance. The public `test.15` project runner also passed the PortareFolium `0.9.2`
upgrade matrix with source and foreign-byte preservation plus tampered-ledger no-mutation. Protected
`main` integration and the five-platform stable candidate passed; only the separately authorized
stable npm publication and its post-publication validation remain.
