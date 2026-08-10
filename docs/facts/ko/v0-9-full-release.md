---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "test.15 수용 유지. Stable source의 무료 platform evidence·external TUF 승인 요청·production 검증 구현 완료, protected main 통합·외부 authorization 대기."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:505cc48a16b2ccc7ca7fe39fdaf47d7b851a19810cb75c784fdfe5a6717c5823"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:2691a98d452eac2b566e97dcd34982c7ef283bf14b01cd8b76508e1c82782403"
  - "repo:docs/guides/signed-update-and-release.md#sha256:aa570e405dc1e568a79fe6291e30807db9e96b7805e570aede152fed4120f5a5"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:01ba525115912b67409e88d9ca73c4021b908f94562df3821dd2bf6a3aeb4d76"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:daa4ab56b05f7403bc1f5f2b44d8471fb99866af"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

Stable `v0.9.0`: 미게시. 수용된 `test.15`: `latest=0.8.0` 유지, Windows 보존형 재설치·develop CI PASS.
Stable source: macOS ad-hoc·Windows unsigned evidence, deterministic TUF 요청, safe extraction,
protected rollback floor, production verifier·exact target byte 결합. 유료 platform certificate: gate 제외.
남은 경계: protected `main`, stable candidate, external 2-of-3 authorization, publication approval.
