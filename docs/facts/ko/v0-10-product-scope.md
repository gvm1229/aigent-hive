---
schema_version: 1
pair_id: v0-10-product-scope
topic_slug: v0-10-product-scope
language: ko
counterpart: ../en/v0-10-product-scope.md
title: "Aigent Hive 0.10.0 제품 범위"
summary: "기존 graph·workflow·upgrade·출시 범위에 자동 한국어 언어 core·humanize-kor·검증된 im-not-ai upstream pack을 추가한 0.10.0 범위."
tags: [knowledge, language, release, scan, v0-10]
aliases: ["0.10.0 scope"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:ff4dfde9029c9024ab260f0366381e1a9bf1ce9d384a1db46b33d1cd842a5578"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a56419242874c459f08f7575ec0b2b6c2249ac696e0efffb053706dfeb6c9f00"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:12c7ebd3b248e881f8bf9b9cf6da969ef8db2998b096c8668fcc1995c1be39bf"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:6bb15c4376924d7e3fcbd389daa09550d6477596"
status: active
---

# Aigent Hive `0.10.0` 제품 범위

관계 검색·무손실 upgrade·host-neutral workflow에 고정된 `im-not-ai` source 기반 자동 한국어
언어 core를 추가합니다. 한국어 응답과 Hive 소유 문서는 Skill 호출 없이 같은 core를 사용하고,
기존 한국어 글의 명시적 윤문은 `humanize-kor`가 담당합니다. Upstream pack은 version·commit·digest·
license·staging·rollback을 고정하며 raw installer를 실행하지 않습니다. 시험판 뒤 제품 동작이
바뀌었으므로 안정판 전 `0.10.0-test.2` 이상이 필요합니다.
