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
  - "repo:.agents/directives/03-workflow.md#sha256:629d32bb289108bbc782e295e4ffda6a4a4d5006fbf151212db0cc79457391f0"
  - "repo:.github/workflows/release.yml#sha256:2f3760d989da12d1b07bfe706b9e7f1cd1e3121d3a53b18843e7825b56d86cac"
  - "repo:docs/public-test-product.json#sha256:127030c1f2d45cce3fa84861eedcefdc6454fceaca888f51663cb19272d10721"
  - "repo:scripts/check-test-release-gate.py#sha256:669dd6cb700c9a169babf8ddf530c8ae4a7114a01096c6d1ae1d0cb63351c54d"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:97928e522edbad00c2fc5c137f246c15fcad06a5"
status: active
---

# 번호 시험판 자동 게시 gate

번호 시험판의 별도 승인 질문 없음. milestone 완료 때 에이전트가 `docs/test-release-intent.json`에 다음 번호·완료 plan ID·제품 지문 기록.
`check-test-release-gate.py`에서 해당 의도·마지막 수용 제품·후보 비교. 새 제품 byte만 후보·게시·공개 수용 자동 진행.
동일 제품과 문서·계획·사실·source-only Skill·지침·시험·CI·안내 변경은 후보 생성 거부.
안정판 명시 승인 유지.
