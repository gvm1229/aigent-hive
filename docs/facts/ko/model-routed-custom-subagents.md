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
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:8dcf64600bf77f630d6f601027ee02a5adf1255a49c4c852ff6006a46f203817"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:6b2eb3faafe345678008fe225dd941026c8eab10911fa19530c2785c8b644f57"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:ffdfb476d4e21dafe5d4dc896fa272f7244d0fe1"
status: active
---

# Model-routed custom subagent

`MRA-*` 계획: bounded implementer·specialist의 Codex·Claude exact model/effort 고정.
User-scope reserved Judge의 Codex 후보는 Sol High, Claude profile은 lifecycle 검증 대기.
Setup 선택: `explicit`은 strict workflow terminal gate만, `implicit`은 material-risk route 추가.
Iterative·team·multi-goal은 선택과 무관하게 terminal acceptance 판정 필수, tick별 호출 금지.
Agent는 verdict만 생성, 외부 signer가 Ed25519 private key 소유, Hive는 bound receipt·quorum 검증.
생성 Skill의 reserved Judge override 금지.
