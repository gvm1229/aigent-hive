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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:c313a53d8ed114aaf9b6303263730d282b11c6d8d52a71c249999b62969214fe"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a56419242874c459f08f7575ec0b2b6c2249ac696e0efffb053706dfeb6c9f00"
  - "repo:docs/plans/active/skill-retirement-migration-0.10.0.md#sha256:66a7fc4b9b18ba9fbe35e252b13188aa19c80488c76e4f573e7671f79aa07ead"
links: [global-onboarding, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:354ea0ab7f22d94d231a5bb2c54385d003a04815"
status: active
---

# Skill Retirement Migration

The historical registry permanently covers every npm or GitHub stable release (`0.8.0` and
`0.9.0` through `0.9.5`) and must append each future stable snapshot before publication. The `0.10.0` updater
authenticates historical Skill bytes directly; it does not install every
intermediate release. The initial registry covers every released 0.8.0 and 0.9.x stable snapshot.
Exact Hive-owned retired files, host projections, manifest entries, and empty
directories are removed atomically with canonical activation. A safe local merge is permitted.
Unknown or foreign bytes are preserved and block successful activation, so no successful upgrade
leaves a retired discoverable Skill.
