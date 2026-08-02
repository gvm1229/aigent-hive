# ADR-0019: Hive-native 반복 실행 소유권

- 상태: proposed
- 날짜: 2026-08-03
- 대상: `0.9.0`
- 정책 전환: ADR-0015의 scheduler·Ralph·team 재구현 금지
- Historical provenance: [`ADR-0004`](ADR-0004-orchestration-ownership.md), [`ADR-0015`](ADR-0015-host-native-skill-composition.md)

## 배경

- 기존 비구현 이유: 핵심 기술 불가능보다 외부 orchestration owner와의 의도 충돌
- 새 사용자 결정: OMX·OMC 의존성 완전 제거와 유용 기능의 Hive clean-room 구현
- 추가 사용자 결정: Sol Advisor 기능 동등성, Codex·Claude task별 custom subagent와
  exact model·thinking level 사전 고정, 목적 기반 custom-agent 생성·자동 route를 `0.9.0`에 포함
- 추가 사용자 결정: Ed25519 Judge를 Sol/Claude exact-model reserved custom agent로 전환하고,
  setup에서 `explicit|implicit` 호출 정책 선택. Strict workflow는 선택값과 무관하게 Judge 강제
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

## Model-routed custom subagent 결정

- 지원 host: OpenAI Codex·Claude Code 한정. Antigravity는 이 기능에서 `unsupported`
- Hive 소유: provider-neutral role 정본, task trigger·negative route, 양쪽 host model/effort,
  user/project scope, projection digest, runtime attestation acceptance
- Host 소유: custom-agent discovery, native dispatch, model call·session·process, runtime metadata
- Functional baseline: primary orchestrator, routine/complex implementer, reserved read-only
  independent Judge, exact role·model·effort·definition digest gate
- 자동 선택: Hive Skill과 같은 narrow semantic description route. 별도 classifier hook 없음
- Model authority: 검증된 exact ID·effort만 활성화, floating alias·silent fallback 결과 수용 금지
- Scope authority: user·project 모두 지원, project 우선. Host file은 preview·명시적 동의·
  Hive ownership digest 일치 때만 생성·교체
- On-demand 생성: 목적 질문→이름·Codex/Claude model/effort·scope·권한 추천→
  `1 수락 | 2 수동 | 3 수정`→검증·projection·auto-route 통합
- Judge authority: user-scope reserved definition만 허용, project shadow·생성 Skill override 금지.
  Codex 후보는 `gpt-5.6-sol/max`; Claude exact profile은 실제 lifecycle 검증 뒤 활성화
- Invocation policy: user setup의 `explicit`은 strict workflow terminal gate만, `implicit`은
  strict gate와 일반 material-risk route. 초기 custom setup 질문과 natural-language reconfigure 지원
- Strict boundary: iterative·team·multi-goal criterion·goal terminal acceptance만 Judge 강제,
  scheduler tick·heartbeat·retry별 호출 금지. Usage 제한 시 성공 우회 없이 pending 중지
- Trust boundary: Agent는 verdict만 생성, 외부 signer가 Ed25519 private key 소유.
  Hive는 assignment·role·exact model/effort·definition digest 결합 서명과 quorum만 검증
- Canonical plan:
  [`model-routed-custom-subagents.md`](../plans/active/model-routed-custom-subagents.md)

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
7. Codex·Claude user/project custom-agent fresh-session lifecycle 실제 proof
8. Exact role·model·effort·scope·definition digest runtime receipt와 mismatch fail-closed
9. Purpose-first 생성 Skill의 3개 decision path와 생성 role automatic route 검증
10. Foreign·user-authored host config overwrite `0건`
11. 두 Judge mode setup·natural-language reconfigure와 strict workflow 강제 terminal gate 검증
12. Project Judge shadow·policy downgrade·signer/model mismatch fail-closed

## 결과

- ADR-0015의 현재 data-only graph는 구현 baseline으로 유지
- Scheduler·Ralph·team 비구현 정책은 신규 개발 지침에서 제거
- Runtime activation은 feasibility 전 금지
- 기존 `V9-*` 완료 상태 재개방 없음; 신규 `NAT-*` fragment 소유
- Exact-model custom subagent는 신규 `MRA-*` fragment 소유, `NAT-016` host adapter와 연계
- Legacy owner in-place 전환 없음

## Rollback

- Activation 이전: 기존 prepare-only graph와 manual recovery 유지
- Shadow mismatch: 신규 dispatch 중지, canonical event·receipt 보존
- Adapter failure: capability별 unsupported 또는 `dispatch-uncertain`
- Foreign state: 삭제·수정 없음
- OMX·OMC 의존성 복귀 없음
