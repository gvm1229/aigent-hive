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
  - "repo:docs/plans/active/release-0.9.5-stable-publication.md#sha256:9c5cdbda5056381989f3d79e639fde0437e631465c808147efdf76eef5e654ae"
  - "repo:docs/plans/active/release-compatibility-qualification-0.9.5.md#sha256:722d961e65b3ed28b344ce2fc27edb1f08453f738cb5c33b11e920ae15c53429"
  - "repo:scripts/accept-public-hive.py#sha256:59a78bea773c38e18248fb6cdefe6e612a69d8f46ae0139eeff7a7b30fa455f2"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [historical-project-base-coverage, release-verification, test-lane-inventory]
reviewed_revision: "git:45a9af8dffd0b46f9a12b11da56de9934d3069fd"
status: active
---

# Release Compatibility Qualification

Public `0.9.5-test.3` remains the prior artifact, not evidence for later corrections. The current
tree adds a real `0.9.2` project-copy acceptance runner and an explicit test update channel. Those
changes require `test.4`, then `test.4 → test.5` user-projection acceptance before stable promotion.
