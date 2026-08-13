---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: ko
counterpart: ../en/public-skill-identity.md
title: "Skill identity"
summary: "Aigent Hive의 product-only 26개 Skill과 지식 Skill의 기능명·정본 영문 ID 병기."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:crates/hive-projection/src/lib.rs#sha256:7b06c12e607a3e6ef8cf547fe8d6d2be67abf70edadfbb45b0612432a2a7b1ff"
  - "repo:docs/plans/active/knowledge-skill-naming-0.9.3.md#sha256:395a33fa2bbab8440265570dd1802605d2157ed0029b86fdc326a825ac1771d8"
  - "repo:docs/skills.md#sha256:89909ed6df13cf089302e226e4df2a27322dfcc0007292302434df18b7a85ae0"
  - "repo:harness/skills/catalog.yml#sha256:640f2ded6bb90de6c8c0797d21028091512569549478400b4721245c47ce3fae"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:da8ff786068c1cf28b0e40862494767ddeffe9c0"
status: active
---

# Skill identity

Aigent Hive product Skill 정본: 26개. 실행·설정 호환을 위해 기존 영문 ID 유지.
지식 Skill의 한국어 표시명은 기능명과 ID를 함께 보이며, 설명은 `(knowledge-...)`로 시작.
`knowledge-capture`: 대화 종료 전 후속 작업에 도움 되는 안전한 지식 하나 기록.
`knowledge-recall`: 현재 작업의 관련 지식 조회. `knowledge-import`: 명시 대상 저장소 스캔.
`knowledge-promote`: 전역 공유. `knowledge-maintain`: 신뢰성 검사·색인 재생성·명시 정리.
