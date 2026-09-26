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
  - "repo:.agents/directives/references/ci-and-candidates.md#sha256:0de7ded9fe08a1e1e2ff78c4653c3781a8b492b59e6f9498ed07f89a5a56c18a"
  - "repo:.github/workflows/release.yml#sha256:993bf1709b27d5f6f5c18df46dab392fbb44570ecb3fcc9e0c4bd321fc5dc664"
  - "repo:docs/public-test-product.json#sha256:5c8c2bf9f0e1b3d5134ee61ba4d8312ec2deaec369bf13e45d56384d97640dd2"
  - "repo:scripts/check-test-release-gate.py#sha256:75a37fd28d2aaf302c7079088b54c4cedb4060bd4497f4aa9219198ff024ce95"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:dd63333a702a7a89585d101d2b9d043ebd0987d8"
status: active
---

# 번호 시험판 자동 게시 gate

번호 시험판의 별도 승인 질문 없음. milestone 완료 때 에이전트가 `docs/test-release-intent.json`에 다음 번호·완료 plan ID·제품 지문 기록.
`check-test-release-gate.py`에서 해당 의도·마지막 수용 제품·후보 비교. 새 제품 byte만 후보·게시·공개 수용 자동 진행.
동일 제품과 문서·계획·사실·source-only Skill·지침·시험·CI·안내 변경은 후보 생성 거부.
안정판 명시 승인 유지.
