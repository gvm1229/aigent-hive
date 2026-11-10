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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:2786721545e082ed59be485a8ce0b21b7c9466a236c593488864725bcd8cc54f"
  - "repo:crates/hive-wiki/src/lib.rs#sha256:8ec6d8eabc5e53d6201c548320a6fa16fd86e219f0a73c357800f6d38d95eb0c"
  - "repo:crates/hive-wiki/src/store.rs#sha256:477bcfc850c27e54549428f527fbf5e91635b5aa6d9e25a0cd9c8a5ca1e5e5a4"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
links: [knowledge-storage, project-onboarding]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# User-root shared index

Enabled user·project Markdown의 projection: user root 아래 disposable SQLite 1개.
Project별 canonical·derived database 생성 0개.
공유 정본 변경은 준비부터 정본 쓰기와 SQLite 재구축이 끝날 때까지 사용자 루트의
작업 잠금을 유지. 별도 내부 게시 잠금은 이 작업 중에도 사용 가능. 따라서 다른
프로세스가 첫 작업의 dirty journal을 관찰하거나 대체하지 못함. 수용 기준: Windows
CI를 포함한 동일 Wiki page 병렬 ingest 및 병렬 추출·통합 회귀 시험 통과. 요청 배경:
다른 플랫폼 검사는 통과했지만 PR #18의 Windows Phase 1 적합성 검사에서 이 경쟁
상태 확인.
