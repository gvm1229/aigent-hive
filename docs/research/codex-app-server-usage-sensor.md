# Codex app-server usage sensor 조사

- 조사일: 2026-07-26
- Local 검증: `codex-cli 0.145.0`
- 판정: Codex native primary sensor 후보 qualification 성공
- 기존 CodexBar: explicit-consent optional fallback 후보

## 공식 surface

Local `codex app-server --help`에서 experimental stdio transport 확인.
Local generated JSON Schema에서 다음 contract 확인:

- initialize request의 `clientInfo`
- `account/rateLimits/read`
- `rateLimits`
- `rateLimitsByLimitId`
- `account/rateLimits/updated`

Upstream app-server 문서도 같은 method와 rolling update notification 제공:

- [OpenAI Codex app-server README](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md)

## Live autonomous probe

Python subprocess sequence:

1. Fixed argv `codex app-server --stdio`
2. JSONL `initialize`
3. `initialized` notification
4. `account/rateLimits/read`
5. Matching request ID response 추출
6. Process 종료

확인 결과:

- Exit: success
- `rateLimitsByLimitId.codex`: 존재
- `usedPercent`: numeric
- `windowDurationMins`: integer
- `resetsAt`: integer timestamp
- `limitId`: `codex`
- `planType`: recognized value
- Native normalized usage와 같은 시각 CodexBar weekly usage: 일치
- API key·OAuth token·account identifier 직접 접근: 0건
- Raw provider payload·credential persistence: 0건

실사용 quota 값과 reset timestamp는 public source 문서에 기록 금지.

## 채택 경계

- `app-server`의 experimental 표시에 따른 exact version·schema qualification 필요
- Executable path·identity의 실행 전후 검증 필요
- Timeout, stdout·stderr size, JSON depth와 process lifecycle bound 필요
- `rateLimitsByLimitId.codex` 우선, legacy `rateLimits`는 compatibility fallback
- Native limited 판정 뒤 CodexBar 재조회·우회 금지
- Native unavailable·unsupported·malformed에서만 CodexBar fallback
- CodexBar 미설치 때 silent install 금지
- CodexBar fallback 설치 전 대상·package manager·command·ownership preview와 explicit consent 필요
- 설치 거절·불가 시 setup·knowledge·update 같은 core 기능 유지, automatic dispatch는
  `usage_unknown`
- Codex credential, private endpoint, browser state와 interactive `/usage` text parsing 금지

## 후속 검증

- Fake app-server protocol fixture와 hostile corpus
- Native success에서 CodexBar invocation 0회
- Native failure에서 CodexBar invocation 최대 1회
- Native limited에서 CodexBar invocation 0회
- Source Python watcher와 shipping Rust one-shot guard의 normalized decision parity
- Opt-in live probe의 actual quota 값 비기록
