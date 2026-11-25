---
schema_version: 1
pair_id: host-usage-directions
topic_slug: host-usage-directions
language: ko
counterpart: ../en/host-usage-directions.md
title: "호스트별 사용량 방향"
summary: "호스트 센서 표 하나로 입력 방향·보호 창·지원 버전 결정"
tags: [guard, sensor, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:50bbe234dff657b08dca42bc8cb17330a75d6aee436b5debd318fb039fd787c4"
  - "repo:crates/hive-cli/src/usage_control/antigravity.rs#sha256:ead5c18719be74afaaebb8fca8fd580e9aff48d018fc4bab4df1d51032aef909"
  - "repo:crates/hive-cli/src/user_setup/setup_engine/host_windows.rs#sha256:41c24ecb384dbc3102eb8836946b37bf9ad33a463ffdb16abd8170eb591e45c2"
  - "repo:crates/hive-core/src/preferences.rs#sha256:6ec5ee3711750ed82cd015e412ade49185152bec1831dbcbf077e52f122cce3b"
links: [usage-guard-thresholds, usage-sensor-policy]
reviewed_revision: "git:9c19940d54e90822e384a95b9482071620324e98"
status: active
---

# 호스트별 사용량 방향

공용 `HOST_USAGE_SENSORS`: 호스트별 사용량 방향·지원 창·센서 버전 범위의 정본. Claude 입력: 사용한 비율, 저장: 남은 비율로 환산. Codex와 Antigravity 입력: 남은 비율. 호스트 기준 변경: 기존 `host_windows` 설정 필수, 선택한 보호 창만 갱신. 사용량 기준 50% 미만: `--confirm` 필수. Antigravity 지원 범위: 1.1.18 이상·2.0.0 미만. 현지 고정물 부재로 실제 1.2.17 콜백 미검증, 관련 시험 무시 상태.
