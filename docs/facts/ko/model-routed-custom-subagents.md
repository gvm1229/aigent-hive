---
schema_version: 1
pair_id: model-routed-custom-subagents
topic_slug: model-routed-custom-subagents
language: ko
counterpart: ../en/model-routed-custom-subagents.md
title: "Model-routed custom subagent"
summary: "Codex·Claude exact-model custom subagent routing과 runtime attestation의 Hive 0.9.0 계획."
tags: [claude, codex, model-routing, subagent, v0-9]
aliases: ["Sol Advisor 기능 동등성", "Task별 model routing"]
sources:
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:fc30a26c372c5dd0881e5fcc36742820f49ae62ef4ee8fac410a60e8d8509fc0"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:b2b6be0e52d1b73542966859842381dcae1805353fd99173742411011fa731cf"
links: [orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:150c4ddd84eb80b0906da731aceb81c4f2d8e059"
status: active
---

# Model-routed custom subagent

`0.9.0` 범위의 `MRA-*` 계획: Codex·Claude 대상 Sol Advisor clean-room 기능 동등성.
Provider-neutral role 정본: host별 exact model·thinking level, user/project scope,
권한, routing description, ownership digest. 결과 수용 조건: host receipt의 예상
role·model·effort·scope·definition digest 일치. Built-in 후보: routine·complex 구현,
fresh review, design, article writing, research, verification. Purpose-first
`hive-custom-subagent-create`: 양쪽 host mapping 추천, 수락·수동·수정 선택,
승인 role의 동일 automatic registry 통합. Activation 조건: 실제 fresh-session proof,
fail-closed mismatch test, projection 명시적 동의, foreign host config overwrite `0건`.
