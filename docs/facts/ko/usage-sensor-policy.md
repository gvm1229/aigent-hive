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
  - "repo:crates/hive-cli/src/main.rs#sha256:bd0a33c9ac1debb73761ff7f492b8d83f384d0ea6c1a5bdd4a42a71f0931b631"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:cffd6c491ffd17dccefa84edb172bbfe64ae925f2fe9cf7c6efd07e6a896a9fd"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:5602025a4eb182cc6e51cc816cab74983f10ee2bdd2f6324649de63fdbddef1f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:aa0e9102c6d4a08a2468f39abf66f2788844c28a989eace52f59f9d2ea919957"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Usage sensor 정책

Host별 우선 surface: qualified native machine sensor. CodexBar: allowlisted native
unavailable·unsupported 결과에서만 explicit consent로 사용하는 optional fallback.
Native limited 판정 우회 금지.

신속 설정은 남은 사용량 `20%`에서 보호를 활성화. 정상 설정은 native-only probe가
allowlisted 실패를 반환한 뒤에만 CodexBar를 설명하고 동의를 요청.
