---
schema_version: 1
pair_id: steering-preservation
topic_slug: steering-preservation
language: ko
counterpart: ../en/steering-preservation.md
title: "추가 지시의 이전 조건 보존"
summary: "0.12.1 첫 차수의 제품 전달 제외·후속 차수 대기, 소스 규칙 유지"
tags: [agent, directives]
aliases: []
sources:
  - "repo:.agents/directives/references/follow-up-instructions.md#sha256:e06c604bfc0f46b38c7affdc30c5a56e71945c340618f7167a96ad69f40c7ea9"
  - "repo:docs/plans/backlog/steering-0.12.1.md#sha256:0d2c963c6af27ad7e137efcdc42f6c2ff16fe59634e6274c2e6652caa8fbd5ce"
  - "repo:docs/research/steering-preservation-0.12.1.md#sha256:f7857c23d4ee8fc5291c2f3571d32848a5988689be21ff7d82321829d0b0dc74"
links: [agent-autonomous-continuation]
reviewed_revision: "git:7673c0ca72551d413d7f9e41f00d88c3f388b027"
status: active
---

# 추가 지시의 이전 조건 보존

- 사용자 요구: 이전 지시 유지, 명시적 교체 없는 충돌의 선택 질문
- 새 메시지의 기본 추가·명시 범위만 교체, 충돌 행동은 답변 전 보류·독립 작업 지속
- 취소 즉시 존중·요약 뒤 미해결 조건 보존. 현재 소스 규칙 유지
- 제품 지침·양언어 전역 전달은 0.12.1 첫 차수 제외·후속 차수 대기, dcc0d242의 변경 분리
- 이전 Windows 세 호스트·위키 비활성 전달 시험은 역사적 근거. 현재 제품 전달·실제 모델 준수·모든 도구 차단 증명 제외
