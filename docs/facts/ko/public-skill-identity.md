---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: ko
counterpart: ../en/public-skill-identity.md
title: "Skill identity"
summary: "Aigent Hive의 product-only 26개 Skill과 지식 Skill의 한국어 기능명 표시·설명 첫머리 정본 영문 ID 표기."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:crates/hive-projection/src/lib.rs#sha256:72c05f8bc6c5a7c3f94a42c34d87828c3beea0d32345171dac598d83a153819b"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:9d445726c92856de8c47781743cfc972aeeebbd1e74cc1660b860dd0ebac573a"
  - "repo:harness/skills/catalog.yml#sha256:d07890ccf090177ed03405d9eae01c278130cdd1fa9797ad3616106d1c67f6c8"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:8fcf8b4794bb7d3d92065ad3f49a03acb33c4c13"
status: active
---

# Skill identity

Aigent Hive product Skill 정본: 26개. 실행·설정 호환을 위해 기존 영문 ID 유지.
지식 Skill 한국어 표시명: 기능명만 표시. `(knowledge-...)` 정본 ID: 설명 첫머리 한 번 표기.
`knowledge-capture`: 대화 종료 전 후속 작업에 도움 되는 안전한 지식 하나 기록.
`knowledge-recall`: 현재 작업의 관련 지식 조회. `knowledge-import`: 명시 대상 저장소 스캔.
`knowledge-promote`: 전역 공유. `knowledge-maintain`: 신뢰성 검사·색인 재생성·명시 정리.
다음 version 미정 반영. `v0.9.4` release·tag·package 변경 없음.
