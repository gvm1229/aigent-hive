---
schema_version: 1
pair_id: shared-index
topic_slug: shared-index
language: ko
counterpart: ../en/shared-index.md
title: "User-root shared index"
summary: "Enabled global·project Markdown의 user-root SQLite projection 1개."
tags: [index, knowledge]
aliases: ["Shared knowledge index"]
sources:
  - "repo:crates/hive-wiki/src/lib.rs#sha256:292a7ce29540a77026fd99620aac10b35e85f51ee7490e003b19f789c6bf6fd4"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:823da60fecfbe3d02cd5025259184212aee703f6d1e184c4854f15683a769e91"
links: [knowledge-storage, project-onboarding]
reviewed_revision: "git:e72e2f95883ad4503335123d487405d064fb36ac"
status: active
---

# User-root shared index

Enabled user·project Markdown의 projection: user root 아래 disposable SQLite 1개.
Project별 canonical·derived database 생성 0개.
공유 정본 변경 전 persistent dirty marker 게시.
낙관적 snapshot 검증의 병렬 변경 감지 시 marker 정리 후 충돌 반환.
효과: 안전한 재시도 경로 유지.
수용 기준: 동일 Wiki page 병렬 ingest 회귀 시험 통과.
