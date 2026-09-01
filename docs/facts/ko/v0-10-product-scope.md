---
schema_version: 1
pair_id: v0-10-product-scope
topic_slug: v0-10-product-scope
language: ko
counterpart: ../en/v0-10-product-scope.md
title: "Aigent Hive 0.10.0 제품 범위"
summary: "한국어 언어 core·선택형 벡터 검색·관계 검색·무손실 지식 이전·검증형 연속 실행을 포함한 0.10.0 정식 출시"
tags: [knowledge, language, release, scan, v0-10]
aliases: ["0.10.0 scope"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:5327d6c3417a62069df8eda30e76fe907c48418806023847eb16189cbe3041ef"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:d84549268a83748e23da88c1e9c1d51163776e9511b258feb2b79c3318239e09"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:2b8007e0cbf5a0f89ebb654ee7f6b44a1b203eee905205fe7ea90629941e4cad"
  - "repo:docs/public-stable-release.json#sha256:3828fade92ec45cdc0eab834aaf8029d95f2619ebc87e034172898371e65668e"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:301147fab8252954b29b7393327dfcff18eb8b1d"
status: active
---

# Aigent Hive `0.10.0` 제품 범위

`0.10.0`: 2026-09-02 정식 출시. 관계 검색·무손실 지식 이전·호스트 중립 검증형 연속 실행·자동 한국어 처리·선택형 벡터 검색 포함.
Windows x64·macOS arm64·Linux musl의 공개 설치 수용과 승인된 main 소스의 정식 후보·게시 성공.
벡터 검색의 스트레스 기준과 한계 기록 유지. SQLite 전문 검색도 계속 사용 가능.
