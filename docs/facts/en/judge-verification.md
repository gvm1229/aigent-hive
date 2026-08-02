---
schema_version: 1
pair_id: judge-verification
topic_slug: judge-verification
language: en
counterpart: ../ko/judge-verification.md
title: "Judge Verification Boundary"
summary: "Hive verifies externally signed Judge artifacts; v0.9.0 plans a reserved exact-model Judge and invocation modes."
tags: [judge, security, verification]
aliases: ["Ed25519 judge quorum"]
sources:
  - "repo:docs/decisions/ADR-0007-ed25519-judge-trust.md#sha256:5a17bcd8f6869437a9e37c025c4fa2da285ff03af16a7144552162bac5a09a1a"
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:cc7e79da6c27052fb9dc256a47e057deab617bbb66a567da8455d9135d6407b8"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:a2779d3f1ebab829c48214fd4486f9505e7207b6de1cdfe3c6af56c9121534ce"
links: [model-routed-custom-subagents, orchestration-ownership, product-non-goals, release-verification]
reviewed_revision: "git:4e750ce659c953d7d71ab6e9536c29968ab1f028"
status: active
---

# Judge Verification Boundary

Current Hive packages bounded evidence and verifies externally signed assignments,
verdicts, and critical human approvals; it never owns private keys or signs artifacts.
The 0.9.0 plan adds a fresh, read-only, user-scope custom Judge with exact model and
effort receipts. `explicit` limits automatic use to strict workflow terminal gates;
`implicit` adds material-risk tasks. Strict gates always require Judge quorum but
never dispatch per scheduler tick. The host runs the agent, an external signer signs,
and Hive validates the digest-bound quorum.
