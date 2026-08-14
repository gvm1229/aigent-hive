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
  - "repo:crates/hive-cli/src/custom_agent_cli.rs#sha256:5726ce3e28f3198b267fc017cba94d53c4a8703efa74544e5499be7c9488d9dd"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:73d95136c28b6742d76d7aca0874144808500168a67fc97accdee9bc2b387481"
  - "repo:crates/hive-core/src/native_workflow.rs#sha256:246f845d21fe73c070abdfa4ffa78d28e829d84b3da498dcc1530355a54a0900"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:9fe4b79c4f4e0be1706600e06b74ab93ee8bbce01e767a38790bbf8bdd21b251"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:9c9bdb1bfc49e06110fe3e1d0f931b03ab2c3b57"
status: active
---

# Model-routed custom subagent

`hive agent recommend`: 외부 보호 host model catalog·분리 attestation·trust root 필수.
위조 서명·exact mapping 누락·지원 밖 lifecycle capability는 activation 전 거부.
`manual|revise` request: exact prior digest·request·scope 결합 검증.
Judge: strict terminal acceptance는 두 정책에서 허용, material-risk는 `implicit`만 허용.
simple·read-only·format-only·scheduler·heartbeat·retry·결정적 실패·unsupported host는 항상 거부.
receipt 부재·role collision·symlink projection path의 foreign byte 보존 hostile regression.
