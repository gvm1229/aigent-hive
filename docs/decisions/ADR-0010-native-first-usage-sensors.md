# ADR-0010: Native-first usage sensor와 CodexBar fallback

- 상태: Accepted
- 기준일: 2026-07-26
- 범위: Source guard와 installed consumer automatic-dispatch guard

## Context

기존 guard의 CodexBar 단일 의존에서 intermittent unknown error 발생. Host 공식 quota
surface와 third-party fallback의 failure·ownership 경계 분리 필요.

확인된 host surface:

- Codex: `app-server` JSON-RPC `account/rateLimits/read`
- Claude Code: status-line stdin JSON의 5-hour·7-day `rate_limits`
- Antigravity: `/usage`·`/quota` interactive TUI, documented machine output 없음

## Decision

공통 sensor 순서:

1. Active host의 qualified native sensor
2. Native unavailable·unsupported·malformed일 때만 CodexBar
3. 두 sensor 모두 불가하면 `usage_unknown`

Host별 결정:

- Codex: app-server JSON-RPC primary
- Claude Code: user가 Claude host의 `/statusline`으로 opt-in한 sanitized JSON capture
  primary; Hive의 `~/.claude/settings.json` mutation 없음
- Antigravity: official structured surface qualification 전 `native=unsupported`
- CodexBar: 세 provider 모두 fallback-only

Antigravity fallback:

- Fixed argv:
  `codexbar usage --provider antigravity --source cli --format json --json-only`
- `default`·`antigravity-claude-gpt`의 provider-defined window와 모든 pool 통과
- Missing·exact `10080` window metadata의 schema v2 `provider` canonical identity
- Schema v1 Antigravity weekly history의 검증 후 one-way comparison bridge
- Multi-pool marker의 `multiple` compatibility
- Incomplete version stdout의 bounded macOS app bundle qualification

Fallback 규칙:

- Native limited 판정 뒤 CodexBar 호출 0회
- CodexBar 미설치 시 필요성·대상·권한·exact command preview
- Current action explicit consent 뒤 supported package-manager install만 허용
- 거절·설치 불가 시 core 기능 유지, automatic dispatch fail-closed
- Provider CLI 재설치와 CodexBar API-key·manual-cookie 설정 제안 없음
- Raw account, quota payload, session ID와 credential persistence 없음

## Consequences

장점:

- Native sensor 성공 경로의 CodexBar unknown error 제거
- Provider별 unsupported·unavailable·limited 상태 구분
- CodexBar 장애와 quota 소진의 분리

비용:

- Codex experimental protocol version qualification 필요
- Claude status-line opt-in과 existing configuration composition 필요
- Antigravity official machine surface 전 CodexBar fallback 의존
- Antigravity provider-defined multi-pool history와 schema v1 migration 유지 비용

Completion:

- 세 host fixture·live qualification
- Native success·limited에서 CodexBar invocation 0회
- Fallback install accept·decline·unavailable conformance
- Source Python·shipping Rust decision parity
