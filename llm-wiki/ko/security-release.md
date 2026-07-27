---
schema_version: 1
pair_id: security-release
topic_slug: security-release
language: ko
counterpart: ../en/security-release.md
title: "Security와 Release Trust"
summary: "Verifier-only trust root, authenticated judge evidence와 protected release publication."
tags: [release, security, trust]
aliases: ["릴리스 신뢰"]
sources:
  - "repo:docs/architecture/release-update-trust-boundary.md#sha256:3f90ab4526cd1ca556a057af529fada637a2a476374927408098e759ae6deb8f"
  - "repo:docs/decisions/ADR-0007-ed25519-judge-trust.md#sha256:5a17bcd8f6869437a9e37c025c4fa2da285ff03af16a7144552162bac5a09a1a"
  - "repo:docs/decisions/ADR-0008-verifier-only-tuf-updates.md#sha256:97989993dba9959f24117f0e4917954a3e67b215cfe659942172e9f22c6ff709"
links: [boundaries, upgrade, usage-hosts]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Security와 Release Trust

Hive trust boundary: verifier-only. Private signing material의 생성·조회·저장·사용 금지.
Judge와 human authority의 근거: external agent-write-denied Ed25519 public trust root와
canonical digest-bound detached attestation.

Release authorization: threshold role, expiry, snapshot consistency, rollback protection, root
rotation, exact target length와 SHA-256 digest를 포함한 TUF-compatible Ed25519 metadata chain.
Running binary는 compiled migration route만 허용. Signed release를 통한 shell, dynamic library,
WebAssembly 또는 기타 executable migration code 전달 금지.

Candidate 생성과 public publication은 분리된 protected workflow. Private credential은 external
signer 소유. Publication 전 TUF, source commit, provenance, platform signing evidence, offline
attestation bundle과 exact candidate bytes의 독립 검증.
