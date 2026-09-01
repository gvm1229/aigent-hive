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
  - "repo:docs/archive/plans/foundations/model-routed-custom-subagents.md#sha256:9fe4b79c4f4e0be1706600e06b74ab93ee8bbce01e767a38790bbf8bdd21b251"
  - "repo:docs/decisions/ADR-0007-ed25519-judge-trust.md#sha256:5a17bcd8f6869437a9e37c025c4fa2da285ff03af16a7144552162bac5a09a1a"
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:3bdc09d7cc02ced6cf4eb2a4dc4fa5e734653581144d8ab4e57914e21f4bb612"
links: [model-routed-custom-subagents, orchestration-ownership, product-non-goals, release-verification]
reviewed_revision: "git:a06262284f558a9ba955c44167bdcc8577102c77"
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
