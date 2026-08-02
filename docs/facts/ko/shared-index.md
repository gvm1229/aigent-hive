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
  - "repo:crates/hive-wiki/src/lib.rs#sha256:e577da51c227170276e09c7961bb24cebb892091c3cb2e9c3be8e8f74b85ecbb"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:44401a82ba3bd9f2bc4048876f5480157720bc5fce005a7c0b63f4d960f63bf1"
links: [knowledge-storage, project-onboarding]
reviewed_revision: "git:5c734e139785b32062587308b41395981c0d209b"
status: active
---

# User-root shared index

Enabled user·project Markdown의 projection: user root 아래 disposable SQLite 1개.
Project별 canonical·derived database 생성 0개.
공유 정본 변경 전 persistent dirty marker 게시.
낙관적 snapshot 검증의 병렬 변경 감지 시 marker 정리 후 충돌 반환.
효과: 안전한 재시도 경로 유지.
수용 기준: 동일 Wiki page 병렬 ingest 회귀 시험 통과.
