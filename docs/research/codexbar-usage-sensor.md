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

## Qualification 결과

- macOS app/CLI `v0.45.2`: 확인
- installed CLI symlink: `/opt/homebrew/bin/codexbar`
- active account digest binding과 `source=codex-cli`: 확인
- OpenAI가 five-hour session limit을 일시 비활성화한 상태의 weekly-only snapshot: 확인
- 2026-07-23 live gate: session 미제공, weekly remaining `51%`, threshold `10%`, allow
- Homebrew CLI symlink를 실제 `CodexBarCLI` executable로 한 번 resolve한 뒤 같은
  executable identity로 version/usage command 실행: 확인
- qualification 전후 executable identity 변경, version/usage timeout, stdout/stderr
  1 MiB 초과, process failure를 `usage_unknown`으로 처리: 확인
- version은 exact `0.45.2`만 허용하고 version 확인 실패 시 usage command를 실행하지
  않음: 확인
- account missing/mismatch/duplicate, provider/source/identity mismatch, stale/future
  timestamp를 `usage_unknown`으로 처리: 확인
- 5h session이 있으면 weekly의 low/malformed/duplicate 상태와 무관하게 session만 선택:
  확인
- session이 없을 때만 단일 10080분 weekly를 fallback으로 선택: 확인
- permit 발급 뒤 represented automatic dispatch 직전에 한 번 소비하고, 재사용·만료 시
  dispatch closure를 실행하지 않는 CLI adapter: 확인

Hostile qualification corpus는
`tests/fixtures/phase5/usage/codexbar_fixture.py`이며 model, subagent 또는 provider
process를 실행하지 않는다. 허용된 child process는 위 두 fixed-argv CodexBar read뿐이다.

## 통합 경계

- `hive run resume`의 기본값과 `--dispatch-intent manual`은 CodexBar 없이 기존
  prepare-only recovery를 유지하며 `enforced=false`, `outcome=not_requested`로 표시한다.
- `--dispatch-intent automatic`은 `--account-digest`와 정확히 한 active `--role`을
  필수로 받는다. threshold authority는 installed canonical
  `.hive/config/harness.toml`의 `usage_stop_remaining_percent`다. CLI `--threshold`는
  생략하거나 같은 값만 전달할 수 있고, downshift·override는 sensor 실행 전에 거부한다.
  Config는 64 KiB 이하 no-follow regular file로 읽으며 missing, malformed, duplicate,
  symlink면 fail-closed다.
- automatic resume는 durable run, owner, role, evidence 검증을 먼저 끝낸 뒤 fresh
  CodexBar sample을 읽고 permit을 평가한다. permit은 dispatch brief prepare closure
  직전에 현재 시각으로 한 번 소비한다.
- Hive는 선택된 account digest를 다시 해시한 filename 아래
  `.hive/runtime/usage-history/*.json`에 선택 window의 provider-neutral snapshot과
  integrity digest만 atomic/no-follow로 기록한다. Raw account, credential, CodexBar
  원문은 기록하지 않는다. 첫 정상 sample은 `history=absent`로 비교 없이 평가하고,
  이후에는 같은 reset remaining 증가, measurement/reset regression을 `usage_unknown`
  으로 처리한다. Malformed, oversized, account-mismatched, symlink history는
  fail-closed며 invalid current sample로 prior history를 덮지 않는다.
- Allow 결과는 정확히 한 selected role의 brief만 포함한다. Run ID, STATUS revision,
  role ID, canonical brief digest를 묶은 deterministic authorization ID를 만들고
  `.hive/runtime/dispatch-authorizations/*.json`에 sanitized `state=issued` claim을
  atomic/no-follow로 기록한다. 같은 binding의 retry/replay는 sensor를 다시 읽지 않고
  `outcome=already_issued`, zero briefs로 거부한다.
- Runtime history와 claim은 consumer `.hive/.gitignore`의 `/runtime/` 아래 Hive-owned
  ephemeral bytes다. Canonical config, PLAN, STATUS, role, handoff, evidence와 foreign
  namespace는 변경하지 않는다.
- Allow 결과만 `enforced=true`, `outcome=authorized`, authorization ID, sanitized
  evidence digest와 선택 window를 반환한다. missing/stale/mismatch/threshold/expiry는
  recovery payload와 빈 `dispatch_briefs`를 반환하며 model·subagent를 실행하지 않는다.
- Trust-boundary 한계: Hive는 같은 durable binding에 두 번째 authorization record를
  발급하지 않지만, 외부 host가 이미 캡처한 성공 JSON을 Hive를 거치지 않고 재사용하는
  행위까지 통제할 수 없다. Host가 실제 model/subagent dispatch와 acknowledgement를
  소유하므로, host는 authorization ID를 자체 one-shot delegation key로 소비해야 한다.
- Runtime integrity digest는 accidental corruption, partial write, account/path mismatch를
  검출하지만 OS의 동일 사용자 권한을 가진 공격자에 대한 인증 MAC은 아니다. 별도 secret을
  저장하지 않는 제품 경계상, 동일 사용자가 snapshot과 digest를 함께 재작성하면 Hive가
  원본성을 증명할 수 없다. Automatic enforcement는 consumer project와 local runtime
  directory가 신뢰되는 동일-user filesystem boundary 안에 있다는 전제를 가진다.
- Windows 지원 표시는 별도 sensor evidence 전까지 `unverified`
