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
  - "repo:.github/workflows/ci.yml#sha256:60945a807302e95f64373efc5d91ff11269e0f2609dc1d21c707c41e5a79db09"
  - "repo:crates/hive-cli/src/usage.rs#sha256:5bd67c08505d00136738ed34751412aa37d7242e43ecb0fbb1c22b5c2f4c0fed"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:141e8070b475ee2b0d81e93a69217093e07af9a9ca61c16dcbb31f111ea1d0f4"
  - "repo:tests/conformance/test_source_usage_guard.py#sha256:6fd6c2db807d251c2cf33c38627114f80dab7541fd26134a687aeb69cdd98df7"
links: [plugin-lifecycle, security-release, workflow]
reviewed_revision: "git:727ec3fa252e2eabbea4a4c57c4b54f0e3830a99"
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

Source-session identity:

- `.omx/` read 0회
- `CODEX_THREAD_ID` 존재 시 current thread key 사용
- thread key 부재 시 live Codex host PID와 mandatory process creation digest 기반 identity
- control binding: thread 또는 process identity + process creation digest
- clean clone 초기화 경로: `.agents/work/usage-guard/` 한정
- 새 thread의 이전 bypass 상속 0건

Recovery boundary:

- `status`, `session-disable`, `session-enable`: quota sensor 초기화 없음
- `status`: 마지막 observation과 freshness 표시
- disabled `gate`: quota sensor 호출 없이 `session_bypass`
- thread 전환: 같은 host process의 이전 watcher 종료
- watcher stop: host creation digest와 locked child lease 일치 필수

State write portability:

- `os.fchmod`: callable host에서만 적용
- pre-stream failure cleanup: descriptor close 후 temporary path 제거
- Windows watcher lease: locked first byte 제외 후 guaranteed JSON object prefix 복원
- Linux·macOS CI: source guard 전체 corpus
- Windows CI: state write, lease read, clean-clone `gate`, `session-disable`, disabled `gate`
- Windows 실행 순서: broader Phase 1 corpus 이전

Regression acceptance:

- `.omx/` 없는 clean clone의 `gate`·`session-disable`
- disable 이후 `gate` allowance
- 새 thread 또는 recreated process로 bypass 전이 0건
- malformed OMX bytes 불변
- 이전 thread watcher 정리
- unrelated process의 genuine PID·creation digest가 담긴 watcher state도 signal 거부

Originating requirement: optional OMX runtime state를 mandatory source-guard authority로 취급한
bootstrap deadlock 제거.
