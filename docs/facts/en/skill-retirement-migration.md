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
  - "repo:.github/workflows/release-publish.yml#sha256:e664105a2734fc5ec7c35f93ddc5ce0362ad5e391ae881c63e326a8c25866bca"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:5a7d279c9ce96bec6792f191e4be0caa69bf17c9062ad535e11478af7f4408c2"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:5327d6c3417a62069df8eda30e76fe907c48418806023847eb16189cbe3041ef"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/skill-retirement-migration-0.10.0.md#sha256:3e2106b90defce8839164efed8054463a8504b873abcb7cd07d7e8a8a45c60bc"
  - "repo:harness/release/stable-skill-ledger.yml#sha256:d9ce44fa49315d495c6d01c7c03004c5f9dbc9e16e67190d9ca1df82dd2ef5d8"
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
