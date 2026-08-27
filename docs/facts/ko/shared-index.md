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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:88e9456a85123bffb76f1574385b556b9e6b6d9d008862f1c0700daba343ec49"
  - "repo:crates/hive-wiki/src/lib.rs#sha256:3267b6cc3e81bf9af57c5ae267fba3f55cfc21facac25534d8c65c31920c9082"
  - "repo:crates/hive-wiki/src/store.rs#sha256:c8ed85d8dfdbe8d2215cc61f31643d1877f18b035f956146f9ae789de25b200a"
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
