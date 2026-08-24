---
schema_version: 1
pair_id: v0-10-product-scope
topic_slug: v0-10-product-scope
language: ko
counterpart: ../en/v0-10-product-scope.md
title: "Aigent Hive 0.10.0 제품 범위"
summary: "자동 한국어 언어 core와 안전한 embedding·격리·rollback·조건부 단일 engine 채택을 위한 vector 재검증을 추가한 0.10.0 범위"
tags: [knowledge, language, release, scan, v0-10]
aliases: ["0.10.0 scope"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:9870204c4032c4c43504b73d20689b2104eba5d8ff826b607016866fd22155b5"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:12de0afadc6995dc5ae6a0151791c461f6f59e3ae38e0fe6f3ca3eb13004f1a3"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:eaed3203ce3fea062acab325a9ce0892348aff02"
status: active
---

# Aigent Hive `0.10.0` 제품 범위

범위: 관계 검색·무손실 upgrade·host-neutral workflow·자동 한국어 core. Vector 재검증 판정:
고유 50,000개 10분 gate 실패 기반 `defer`, product dependency `0건`. 한국어 core·`humanize-kor`·
고정 pack·bounded host adapter·rollback 구현 완료. 안정판 승인 전 `0.10.0-test.2` 이상과 세
운영체제 수용 필요.
