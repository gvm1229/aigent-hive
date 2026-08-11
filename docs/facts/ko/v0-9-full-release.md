---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "기존 candidate는 과거 증거로만 유지. v0.9.0 게시는 knowledge autocapture 보정과 replacement stable candidate 필수."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:505cc48a16b2ccc7ca7fe39fdaf47d7b851a19810cb75c784fdfe5a6717c5823"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:d2b7908ed7cdbcba844c47f406d91eb74b5d9bceb5aae562908e14b101800927"
  - "repo:docs/guides/signed-update-and-release.md#sha256:41b38d004edd0a2305919b183b706d65705c3f0b8b3998ac63308f529ae7a549"
  - "repo:docs/plans/active/release-0.9.0-stable-publication.md#sha256:55725c877b0bbcd94e3197f86c38df08c99e3161faf72ec6c21f86e182e74cf4"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:c40863c90f3b8947dfe52bfe43ef1f52ae5f1ed72150f6fcc2921e10bcfaa39f"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:1b7ea99554fcf2e475cc77dcb1a3452a7805315f"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

Stable `v0.9.0`: 미게시. `test.16`: `latest=0.8.0` 유지. protected-main candidate `31482918509`:
mandatory knowledge autocapture 보정 전 historical qualification. Windows Codex fresh-session
write·recall, replacement stable candidate, 분리된 Mac 2-of-3 Ed25519 authorization·publication
approval: 출시 gate. `test.16` binary embedded date `2026-08-01`: historical 오류. 기존 byte 불변,
별도 테스트 배포 없이 다음 정상 배포에서 actual UTC date 입력·표시 검증 필요.
