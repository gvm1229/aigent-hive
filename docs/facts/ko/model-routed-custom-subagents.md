---
schema_version: 1
pair_id: model-routed-custom-subagents
topic_slug: model-routed-custom-subagents
language: ko
counterpart: ../en/model-routed-custom-subagents.md
title: "Model-routed custom subagent"
summary: "Hive 0.9.3 사용자 정의 에이전트 activation의 전체 host lifecycle capability와 Judge 호출 정책 값 검증"
tags: [claude, codex, model-routing, subagent, v0-9]
aliases: ["Sol Advisor 기능 동등성", "Task별 model routing"]
sources:
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:42506fc775e4a456f724c73fc71a2fb1fc80c12967606accb909de3ef323c888"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:c713d13630d17bb216771b6e18b9efb0f4fb38bb0f9de312b3220e5b48ae1282"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:271ad81dbf3389ba8932c5617738cf5f9022c5c9"
status: active
---

# Model-routed custom subagent

`MRA-*` 계획: bounded implementer·specialist의 Codex·Claude exact model/effort 고정.
`hive agent recommend`의 외부 보호 호스트 모델 카탈로그·분리 attestation·trust root 필수.
위조 서명 또는 exact mapping 누락 시 추천 결정 거부.
Activation 전 fresh host capability·lifecycle 수용 필요.
`manual|revise` request의 exact prior digest·prior request·scope 결합 검증.
dispatch·acknowledgement·result·cancel·lookup·idempotency를 포함한 lifecycle capability의
non-supported activation 거부. User setup의 Judge 호출 정책: `explicit|implicit` 두 값과 동일한
기본값·공개 설정 계약 검증.
receipt 부재·role collision·symlink projection path의 foreign byte 보존 hostile regression.
