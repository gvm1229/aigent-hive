# ADR-0019: Hive-native 반복 실행 소유권

- 상태: proposed
- 날짜: 2026-08-02
- 대상: `0.9.x` 이후 구현
- 정책 전환: ADR-0015의 scheduler·Ralph·team 재구현 금지
- Historical provenance: [`ADR-0004`](ADR-0004-orchestration-ownership.md), [`ADR-0015`](ADR-0015-host-native-skill-composition.md)

## 배경

- 기존 비구현 이유: 핵심 기술 불가능보다 외부 orchestration owner와의 의도 충돌
- 새 사용자 결정: OMX·OMC 의존성 완전 제거와 유용 기능의 Hive clean-room 구현
- 직접 계기: selected session pointer와 실제 session ID 불일치로 Stop 권한 실패,
  반복 no-op 응답, cancel·usage control 복구 방해
- 기존 기반: immutable graph revision, evidence, retry data, steering, prepare-only envelope,
  session·process bound usage control

## 결정

- Hive 소유: iterative judgment, deterministic reducer, logical scheduler, lease·budget,
  authority, receipt, cancel·recover, team mailbox·barrier, multi-goal ledger, quality gate
- Host 소유: model call, model·subagent process, native task identity, declarative envelope consume
- 신규 workflow의 OMX·OMC 실행·명령·namespace dependency 없음
- Legacy external-owner run: read-only provenance, explicit migration의 새 native identity
- Provider API·SDK·credential·direct model/subagent process spawn 없음
- Feature 기본값: `off`; host feasibility와 qualification 뒤 별도 activation 결정

## 선택지

| 선택지 | 판정 | 이유 |
| --- | --- | --- |
| Stop hook 중심 continuation | 기각 | Ambient pointer·hook session authority 재발 위험 |
| Hive daemon의 model/session 직접 소유 | 기각 | Subscription host·credential·provider boundary 위반 |
| Cooperative scheduler + host envelope | 채택 | Provider-neutral durable control과 host 실행 경계 양립 |
| Host별 Skill의 독립 loop state | 기각 | Canonical state 분산·migration·verification drift |

## 핵심 긴장과 해법

- 긴장: Hive의 durable 판단·복구성과 host의 opaque inference/session transport
- 한계: Host idempotency·receipt·cancel guarantee 부재 상태의 exactly-once·강제 cancel 보장 불가
- 해법: Intent·event·fencing·receipt·truthful uncertainty는 Hive 소유, 실제 실행은 host 소유
- Ack 유실: safe-reclaim proof 부재 시 `dispatch-uncertain` 중지
- Cancel race: consume 직전 exact head·control epoch 재검증과 late receipt quarantine

## Authority 결정

- Selected session pointer: target selector만 허용, authority 사용 금지
- Mutation authority: exact target, expected event head, control epoch, request digest,
  external trust root로 검증한 one-time action capability
- 정상 cancel: cancel event publish 뒤 `EVENT-CURRENT.toml` CAS 단일 commit
- `CONTROL.md`: committed head projection
- Corrupt head: 별도 인증된 emergency cancel intent와 정상 event 승격
- Cancel·status·recover·usage guard: scheduler lock과 pointer 독립 접근

## Canonical 결정

- 정본 형식: Markdown·tracked TOML
- 단일 linearization point: immutable event revision + `EVENT-CURRENT.toml`
- JSON: Host envelope·result·derived cache 한정
- Receipt kind: `claim|launch-ack|heartbeat|lookup|non-launch-proof|cancel-ack|final-result`
- Dispatch state: `reserved|prepared|claimed|dispatch-uncertain|acknowledged|running|cancel-requested|result-received|expired|quarantined`

## OMX·OMC 기능 채택

- 전체 기능 inventory 재개방
- 채택 방식: provider-neutral clean-room contract와 Hive naming·state model
- 제외 가능 사유: `unsafe|provider-specific|redundant|non-useful`
- 제외 불가 사유: `ownership-collision` 단독 근거
- 외부 source byte·runtime state·private prompt 직접 복사 없음

## Acceptance 전 필수 gate

1. Codex·Claude Code·Antigravity envelope·receipt·cancel·lookup feasibility matrix
2. 최소 한 host complete lifecycle 실제 proof
3. Event linearization·authority·typed receipt protocol freeze
4. Shadow reducer와 cancel/guard fail-safe qualification
5. Provider SDK·credential·process spawn static finding `0건`
6. Stale-pointer hostile E2E와 migration recovery 통과

## 결과

- ADR-0015의 현재 data-only graph는 구현 baseline으로 유지
- Scheduler·Ralph·team 비구현 정책은 신규 개발 지침에서 제거
- Runtime activation은 feasibility 전 금지
- 기존 `V9-*` 완료 상태 재개방 없음; 신규 `NAT-*` fragment 소유
- Legacy owner in-place 전환 없음

## Rollback

- Activation 이전: 기존 prepare-only graph와 manual recovery 유지
- Shadow mismatch: 신규 dispatch 중지, canonical event·receipt 보존
- Adapter failure: capability별 unsupported 또는 `dispatch-uncertain`
- Foreign state: 삭제·수정 없음
- OMX·OMC 의존성 복귀 없음
