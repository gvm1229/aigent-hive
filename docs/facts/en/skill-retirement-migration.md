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
  - "repo:.github/workflows/release-publish.yml#sha256:240f5376bc7ea26109a1c34be965c6e74f9dc840e400af78bb1e55a2bac01263"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:ee4aa374f3452d250a376e66c1d1d49ce28c076b95aab8a8bdb9b155e05bca7e"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:5327d6c3417a62069df8eda30e76fe907c48418806023847eb16189cbe3041ef"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3620f38dc575abdd65508175d3a33b7e998b957d70b05c8c807337eafa5a0321"
  - "repo:docs/plans/active/skill-retirement-migration-0.10.0.md#sha256:3e2106b90defce8839164efed8054463a8504b873abcb7cd07d7e8a8a45c60bc"
  - "repo:harness/release/stable-skill-ledger.yml#sha256:eb54db4f5e5bac7900f9044fd0bf079b3466f5dd52b1475a89bd9699abd18ac0"
  - "repo:scripts/check-stable-skill-ledger.py#sha256:0aa6a3582d31854106dd69d7faf05072a9027a3e3cfa707e00fce43b7090de58"
links: [global-onboarding, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:9fc4580a60a40af3161efb32b68726d66b41f294"
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
