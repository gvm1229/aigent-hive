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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:b88eaf08d187d6f83cfac8b9e3a186791f08b71d0d5287f5dafe4d2e7aaa8151"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a56419242874c459f08f7575ec0b2b6c2249ac696e0efffb053706dfeb6c9f00"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:12c7ebd3b248e881f8bf9b9cf6da969ef8db2998b096c8668fcc1995c1be39bf"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:e8bbe0529513df56e73f84cf5797bb334f4184ec"
status: active
---

# Aigent Hive `0.10.0` 제품 범위

관계 검색·무손실 upgrade·host-neutral workflow·자동 한국어 core에 vector 재검증을 추가합니다.
Vector는 반복 합성 자료와 unique chunk를 분리하고 재개 가능한 embedding·end-to-end 속도·scope
격리·원자 generation·rollback·세 운영체제 근거를 요구합니다. Engine 선결정은 없으며 모든 gate를
통과한 한 조합만 선택형으로 채택합니다. 새 한국어 core와 조건부 vector adapter는 안정판 전
`0.10.0-test.2` 이상에서 다시 수용해야 합니다.
