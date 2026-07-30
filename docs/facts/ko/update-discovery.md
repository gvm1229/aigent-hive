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
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:7631f6a1b322510cf6b9b1d6e826681362cb494d250cf7137b4a29c446402b35"
links: [global-onboarding, test-distribution]
reviewed_revision: "git:4e3fb80b77c2e105029c9d6794922c4ce1b2fdeb"
status: active
---

# Update 확인

일일 update 확인은 explicit opt-in. 성공 확인 뒤 24시간 registry 요청 throttle.
Offline·malformed 결과는 성공 시각을 기록하지 않고 다음 host session에서 재시도.
확인은 availability만 알리고 설치 금지.
