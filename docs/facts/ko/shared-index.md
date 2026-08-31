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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:fdf0da018c63535231b564ed03ca7b31739f402c9640d59f9c2a76cc9eb00a04"
  - "repo:crates/hive-wiki/src/lib.rs#sha256:51423b7c92dca5ac711ffb9456861a283f1c70764954ee4ab0bc6cc69c471b66"
  - "repo:crates/hive-wiki/src/store.rs#sha256:6d6a377a6cd0c0c38ca48a85e89e871210ef4e87bbe05cf80c17713a566ae9a0"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
links: [knowledge-storage, project-onboarding]
reviewed_revision: "git:e5c2c599562121ed3dc43143c16a0b1f063cefa2"
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
