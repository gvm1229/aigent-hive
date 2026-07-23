# CodexBar usage sensor 조사

- 조사일: 2026-07-23
- 검증 release: `steipete/CodexBar` `v0.45.2`
- tag commit: `91560ca98e776b96fdf910d4a0423c2f0c07a3b9`
- 판정: optional local command adapter 후보, linked dependency·자동 설치 금지

## 채택 범위

Hive는 CodexBar를 설치하거나 provider credential을 읽지 않는다. 감지된 `codexbar` executable을 shell 없이 고정 argv로 bounded 실행하고, 결과를 provider-neutral `UsageSnapshot`으로 정규화한다.

`codexbar guard`는 inclusive threshold와 stable exit code를 제공하지만 account, source, measured time과 expiry를 포함하지 않는다. 따라서 Stage 7 freshness·scope 증거로 단독 사용하지 않는다.

정규화 입력은 다음 command의 stdout JSON이다.

```text
codexbar usage --provider codex --all-accounts --source cli --format json --json-only
```

필수 검증:

- exact qualified CodexBar version
- `provider=codex`
- active account digest 일치
- row-level `error` 부재
- local CLI source
- `usage.updatedAt` freshness
- 제공되는 경우 `primary.windowMinutes=300` session window
- session limit 미제공 시 `secondary.windowMinutes=10080` weekly fallback
- 선택된 window의 finite `usedPercent`, reset time과 monotonicity

어느 조건이든 증명하지 못하면 `usage_unknown`이며 다음 automatic dispatch를 허용하지 않는다. Session과 weekly가 함께 있으면 session이 우선한다. CodexBar 미설치 상태에서도 setup, knowledge, update 같은 비-dispatch core 기능은 정상 동작해야 한다.

## 근거

- [CodexBar CLI](https://github.com/steipete/CodexBar/blob/91560ca98e776b96fdf910d4a0423c2f0c07a3b9/docs/cli.md)
- [Codex provider](https://github.com/steipete/CodexBar/blob/91560ca98e776b96fdf910d4a0423c2f0c07a3b9/docs/codex.md)
- [v0.45.2 release](https://github.com/steipete/CodexBar/releases/tag/v0.45.2)
- [MIT license](https://github.com/steipete/CodexBar/blob/91560ca98e776b96fdf910d4a0423c2f0c07a3b9/LICENSE)

## Local qualification

- macOS app/CLI `v0.45.2`: 확인
- installed CLI symlink: `/opt/homebrew/bin/codexbar`
- active account digest binding과 `source=codex-cli`: 확인
- OpenAI가 five-hour session limit을 일시 비활성화한 상태의 weekly-only snapshot: 확인
- 2026-07-23 live gate: session 미제공, weekly remaining `51%`, threshold `10%`, allow
- Homebrew CLI symlink를 실제 `CodexBarCLI` executable로 resolve한 뒤 version/usage command 실행: 확인

## 미완료 qualification

- OMX·host-native dispatch boundary integration
- Windows 지원 표시는 별도 sensor evidence 전까지 `unverified`
