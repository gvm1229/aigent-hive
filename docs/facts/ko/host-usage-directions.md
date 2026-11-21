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
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:b830054a7de89161421441148b7683d339ab383d31f3a1441690556a10fd2daf"
  - "repo:crates/hive-cli/src/usage_control/antigravity.rs#sha256:16907c56509e4dbc868d045112af64c55cbb402d227bc5384bb254cf0ae8fe6a"
  - "repo:crates/hive-cli/src/user_setup/setup_engine/host_windows.rs#sha256:26016906f758d48f982e3f378034b569d079b3c3448c738dba0cf4276c408420"
  - "repo:crates/hive-core/src/preferences.rs#sha256:6ec5ee3711750ed82cd015e412ade49185152bec1831dbcbf077e52f122cce3b"
links: [usage-guard-thresholds, usage-sensor-policy]
reviewed_revision: "git:9c19940d54e90822e384a95b9482071620324e98"
status: active
---

# 호스트별 사용량 방향

공용 `HOST_USAGE_SENSORS`: 호스트별 사용량 방향·지원 창·센서 버전 범위의 정본. Claude 입력: 사용한 비율, 저장: 남은 비율로 환산. Codex와 Antigravity 입력: 남은 비율. 호스트 기준 변경: 기존 `host_windows` 설정 필수, 선택한 보호 창만 갱신. 사용량 기준 50% 미만: `--confirm` 필수. Antigravity 지원 범위: 1.1.18 이상·2.0.0 미만. 현지 고정물 부재로 실제 1.2.17 콜백 미검증, 관련 시험 무시 상태.
