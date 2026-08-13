---
schema_version: 1
pair_id: knowledge-portability-scan
topic_slug: knowledge-portability-scan
language: ko
counterpart: ../en/knowledge-portability-scan.md
title: "Knowledge 이식과 directory scan"
summary: "v0.9의 checksummed canonical bundle, normalized collection, evidence-qualified scan·automatic query와 50,000 chunk 부하 검증."
tags: [collection, knowledge, portability, scan, v0-9]
aliases: ["Directory knowledge scan", "Hive knowledge bundle"]
sources:
  - "repo:crates/hive-wiki/tests/v09_bundle_qualification.rs#sha256:503ae836a837ec541a6217d8afbc9160c9031b21320409e85e961ff8ed1d7005"
  - "repo:crates/hive-wiki/tests/v09_rag_qualification.rs#sha256:5ab69e6461b2583aa2bdc095ffe71b05476cde24e92d9d76ba81bf0b40f68a4c"
  - "repo:docs/decisions/ADR-0016-global-knowledge-rag.md#sha256:196117cadc85737e0dbe35c8fcc6699e5180632d919782c2312453f588b3ab7a"
  - "repo:docs/plans/active/v0.9.0-knowledge-portability-scan.md#sha256:976150863fbb552b17b456b5bdaf4f6ce2780dcd7ed9af45ebcf565aae709e05"
  - "repo:docs/research/knowledge-portability-ingestion-retrieval.md#sha256:983844189f92fca165ed1c85eadf975dc404b46ddea4111ab956823448b15de6"
links: [global-knowledge-rag, knowledge-storage, shared-index, v0-9-skill-suite-plan]
reviewed_revision: "git:159e11c2f37f760b3e2bafedfb8b74cc735ff5f4"
status: active
---

# Knowledge 이식과 directory scan

Markdown mode v0.9: deterministic `.hivekb` bundle, normalized `collection_id`,
evidence-qualified scan, root promotion, bounded automatic query 지원. SQLite portable source
사용 0건, Notion mode export는 이 claim 밖.

부하 검증: 의도적으로 길게 만든 Wiki page 25개 × 검색 단위 chunk 2,000개 = 총 50,000 chunk,
portable collection registry 100개. 2026-08-13 배포용 build·local SSD p95 통과: fresh·warm
검색 `169.7758 ms`·`0.1343 ms`(기준 `500 ms`·`100 ms`), bundle export·import/rebuild
`1042.8145 ms`·`3267.0567 ms`(기준 `5 s`·`15 s`). Chunk: 전체 문서 아닌 검색용 지식 조각;
수치: hardware 독립 보장 아닌 부하 검증 근거.
