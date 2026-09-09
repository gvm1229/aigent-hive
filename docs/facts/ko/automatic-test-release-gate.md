---
schema_version: 1
pair_id: automatic-test-release-gate
topic_slug: automatic-test-release-gate
language: ko
counterpart: ../en/automatic-test-release-gate.md
title: "번호 시험판 자동 게시 gate"
summary: "완료된 승인 제품 milestone의 시험판 자동 게시·수용, source-only 변경·동일 제품의 후보 생성 차단"
tags: [automation, product, release]
aliases: ["번호 공개 시험 gate"]
sources:
  - "repo:.agents/directives/03-workflow.md#sha256:8d3afcb2e885232dcb7e7775d55d0b48477358ddbc0266ff4a48de80af34e9fc"
  - "repo:.github/workflows/release.yml#sha256:0b800d9f74b331f34aad1507b57129fb319fdf49934815026c6352c6aa91a5d7"
  - "repo:docs/public-test-product.json#sha256:a6935695f09d44816e802151166d22a89b606e3cdc2c5f81693ea5c49a9d7025"
  - "repo:scripts/check-test-release-gate.py#sha256:d0cc06433201fadb70cc765be20e5083de26e44c3153cf6fdefc7098c10f5ced"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:97928e522edbad00c2fc5c137f246c15fcad06a5"
status: active
---

# 번호 시험판 자동 게시 gate

번호 시험판의 별도 승인 질문 없음. milestone 완료 때 에이전트가 `docs/test-release-intent.json`에 다음 번호·완료 plan ID·제품 지문 기록.
`check-test-release-gate.py`에서 해당 의도·마지막 수용 제품·후보 비교. 새 제품 byte만 후보·게시·공개 수용 자동 진행.
동일 제품과 문서·계획·사실·source-only Skill·지침·시험·CI·안내 변경은 후보 생성 거부.
안정판 명시 승인 유지.
