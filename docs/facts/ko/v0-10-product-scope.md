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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:08b5950d158c3e374752b625ba93a715d44c990d32c2ef39490bef2a1b9b084d"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:d84549268a83748e23da88c1e9c1d51163776e9511b258feb2b79c3318239e09"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:d9cb7733237df4e5cc14824cb2df13ff75009776"
status: active
---

# Aigent Hive `0.10.0` 제품 범위

범위: 관계 검색·무손실 갱신·호스트 중립 연속 실행·자동 한국어 처리·선택형 벡터 검색.
비벡터 수정은 `develop` 통합과 test.4 세 운영체제 수용 완료. 전용 브랜치의 벡터 기능 구현·검증 완료.
성능 기준 미달로 채택·공개 수용은 미완료, 과거 실패 기록 보존.
벡터 포함 다음 번호 시험판은 test.5. 안정판 통합·게시·설치는 버전별 명시 승인 전 금지.
