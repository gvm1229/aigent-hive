---
schema_version: 1
pair_id: release-verification
topic_slug: release-verification
language: ko
counterpart: ../en/release-verification.md
title: "Release verification"
summary: "TUF metadata·provenance·signing evidence·artifact byte 검증."
tags: [release, security, verification]
aliases: ["Verifier-only release"]
sources:
  - "repo:docs/decisions/ADR-0008-verifier-only-tuf-updates.md#sha256:97989993dba9959f24117f0e4917954a3e67b215cfe659942172e9f22c6ff709"
links: [judge-verification, update-transaction]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Release verification

Hive 검증 범위: external TUF-compatible Ed25519 trust chain, target length·SHA-256,
rollback floor, provenance, platform-signing evidence, exact candidate bytes.
Private key·signing·publication authority: external owner.
