---
schema_version: 1
pair_id: v0-10-product-scope
topic_slug: v0-10-product-scope
language: ko
counterpart: ../en/v0-10-product-scope.md
title: "Aigent Hive 0.10.0 제품 범위"
summary: "추가 Backlog 승격 없이 관계 검색·nested project scan·host-owned Skill 예약·무손실 upgrade·출시 수용을 결합한 최종 0.10.0 범위."
tags: [knowledge, release, scan, v0-10]
aliases: ["0.10.0 scope"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:9d595ccdbc341b4a532bf6266106ca890edc983ce96b69316c1e6731764987a7"
  - "repo:docs/decisions/product-release-decisions.md#sha256:b83e0f586453e6aeb0356f5a4eee84591b2946ccb06ecbffde35131027878de6"
links: [consumer-session-coordination, graphify-0-10-adoption, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:8a99ccacb96a249692d01e6835efc872adb7fe95"
status: active
---

# Aigent Hive `0.10.0` 제품 범위

- 관계 검색: Hive-native Markdown graph, 선택형 Graphify full-rebuild code 추출, FTS routing
- 지식 운영: Metadata-first 조회, 생명주기·비용·drift evidence, scope 격리
- Project 안전: nested scan, host-owned Skill 예약, canonical byte 보존 upgrade
- 출시: Windows·macOS·Linux 공개 시험과 same-byte stable
- 추가 Backlog·Archive 승격: 없음
