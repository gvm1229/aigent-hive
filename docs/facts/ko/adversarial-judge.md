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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:1645eb2249265b75d27b0c65a709806f4999a0ec425e8e874336bcda084b702c"
  - "repo:docs/decisions/product-release-decisions.md#sha256:25bd2880270b2dd21bf09d5efe576f4164b8d02fadd8366f8649d8d50d38bded"
  - "repo:docs/plans/active/adversarial-judge-0.10.0.md#sha256:c26efcfc99708c0c6edb6d1d3b4e0b473172a3c24c247938097e6515de7fdaf5"
links: [judge-verification, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:a2518fa364c40efb4e676fe31b694562f73dd819"
status: active
---

# Adversarial Judge Skill

- 기존 기능: Package·quorum 검증 보유, 명시적 Judge launch 단계 부재
- 새 역할: Clean-context request·provider-neutral dispatch envelope 준비
- 실제 launch: Active host 소유
- 판정: Finding은 diagnostic, 기존 authenticated quorum 통과 뒤에만 acceptance authority
- 금지: Provider 호출·credential·Hive direct process spawn
