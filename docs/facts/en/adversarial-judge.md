---
schema_version: 1
pair_id: adversarial-judge
topic_slug: adversarial-judge
language: en
counterpart: ../ko/adversarial-judge.md
title: "Adversarial Judge Skill"
summary: "0.10.0 adds an explicit adversarial-judge step that prepares a clean-context host-native Judge request and reuses authenticated Hive quorum verification."
tags: [judge, skills, v0-10]
aliases: ["Adversarial review"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:fe327177fca73ccbdb3267a1cfca7b579b984e8bd3a24e74457a7d062020f2ec"
  - "repo:docs/decisions/product-release-decisions.md#sha256:59e330c3bd0a5a8133e00c447c99db44e30274dbf92770b662d3cf4c14b50e0f"
  - "repo:docs/plans/active/adversarial-judge-0.10.0.md#sha256:ed57c7d40872de6bd6963ec2f3d3a77488b1d9ca849b9d8236b47b7ef3745db2"
links: [judge-verification, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:26e5fd299f961d79c6b8237c212b4b07e9e99770"
status: active
---

# Adversarial Judge Skill

Hive has `judge-evidence` package and quorum verification but no explicit Judge-launch step. `adversarial-judge`
prepares a clean-context request and a provider-neutral dispatch envelope. The active host launches
the separate Judge and returns typed evidence. Findings remain diagnostic until the existing
authenticated quorum contract authorizes acceptance. Hive never calls a provider or spawns a process.
