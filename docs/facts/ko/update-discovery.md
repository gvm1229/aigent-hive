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
  - "repo:crates/hive-cli/src/update_discovery.rs#sha256:64f7078b4994a5131506a7258484c63da9eebb2d0b99f746e07729b278231f2f"
links: [global-onboarding, test-distribution]
reviewed_revision: "git:bd6d9249b8641590269d32deb97d13b2816ba75e"
status: active
---

# Update 확인

일일 update 확인은 explicit opt-in. 성공 확인 뒤 24시간 registry 요청 throttle.
Offline·malformed 결과는 성공 시각을 기록하지 않고 다음 host session에서 재시도.
확인은 availability만 알리고 설치 금지.
