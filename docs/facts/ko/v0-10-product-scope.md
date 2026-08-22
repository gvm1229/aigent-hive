---
schema_version: 1
pair_id: v0-10-product-scope
topic_slug: v0-10-product-scope
language: ko
counterpart: ../en/v0-10-product-scope.md
title: "Aigent Hive 0.10.0 제품 범위"
summary: "관계 검색·조건부 hybrid vector gate·nested project scan·host-owned Skill 예약·무손실 upgrade·출시 수용을 결합한 0.10.0 범위."
tags: [knowledge, release, scan, v0-10]
aliases: ["0.10.0 scope"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:1645eb2249265b75d27b0c65a709806f4999a0ec425e8e874336bcda084b702c"
  - "repo:docs/decisions/product-release-decisions.md#sha256:25bd2880270b2dd21bf09d5efe576f4164b8d02fadd8366f8649d8d50d38bded"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:a0f288b6b962cd5bede27065fa39f708764a621f"
status: active
---

# Aigent Hive `0.10.0` 제품 범위

- 관계 검색: Hive-native Markdown graph, 선택형 Graphify full-rebuild code 추출, FTS routing
- Semantic 검색: Hard gate 통과 시 optional local vector adapter, 실패 시 dependency `0건`
- 지식 운영: Metadata-first 조회, 생명주기·비용·drift evidence, scope 격리
- Project 안전: nested scan, host-owned Skill 예약, canonical byte 보존 upgrade
- 출시: Windows·macOS·Linux 공개 시험과 유지보수자 명시 승인 뒤 same-byte stable
- 추가 후보: 조건부 vector feasibility·implementation gate
- 실행 workflow: 자연어 continuation의 `verified-workflow` 자동 routing과 명시적 `adversarial-judge`
