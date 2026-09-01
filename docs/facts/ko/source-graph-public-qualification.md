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
  - "repo:.github/workflows/release.yml#sha256:2f3760d989da12d1b07bfe706b9e7f1cd1e3121d3a53b18843e7825b56d86cac"
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:676c827ebe9ed1c4e51d73cf35289b8dfa9149f52287bcfad1a33d0b872b3be2"
  - "repo:crates/hive-wiki/src/source.rs#sha256:f9d3cae67f645e63b1483e83303fd0ecce41d50a9f66d3cd9540fc6e6f824f52"
  - "repo:scripts/qualify-source-graph.py#sha256:62e74cb2994404d7607f33a38da73b1973592609fc1b6af3686a7920c2086710"
links: [graphify-0-10-adoption, hybrid-vector-search-0-10]
reviewed_revision: "git:5b0c964f1ee65f4c21f301d51f85ce81e07b84e4"
status: active
---

# Source graph 공개 자격 검증

- 공개 명령: `hive source-wiki graph`
- 결합 방식: 영어 FTS 검색 결과에서 최대 50개의 `EXTRACTED` 관계 edge 반환
- 저장 경계: source 파생 graph·Graphify 동의는 `.agents/work` 아래에만 저장
- 출시 후보 검증: 직접 사실 30개와 관계 질문 30개
- 통과 기준: 직접 사실 Recall@10 100%, 근거 관계 Recall@10 90% 이상, cold CLI p95 2초 이하
- 보존 기준: canonical fact byte 변경·provider API·API key·query log 모두 0건
- 고정 질문: 같은 시점의 사실·인용 파일을 임시 복원하고 현재 실행 파일로 검사
- 현재 정본: 색인·관계 그래프·문서 검사 별도 실행. 두 사실 집합 변경·오류·경고 0건
