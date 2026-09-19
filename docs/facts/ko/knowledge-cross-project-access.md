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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:0ffab09ca1eaac47b41608e04003fdbc51ee7e5edbaf2386554c885d9f55ec58"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:78a309398bc924b72c98eb522ac4ef7d944f172f769eab72d8b8916444c257cc"
  - "repo:crates/hive-wiki/src/rag.rs#sha256:2bb27720c34a60bfd3b0003e27348288f3f17062ab1e270f3c2d624487e1eff4"
  - "repo:crates/hive-wiki/src/store.rs#sha256:a804c6475388324fe8e785f4afdb852a3318cfc4ba2e2fba1c9ae8eb313d2e20"
  - "repo:crates/hive-wiki/src/store/freshness.rs#sha256:6eed0fc1dfcee4d063ba6e3877e0ae2043dc36b8abad8b358551ad1515ddc57e"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:7ca8ab883c9dc93c06e585f75be3a4ff0dd2ee0dc6c9adcbf68cf5798a43a416"
  - "repo:harness/skills/knowledge-scan/SKILL.md#sha256:b8c3928df97c6f5e84f60b5a20ed9944c3ccd785cc408ffa5aa4a1db4d4b2aef"
links: [global-knowledge-rag, knowledge-portability-scan, shared-index]
reviewed_revision: "git:dd63333a702a7a89585d101d2b9d043ebd0987d8"
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
