# Host-neutral 연속 실행 `0.10.0`

> Checklist owner: `CON10-*`
> 상태: 조사·계약 확정 뒤 조건부 구현
> 목표: Agent 소유 작업의 중간 종료 방지와 사용자 취소의 즉시 보장

## 원칙

- Hive Markdown run·plan: provider-neutral 정본
- Host goal·task: 실제 연속 실행 주체
- Hive CLI closure gate: 최종 응답 가능 여부의 결정론적 판정
- Host hook: 선택형 종료 전 adapter, 실행 주체 아님
- Provider API·credential·직접 model/subagent process 실행 없음
- Hook 미지원 host: Goal·task 또는 수동 resume 경로, 지원을 가장한 대체 동작 없음

## Checklist

- [x] [CON10-001] Codex Goal·Claude task/hook·Antigravity task/hook의 공식 기능과 `oh-my-codex@3ad79a8` 구현 비교, 지원·부분 지원·미지원 판정 — [`host-neutral-continuation-hooks`](../../research/host-neutral-continuation-hooks-0.10-feasibility-2026-08-22.md)
- [ ] [CON10-002] `agent-owned|awaiting-user-authority|awaiting-external-evidence|blocked|excluded` closure schema와 `ready_for_final` 판정 계약
- [ ] [CON10-003] `hive run closure --target <root> --run <id> --output json` read-only CLI와 exact checklist·evidence·제외 ID 검증
- [ ] [CON10-004] Host-neutral continuation envelope: run ID·revision·session binding·next action·retry budget·closure digest·cancel state
- [ ] [CON10-005] Codex·Claude·Antigravity adapter별 Goal·task 지속과 Stop hook capability mapping, unsupported의 mutation `0건`
- [ ] [CON10-006] 선택형 hook preview·exact digest 승인·non-clobber merge·disable·uninstall·rollback 계약
- [ ] [CON10-007] 과도한 지속 방지: 사용자 interrupt·cancel 즉시 허용, `blocked_on_user`·terminal·stale·malformed·foreign session fail-open, revision당 nudge 1회, bounded consecutive block cap
- [ ] [CON10-008] three-host fixture와 중간 종료·정상 완료·취소·stale state·hook 손상·host 미지원 회귀 검증, 채택 또는 defer 결정

## Hook 최소 계약

Hook의 유일한 blocking 조건:

```text
current session binding valid
AND run active
AND closure.ready_for_final = false
AND closure.agent_owned > 0
AND retry budget remains
AND no user cancel or interrupt
```

그 밖의 상태: 종료 허용. Hook에서 goal·canonical run state mutation 금지.

## 단계

1. 공식 host capability와 `oh-my-codex` 사례 조사
2. Closure·continuation envelope schema 확정
3. Read-only CLI와 host-neutral fixture 구현
4. 지원 host별 선택형 adapter 구현
5. 취소·상한·복구·non-clobber 검증
6. 기본 비활성 공개 시험 뒤 채택 또는 defer

## 제외

- 무제한 Stop block
- Hook의 task·goal 자동 생성·완료·취소
- Transcript를 canonical state로 사용
- Provider별 state를 Hive Markdown 정본으로 승격
- Hook 미지원 host의 watcher·polling daemon 대체
