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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:6b4d501127b0ba69d8d40d07cb57da9992ca34380d7c728f17e39deb9b6b11ee"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:35c9814239dde8505142300ca75f046ed17ac6c36e717a70bb2c358c312ee239"
  - "repo:crates/hive-wiki/src/rag.rs#sha256:daec035ff6b447bbcb64ca2e9dc9b8d97f286d0622bee15d324a681aea7f3bf8"
  - "repo:crates/hive-wiki/src/store.rs#sha256:2d8660c5455ba2a287a96e93a4817db413338a8bf46cfb6f034108ae23ab670b"
  - "repo:crates/hive-wiki/src/store/freshness.rs#sha256:aaea806d5543632cf6ebf6c9faee349e6d67a97f9b8c8bd753bd0da9e02582b7"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:2233fdf8eca297da93e9ae5785598ba1d96fcf84e39ebf44cc9cfa212a23e556"
  - "repo:harness/skills/knowledge-scan/SKILL.md#sha256:b8c3928df97c6f5e84f60b5a20ed9944c3ccd785cc408ffa5aa4a1db4d4b2aef"
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
