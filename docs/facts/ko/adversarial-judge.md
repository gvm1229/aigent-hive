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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:c313a53d8ed114aaf9b6303263730d282b11c6d8d52a71c249999b62969214fe"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a56419242874c459f08f7575ec0b2b6c2249ac696e0efffb053706dfeb6c9f00"
  - "repo:docs/plans/active/adversarial-judge-0.10.0.md#sha256:2334648e6f6ab90c67010884b7c18ad55fd2f7607383cb834f0a9205fae02bc9"
links: [judge-verification, v0-10-product-scope, verified-workflow]
reviewed_revision: "git:ac178d3c45a5b22903488042bab9ef3ed662fb12"
status: active
---

# Adversarial Judge Skill

- 기존 기능: `judge-evidence` package·quorum 검증 보유, 명시적 Judge launch 단계 부재
- 새 역할: Clean-context request·provider-neutral dispatch envelope 준비
- 실제 launch: Active host 소유
- 판정: Finding은 diagnostic, 기존 authenticated quorum 통과 뒤에만 acceptance authority
- 금지: Provider 호출·credential·Hive direct process spawn
