---
schema_version: 1
pair_id: judge-verification
topic_slug: judge-verification
language: ko
counterpart: ../en/judge-verification.md
title: "Judge verification 경계"
summary: "Clean-context judge artifact 검증과 judge 실행·signing 비소유."
tags: [judge, security, verification]
aliases: ["Ed25519 judge quorum"]
sources:
  - "repo:docs/decisions/ADR-0007-ed25519-judge-trust.md#sha256:5a17bcd8f6869437a9e37c025c4fa2da285ff03af16a7144552162bac5a09a1a"
links: [product-non-goals, release-verification]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Judge verification 경계

Hive 범위: bounded clean-context evidence package, external signed assignment·verdict·
critical human approval 검증. 비소유: private key 생성, signing, judge 실행, 판단
진실성 보증.
