---
schema_version: 1
pair_id: source-graph-public-qualification
topic_slug: source-graph-public-qualification
language: ko
counterpart: ../en/source-graph-public-qualification.md
title: "Source graph 공개 자격 검증"
summary: "0.10.0 source graph가 Source Wiki FTS와 근거 있는 Markdown edge를 결합하고 출시 후보마다 직접 사실 30개·관계 질문 30개를 검증함."
tags: [graph, knowledge, qualification, v0-10]
aliases: ["source graph 수용", "source 관계 자격 검증"]
sources:
  - "repo:.github/workflows/release.yml#sha256:98fc01b94dd0cc9c5fa839c4fc68a32c8398fd6a624e065c1a1631001173e777"
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:ee2e1628368de52fe46f08547c2866bcd271fac881b61127e0e00db67b297c1e"
  - "repo:crates/hive-wiki/src/source.rs#sha256:2fcd76ad7d212a94ede2390b2b0532f4d8fba6ac053256c8a21c86256bd95dd6"
  - "repo:scripts/qualify-source-graph.py#sha256:20181b16b7dee6e78992cab8c2e3411614153a6e7345ab3e928a7ce61e6a4478"
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
