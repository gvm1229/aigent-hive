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
  - "repo:crates/hive-cli/src/update_discovery.rs#sha256:9841e0c913da22987396f488e22bc0459062aa41f3508d25269ef55a277c6c29"
links: [global-onboarding, test-distribution]
reviewed_revision: "git:4e3fb80b77c2e105029c9d6794922c4ce1b2fdeb"
status: active
---

# Update 확인

일일 update 확인은 explicit opt-in. 성공 확인 뒤 24시간 registry 요청 throttle.
Offline·malformed 결과는 성공 시각을 기록하지 않고 다음 host session에서 재시도.
확인은 availability만 알리고 설치 금지.
