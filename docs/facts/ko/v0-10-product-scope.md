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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:bfc5c5dd278e74d6e5e8a1260d3d5ac883e928d682e7773edcaa568e6c1fb78c"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:d84549268a83748e23da88c1e9c1d51163776e9511b258feb2b79c3318239e09"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:c0e9af4be59bf065286805e70316fe07e821fbe3"
status: active
---

# Aigent Hive `0.10.0` 제품 범위

범위: 관계 검색·무손실 갱신·호스트 중립 연속 실행·자동 한국어 처리·선택형 벡터 검색.
비벡터 수정은 `develop` 통합과 test.4 세 운영체제 수용 완료. 벡터는 전용 브랜치에서 구현 중.
이전 5만 청크 생성의 10분 기준 실패 기록 유지, 최종 구성의 품질·속도·격리 재검증 필수.
벡터 포함 다음 번호 시험판은 test.5. 안정판 통합·게시·설치는 버전별 명시 승인 전 금지.
