---
schema_version: 1
pair_id: skill-retirement-migration
topic_slug: skill-retirement-migration
language: en
counterpart: ../ko/skill-retirement-migration.md
title: "Skill Retirement Migration"
summary: "A successful direct 0.10.0 upgrade removes authenticated retired Skill artifacts from every supported predecessor while preserving foreign bytes through a blocking conflict."
tags: [migration, skills, upgrade, v0-10]
aliases: ["Retired Skill cleanup"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:6d9b351dfbe99fef461d642285a5bc37730ef6ba29d3c62d38c800bdd8e6220f"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:7a5c873834ba9a77e6efdedc60a5eed953fa40102dfcf88c084db5b591f465c3"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:586e27426f1c48ebc8ad92754d478b731d9b07bbba01e61a34e9f0469c43c031"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/skill-retirement-migration-0.10.0.md#sha256:4801b99b3ee38888a2ece0abd9671e14f3c536519d47080af91c8298cd5d69e6"
  - "repo:harness/release/stable-skill-ledger.yml#sha256:e1852b13986655af87b6271433d89734b8a260d1e565e81a27dcd5d6081a233a"
  - "repo:scripts/check-stable-skill-ledger.py#sha256:b19da205df0303dc56e9e8ceeed1ac84f26db8abc337b7e75c0dc06e5a35ed24"
links: [global-onboarding, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
status: active
---

# Skill Retirement Migration

The historical registry permanently covers every npm or GitHub stable release (`0.8.0` and
`0.9.0` through `0.9.5`) and must append each future stable snapshot before publication. The
`0.10.0` updater authenticates historical Skill bytes directly; it does not install every
intermediate release.
The lifecycle ledger requires version and direct replacement for each 0.10.0 rename or merge.
Exact Hive-owned retired files, host projections, manifest entries, and empty
directories are removed atomically with canonical activation. A safe local merge is permitted.
Unknown or foreign bytes are preserved and block successful activation, so no successful upgrade
leaves a retired discoverable Skill.
