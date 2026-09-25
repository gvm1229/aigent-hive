---
schema_version: 1
pair_id: directive-context-recovery
topic_slug: directive-context-recovery
language: ko
counterpart: ../en/directive-context-recovery.md
title: "반복 압축 뒤 지침 복구"
summary: "승인 구절 복원 구현과 실제 압축 준수 검증의 분리"
tags: [context, hooks, policy]
aliases: []
sources:
  - "repo:docs/guides/directive-context-recovery.md#sha256:20a1f22a3570c4dd8ad993c0ae04fee31ebcbc43646f90c72811501d30ea0438"
  - "repo:docs/research/directive-context-implementation-0.11.0.md#sha256:eb38786a9b205550a61a3a9a52f98aac7766b04999dbd31fcdd8bd51e6da2e6e"
  - "repo:docs/research/directive-context-instrumented-tests-2026-09-26.md#sha256:e2a20c1a87519399eca28095c6a4de7e267a3fe87ca9f4c103ab252a5b07c9ab"
links: [project-policy-enforcement]
reviewed_revision: "git:b90d51eb7d60c5f5f6b555d329cfef7800330c60"
status: active
---

# 반복 압축 뒤 지침 복구

- test.8: 승인 구절 복원 4,096바이트 상한·경로 안내·파일 보호
- 매 압축·하위 작업 재전달, 부모 전달 상태 공유·추가 모델 호출 제외
- 바뀐 원문 주입 금지, 설정 손상 시 편집 거부·철회 가능
- 공개 실행 파일: Windows·macOS·Linux 각 26개 계약 통과
- Windows 수동 압축 10회 보호 확인. 호스트 단독 자동 압축 한 턴 3회 관측
- 철회 후 재등록 결함 수정: 27개 통과·권한 조건 1개 제외. 신뢰 후 자동 압축 3회 복원 확인. 문구 비교 입력 1,662토큰 감소, 과금 절감 미증명
- 목표: 지침 유지·불필요한 강제와 비용 최소화
