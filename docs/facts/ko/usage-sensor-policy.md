---
schema_version: 1
pair_id: usage-sensor-policy
topic_slug: usage-sensor-policy
language: ko
counterpart: ../en/usage-sensor-policy.md
title: "Usage sensor 정책"
summary: "Qualified host-native sensor 우선과 optional CodexBar fallback."
tags: [sensor, usage]
aliases: ["Native-first usage"]
sources:
  - "repo:crates/hive-cli/src/main.rs#sha256:a21d2797007b8b826e639cd77c3c42a2ab630586815ec37883899b77afdbcf41"
  - "repo:crates/hive-cli/src/usage.rs#sha256:08df602b839ca6ced6cd1571c37111b300853def9bba4e25c9c24774453535f6"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:41e5712b85c91de0f0df5505e999b245e60a530d5764f8cf4a35c285b8c4f730"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:ad412a7b2aa6bba927f5f841e0e5161f4fb60488"
status: active
---

# Usage sensor 정책

Host별 우선 surface: qualified native machine sensor. CodexBar: allowlisted native
unavailable·unsupported 결과에서만 explicit consent로 사용하는 optional fallback.
Native limited 판정 우회 금지.

supplied Codex account digest를 찾지 못하면 native sensor가 완전한 authenticated account
하나를 반환할 때만 digest 없이 한 번 재측정. identity 누락·복수·malformed·stale·limited는
계속 fail-closed이며 CodexBar 호출 없음.

신속 설정은 남은 사용량 `20%`에서 보호를 활성화. 정상 설정은 native-only probe가
allowlisted 실패를 반환한 뒤에만 CodexBar를 설명하고 동의를 요청.
