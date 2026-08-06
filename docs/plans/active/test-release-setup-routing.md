# 시험판 global setup routing·복구 계획

> Checklist owner: `TUR-*`
> Target: `0.9.0-test.3`
> Scope: global user-scope 설정과 project harness 설정의 명시적 분리, legacy·numbered test
> user projection 인증 연속성

## 목표

- Global preference 요청의 `setup-hive` 자동 선택
- 명시 project·repository·folder·path 요청의 `setup-harness` 자동 선택
- 두 범위 동시 요청: global setup 완료 뒤 project setup의 별도 확인
- legacy `0.7.0`, `0.9.0-test.1|.2` authenticated user projection의 `0.9.0-test.3` 안전한 갱신
- 무인증·변조 ownership manifest의 write preview·mutation 0건 유지

## Checklist

- [x] [TUR-001] `setup-hive`·`setup-harness`의 discovery description·routing boundary·Codex
  metadata와 current projection parity
- [x] [TUR-002] legacy `0.7.0`, `0.9.0-test.1|.2` exact user projection inventory의 authenticated
  prior-release validation과 foreign·변조 byte fail-closed 유지
- [x] [TUR-003] Global-only·project-only·combined request와 test projection update의 Rust·Python
  regression
- [x] [TUR-004] `0.9.0-test.3|test` candidate·publication·existing test projection의 global
  setup routing acceptance

## 완료 기준

- Global preference prompt의 project root 요청·inspection 0건
- Project prompt의 `hive setup --scope user` 재구성 혼입 0건
- Known legacy·test predecessor: exact inventory 검증 뒤 user projection preview·apply 가능
- Unknown·modified predecessor: write preview·apply 0건과 exact conflict
- `test` 갱신, `latest=0.8.0` 유지

## 완료 증거

- candidate [`31090062784`](https://github.com/gvm1229/aigent-hive/actions/runs/31090062784):
  `5341bdf` 5 native target·npm umbrella·direct installer PASS
- publication [`31090917408`](https://github.com/gvm1229/aigent-hive/actions/runs/31090917408):
  여섯 package `test=0.9.0-test.3`, `latest=0.8.0`, annotated `v0.9.0-test.3`, 22-asset prerelease PASS
- 기존 Codex legacy user installation의 source test.3 dry-run: authenticated preview PASS
