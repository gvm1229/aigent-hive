---
schema_version: 1
pair_id: adversarial-judge
topic_slug: adversarial-judge
language: ko
counterpart: ../en/adversarial-judge.md
title: "Adversarial Judge Skill"
summary: "Clean-context host-native Judge 요청을 준비하고 기존 authenticated quorum 검증을 재사용하는 0.10.0 명시 단계"
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

- 기존 기능: `judge-evidence` package·quorum 검증 보유, 명시적 Judge launch 단계 부재
- 새 역할: Clean-context request·provider-neutral dispatch envelope 준비
- 실제 launch: Active host 소유
- 판정: Finding은 diagnostic, 기존 authenticated quorum 통과 뒤에만 acceptance authority
- 금지: Provider 호출·credential·Hive direct process spawn
