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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:c313a53d8ed114aaf9b6303263730d282b11c6d8d52a71c249999b62969214fe"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a56419242874c459f08f7575ec0b2b6c2249ac696e0efffb053706dfeb6c9f00"
links: [consumer-session-coordination, graphify-0-10-adoption, hybrid-vector-search-0-10, knowledge-storage, nested-project-scan-0-10]
reviewed_revision: "git:a0f288b6b962cd5bede27065fa39f708764a621f"
status: active
---

# Aigent Hive `0.10.0` 제품 범위

- 검색: Native Markdown graph, 선택형 Graphify code 추출, FTS, 조건부 vector gate
- 실행: Host-neutral continuation, `verified-workflow`, 명시적 `adversarial-judge`
- Upgrade: Authenticated retired Skill·projection `0건`; modified·foreign bytes는 activation conflict
- 보존: Canonical Markdown·SQLite direct 검색·nested project byte
- 출시: 세 OS 공개 시험과 유지보수자 명시 승인
