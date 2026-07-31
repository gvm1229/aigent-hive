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
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:a958146a8ea6d747fa485cf5ba0ec81f0471567723589fe82a6aa90815cece06"
links: [global-onboarding, test-distribution]
reviewed_revision: "git:a7be86f2558442c2cec3596abe2f481dd91d268f"
status: active
---

# Update 확인

일일 update 확인은 explicit opt-in. 성공 확인 뒤 24시간 registry 요청 throttle.
Offline·malformed 결과는 성공 시각을 기록하지 않고 다음 host session에서 재시도.
확인은 availability만 알리고 설치 금지.
