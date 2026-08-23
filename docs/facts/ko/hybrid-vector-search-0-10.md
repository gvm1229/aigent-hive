---
schema_version: 1
pair_id: hybrid-vector-search-0-10
topic_slug: hybrid-vector-search-0-10
language: ko
counterpart: ../en/hybrid-vector-search-0-10.md
title: "0.10.0 Hybrid vector search gate"
summary: "중복 제거·unique chunk·재개 가능한 embedding·end-to-end 속도·격리·rollback·세 운영체제로 vector 자격을 다시 검증하는 0.10.0 범위"
tags: [knowledge, retrieval, v0-10, vector]
aliases: ["Vector database gate"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:b88eaf08d187d6f83cfac8b9e3a186791f08b71d0d5287f5dafe4d2e7aaa8151"
  - "repo:docs/plans/active/hybrid-vector-search-0.10.0.md#sha256:1010a111e32833722e0c00b9bda1de421b21e98a025366a856dea661f2ec8ad9"
  - "repo:docs/research/evidence/vector-hard-gate-windows-2026-08-23.json#sha256:41517d801330c1c299178b5b1ae75ed27fb5106c8af6ce4e2083b66cec30f09a"
  - "repo:docs/research/vector-memory-0.10-feasibility-2026-08-22.md#sha256:03dca07c4f6b5928268f4bc7c5337d1604371eadcd5b8a7b85b88ec3f65f215c"
links: [global-knowledge-rag, graphify-0-10-adoption, knowledge-storage, v0-10-product-scope]
reviewed_revision: "git:e8bbe0529513df56e73f84cf5797bb334f4184ec"
status: active
---

# `0.10.0` hybrid vector search gate

첫 gate에서 semantic Recall@10 `+15.0 points`와 빠른 vector engine을 확인했지만 naive
50,000 embedding build가 10분을 넘었습니다. 재검증은 반복 합성 자료와 50,000 unique chunk를
분리하고 digest 재사용·중단 재개·증분 build·query embedding 속도·scope 격리·원자 generation·
rollback·세 운영체제 근거를 추가합니다. Engine 선결정은 없습니다. 모든 gate를 통과한 한 조합만
선택형 hybrid adapter로 채택하고, 실패하면 FTS·graph와 product dependency `0건`을 유지합니다.
