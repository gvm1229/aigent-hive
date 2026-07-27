---
schema_version: 1
pair_id: usage-hosts
topic_slug: usage-hosts
language: ko
counterpart: ../en/usage-hosts.md
title: "Usage Guard와 Host Sensor"
summary: "Native-first quota sensing, CodexBar fallback 경계와 source-session enforcement."
tags: [guard, hosts, usage]
aliases: ["사용량 가드 호스트"]
sources:
  - "repo:.agents/skills/hive-usage-guard/scripts/guard.py#sha256:9be7431e5f63d3bfbdcab93b902cb736cd5e13b59622d0817e576f738b1e6df1"
  - "repo:crates/hive-cli/src/usage.rs#sha256:5bd67c08505d00136738ed34751412aa37d7242e43ecb0fbb1c22b5c2f4c0fed"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:141e8070b475ee2b0d81e93a69217093e07af9a9ca61c16dcbb31f111ea1d0f4"
links: [plugin-lifecycle, security-release, workflow]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Usage Guard와 Host Sensor

Automatic work boundary 전 configured inclusive remaining-quota threshold 확인. Durable state에
허용된 정보: sanitized sensor identity, window, timing과 decision. Account payload, provider
credential과 raw quota response 저장 금지.

Sensor 순서: native first, CodexBar fallback-only.

- Codex: local app-server rate-limit method
- Claude: explicit configuration을 거친 sanitized status-line capture
- Antigravity: qualified official structured output 부재로 native unsupported
- CodexBar: 세 provider 공통 fallback-only, 설치 전 explicit consent 필요

Source-development Python watcher와 boundary gate는 shipping one-shot dispatch guard와 별도.
Transient `unknown`: 3초 대기 후 1회 재시도. 반복되는 짧은 glitch는 observation에 유지하되
새 halt marker 생성 없음. Confirmed quota exhaustion과 filesystem, session 또는 sensor-integrity
오류는 fail-closed.
