---
schema_version: 1
pair_id: release-verification
topic_slug: release-verification
language: en
counterpart: ../ko/release-verification.md
title: "Release Verification"
summary: "Hive verifies TUF-compatible metadata, provenance, signing evidence, and artifact bytes."
tags: [release, security, verification]
aliases: ["Verifier-only release"]
sources:
  - "repo:docs/decisions/ADR-0008-verifier-only-tuf-updates.md#sha256:97989993dba9959f24117f0e4917954a3e67b215cfe659942172e9f22c6ff709"
links: [judge-verification, update-transaction]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Release Verification

Hive verifies an external TUF-compatible Ed25519 trust chain, target length and
SHA-256, rollback floors, provenance, platform-signing evidence, and exact candidate
bytes. Private keys, signing, and publication authority remain external.
