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
  - "repo:docs/decisions/ADR-0013-preview-release-scope.md#sha256:1ba89150ac521f638f686cd4fa9ff6d8cf256a0e561c0c3b80616fd1a3989f2f"
  - "repo:docs/guides/signed-update-and-release.md#sha256:e457b425ef3f8bf88599ad1ba576a9ab1a27d60c4ade9115665557586d6cf8e8"
  - "repo:docs/plans/active/windows-shell-install.md#sha256:0c79a70672c69438c85c9b3f9406036f6d90d616959e5030179147737eaed0f7"
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

Windows consumer 경계의 PowerShell 7 dependency 없음. `hive.exe`와 installed harness의
PowerShell 7 탐지·설치 제안·설치·update·uninstall 없음. Direct installer는 Windows
기본 PowerShell 5.1 지원. `cmd.exe`는 동일 exact-version bootstrap 경로 호출.

PowerShell 7.6.4 LTS는 source development·release dependency 전용. Optional source
helper는 exact WinGet command·package·version·user scope를 preview하고 explicit consent
전 mutation 없음. Microsoft package에 설치를 위임한 뒤 결과 재검증. PowerShell
설치·update·제거 ownership은 Microsoft 또는 선택한 package manager에 유지.
