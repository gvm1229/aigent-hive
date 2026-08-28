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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:1229cfa84e1fb0357c943fd0ef2910f3cdb5dd7e70f67879f0832db0ea26c800"
  - "repo:crates/hive-wiki/src/rag.rs#sha256:1f1d62c4fcb370f6792024d6a1caccc5c1a4dc16849c17df1df69aaed7effa98"
  - "repo:crates/hive-wiki/src/store.rs#sha256:1c31e8e950647b535fdd7118b2477b603cd2b2764ab37adb7d1c6e6917899989"
  - "repo:harness/skills/knowledge-import/SKILL.md#sha256:b73e6c82eb5ef9105781383f59211a982ed70b0f0ecf1690619f3b9b30f4730d"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:9e169f3daff2b4fbe6cff4d9a93d7e45cca6e9a6e78d1784b83458b50d3aa267"
links: [global-knowledge-rag, knowledge-portability-scan, shared-index]
reviewed_revision: "git:e5c2c599562121ed3dc43143c16a0b1f063cefa2"
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
