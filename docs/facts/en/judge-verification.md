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
  - "repo:docs/decisions/ADR-0019-hive-native-iterative-execution.md#sha256:8dcf64600bf77f630d6f601027ee02a5adf1255a49c4c852ff6006a46f203817"
  - "repo:docs/plans/active/model-routed-custom-subagents.md#sha256:6b2eb3faafe345678008fe225dd941026c8eab10911fa19530c2785c8b644f57"
links: [model-routed-custom-subagents, orchestration-ownership, product-non-goals, release-verification]
reviewed_revision: "git:ffdfb476d4e21dafe5d4dc896fa272f7244d0fe1"
status: active
---

# Judge Verification Boundary

Current Hive packages bounded evidence and verifies externally signed assignments,
verdicts, and critical human approvals; it never owns private keys or signs artifacts.
The 0.9.0 plan adds a fresh, read-only, user-scope custom Judge using Sol High on
Codex with exact model and effort receipts. `explicit` limits use to strict workflow terminal gates;
`implicit` adds material-risk tasks. Strict gates always require Judge quorum but
never dispatch per scheduler tick. The host runs the agent, an external signer signs,
and Hive validates the digest-bound quorum.
