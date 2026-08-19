---
schema_version: 1
pair_id: docs-wiki-architecture
topic_slug: docs-wiki-architecture
language: ko
counterpart: ../en/docs-wiki-architecture.md
title: "`docs/` Wiki 구조"
summary: "현재 문서·미래 backlog·동결 기록의 수명주기 분리."
tags: [documentation, wiki]
aliases: ["Source docs Wiki"]
sources:
  - "repo:docs/archive/README.md#sha256:4fa687e5a3603890bca9e557df8ad8e80de9f87eafa76d18e7bdf11c827eef6f"
  - "repo:docs/decisions/ADR-0011-source-wiki-independence.md#sha256:c9e698b54b31db5561a9b3611164ebc2d851bd7fa92087161864ea2092801b93"
  - "repo:docs/decisions/ADR-0014-docs-wiki-architecture.md#sha256:ec0fe3e284ab7ea2effc9330f4a82918bc643c58aaae88866fc8af28e2be477f"
  - "repo:docs/plans/README.md#sha256:85944730779c8686d4f436fe735f8e65b0ee34f8e5dee048103a8e85cd3f508a"
links: [knowledge-preservation, knowledge-storage]
reviewed_revision: "git:41f05a55741e319594e5f7ffe811e0e623ade499"
status: active
---

# `docs/` Wiki 구조

단일 graph: `docs/` home, 전체 index, topic MOC, 사람이 읽는 architecture·guide,
`docs/facts/`의 bilingual atomic fact.
이전 standalone source Wiki layout·명칭: tracked source에서 제거. Current CLI·Skill·시험·
index: `docs/facts/` 사용.
Source workspace 자동 조회: `hive-source.json` 확인 뒤 `hive source-wiki query` 사용.
Consumer `hive knowledge retrieve`: 등록된 외부 project 전용.
현재 작업 확인: `PLAN.md`·`CURRENT.md`·소유 active fragment만 사용.
버전 미정 후보: `docs/plans/backlog/`. 완료·대체 기록: `docs/archive/`.
Archive: 자동 작업 문맥과 현재 문서 검사에서 제외.
