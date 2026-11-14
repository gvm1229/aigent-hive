---
schema_version: 1
pair_id: orchestration-ownership
topic_slug: orchestration-ownership
language: ko
counterpart: ../en/orchestration-ownership.md
title: "작업 조율 소유권"
summary: "Hive의 결정적 조율·공식 확장점 연결과 호스트의 모델 실행 소유권"
tags: [orchestration, ownership]
aliases: ["Orchestration owner"]
sources:
  - "repo:.agents/directives/02-architecture.md#sha256:6f4b5504814c95be4ef7c7035bd090db945804356dafdaf19944af77eb9c3d3b"
  - "repo:docs/decisions/ADR-0004-orchestration-ownership.md#sha256:0400842448b5e73cedabe1d2eb941abf343a0e1564b2e161c8e54d6677af017e"
  - "repo:docs/decisions/ADR-0015-host-native-skill-composition.md#sha256:c122052f10778e4c0e3c56c9511c2fdb6fc48528ba3d0dba599f91d3be77a5b5"
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:3bdc09d7cc02ced6cf4eb2a4dc4fa5e734653581144d8ab4e57914e21f4bb612"
  - "repo:docs/decisions/ADR-0026-official-cli-extension-points.md#sha256:ce73d78bda06047d4fe899e1f62f669389978675d668fe37a5a6800bfb76bd10"
links: [judge-verification, model-routed-custom-subagents, product-non-goals, skill-routing, v0-9-skill-suite-plan]
reviewed_revision: "git:3313076aab5bcc9c162b9e9beabc48a777aa5627"
status: active
---

# 작업 조율 소유권

ADR-0019: 사건·일정·임대·확인증·취소·팀·다중 목표 상태는 Hive, 모델·하위 에이전트 실행은 호스트 소유. 신규 OMX·OMC 의존 없음. 외부 실행은 읽기 전용, 명시 이전은 소유자 교체 대신 새 실행 생성. 엄격 반복·팀·다중 목표의 최종 검증은 두 호출 모드 모두 인증 Judge 필수, 주기·재시도별 호출 금지.

2026-11-12 ADR-0026: 공식 확장점·파일 수신 상자만 사용. Claude 대화형, Codex·Gemini 구독 비대화형 작업자. CLI 키 주입·화면 캡처·Claude 비대화형 금지. ADR-0019 불변, 실제 동작 검증과 구분.
