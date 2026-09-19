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
  - "repo:crates/hive-cli/src/main.rs#sha256:c4a2a3267134b6572a39f37cec8a76b15ed4da0896a02ce701cf508525224785"
  - "repo:crates/hive-cli/src/usage.rs#sha256:c60a6eecaa243ef0528c292303baca85f0bf4c4c4f654612bf97d15fa52ffe69"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:267309f9d7f56095b6cc00f4602aeaf1249e0e8988f7fc5bd897dff04e2be5f5"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:e1e23470bdd37528700da00641cedef9e510c525863c991c9d189ab761e9bde1"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:4e753ff25c9c2c604b59b60d27cace205a8e5f7cf377538db6dd6156835f0408"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:d1105d7d3ebc2c5b8dbf100c4383ecd0933a8425c07501785b84ccdd4c6d2719"
links: [automatic-dispatch-guard, supported-hosts]
reviewed_revision: "git:4afd5ba4483d98f63ae42c065837f9b010506657"
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
