---
schema_version: 1
pair_id: security-release
topic_slug: security-release
language: ko
counterpart: ../en/security-release.md
title: "Security와 Release Trust"
summary: "Pre-1.0 provenance, verifier-only hardened trust와 protected release publication."
tags: [release, security, trust]
aliases: ["릴리스 신뢰"]
sources:
  - "repo:docs/architecture/release-update-trust-boundary.md#sha256:3f90ab4526cd1ca556a057af529fada637a2a476374927408098e759ae6deb8f"
  - "repo:docs/decisions/ADR-0007-ed25519-judge-trust.md#sha256:5a17bcd8f6869437a9e37c025c4fa2da285ff03af16a7144552162bac5a09a1a"
  - "repo:docs/decisions/ADR-0008-verifier-only-tuf-updates.md#sha256:97989993dba9959f24117f0e4917954a3e67b215cfe659942172e9f22c6ff709"
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:3b5b29532bd3e353aaaea9b95637a6582bf6c6ab5dab01ebfc61bc7967ecd613"
  - "repo:docs/guides/signed-update-and-release.md#sha256:e457b425ef3f8bf88599ad1ba576a9ab1a27d60c4ade9115665557586d6cf8e8"
  - "repo:docs/plans/active/release-0.8.0.md#sha256:e6ae8f6c48b5018960533264670696f22a17b0a786c9b4b8087bbc11fe0f515d"
  - "repo:docs/plans/active/windows-shell-install.md#sha256:d6a7e05eae8d1a4328a9fd58414b087442d32a7423ff92cb9e0d158bbb4ae179"
  - "repo:docs/state/CURRENT.md#sha256:de64dfc9f37a949e805fae83aaab84878e61565ba0b0c7b67887dc2f67cb5eaf"
links: [boundaries, product-intent, upgrade, usage-hosts]
reviewed_revision: "git:51f40e24316e9f776626ddf73676f7719b020a42"
status: active
---

# Security와 Release Trust

공개 identity: `Aigent Hive 0.8.0`. Preview label·GitHub prerelease·npm preview dist-tag 없음.
Pre-1.0 SemVer를 성숙도 신호로 사용. Codex·Antigravity 실제 host evidence 확보.
Claude evidence: package·fixture·projection conformance, subscription-backed session 없음.

Current source `9fb2552`: clean-clone CI 7/7 PASS. `d39ce7f` historical native runtime:
macOS arm64·Intel, Windows x86_64. Linux musl x86_64·arm64 release qualification 미완료.

Release baseline: protected exact tag, 5개 native archive, SHA-256, GitHub artifact attestation,
source provenance, GitHub·npm binary byte identity. 주 설치 명령:
`npm install -g aigent-hive`. Unix·PowerShell·CMD 검증 installer는 Node-free 병렬 channel.
Network self-update 비활성.

Hardened trust boundary의 verifier-only 원칙 유지. Private signing material의 생성·조회·
저장·사용 금지. Existing TUF-compatible verifier, compiled-only migration, rollback protection,
external signer design은 future hardened update channel용으로 보존. Candidate creation과
public publication은 분리된 protected workflow.

`0.8.0` workflow 요구: 5개 target build, npm platform package staging, artifact attestation,
platform package 선행·`aigent-hive` umbrella 후행 publication. GitHub normal release와
npm `latest` 사용. Developer ID·notarization·Authenticode·Azure signing·external TUF는
secret 부재 시 필수 조건이 아닌 후속 opt-in hardening. Public GitHub·npm publication은
최종 사용자 확인과 registry ownership 필수.

Windows consumer 경계의 PowerShell 7 dependency 없음. `hive.exe`와 installed harness의
PowerShell 7 탐지·설치 제안·설치·update·uninstall 없음. Direct installer는 Windows
기본 PowerShell 5.1 지원. `cmd.exe`는 동일 exact-version bootstrap 경로 호출.

PowerShell 7.6.4 LTS는 source development·release dependency 전용. Optional source
helper는 exact WinGet command·package·version·user scope를 preview하고 explicit consent
전 mutation 없음. Microsoft package에 설치를 위임한 뒤 결과 재검증. PowerShell
설치·update·제거 ownership은 Microsoft 또는 선택한 package manager에 유지.
