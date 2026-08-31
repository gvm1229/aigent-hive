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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:586e27426f1c48ebc8ad92754d478b731d9b07bbba01e61a34e9f0469c43c031"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:d84549268a83748e23da88c1e9c1d51163776e9511b258feb2b79c3318239e09"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:d331dc879cf51eab078c5e189b2fe7b8d729e541"
status: active
---

# Aigent Hive `0.10.0` 제품 범위

범위: 관계 검색·무손실 갱신·호스트 중립 연속 실행·자동 한국어 처리·선택형 벡터 검색.
비벡터 수정은 `develop` 통합과 test.4 세 운영체제 수용 완료. 전용 브랜치의 벡터 기능 구현·검증 완료.
2026-08-29 스트레스 성능 수용과 새 기준으로 채택 승인. 과거 실패 기록 보존.
벡터 포함 test.6 공개·세 운영체제 설치 수용 완료. 안정판 통합·게시·설치는 버전별 명시 승인 전 금지.
