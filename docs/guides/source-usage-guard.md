# Source 개발 usage guard

이 문서는 Aigent Hive source workspace에서 장시간 Codex 작업을 수행할 때 사용하는
개발 전용 `hive-usage-guard` Skill을 설명. Consumer harness에 출하되는 제품
기능은 아니며 product version 변경 없음.

## 문제와 보장 범위

Consumer 제품 guard는 새 automatic dispatch 직전에 one-shot permit을 발급. 이미
진행 중인 Codex App task, durable Codex goal 또는 Ultragoal artifact를 종료하는
기능 범위에서 제외. `omx cancel`도 session-scoped OMX mode는 정리하지만 durable goal
artifact나 Codex App task 종료는 증명 범위 밖.

Source guard는 이 경계를 다음 두 계층으로 보완.

1. 별도 watcher: 기본 15초 간격으로 qualified CodexBar `0.45.2`를 읽어 현재 session의
   observation과 halt marker를 갱신.
2. source directive가 매 user turn의 simple-question 판별 전과 각 tool, mutation,
   delegation, external write, push, 최종 응답 전에 fresh `gate`를 요구.
   `halted` 또는 `usage_unknown`이면 guard 제어 외의 다음 action 시작 금지.

Watcher의 Codex App process signal 전송과 `.omx/` state 수정 금지.
따라서 한 번의 model inference 중간 강제 중단 주장은 제외. 독립 polling은
중지선을 빠르게 감지하고, mandatory pre-action check는 감지 뒤 추가 작업이 시작되는
요청을 차단.

## Quota 선택

- CodexBar primary session window가 있으면 항상 그것을 선택.
- Primary가 명시적으로 없을 때만 weekly secondary window를 사용.
- Primary가 존재하지만 malformed이면 weekly fallback 없이 `usage_unknown`.
- Session은 exact `300`분, weekly는 exact `10080`분 window만 허용하고 duplicate JSON
  key를 포함한 ambiguous payload는 `usage_unknown`.
- 기본 중지선은 remaining `10%`이며 `remaining <= threshold`가 inclusive halt.
- Snapshot stale, sensor/version mismatch, 다중 account 또는 malformed output은
  fail-closed `usage_unknown`.

Raw account, credential과 CodexBar 원문 저장 금지. Observation에는 sensor
version, 선택 window, used/remaining, 측정 시각과 decision만 남김.

## 실제 session에서 사용

Skill 이름을 말하지 않아도 아래처럼 의도가 분명하면 자동으로 적용.

| 목적 | 자연어 prompt 예시 |
| --- | --- |
| 중지선 설정·활성화 | `사용량 가드 중지선을 잔여 10%로 설정하고 켜 줘.` |
| 현재 상태 확인 | `사용량 가드의 선택 window, 중지선, session 우회와 watcher 상태를 보여 줘.` |
| 현재 session만 우회 | `이 session에서 사용량 가드를 우회하고 남은 quota를 사용해.` |
| 다시 활성화 | `session 우회를 해제하고 사용량 가드를 다시 켜 줘.` |
| 중지선 변경 | `사용량 가드 중지선을 잔여 15%로 바꿔 줘.` |

명시적인 `$hive-usage-guard` 호출도 같은 동작을 하지만 필수 요건에서 제외.

```text
$hive-usage-guard 사용량 가드 상태를 보여 줘.
$hive-usage-guard 이 session에서 가드를 우회해.
$hive-usage-guard 가드를 다시 켜 줘.
```

중지선에 도달하면 같은 session의 일반 질문, 계획, Skill, tool, write와 후속 task를
모두 차단. `계속해`, `resume`, `끝내 줘`처럼 guard·quota·중지선·잔여량을 명시하지
않은 말은 우회 승인으로 인정 불가. 차단 뒤 계속하려면
`이 session에서 사용량 가드를 우회하고 계속해.`처럼 현재 session 우회 명시 필수.
다시 켠 시점에 이미 중지선 이하라면 일반 작업은 즉시 다시 차단.

Watcher의 진행 중인 model inference 강제 종료 기능 없음. 15초 polling과
mandatory `gate`가 다음 관측 가능한 실행 경계부터 작업을 중지.

## 직접 명령

상태 확인:

```bash
python3 .agents/skills/hive-usage-guard/scripts/guard.py status --json
```

Watcher 시작:

```bash
python3 .agents/skills/hive-usage-guard/scripts/guard.py watch-start --json
```

중지선 변경:

```bash
python3 .agents/skills/hive-usage-guard/scripts/guard.py set-threshold 15 --json
```

현재 session에서만 guard 비활성화:

```bash
python3 .agents/skills/hive-usage-guard/scripts/guard.py \
  session-disable --confirm-session-disable --json
```

다시 활성화:

```bash
python3 .agents/skills/hive-usage-guard/scripts/guard.py session-enable --json
python3 .agents/skills/hive-usage-guard/scripts/guard.py gate --json
```

`session-toggle`도 제공하지만 off 방향은 동일한
`--confirm-session-disable`을 요구. Threshold는 source-local 설정으로 유지되며,
disable은 current session ID와 Codex process ID에 결합. 새 session은 항상 enabled
default로 시작.

## 상태와 ownership

모든 runtime state는 Git에서 제외된 다음 경로에만 씀.

```text
.agents/work/usage-guard/
├── settings.json
└── sessions/<session-id>/
    ├── control.json
    ├── observation.json
    ├── halt.json
    ├── watcher.json
    └── watcher.log
```

State read/write는 symlink를 거부하고 source root 밖 접근을 차단. Watcher stop은
PID만 믿지 않고 exact script path, `watch` command와 session ID가 모두 일치하는
process만 종료.

## Consumer 제품 상태

구현 완료: shipping `hive-usage-guard` Skill, typed one-shot `hive usage enforce`,
installed threshold ownership, host/session/PID binding, ignored runtime marker와
session-first·weekly-fallback sensor 선택.

남은 qualification: 실제 Codex·Claude Code·Gemini Antigravity host E2E와
macOS·Windows의 로컬 CodexBar sensor evidence. 제품 watcher는 금지하며 각 host가
turn boundary에서 one-shot `enforce`를 호출.
