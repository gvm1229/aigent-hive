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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:7d98098cff64b2ec197c3fa3f4f120399cf35dd9a578c3eb3aee05e224c43031"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3fbe246c3a5b7d2b8ec002d40f73874c056c48ae3a888dede3e40db12eddddac"
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
