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
  - "repo:crates/hive-cli/src/main.rs#sha256:ca5d0af23e3719732dec1a6d3a38dcde959a7dfa1426ef7f7be9edc2623b0a4d"
  - "repo:crates/hive-cli/src/usage.rs#sha256:1775b5a413935ff5c714eef1700d8c91adbef83fbc9509e2e50dd22997e06777"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b2d3c7a9a42ce53e2ab8806401efb6e7076d7550843f0a09dd7158de56eee08f"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:e1e23470bdd37528700da00641cedef9e510c525863c991c9d189ab761e9bde1"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:72bd93df896956e10c30013db3bc9232b7a4a3ca"
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
