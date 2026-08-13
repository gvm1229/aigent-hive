---
schema_version: 1
pair_id: model-routed-custom-subagents
topic_slug: model-routed-custom-subagents
language: ko
counterpart: ../en/model-routed-custom-subagents.md
title: "Model-routed custom subagent"
summary: "Hive 0.9.3 사용자 정의 에이전트 activation의 전체 host lifecycle capability와 Judge 호출 범위의 closed policy"
tags: [claude, codex, model-routing, subagent, v0-9]
aliases: ["Sol Advisor 기능 동등성", "Task별 model routing"]
sources:
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:cffd6c491ffd17dccefa84edb172bbfe64ae925f2fe9cf7c6efd07e6a896a9fd"
  - "repo:crates/hive-core/src/native_workflow.rs#sha256:246f845d21fe73c070abdfa4ffa78d28e829d84b3da498dcc1530355a54a0900"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:b2003c6e94c978fc051aeba5240a6cf80f8c2700940ca5fb9b368dbaa4fe0404"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:da8ff786068c1cf28b0e40862494767ddeffe9c0"
status: active
---

# Model-routed custom subagent

`hive agent recommend`: 외부 보호 host model catalog·분리 attestation·trust root 필수.
위조 서명·exact mapping 누락·지원 밖 lifecycle capability는 activation 전 거부.
`manual|revise` request: exact prior digest·request·scope 결합 검증.
Judge: strict terminal acceptance는 두 정책에서 허용, material-risk는 `implicit`만 허용.
simple·read-only·format-only·scheduler·heartbeat·retry·결정적 실패·unsupported host는 항상 거부.
receipt 부재·role collision·symlink projection path의 foreign byte 보존 hostile regression.
