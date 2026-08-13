---
schema_version: 1
pair_id: model-routed-custom-subagents
topic_slug: model-routed-custom-subagents
language: ko
counterpart: ../en/model-routed-custom-subagents.md
title: "Model-routed custom subagent"
summary: "Hive 0.9.3의 사용자 정의 에이전트 추천 전 서명된 호스트 모델 카탈로그 근거 검증."
tags: [claude, codex, model-routing, subagent, v0-9]
aliases: ["Sol Advisor 기능 동등성", "Task별 model routing"]
sources:
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:42506fc775e4a456f724c73fc71a2fb1fc80c12967606accb909de3ef323c888"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:616b6850533a66d89c369cf1660987ca3760468a74054fc35c3871a98dda464d"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:e659d878b301c03648849ad0fd73bf59e31730fd"
status: active
---

# Model-routed custom subagent

`MRA-*` 계획: bounded implementer·specialist의 Codex·Claude exact model/effort 고정.
`hive agent recommend`의 외부 보호 호스트 모델 카탈로그·분리 attestation·trust root 필수.
위조 서명 또는 exact mapping 누락 시 추천 결정 거부.
Activation 전 fresh host capability·lifecycle 수용 필요.
`manual|revise` request의 exact prior digest·prior request·scope 결합 검증.
