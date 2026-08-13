---
schema_version: 1
pair_id: judge-verification
topic_slug: judge-verification
language: ko
counterpart: ../en/judge-verification.md
title: "Judge verification 경계"
summary: "루프 terminal evidence 수용 전 외부 서명 Ed25519 Judge quorum 재검증."
tags: [judge, security, verification]
aliases: ["Ed25519 judge quorum"]
sources:
  - "repo:docs/architecture/judge-trust-boundary.md#sha256:ba816f14dd830e1299ef1a41baaeddffead88cffb23e29ee0599423bd02f3fa1"
  - "repo:docs/decisions/ADR-0007-ed25519-judge-trust.md#sha256:5a17bcd8f6869437a9e37c025c4fa2da285ff03af16a7144552162bac5a09a1a"
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:42506fc775e4a456f724c73fc71a2fb1fc80c12967606accb909de3ef323c888"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:3c19d66b868d0b07f03d7d7eda62c0cd4c3d2db46920e9cfc65f8c5b0967f165"
links: [model-routed-custom-subagents, orchestration-ownership, product-non-goals, release-verification]
reviewed_revision: "git:8d377f6ad981702927c351e155b4f08a400a80ea"
status: active
---

# Judge verification 경계

Hive 범위: bounded evidence package와 외부 서명 assignment·verdict·critical human approval 검증.
Private key·signing·agent 실행 비소유. 루프 Judge verifier의 필수 결합: v2 quorum request digest,
외부 보호 trust root, exact run·revision·node·attempt·evidence ID. Hive의 evidence 수용 조건:
Ed25519 quorum 재검증 뒤 authenticated PASS와 동일 subject 확인. Completion 권한 제외 대상:
단순 authentication flag, unsigned request, 바뀐 request bytes, target 안 trust root.
