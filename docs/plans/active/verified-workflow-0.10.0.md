# Verified workflow `0.10.0`

> Checklist owner: `VWF10-*`
> 이전 이름: `ralph-loop`
> 역할: 증거·의존성·bounded retry·독립 검증이 필요한 실행 graph
> 병합 source: `ralph-loop`, `iterative-execution`

## Checklist

- [x] [VWF10-001] Canonical Skill ID `verified-workflow`·한국어 표시 `검증형 작업 흐름`과 `iterative-execution` protocol 병합 유지보수자 승인
- [x] [VWF10-002] `ralph-loop` graph 계약과 `iterative-execution` receipt·dispatch-uncertain·budget·cancel·recovery 계약을 하나의 `verified-workflow` canonical Skill로 병합 — `cd3379a`; Codex·Claude·Antigravity projection parity, `hive-projection` 34·`hive-render` 61·`user_setup` 46 Rust tests, Skill contract 38 Python tests 통과
- [x] [VWF10-003] 자연어 continuation의 자동 routing: dependency·중간 evidence gate·bounded retry·독립 verifier·steering·recovery 중 2개 이상, reason code와 `simple|verified-workflow|required-but-unsupported` 결과 — `c032030`; `hive-projection` 38 Rust tests·routing contract 20 Python tests 통과
- [x] [VWF10-004] 작업 길이·bare `continue`만으로 자동 선택 금지, `간단한 continuation|검증형 workflow|retry 없음` 사용자 override — `c032030`; 단순 continuation·`NoRetry`·inactive host 회귀 포함
- [x] [VWF10-005] Host Goal·task를 outer owner로 유지하고 verified workflow graph를 nested execution contract로 결합, Hive의 model·subagent process spawn `0건` — `c37e8cb`; `hive run closure`의 read-only continuation envelope·outer owner·`task_launch=host-owned`·`spawned=false` 검증
- [x] [VWF10-006] `SKM10-*` cleanup 계약을 통한 기존 `ralph-loop|iterative-execution` run·setup·update·projection 무손실 migration과 three-host routing·복구 검증 — `f494053`, `0b8328d`; direct-jump·projection·routing·rollback 회귀 통과

## 일회성 통합 수용

- [x] [VWA10-001] 자연어 요청의 `dependency-graph|independent-verifier` 정규화 신호가 공개 `hive route`에서 `verified-workflow`를 자동 선택 — 수용 영수증 `sha256:a2fefa0da9027582bcfbcdc44da5dae33d22491bcb6c323cee78bcc0b0e81169`
- [x] [VWA10-002] `tests/work/` 격리 작업에서 `disposable-run` revision 1 생성·검증, graph digest `sha256:0597baef2c88d1d3add59e27ab74540138e6760fa89a7d0f3bee7d8d4c52f587` 고정
- [x] [VWA10-003] 의도한 attempt 1 실패 뒤 continuation retry 잔여 2회 확인, attempt 2 성공
- [x] [VWA10-004] task agent와 다른 Codex Judge identity의 host-owned 결과 receipt 검증, `spawned=false`, 단일 receipt 완료 권한 없음, quorum 필요
- [x] [VWA10-005] 다른 CLI process와 새 session identity에서 `session-binding-mismatch`를 구분하고 동일 graph digest를 canonical Markdown에서 복구
- [x] [VWA10-006] 취소 뒤 closure terminal·`retry_permitted=false`·`nudge_claimed=false`·`spawned=false` 확인, 단일 JSON 수용 영수증 생성

수용 경계:

- 제품·사용자 지식·전역 설정을 변경하지 않고 `tests/work/`만 실행 상태로 사용
- provider process·model API·subagent process를 실행하지 않음
- 실제 Codex 응용 프로그램 재시작 대신 새 CLI 프로세스와 새 session identity의 복구를 검증하며, 응용 프로그램 자체 재시작까지 증명했다고 표현하지 않음
- `0.10.0` 안정판 tag·게시·설치 작업은 포함하지 않음

## 자동 선택 기준

```text
strong signals >= 2
AND host capability verified
AND user did not opt out
```

Strong signal:

- 세 단계 이상의 실제 dependency
- 중간 evidence가 다음 edge의 조건
- node별 retry·backoff·반복 failure fingerprint 제한
- 실행자와 별도 verifier
- topology steering audit
- session·compaction 뒤 exact node recovery

## 역할 경계

- Natural continuation: 전체 run 지속·closure·cancel
- `verified-workflow`: 복잡한 내부 graph·retry·evidence·verification
- Host: Goal·task 실행과 native agent launch
- Stop hook: bounded accidental-stop nudge
