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
  - "repo:docs/research/0.11.0-test9-qualification.md#sha256:a2866f12daf03e438b220264686b045d82f41a4fb7a793cfb3781cd62d6a03c4"
  - "repo:docs/research/directive-context-host-acceptance-2026-09-27.md#sha256:aca52c206a312d7b612acea527e6789c3fd6e88b12c52652eb989c7936a58017"
  - "repo:docs/research/directive-context-implementation-0.11.0.md#sha256:eb38786a9b205550a61a3a9a52f98aac7766b04999dbd31fcdd8bd51e6da2e6e"
  - "repo:docs/research/directive-context-instrumented-tests-2026-09-26.md#sha256:e2a20c1a87519399eca28095c6a4de7e267a3fe87ca9f4c103ab252a5b07c9ab"
links: [project-policy-enforcement]
reviewed_revision: "git:b90d51eb7d60c5f5f6b555d329cfef7800330c60"
status: active
---

# 반복 압축 뒤 지침 복구

- test.9: 승인 구절 4,096바이트 상한·경로 안내·파일 보호. 부모 전달 상태 공유·추가 모델 호출 제외
- 바뀐 원문 주입 금지, 철회 유지. 공개 훅 계약 세 운영체제 각 29개 통과
- Windows Codex·gpt-6-astra: 세 조건 각 5개 기준, 재개 3회·하위 작업 3개 확인
- 추가 안내 입력 2,239→526토큰(76.5% 감소), 명령 p95 증가 0.10%
- 예산 초과 실행의 실패 기록 유지, 관측된 제품 결과는 독립 오프라인 판정. 보편적 준수·과금 절감 보장 제외
