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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:c313a53d8ed114aaf9b6303263730d282b11c6d8d52a71c249999b62969214fe"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a56419242874c459f08f7575ec0b2b6c2249ac696e0efffb053706dfeb6c9f00"
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
