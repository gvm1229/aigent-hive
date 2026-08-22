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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:fe327177fca73ccbdb3267a1cfca7b579b984e8bd3a24e74457a7d062020f2ec"
  - "repo:docs/decisions/product-release-decisions.md#sha256:59e330c3bd0a5a8133e00c447c99db44e30274dbf92770b662d3cf4c14b50e0f"
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
