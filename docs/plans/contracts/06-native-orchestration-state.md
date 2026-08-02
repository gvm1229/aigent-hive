# 6. Hive-native orchestration 상태 계약

## 소유 경계

| 소유자 | 범위 |
| --- | --- |
| Hive | Event reducer, logical scheduler, lease·budget, authority, receipt, cancel, team·goal ledger |
| Host | Model call, model·subagent process, native task identity, envelope consume, lifecycle receipt |
| User·external trust root | Authority issuer·revoker public identity와 private-key custody |

금지 범위: provider SDK·endpoint·API key·host session token 저장, direct model·subagent spawn,
ambient session pointer authority, foreign OMX·OMC runtime mutation.

## Canonical tree

```text
.hive/runs/<run-id>/
├── PLAN.md
├── STATUS.md
├── AUTHORITY.md
├── CONTROL.md
├── EVENT-CURRENT.toml
├── events/
│   ├── revisions/<sequence>.md
│   ├── authorities/<authority-id>.toml
│   ├── segments/<segment-id>.md
│   └── snapshots/<sequence>.md
├── leases/<lease-id>.md
├── receipts/<receipt-id>.md
├── team/
├── goals/
└── evidence/
```

- 정본: Markdown과 tracked TOML
- JSON: Host envelope·`ActionResult`·derived cache 한정
- 유일한 정상 commit 지점: immutable event publish 뒤 `EVENT-CURRENT.toml` generation·sequence·digest CAS
- `STATUS.md`, `CONTROL.md`, `AUTHORITY.md`, lease·team·goal 문서: committed head 기반 materialized projection
- Projection digest 불일치: 신규 dispatch 차단과 deterministic rebuild
- Event 보존: bounded immutable segment, snapshot provenance, replay 상한, retention benchmark 후 schema freeze

## Authority

Canonical trust 경로:

```text
<user-root>/trust/orchestration-root.toml
.hive/config/harness.toml
.hive/roles/<role-id>.md
.hive/runs/<run-id>/AUTHORITY.md
.hive/runs/<run-id>/events/authorities/<authority-id>.toml
```

- Root: consumer target 밖의 agent-write-denied Ed25519 public-key trust root
- Project binding: exact project identity·root digest·allowed issuer
- Run ACL: principal·role·allowed action·target·validity·revocation head
- Single-action authority: target, expected head, control epoch, action, request digest, nonce,
  expiry, issuer, detached signature
- 모든 mutation 필수 입력: `--target --expected-head --control-epoch --authority --request-digest`
- Authority 소비: event-head CAS 성공과 같은 generation에서 one-time consumed event 기록
- 실패 CAS·expiry·revocation·nonce replay: mutation `0건`
- Target-contained trust root와 caller-writable key: 검증 거부

## Dispatch 상태

| 상태 | 진입 증거 | 다음 상태 | 제한 |
| --- | --- | --- | --- |
| `reserved` | Budget·priority reservation event | `prepared|expired` | Dispatch 불가 |
| `prepared` | Exact envelope digest·authority | `claimed|cancel-requested|expired` | Host consume 전 head 재검증 |
| `claimed` | Typed claim receipt | `acknowledged|dispatch-uncertain|cancel-requested` | 같은 idempotency key 재claim 금지 |
| `dispatch-uncertain` | Claim 뒤 launch 결과 불명 | `acknowledged|cancel-requested|quarantined` | Proof 없는 reclaim 금지 |
| `acknowledged` | Launch ack·native task identity | `running|cancel-requested|result-received` | Fencing token 필수 |
| `running` | Heartbeat·lookup evidence | `cancel-requested|result-received|quarantined` | Stale heartbeat 성공 오표시 금지 |
| `cancel-requested` | Committed cancel event | `result-received|quarantined` | Late launch·receipt quarantine |
| `result-received` | Final result receipt·evidence | Terminal reducer 결과 | Verifier 이전 success 금지 |
| `expired` | Lease deadline event | `prepared|quarantined` | Safe reclaim proof 필수 |
| `quarantined` | Conflict·late·invalid receipt | Explicit recover | 자동 dispatch 금지 |

## Typed host receipt

Schema: `schemas/host-orchestration-receipt.schema.json`의 discriminated `oneOf`.

| Kind | 필수 내용 | 주요 전이 |
| --- | --- | --- |
| `claim` | Idempotency key, fencing token, consumer identity | `prepared → claimed` |
| `launch-ack` | Native task identity, launch time | `claimed|dispatch-uncertain → acknowledged` |
| `heartbeat` | Native task identity, progress sequence | `acknowledged → running` 또는 liveness 갱신 |
| `lookup` | Exact key lookup result·provenance | Uncertain reconciliation |
| `non-launch-proof` | Qualified host proof | Safe reclaim eligibility |
| `cancel-ack` | `cancelled|not-found|too-late` | Quiescent 또는 quarantine |
| `final-result` | Outcome, evidence locator, native task binding | `running|acknowledged → result-received` |

공통 필드: receipt ID, run·action ID, exact event head, control epoch, idempotency key,
fencing token, native task identity, host capability digest, source locator·digest, issued/received
time, signature 또는 qualified provenance.

- Exact duplicate: no-op
- 같은 ID의 다른 bytes: conflict·quarantine
- Cancel 뒤 launch·result: late receipt quarantine
- `lookup absent` 단독 safe reclaim 금지
- Exactly-once claim: host의 single-consume·lookup·receipt attestation qualification 뒤에만 사용

## Cancel

- 정상 경로: cancel event artifact publish → `EVENT-CURRENT.toml` CAS
- `CONTROL.md`: committed `control_epoch`, desired state, actor, reason, request digest projection
- Consume 직전 host 재검증: exact event head·control epoch·authority
- Cancel CAS 승리 뒤 stale prepared envelope launch 금지
- Valid head + malformed `CONTROL.md`: projection rebuild
- Corrupt·ambiguous head: 별도 인증된 `EMERGENCY-CANCEL.toml` exclusive intent
- Emergency 승격: predecessor 복구 → intent 검증 → 정상 cancel event publish → head CAS →
  `CONTROL.md` materialize → promoted digest 기록·exact cleanup
- Status·cancel·recover·usage guard: scheduler lock·derived index·selected pointer 비의존

## Lease·scheduler

- Priority: explicit priority → not-before → aging → stable run/action ID tie-break
- Starvation bound와 per-run·global quota
- Lease issuer fencing epoch와 monotonic receipt sequence
- Trusted wall clock + monotonic elapsed time, clock rollback quarantine
- Atomic budget reservation·refund와 nested goal budget 상한
- Backpressure: global active lease·host lane·usage allowance 상한
- Reclaim: host-qualified `non-launch-proof` 또는 safe-reclaim capability만 허용
