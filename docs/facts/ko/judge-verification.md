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
  - "repo:docs/archive/plans/foundations/model-routed-custom-subagents.md#sha256:9fe4b79c4f4e0be1706600e06b74ab93ee8bbce01e767a38790bbf8bdd21b251"
  - "repo:docs/decisions/ADR-0007-ed25519-judge-trust.md#sha256:5a17bcd8f6869437a9e37c025c4fa2da285ff03af16a7144552162bac5a09a1a"
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:a15a00e40b63abb6aa312ed24ee3d80c491f0b79056fa628e165431858e51551"
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
