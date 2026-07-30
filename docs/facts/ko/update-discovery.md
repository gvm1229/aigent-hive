---
schema_version: 1
pair_id: update-discovery
topic_slug: update-discovery
language: ko
counterpart: ../en/update-discovery.md
title: "Update 확인"
summary: "설치 없는 opt-in 일일 update 확인."
tags: [discovery, update]
aliases: ["일일 update 확인"]
sources:
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:2fb97b133d567155c0f333cbe7a401fc7473e849d88db2e2f9b897d7acecb39e"
links: [global-onboarding, test-distribution]
reviewed_revision: "git:99f39edd08cc4b9d513f073d297bed05e2772c9d"
status: active
---

# Update 확인

일일 update 확인은 explicit opt-in. 성공 확인 뒤 24시간 registry 요청 throttle.
Offline·malformed 결과는 성공 시각을 기록하지 않고 다음 host session에서 재시도.
확인은 availability만 알리고 설치 금지.
