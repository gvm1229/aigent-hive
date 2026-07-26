# 세 host native usage sensor 전환 계획

> Checklist owner: `NUS-*`
> Load condition: Codex·Claude Code·Antigravity usage sensor와 CodexBar fallback 구현·검증
> Research: [`Codex`](../../research/codex-app-server-usage-sensor.md) ·
> [`Claude Code`](../../research/claude-code-native-usage-sensor.md) ·
> [`Antigravity`](../../research/antigravity-native-usage-sensor.md)
> Decision: [`ADR-0010`](../../decisions/ADR-0010-native-first-usage-sensors.md)

## 목표

- Host별 qualified native surface를 source·shipping guard의 기본 sensor로 사용
- CodexBar를 세 provider 모두의 명시적 동의 기반 optional fallback으로 강등
- Sensor 오류와 quota 소진의 분리, 자동 dispatch fail-closed 유지
- Provider credential·private endpoint·interactive TUI parsing 없음

## 검증된 기준선

| 범위 | 결과 |
| --- | --- |
| Local Codex | `codex-cli 0.145.0`, `app-server --stdio` 제공 |
| Protocol discovery | Local generated schema에 `account/rateLimits/read` 존재 |
| Autonomous probe | Python subprocess의 initialize → initialized → rate-limit read 성공 |
| Response | `usedPercent`, `windowDurationMins`, `resetsAt`, `limitId`, `planType` 확인 |
| Cross-check | Native `codex` bucket과 CodexBar weekly 사용률 일치 |
| Claude Code | Status-line stdin JSON의 5-hour·7-day `rate_limits` 공식 문서 확인 |
| Antigravity | `/usage`·`/quota` interactive TUI만 공식 문서화; machine surface 미확인 |

## 1. Codex native sensor

- [x] [NUS-001] Current Codex session에서 credential 직접 접근 없는
  `account/rateLimits/read` autonomous probe와 CodexBar cross-check
- [x] [NUS-002] Supported Codex version·method·response schema와 experimental surface
  compatibility matrix 고정
- [x] [NUS-003] Active Codex executable의 canonical path·file identity를 실행 전후
  검증하고 shell 없는 fixed argv `app-server --stdio` adapter 구현
- [x] [NUS-004] JSONL initialize handshake, request ID correlation, bounded timeout·output,
  stderr·process failure와 graceful termination 구현
- [x] [NUS-005] `rateLimitsByLimitId.codex` 우선, backward-compatible `rateLimits`
  fallback, primary·secondary window의 provider-neutral `UsageSnapshot` 정규화
- [x] [NUS-006] Missing·duplicate·non-finite·stale·regressed window와 unexpected
  `limitId`·plan·protocol payload를 sanitized `usage_unknown`으로 처리

## 2. Claude Code native sensor

- [x] [NUS-016] Official status-line JSON의
  `rate_limits.five_hour|seven_day.used_percentage|resets_at`, subscriber-only와 first
  API response 이후 availability contract 고정
- [x] [NUS-017] Plugin `bin/`에 bounded `hive usage capture --host claude --stdin-json`
  projection; 사용자의 host-owned `/statusline` opt-in만 허용하고 Hive의
  `~/.claude/settings.json` mutation 0회
- [x] [NUS-018] 5-hour 우선·7-day fallback 정규화, exact `session_id` binding,
  callback receipt time 기반 freshness와 sanitized ignored snapshot 구현
- [x] [NUS-019] Existing status line non-clobbering integration snippet·preview 제공,
  미설정·trust 거절·first response 이전·stale capture를 native unavailable로 분류
- [ ] [NUS-020] 실제 Claude Pro/Max host에서 5-hour·7-day parity, 독립 window
  omission, existing status line non-clobber qualification

## 3. Antigravity native sensor

- [x] [NUS-021] Antigravity CLI `1.1.7`의 `/usage`·`/quota`가 backend refresh 후
  interactive TUI만 제공하며 documented machine JSON mode가 없음을 dated matrix에 고정
- [x] [NUS-022] Release qualification마다 official CLI·SDK의 structured
  quota JSON·event·local IPC surface 재검사와 supported-version matrix 갱신
- [x] [NUS-023] Official structured surface와 live probe가 모두 확인된 뒤에만 native
  adapter 활성화; TUI text·screen·private LSP/HTTP·credential·browser state parsing 금지
- [x] [NUS-024] Structured surface 부재 시 `native=unsupported`를 명시하고
  qualified CodexBar Antigravity adapter만 fallback으로 허용
- [ ] [NUS-025] 실제 Antigravity host의 truthful unsupported·CodexBar fallback
  parity qualification

## 4. Source·shipping integration

- [x] [NUS-007] Source Python `hive-usage-guard`와 shipping Rust `hive usage`에
  `active-host native → CodexBar` 순서 적용, 기존 threshold·history·halt·permit 보존
- [x] [NUS-008] Native unavailable·unsupported·malformed일 때만 CodexBar fallback,
  native quota limited 상태에서는 fallback 우회 금지
- [x] [NUS-009] 어느 provider든 CodexBar 미설치 시 fallback 필요성·설치 대상·권한·
  command preview를
  알리고 autonomous CodexBar 설치 동의 요청; 수락 시 supported package-manager
  adapter만 실행, 거절 시 core 기능 유지와 automatic dispatch `usage_unknown`
- [x] [NUS-010] 설치 동의의 current action 한정, silent install·host credential
  접근·provider CLI 재설치·CodexBar API-key/manual-cookie 설정 제안·package-manager
  ownership 침범 금지

## 5. Qualification

- [x] [NUS-011] 현재 구현된 native adapter별 적용 가능한 timeout,
  oversized·malformed input, process identity change, stale snapshot, wrong session과
  unsupported-version hostile fixture
- [x] [NUS-012] Qualified native provider의 success·limited에서 CodexBar 0회,
  unavailable·unsupported·malformed에서 fallback 최대 1회, native surface 부재
  provider의 truthful unsupported와 fallback 최대 1회
- [x] [NUS-013] Source watcher·one-shot shipping guard·automatic resume의 existing
  fail-closed, session binding, authorization replay와 monotonicity regression 전수 통과
- [ ] [NUS-014] Opt-in 세 host qualification에서 native/CodexBar normalized
  snapshot parity 또는 truthful unsupported, raw account·quota·credential persistence 0건
- [x] [NUS-015] Provider별 notification의 install accept·decline·package-manager
  unavailable, non-interactive 실행과 source/consumer guidance parity

## 해석

- 설치 제안 대상: `CodexBar`
- Codex·Claude Code·Antigravity CLI 재설치 제안 없음
- CodexBar 분류: 모든 provider에서 fallback-only
- Antigravity의 현재 상태: native machine sensor `unsupported`, CodexBar fallback 허용

## 잔여 evidence

- `NUS-014`: 실제 세 host opt-in parity 잔여
- `NUS-020`: 실제 Claude Pro/Max qualification 잔여
- `NUS-025`: 실제 Antigravity native `unsupported`→CodexBar `0.45.2` fallback 호출,
  provider error의 sanitized `usage_unknown`·exit `3`·raw persistence 0건 확인
- `NUS-025`: CodexBar 성공 또는 limited snapshot의 normalized quota parity 잔여
