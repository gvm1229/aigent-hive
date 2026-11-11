---
schema_version: 1
pair_id: harness-improvement-research
topic_slug: harness-improvement-research
language: ko
counterpart: ../en/harness-improvement-research.md
title: "점수 기반 Hive 개선 방향"
summary: "Hive 실행 통제·작업별 점수·제한된 후보 시험·독립 검증·조건부 경험 재사용의 결합"
tags: [evaluation, orchestration, research]
aliases: ["Harness²", "점수 기반 평가"]
sources:
  - "repo:docs/decisions/score-driven-improvement.md#sha256:8c19a8b5cba675d3789919303e76008546e81e5ddcabb94c0837072f92e6c344"
  - "repo:docs/plans/score-driven-improvement-0.14.0.md#sha256:a138c9ad92622a2460daa5046ecf71d8c486aef36939996c89fadab0356ee8e0"
links: [ai-learning-application-review, orchestration-ownership]
reviewed_revision: "git:7a7f314197b23f93f72e839bbb9f6226470a8f9d"
status: active
---

# 점수 기반 Hive 개선 방향

- Harness² 최종 목표 철회, 참고 방법으로만 활용
- 방향: Hive 실행 통제·작업별 점수·제한된 후보 시험·독립 검증·조건부 경험 재사용
- 정본: docs/plans/score-driven-improvement-0.14.0.md, 구현 항목12개
- 산출물 수정·하네스 개선: 별도 상태와 하나의 상위 예산
- 승인 범위: 계획 작성, 구현·설치·출시 제외
- 기존 CLI 작업: 실행 시 계획 통합 전까지 별도 유지
