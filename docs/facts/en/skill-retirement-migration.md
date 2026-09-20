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
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b57e2ac894503cf91d6efc41940284e79cc11e2ca70f6718d5b88926a917ec67"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:5327d6c3417a62069df8eda30e76fe907c48418806023847eb16189cbe3041ef"
  - "repo:docs/decisions/product-release-decisions.md#sha256:9f234ef3fede8030ab6ad57fa4b560a71f30f468622c4e4ad56b83864d0e05ce"
  - "repo:docs/plans/active/skill-retirement-migration-0.10.0.md#sha256:3e2106b90defce8839164efed8054463a8504b873abcb7cd07d7e8a8a45c60bc"
  - "repo:harness/release/stable-skill-ledger.yml#sha256:1bbe29fda34d23fc2a3accb4bc30714f68b2d1c1f1a7522850d09fe03843de73"
  - "repo:scripts/check-stable-skill-ledger.py#sha256:0aa6a3582d31854106dd69d7faf05072a9027a3e3cfa707e00fce43b7090de58"
links: [global-onboarding, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:4afd5ba4483d98f63ae42c065837f9b010506657"
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
