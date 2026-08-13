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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:c61df64b6ae0367c4e146346472a108d22576e6f8eee07e8581b29d7ffe25784"
  - "repo:crates/hive-wiki/src/rag.rs#sha256:15a09e0b770055a0cdab1191048d53c0323a892e2ba8eb374d4bf30cb5491c13"
  - "repo:crates/hive-wiki/src/store.rs#sha256:d2933a18fede81c05988727a07f5b9de019d4f488cce4d0d8876113d2be1d4b2"
  - "repo:harness/skills/knowledge-import/SKILL.md#sha256:c20be7748412c966c9fe87d6a97281ac7eb00381607b4443d2cfe555c07e01f3"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:7b5d334b67e9db1b981273f9fb134adca8129250f241d7359f6f1bc5bda88c1e"
links: [global-knowledge-rag, knowledge-portability-scan, shared-index]
reviewed_revision: "git:6d5798e1a4ed03a79f0d97ed596d3229121af5e8"
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
