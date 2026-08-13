---
schema_version: 1
pair_id: judge-verification
topic_slug: judge-verification
language: en
counterpart: ../ko/judge-verification.md
title: "Judge Verification Boundary"
summary: "Hive re-verifies an externally signed Ed25519 Judge quorum before accepting loop terminal evidence."
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

# Judge Verification Boundary

Hive packages bounded evidence and verifies externally signed assignments, verdicts,
and critical human approvals; it never owns private keys or signs artifacts. A loop
Judge verifier must bind a v2 quorum request digest, an external protected trust root,
and its exact run, revision, node, attempt, and evidence ID. Hive re-evaluates the
Ed25519 quorum and accepts the evidence only for an authenticated PASS with the same
subject. A bare authentication flag, an unsigned request, changed request bytes, or a
target-contained trust root cannot authorize completion.
