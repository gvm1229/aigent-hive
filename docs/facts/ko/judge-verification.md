---
schema_version: 1
pair_id: judge-verification
topic_slug: judge-verification
language: ko
counterpart: ../en/judge-verification.md
title: "Judge verification 경계"
summary: "외부 서명 Judge artifact 검증과 v0.9.0 reserved exact-model Judge 호출 정책."
tags: [judge, security, verification]
aliases: ["Ed25519 judge quorum"]
sources:
  - "repo:docs/decisions/ADR-0007-ed25519-judge-trust.md#sha256:5a17bcd8f6869437a9e37c025c4fa2da285ff03af16a7144552162bac5a09a1a"
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:8dcf64600bf77f630d6f601027ee02a5adf1255a49c4c852ff6006a46f203817"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:6b2eb3faafe345678008fe225dd941026c8eab10911fa19530c2785c8b644f57"
links: [model-routed-custom-subagents, orchestration-ownership, product-non-goals, release-verification]
reviewed_revision: "git:ffdfb476d4e21dafe5d4dc896fa272f7244d0fe1"
status: active
---

# Judge verification 경계

현재 Hive 범위: bounded evidence package와 외부 서명 assignment·verdict·critical human approval 검증.
Private key·signing·agent 실행 비소유. `0.9.0` 계획: Codex Sol High와 exact model/effort receipt의
fresh read-only user-scope custom Judge. `explicit`은 strict terminal gate 한정, `implicit`은 material-risk task 추가.
Strict gate는 항상 quorum 필수, scheduler tick별 dispatch 금지. Host는 agent 실행, 외부 signer는 서명,
Hive는 digest-bound quorum 검증.
