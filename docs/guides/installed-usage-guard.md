# 설치본 usage guard의 source 적용

Source 개발도 설치된 `hive usage`를 단일 정본으로 사용. 별도 Python gate, 15초 watcher,
source-local threshold·halt state 없음.

## 대상 분류

| 대상 | 사용량 보호 | Threshold |
| --- | --- | --- |
| `.hive/config/harness.toml` 보유 Hive project | 활성 | `max(global, project)` |
| `hive-source.json` 보유 Aigent Hive source | 활성 | global |
| 자체 `AGENTS.md`만 보유한 project | 비활성 | 없음 |
| 빈 folder | 비활성 | 없음 |

Non-Hive target의 `status`, `enforce`, threshold 변경, session override, halt marker와 runtime
file 생성 없음. Project threshold 요청의 global 변경 재해석 금지.

## Hive 기능과 guard 분리

Non-Hive의 guard 비활성은 Hive 전체 비활성과 별개. `quick-answer`, prompt 개선, user-root
지식처럼 project state가 불필요한 Skill은 setup·usage preflight 없이 사용. Project plan,
run, file ownership이 필요한 workflow만 한 번의 명확한 project 활성화 승인 대상으로 분리.
승인 뒤 capability 확인과 run bootstrap은 해당 workflow가 자동 처리하며 usage guard 오류로
내부 전제 노출 금지.

## Source task preflight

Source task 시작 시 active host의 정확한 session ID·process ID 사용:

```text
hive usage enforce --target <source-root> --host <active-host> --session-id <current-session-id> --process-id <current-process-id> --user-root <user-root> --output json
```

Task당 one-shot 확인 1회. Tool·mutation·push·최종 응답 전 반복 gate와 background watcher 없음.
Exit `3`, `hive.usage-limited`, `hive.usage-unknown`: guard control·동의한 fallback recovery 외
작업 중단.

## Quota Reset Guard

- `0.11.0` 개발 범위: `usage_guard.quota_reset_guard_enabled` 기본값 `true`
- 같은 세션·계정·사용량 묶음·집계 구간의 유효한 두 측정에서 잔량 증가 시 `hive.usage-reset` 자동 실행 차단
- 첫 측정: 비교 기준만 기록. 초기화 시각 예약·외부 소식 추적 제외
- 비교 시점: `enforce`의 새 측정. 진행 중 추론의 15초 감시·호스트 프로세스 중단은 미지원
- 제한 표시: `status`의 `quota_reset_guard_monitoring=false`. 공개 설치본의 지원 여부와 개발 구현 구분
- 초기화 중단: 같은 값 재조회·프로세스 변경 뒤에도 유지. 현재 중단 표식의 지문과 명시적 사용자 확인 필요

초기화 사실 확인 뒤 재개 요청을 받은 경우의 개발 명령:

```text
hive usage session --target <hive-target> --host <active-host> --session-id <current-session-id> --process-id <current-process-id> --user-root <user-root> --action acknowledge-reset --confirm-reset <halt-digest> --output json
```

`halt-digest`는 중단 결과의 `reset_acknowledgement_digest` 또는 `status`의 `halt_digest` 값.
확인 후 동일한 대상·세션·사용자 루트와 현재 계정 정보로 `enforce` 재실행 필수.
확인은 잔여량 보호 해제나 작업 실행 허가와 구분. 낮거나 확인할 수 없는 잔여량은 계속 차단.
공개 `0.10.3` 설치본에 새 명령이 있다는 가정 금지.

## Threshold 변경

명시적 global 변경:

```text
hive usage threshold --user-root <user-root> --remaining-percent <1..99> --output json
```

설정 완료 Hive project의 조기 중지 변경:

```text
hive usage threshold --target <project-root> --remaining-percent <1..99> --output json
```

Global `20%`, project `40%`: 적용값 `40%`. Non-Hive target의 project `40%` 요청: mutation
`0건`, 적용 threshold 없음.

## Session control

현재 binding만 비활성화:

```text
hive usage session --target <hive-target> --host <active-host> --session-id <current-session-id> --process-id <current-process-id> --user-root <user-root> --action disable --confirm-session-disable --output json
```

재활성화:

```text
hive usage session --target <hive-target> --host <active-host> --session-id <current-session-id> --process-id <current-process-id> --user-root <user-root> --action enable --output json
```

새 session 기본값: 활성. Raw account·session ID 저장 없음. Native sensor 우선, CodexBar는
별도 동의를 받은 failure-only fallback. Provider credential·API 호출·OMX/OMC·host process signal
경로 없음.
