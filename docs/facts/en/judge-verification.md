---
schema_version: 1
pair_id: judge-verification
topic_slug: judge-verification
language: en
counterpart: ../ko/judge-verification.md
title: "Judge Verification Boundary"
summary: "Hive verifies clean-context judge artifacts but never runs or signs for a judge."
tags: [judge, security, verification]
aliases: ["Ed25519 judge quorum"]
sources:
  - "repo:docs/decisions/ADR-0007-ed25519-judge-trust.md#sha256:5a17bcd8f6869437a9e37c025c4fa2da285ff03af16a7144552162bac5a09a1a"
links: [product-non-goals, release-verification]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Judge Verification Boundary

Hive packages bounded clean-context evidence and verifies externally signed
assignments, verdicts, and critical human approvals. It does not generate private
keys, sign artifacts, run a judge, or claim that a valid signature proves judgment truth.
