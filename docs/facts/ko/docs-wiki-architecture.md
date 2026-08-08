---
schema_version: 1
pair_id: docs-wiki-architecture
topic_slug: docs-wiki-architecture
language: ko
counterpart: ../en/docs-wiki-architecture.md
title: "`docs/` Wiki 구조"
summary: "사람용 topic 문서와 atomic fact의 단일 docs Wiki."
tags: [documentation, wiki]
aliases: ["Source docs Wiki"]
sources:
  - "repo:docs/decisions/ADR-0011-source-wiki-independence.md#sha256:82bc9dd03fc23f591540c8808cf5dba27e224dba3a41db2c81b2bafcb30f99fe"
  - "repo:docs/decisions/ADR-0014-docs-wiki-architecture.md#sha256:ec0fe3e284ab7ea2effc9330f4a82918bc643c58aaae88866fc8af28e2be477f"
links: [knowledge-preservation, knowledge-storage]
reviewed_revision: "git:ef4bc28e9bd13003d70072b968314558633bf31f"
status: active
---

# `docs/` Wiki 구조

단일 graph: `docs/` home, 전체 index, topic MOC, 사람이 읽는 architecture·guide,
`docs/facts/`의 bilingual atomic fact.
이전 standalone source Wiki layout·명칭: tracked source에서 제거. Current CLI·Skill·시험·
index: `docs/facts/` 사용.
Source workspace 자동 조회: `hive-source.json` 확인 뒤 `hive source-wiki query` 사용.
Consumer `hive knowledge retrieve`: 등록된 외부 project 전용.
