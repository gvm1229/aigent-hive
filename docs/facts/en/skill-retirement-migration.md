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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:fe327177fca73ccbdb3267a1cfca7b579b984e8bd3a24e74457a7d062020f2ec"
  - "repo:docs/decisions/product-release-decisions.md#sha256:59e330c3bd0a5a8133e00c447c99db44e30274dbf92770b662d3cf4c14b50e0f"
  - "repo:docs/plans/active/skill-retirement-migration-0.10.0.md#sha256:48117baa1a1de7d252946c6f270c6eef67b526223e954dab98fb5c585bd3b8a2"
links: [global-onboarding, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:26e5fd299f961d79c6b8237c212b4b07e9e99770"
status: active
---

# Skill Retirement Migration

The `0.10.0` updater authenticates historical Skill bytes directly; it does not install every
intermediate release. Exact Hive-owned retired files, host projections, manifest entries, and empty
directories are removed atomically with canonical activation. A safe local merge is permitted.
Unknown or foreign bytes are preserved and block successful activation, so no successful upgrade
leaves a retired discoverable Skill.
