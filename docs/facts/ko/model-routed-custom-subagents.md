---
schema_version: 1
pair_id: model-routed-custom-subagents
topic_slug: model-routed-custom-subagents
language: ko
counterpart: ../en/model-routed-custom-subagents.md
title: "Model-routed custom subagent"
summary: "Exact-model Codex·Claude role과 설정 가능한 authenticated Judge의 Hive 0.9.0 계획."
tags: [claude, codex, model-routing, subagent, v0-9]
aliases: ["Sol Advisor 기능 동등성", "Task별 model routing"]
sources:
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:cc7e79da6c27052fb9dc256a47e057deab617bbb66a567da8455d9135d6407b8"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:a2779d3f1ebab829c48214fd4486f9505e7207b6de1cdfe3c6af56c9121534ce"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:4e750ce659c953d7d71ab6e9536c29968ab1f028"
status: active
---

# Model-routed custom subagent

`MRA-*` 계획: bounded implementer·specialist의 Codex·Claude exact model/effort 고정.
User-scope reserved Judge의 Codex 후보는 Sol Max, Claude profile은 lifecycle 검증 대기.
Setup 선택: `explicit`은 strict workflow terminal gate만, `implicit`은 material-risk route 추가.
Iterative·team·multi-goal은 선택과 무관하게 terminal acceptance 판정 필수, tick별 호출 금지.
Agent는 verdict만 생성, 외부 signer가 Ed25519 private key 소유, Hive는 bound receipt·quorum 검증.
생성 Skill의 reserved Judge override 금지.
