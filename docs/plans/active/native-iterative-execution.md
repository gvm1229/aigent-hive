# Hive-native 반복 실행 계획

> Checklist owner: `NAT-*`
> 상태: 정책·계획 정본화 완료, host feasibility 미착수
> Decision: [`ADR-0019`](../../decisions/ADR-0019-hive-native-iterative-execution.md)
> State contract: [`06-native-orchestration-state.md`](../contracts/06-native-orchestration-state.md)
> Workflow contract: [`07-native-orchestration-workflows.md`](../contracts/07-native-orchestration-workflows.md)

## 목표

- OMX·OMC 기능 의존성 없는 Hive-native 반복 실행·팀·다중 목표 Skill
- Hive 소유의 결정론적 상태 전이·logical scheduler·lease·receipt·cancel·복구
- Host 소유의 실제 model·subagent 실행과 선언형 envelope 소비
- Provider API·credential·model runtime·직접 process spawn `0건`
- Ambient·selected session pointer 기반 mutation authority `0건`
- 호출 정책과 무관한 strict workflow criterion·goal terminal authenticated Judge
- Legacy OMX·OMC run의 read-only provenance와 새 native identity 기반 명시적 migration

## 채택 원칙

1. 유용한 기능의 clean-room 채택 우선
2. `ownership-collision` 단독 제외 사유 금지
3. 제외 사유: `unsafe|provider-specific|redundant|non-useful`
4. Host capability 미확인 상태의 성공·exactly-once·강제 cancel 주장 금지
5. 지원 불충분 상태의 `dispatch-uncertain|unsupported` 정직한 중지
6. Usage guard·cancel·status·recover의 scheduler lock·pointer 독립 접근
7. Judge는 terminal acceptance gate 한정, scheduler tick·heartbeat·retry별 호출 금지

## 구현 순서

### A. 정본·feasibility

- [x] [NAT-001] 사용자 결정, RALPLAN-DR, Architect·Critic 승인 결과의 정본 plan·proposed ADR·source directive 반영
- [x] [NAT-002] OMX·OMC 전체 기능 inventory 재개방과 clean-room `adopt|merge|exclude` 재분류
  - Evidence: `docs/research/v0.9-omx-omc-capability-inventory.md`, reapply commit `1f8c1b3`,
    external runtime·provider credential·copied byte exclusion contract
- [x] [NAT-003] Codex·Claude Code·Antigravity의 envelope consume·claim·launch ack·result·cancel·lookup·idempotency capability matrix
  - Evidence: `schemas/host-orchestration-capability.schema.json`, 세 host fixture와
    `python tests/conformance/test_native_orchestration_feasibility.py -v` 4 PASS
- [ ] [NAT-004] 세 host fixture와 최소 한 host 실제 lifecycle spike, unsupported capability의 정직한 판정
- [ ] [NAT-005] Feasibility 결과 기반 ADR-0019 acceptance·default-off 유지·중단 조건 확정

### B. Canonical protocol

- [x] [NAT-006] Immutable event revision과 `EVENT-CURRENT.toml` 단일 linearization point
- [x] [NAT-007] Run ACL·role assignment·single-action authority의 external trust root·Ed25519 발급·회수·one-time consume
- [x] [NAT-008] `claim|launch-ack|heartbeat|lookup|non-launch-proof|cancel-ack|final-result` typed receipt schema
- [x] [NAT-009] `reserved|prepared|claimed|dispatch-uncertain|acknowledged|running|cancel-requested|result-received|expired|quarantined` 전이 reducer
- [x] [NAT-010] Normal cancel event commit과 corrupt-head `EMERGENCY-CANCEL.toml` 승격·복구
  - Evidence: reapply commits `a0d86aa`·`22d88bc`·`c11ea2a`; core reducer·authority 8 PASS,
    CLI authority·head·emergency·migration 5 PASS

### C. Core·CLI

- [x] [NAT-011] Deterministic priority·tie-break·aging·starvation bound·quota·backpressure scheduler core
- [x] [NAT-012] Lease fencing epoch·clock skew·not-before·budget reservation/refund·safe reclaim
- [x] [NAT-013] Event replay·snapshot·bounded segment·crash recovery와 derived projection rebuild
- [x] [NAT-014] `hive orchestration` status·plan·dispatch·receipt·cancel·recover·authority CLI
- [x] [NAT-015] Legacy `hive orchestration migrate --from-run ... --dry-run|--apply|--recover`와 원본 byte 불변
  - Evidence: reapply commits `a0d86aa`·`22d88bc`·`c11ea2a`; `cargo check -p hive-cli` PASS,
    core scheduler·lease·migration·receipt tests 8 PASS, CLI mutation tests 5 PASS

### D. Host·Skill

- [ ] [NAT-016] 세 host declarative envelope adapter, exact role receipt와 reserved Judge profile 소비
- [x] [NAT-017] Ralph급 persistent criterion loop와 criterion terminal Judge gate의 `iterative-execution` Skill
  - Evidence: commit `b6679a6`, usage gate→event/receipt→criterion evidence→terminal-only
    independent Judge 흐름과 direct spawn·provider API·OMX/OMC 금지 제품 Skill
- [x] [NAT-018] Mailbox·barrier·shared-path lease·lane cancel·goal terminal Judge 기반 `team-execution` Skill
  - Evidence: commit `b6679a6`, immutable message dedupe/conflict·sender sequence·bounded bytes,
    exact membership barrier·ASCII casefold/path overlap·failed lane 정책 core 시험
- [x] [NAT-019] AND·OR·quorum criterion·budget·terminal lattice·nested cancel·goal/aggregate Judge gate 기반 `multi-goal` Skill
  - Evidence: commit `b6679a6`, aggregation·verified evidence·terminal Judge, parent allocation·single
    refund·nested cancel 규칙 core 시험과 제품 Skill projection
- [ ] [NAT-020] Planning·review·QA·research·performance loop parity와 explicit/implicit 무관 strict Judge 정책 통합

### E. Qualification·activation

- [ ] [NAT-021] Stale pointer·wrong session·100회 Stop no-op·cancel/guard 독립 접근 회귀 시험
  - 구현 증거: stale checkpoint의 `CURRENT.md` no-mutation, wrong usage session의 prepare
    no-mutation, `Stop` neutral payload 100회 반복 회귀. cancel/guard의 selected pointer 독립
    control-plane E2E는 host lifecycle 수용과 함께 계속 확인
- [x] [NAT-022] Ack loss·duplicate/late receipt·two-scheduler race·cancel-vs-consume·clock rollback property 시험
  - Evidence: `dispatch-uncertain`에서 authenticated non-launch proof 전 reprepare·final result
    거부, duplicate/conflict receipt·late result·clock rollback/refund replay, concurrent prepare의
    단일 executable response. core 9·loop CLI 20·strict Clippy PASS
- [ ] [NAT-023] Team barrier·mailbox dedupe·path overlap·multi-goal budget·migration partial publish E2E
- [ ] [NAT-024] Clean clone·세 host·보안·관찰성 gate 뒤 default activation 결정과 신규 OMX·OMC 경로 제거

## 수락 기준

- `NAT-001–024` evidence-backed 완료
- Provider SDK·endpoint·credential locator·model/subagent process spawn `0건`
- 신규 workflow의 OMX·OMC command·namespace·runtime dependency `0건`
- Canonical JSON lease `0건`; Markdown/TOML 정본과 derived JSON 분리
- Normal mutation의 exact target·expected head·control epoch·authority·request digest 결합 `100%`
- 잘못된 selected pointer와 Stop 100회에서 canonical mutation `0건`
- Pointer mismatch 중 exact-ID cancel·status·recover·usage guard control 성공
- Host idempotency proof 부재의 automatic reclaim `0건`
- Authenticated Judge 없이 strict criterion·goal 완료 전이 `0건`; usage 제한은 pending 중지
- Legacy run migration 뒤 원본 owner·foreign bytes 변경 `0건`
- 기능 activation 전 feasibility·ADR acceptance·schema·security gate 완료

## 중단 조건

- 세 host 모두 exact envelope consume 또는 result/cancel receipt 제공 불가
- Provider API·credential·direct process spawn 없이는 필수 lifecycle 구현 불가
- Canonical single-commit 복구나 independent cancel control 보장 불가
- 외부 runtime byte import 없이는 parity 확보 불가

중단 시 결과: 지원 가능한 data-only 기능 유지, unsupported capability 공개, runtime 성공 주장 `0건`.

## Pre-mortem

| 실패 시나리오 | 조기 신호 | 완화 |
| --- | --- | --- |
| Scheduler의 hidden provider runtime 팽창 | Host token·SDK·process launcher 요구 | Declarative envelope 경계와 static dependency gate |
| Pointer mismatch의 control-plane 재봉쇄 | Cancel·guard가 selected session 해석에 의존 | Exact target·authority·event head와 emergency cancel 독립 경로 |
| Team·multi-goal 병렬 상태 손상 | 중복 lease·late receipt·path overlap | Event CAS·fencing·quarantine·path canonicalization·property 시험 |

## Commit 순서

1. Host feasibility evidence
2. Capability parity inventory
3. Proposed boundary·plan·ADR
4. Schema·reducer·authority
5. Control CLI·cancel·recovery
6. Host adapter·Skills
7. Migration·qualification·activation

각 concern의 nearest verification 뒤 독립 commit. Feasibility 전 ADR acceptance·runtime schema/core activation 금지.
