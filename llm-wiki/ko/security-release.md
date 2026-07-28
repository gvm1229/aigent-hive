---
schema_version: 1
pair_id: security-release
topic_slug: security-release
language: ko
counterpart: ../en/security-release.md
title: "Security와 Release Trust"
summary: "Preview provenance, verifier-only hardened trust와 protected release publication."
tags: [release, security, trust]
aliases: ["릴리스 신뢰"]
sources:
  - "repo:docs/architecture/release-update-trust-boundary.md#sha256:3f90ab4526cd1ca556a057af529fada637a2a476374927408098e759ae6deb8f"
  - "repo:docs/decisions/ADR-0007-ed25519-judge-trust.md#sha256:5a17bcd8f6869437a9e37c025c4fa2da285ff03af16a7144552162bac5a09a1a"
  - "repo:docs/decisions/ADR-0008-verifier-only-tuf-updates.md#sha256:97989993dba9959f24117f0e4917954a3e67b215cfe659942172e9f22c6ff709"
  - "repo:docs/decisions/ADR-0013-preview-release-scope.md#sha256:eb5f53e2cc1168888bb5117fdd91ede7016312ee79f8894581017e2e1b1976c5"
links: [boundaries, product-intent, upgrade, usage-hosts]
reviewed_revision: "git:d46e9b7deb5c54fc7cec00c38483388ce563ff1d"
status: active
---

# Security와 Release Trust

`0.8.0` 공개 label: `Claude-unverified preview`. Codex·Antigravity 실제 host evidence 확보.
Claude evidence: package·fixture·projection conformance, subscription-backed session 없음.
실제 Windows 기기 acceptance는 release gate 유지.

Preview trust baseline: protected exact tag, release asset SHA-256, GitHub artifact attestation,
source provenance, package-manager 또는 digest 고정 수동 update. Network self-update 비활성.
Developer ID·notarization, Authenticode와 external TUF 2-of-3 authorization은 deferred.

Hardened trust boundary의 verifier-only 원칙 유지. Private signing material의 생성·조회·
저장·사용 금지. Existing TUF-compatible verifier, compiled-only migration, rollback protection,
external signer design은 future hardened update channel용으로 보존. Candidate creation과
public publication은 분리된 protected workflow.
