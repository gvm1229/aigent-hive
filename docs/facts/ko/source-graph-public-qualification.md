---
schema_version: 1
pair_id: source-graph-public-qualification
topic_slug: source-graph-public-qualification
language: ko
counterpart: ../en/source-graph-public-qualification.md
title: "Source graph 공개 자격 검증"
summary: "0.10.0 source graph의 Source Wiki FTS·근거 있는 Markdown edge 결합과 출시 후보별 직접 사실 30개·관계 질문 30개 검증."
tags: [graph, knowledge, qualification, v0-10]
aliases: ["source graph 수용", "source 관계 자격 검증"]
sources:
  - "repo:.github/workflows/release.yml#sha256:88394c81a55cb27a5fea46cc1adddd6877e0a3006c24567a1865e34b2bef26bb"
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:1229cfa84e1fb0357c943fd0ef2910f3cdb5dd7e70f67879f0832db0ea26c800"
  - "repo:crates/hive-wiki/src/source.rs#sha256:334881a8ed13d2e960d95d924c71391f029be131b67f8315c54a7385f1205a0f"
  - "repo:scripts/qualify-source-graph.py#sha256:10896324388562ef86b75c443f463c734d1323327a3631957c0aa86e88649a79"
links: [graphify-0-10-adoption, hybrid-vector-search-0-10]
reviewed_revision: "git:56db5d7f6b1fd49f4ed817617d2bc635fd0bbf63"
status: active
---

# Source graph 공개 자격 검증

- 공개 명령: `hive source-wiki graph`
- 결합 방식: 영어 FTS 검색 결과에서 최대 50개의 `EXTRACTED` 관계 edge 반환
- 저장 경계: source 파생 graph·Graphify 동의는 `.agents/work` 아래에만 저장
- 출시 후보 검증: 직접 사실 30개와 관계 질문 30개
- 통과 기준: 직접 사실 Recall@10 100%, 근거 관계 Recall@10 90% 이상, cold CLI p95 2초 이하
- 보존 기준: canonical fact byte 변경·provider API·API key·query log 모두 0건
