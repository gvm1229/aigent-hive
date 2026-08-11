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
  - "repo:docs/plans/active/release-0.9.0.md#sha256:0eb164fc7c9a028804b50c78f78cd8c673d6525817afffb6e1d202e531ff1445"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:1b7ea99554fcf2e475cc77dcb1a3452a7805315f"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

Stable `v0.9.0`: 미게시. 수용된 `test.15`: `latest=0.8.0` 유지. protected-main candidate
`31482918509`: 5개 native archive·npm 6개 package·direct installer·attestation·public authorization
request 통과. macOS ad-hoc·Windows unsigned evidence 허용, 유료 platform certificate gate 제외.
external 2-of-3 Ed25519 authorization: Windows 수용 호스트가 아닌 분리된 Mac. publication approval 대기.
해당 candidate는 mandatory knowledge autocapture 보정 전 생성되어 과거 qualification으로만 유지하며
게시할 수 없다. Windows Codex fresh-session write·recall 수용과 replacement stable candidate를
`v0.9.0` 추가 gate로 적용한다.
