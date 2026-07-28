---
schema_version: 1
pair_id: security-release
topic_slug: security-release
language: en
counterpart: ../ko/security-release.md
title: "Security and Release Trust"
summary: "Preview provenance, verifier-only hardened trust, and protected release publication."
tags: [release, security, trust]
aliases: ["release trust"]
sources:
  - "repo:docs/architecture/release-update-trust-boundary.md#sha256:3f90ab4526cd1ca556a057af529fada637a2a476374927408098e759ae6deb8f"
  - "repo:docs/decisions/ADR-0007-ed25519-judge-trust.md#sha256:5a17bcd8f6869437a9e37c025c4fa2da285ff03af16a7144552162bac5a09a1a"
  - "repo:docs/decisions/ADR-0008-verifier-only-tuf-updates.md#sha256:97989993dba9959f24117f0e4917954a3e67b215cfe659942172e9f22c6ff709"
  - "repo:docs/decisions/ADR-0013-preview-release-scope.md#sha256:eb5f53e2cc1168888bb5117fdd91ede7016312ee79f8894581017e2e1b1976c5"
links: [boundaries, product-intent, upgrade, usage-hosts]
reviewed_revision: "git:d46e9b7deb5c54fc7cec00c38483388ce563ff1d"
status: active
---

# Security and Release Trust

The `0.8.0` public label is `Claude-unverified preview`. Codex and Antigravity have live host
evidence; Claude has package, fixture, and projection conformance without a subscription-backed
session. Real Windows-machine acceptance remains a release gate.

The preview trust baseline is a protected exact tag, release-asset SHA-256, GitHub artifact
attestation, source provenance, and package-manager or digest-pinned manual update. Network
self-update is disabled. Developer ID, notarization, Authenticode, and external TUF 2-of-3
authorization are deferred rather than falsely claimed.

Hive remains verifier-only at the hardened trust boundary. It never generates, reads, stores, or
uses private signing material. The existing TUF-compatible verifier, compiled-only migrations,
rollback protection, and external signer design remain available for a future hardened update
channel. Candidate creation and public publication stay separate protected workflows.
