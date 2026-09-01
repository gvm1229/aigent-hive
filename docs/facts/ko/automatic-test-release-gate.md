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
  - "repo:.agents/directives/03-workflow.md#sha256:9133a979df415b6df62b8669e3d0a1a6c069c9a441451948f16473bd5527878d"
  - "repo:.github/workflows/release.yml#sha256:b530af22eb2e6f932558e2f2699038d59c1bd8f2c48cedf37433417dac4a66bf"
  - "repo:docs/public-test-product.json#sha256:127030c1f2d45cce3fa84861eedcefdc6454fceaca888f51663cb19272d10721"
  - "repo:scripts/check-test-release-gate.py#sha256:06af753c2dc6a4568e5173676c455b3e618ab9daeea8aa91230e892613241c29"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:97928e522edbad00c2fc5c137f246c15fcad06a5"
status: active
---

# 번호 시험판 자동 게시 gate

번호 시험판의 별도 승인 질문 없음. milestone 완료 때 `check-test-release-gate.py`에서 마지막 수용 제품과 후보 비교,
완료된 비출시 구현 plan ID 요구. 새 제품 byte만 후보·게시·공개 수용 자동 진행.
동일 제품과 문서·계획·사실·source-only Skill·지침·시험·CI·안내 변경은 후보 생성 거부.
안정판 명시 승인 유지.
