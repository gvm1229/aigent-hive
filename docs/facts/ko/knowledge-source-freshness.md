---
schema_version: 1
pair_id: knowledge-source-freshness
topic_slug: knowledge-source-freshness
language: ko
counterpart: ../en/knowledge-source-freshness.md
title: "과거 지식과 현재 원본의 구분"
summary: "과거 기록의 무결성과 현재 원본의 일치 여부를 분리하는 조회 계약"
tags: [freshness, knowledge, portability]
aliases: []
sources:
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:a72952ecc2423f82bd6710e9d88a88cd6b59f0d87500b30c1a1d68151879bcf1"
  - "repo:crates/hive-wiki/src/store/freshness.rs#sha256:aaea806d5543632cf6ebf6c9faee349e6d67a97f9b8c8bd753bd0da9e02582b7"
  - "repo:docs/decisions/ADR-0023-foundation-refactor.md#sha256:ae98a6a0eac8e6ee53954df0f0162721c25fd4ebea8eff09240948e673c5746e"
  - "repo:schemas/knowledge-retrieval-result.schema.json#sha256:6d8499cf211d663d928c3fa1a31a0e375e6c970736414a1298f1b42180953ac9"
links: [foundation-refactor, knowledge-portability-scan]
reviewed_revision: "git:1cdebf8aef9dde963f5aa88a36042186fbf37e80"
status: active
---

# 과거 지식과 현재 원본의 구분

- 이식 뒤 원본 파일 부재: 정본 무결성을 확인한 과거 기록 조회 유지
- `source_freshness=verified-current`: 해당 조회 시점의 원본 내용 일치
- `historical-unverified`: 현재 코드 근거로 사용 금지, 원본 복원·재스캔·검토 경로 안내
- 정본 변조·잘못된 출처 연결·확인된 원본 변경은 계속 거부
- 일부 근거 부재로 다른 근거의 내용 변경을 숨기는 처리 금지
- 조회의 정본 수정·색인 복구 없음. 유지보수자가 선택한 `0.11.0` 이식 계약
