---
schema_version: 1
pair_id: implementation-plan-handoff
topic_slug: implementation-plan-handoff
language: ko
counterpart: ../en/implementation-plan-handoff.md
title: "구현 계획 인계"
summary: "다른 모델의 구현 전에 계획자가 중요한 설계 선택을 확정하는 소스 작업 규칙"
tags: [directives, planning]
aliases: []
sources:
  - "repo:.agents/directives/references/planning-contract.md#sha256:4136a403ed93af42bed844f30e7ed48309ab9ed535c3deb91b1d264dab2fda33"
links: [agent-directive-ownership, plan-persistence]
reviewed_revision: "git:453bda5786f55a11c6e2c03a88bcb13bce5171ac"
status: active
---

# 구현 계획 인계

- 계획자의 대화·추론 능력을 공유하지 않는 구현자를 위한 자기완결적인 계획
- 파일·함수·설계 결정·수정 순서·자료/오류/호환성·검증·복구 조건 명시
- 중요한 설계 불일치는 종속 편집 전에 계획자에게 반환, 독립적인 계획 단계는 계속 진행
- 구현자는 소유 단계만 읽고 계획 작성 참고자료를 매번 다시 읽지 않는 방식
- Hive 소스 개발 규칙이며 설치된 소비자 지침의 변경과 구분
