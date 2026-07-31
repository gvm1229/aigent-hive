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
  - "repo:crates/hive-cli/src/update_discovery.rs#sha256:650d13734e29745a8ae4634c0dd7f3f8477222e144875b111ce6925fccd86a19"
links: [global-onboarding, test-distribution]
reviewed_revision: "git:bf9e28d8af36ef8d672694fc3c23fdd1a39233ee"
status: active
---

# Update 확인

일일 update 확인은 explicit opt-in. 성공 확인 뒤 24시간 registry 요청 throttle.
Offline·malformed 결과는 성공 시각을 기록하지 않고 다음 host session에서 재시도.
확인은 availability만 알리고 설치 금지.
