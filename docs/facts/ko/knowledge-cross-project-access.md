---
schema_version: 1
pair_id: knowledge-cross-project-access
topic_slug: knowledge-cross-project-access
language: ko
counterpart: ../en/knowledge-cross-project-access.md
title: "명시 프로젝트 간 지식 접근"
summary: "자동 조회 격리, 명시 collection 직접 조회, 검토된 일반 지식의 수입 시 자동 승격"
tags: [collection, knowledge, promotion, retrieval, v0-9-3]
aliases: ["자동 지식 승격", "프로젝트 간 지식"]
sources:
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:cb4e53566ee902c209f8e0593929b05b8c5d22b87128736c638f4aa57d7b0f21"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:a0cf1b55b6cfc9d02920f117d25fb752e25267093d1be960b33267b8f6b0a94c"
  - "repo:crates/hive-wiki/src/rag.rs#sha256:631d823968b135efa90bb0515185fed724c1c1522af3623aa1a26e7c9842d1a7"
  - "repo:crates/hive-wiki/src/store.rs#sha256:812c302031f36694ad109479698f9d89b125392dba55468c0131e52d45f47b44"
  - "repo:crates/hive-wiki/src/store/freshness.rs#sha256:aaea806d5543632cf6ebf6c9faee349e6d67a97f9b8c8bd753bd0da9e02582b7"
  - "repo:harness/skills/knowledge-capture/references/scan.md#sha256:83063c0c0a97e81bd71170c1325df6a118d21e0659a806eecd6723912c1b26f0"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:d80f5871428941be1d05b12d488857f40bb894752c0c4354cee12f15a46578f6"
links: [global-knowledge-rag, knowledge-portability-scan, shared-index]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# 명시 프로젝트 간 지식 접근

Project A의 자동 조회 범위: A·`user-root`·검증된 shared 지식. Project B의 비공유 지식 혼입 없음.

사용자가 Project B 또는 unique collection alias를 명시한 경우: Hive의 fail-closed 해석 뒤 B
collection 직접 조회. 결과 범위: A·`user-root`·무관한 shared collection 제외. Confidential 지식:
exact query authorization 유지.

명시 applicability를 갖춘 검토된 safe-general decision·convention·workflow: scan apply와 rescan
maintenance 중 자동 승격. source provenance·promotion status 보존, source evidence 변경 시 파생
shared claim 무효화. 조회 시점 승격 0건.
