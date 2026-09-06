---
schema_version: 1
pair_id: vector-onboarding-0-10
topic_slug: vector-onboarding-0-10
language: ko
counterpart: ../en/vector-onboarding-0-10.md
title: "0.10.0 벡터 검색 최초 설정"
summary: "한 번의 벡터 검색 답변, 짧은 local claim, 고정 범위 새 세션 설정 안내문"
tags: [knowledge, onboarding, v0-10, vector]
aliases: ["벡터 최초 설정"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:5327d6c3417a62069df8eda30e76fe907c48418806023847eb16189cbe3041ef"
  - "repo:docs/guides/vector-search.md#sha256:ec476f82aa26bba2e8a1605af7620974b4620ee33d1f855c0d7669fa10d5df18"
  - "repo:docs/plans/active/vector-onboarding-0.10.0.md#sha256:96f2f93129dc8ac8d7a70789e940b57d217d360ae112ea1c91521097abf2b086"
links: [hive-preserving-uninstall, hybrid-vector-search-0-10, v0-10-product-scope]
reviewed_revision: "git:64a9f1929b96fcd3a274f2dd0e86b7d9e7c4399c"
status: active
---

# `0.10.0` 벡터 검색 최초 설정

사용자 답변과 설정·색인 상태 분리. `예`: 고정 범위 안내문 제공. `아니요`: 자동 질문 중지.
짧은 local claim으로 동시 중복 질문 방지, host session 식별자 저장 없음. 갱신·보존형 재설치 뒤 답변 유지.
답변만으로 정본 Markdown·FTS·기존 파생 벡터 자료 변경 없음.
