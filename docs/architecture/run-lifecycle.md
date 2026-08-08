# Durable run and loop lifecycle

## 소유 경계

- Hive 소유: `.hive/runs/<run-id>/`의 tracked Markdown, immutable graph revision,
  evidence 검증, checkpoint·resume·steering·prepare 계약과 예정된 event reducer·logical
  scheduler·lease·receipt·cancel·team·multi-goal state
- Host 소유: model 실행, subagent 생성, native task identity와 declarative envelope 소비
- Hive 금지 범위: model call, model·subagent process spawn, provider session daemon,
  provider API·credential, `omx|omc` command 호출

현재 구현 baseline: fresh evidence로 검증된 `host-native` owner의 prepare-only dispatch.
예정된 native control plane: ADR-0019 feasibility·acceptance 전 default-off.

| Run 상태 | Owner 결정 | 부족한 필수 capability |
| --- | --- | --- |
| 현재 새 v0.9 run | `host-native` prepare-only 기본값 | `unsupported|unverified` → `host_capability_unsupported`, exit `4`, mutation 0건 |
| 신규 native orchestration | Hive control + host executor | Feasibility·ADR acceptance 전 default-off |
| 기존 run | `STATUS.md`에 고정된 owner 유지 | 중간 owner 전환 없음 |
| 기존 0.8.x external run | 고정 owner의 read-only provenance | 명시적 migration은 새 native run identity 생성 |

## Canonical artifact

```text
.hive/runs/<run-id>/
├── PLAN.md
├── STATUS.md
├── HANDOFF.md
├── evidence/
└── graph/
    ├── CURRENT.md
    ├── revisions/
    │   └── <16-digit-revision>.md
    └── prepared/
        └── <usage-authorization-id>.json
```

- `PLAN.md`: checkbox criterion ID 기준의 required acceptance 정본
- `STATUS.md`: `schemas/run-status.schema.json` 기준의 canonical JSON-compatible
  frontmatter와 exact Markdown body
- `HANDOFF.md`: 여러 active role이 공유하는 역할별 handoff envelope;
  [`role-lifecycle.md`](role-lifecycle.md)의 strict RFC 3339 `date-time` 계약
- `evidence/`: checkpoint와 graph transition이 참조하는 bounded local file
- `graph/revisions/`: content-bound immutable graph revision chain
- `graph/CURRENT.md`: exact current revision과 digest pointer
- `graph/prepared/`: host-owned dispatch 직전의 one-time authorization-bound data record
- Resume 정본 제외 대상: SQLite, transcript, host runtime state

Evidence locator의 exact 형식:

```text
.hive/runs/<run-id>/evidence/<safe-relative-file>#sha256:<64-lowercase-hex>
```

각 passed criterion의 `criterion_evidence` 필수. Checkpoint, resume, loop validation과
prepare에서 referenced file의 no-follow bounded read와 exact digest 재계산. Missing,
unsafe, stale 또는 tampered evidence: status write·graph transition·dispatch preparation
차단.

## Loop graph lifecycle

| Command | 현재 계약 |
| --- | --- |
| `hive loop initialize` | revision 1, `CURRENT.md`, immutable graph source의 원자적 초기화 |
| `hive loop validate` | current pointer·revision chain·DAG·evidence·terminal invariant 검증 |
| `hive loop checkpoint` | attempt, evidence, retry 또는 terminal outcome의 새 immutable revision |
| `hive loop steer` | user-bound proposal과 영향 edge를 포함한 명시적 graph revision |
| `hive loop prepare` | exact node·retry·steering dispatch의 data-only 준비 |
| `hive loop recover` | bounded revision chain과 current state의 fresh-session 복구 |

Graph static gate:

- cycle·self-edge·unreachable node·orphan acceptance criterion 거부
- success edge마다 exact evidence predicate와 독립 verification role 필수
- node별 bounded retry, deterministic backoff, 동일 failure fingerprint 반복 중지
- terminal outcome: `blocked|failed|complete`
- steering 필수 결합: base revision·reason·affected edge·user boundary·proposal digest
- steering에 의한 기존 revision 수정 없음; 새 immutable revision만 허용
- 전체 revision-chain recovery 상한: 32 MiB

## Fresh capability input

Checkpoint, resume와 loop prepare의 명시적 fresh capability JSON 필수. Capability path의
no-follow regular-file preflight, nonblocking·no-follow open, opened handle의 type·identity,
size·`mtime`·read length 안정성 검증. 미래 `mtime` 또는 60초 이상 stale evidence:
JSON parse, owner resolution·continuity, status write와 dispatch preparation 전 거부.

Graph의 capability snapshot과 current host resolution의 exact digest 결합. Required
capability의 minimum support 미달: exact `host_capability_unsupported`, prepared record
및 다른 mutation 0건.

## Run checkpoint

`hive run checkpoint`: request, fresh capability resolution, `PLAN.md`, active role과 shared
handoff 검증 뒤 `STATUS.md` 하나의 optimistic transaction.

- 첫 status: `expected_revision: 0`; 이후 update: current revision exact match 필수
- 동일 request retry: byte-identical no-op
- 같은 revision의 다른 bytes 또는 revision 손실: conflict, write 0건
- `succeeded`: 모든 required criterion passed, failed criterion 0건, 각 pass의 verified
  evidence 필수
- Owner field: request 입력 제외; 첫 checkpoint의 fresh capability evidence에서 자동 pin

Owner pin의 full object: `host`, `host_version`, `surface`, `external_runtime`,
`resolved_owner`, `resolution_evidence_digest`, `subagent_support`. 이후 checkpoint,
resume와 loop prepare에서 불변.

신규 `STATUS.md`와 shared `HANDOFF.md`의 첫 publish: exclusive hard-link primitive,
published inode 확인, foreign racing destination 보존, exact recovery link 또는 rollback.

## Owner continuity

Run 도중 fresh evidence와 pin 불일치 시 owner 교체 없음.

- pinned external runtime missing 또는 `unknown`: blocked, exit `3`
- pinned external runtime incompatible: unsupported, exit `4`
- host·version·surface 또는 full resolution digest drift: blocked, exit `3`
- malformed capability 또는 digest mismatch: verification failure, exit `5`
- 모든 실패: `changed_paths: []`; canonical run·role·handoff·evidence와 `.omx/.omc`
  namespace 불변

## Resume와 one-time dispatch authorization

`hive run resume`: canonical PLAN·STATUS·active role·handoff·evidence의 read-only 검증과
provider-neutral recovery data 생성. Automatic intent에서만 Git-ignored usage history와
dispatch authorization record의 bounded 갱신 허용.

| Durable state 또는 support | 결과 |
| --- | --- |
| `executing`, `verifying` + minimum support 충족 | role별 `DispatchBrief`, `prepared_only: true`, `spawned: false` |
| `executing`, `verifying` + `unsupported|unverified` | exit `4`, brief·spawn data 0건 |
| `blocked`, `usage-limited` | exit `3`, recovery data만 반환 |
| `resume-ready` | recovery data만 반환; hidden transition 없음 |
| `succeeded`, `cancelled` | terminal recovery data만 반환; continuation 없음 |

Manual intent: CodexBar와 usage runtime record read/write 0건, prepare-only brief만 반환,
usage enforcement 주장 없음. Automatic intent: SHA-256 account digest, current host
session·process, active role ID와 installed `usage_stop_remaining_percent`의 exact binding.
Optional threshold 인수는 installed 값과 일치할 때만 허용.

Automatic dispatch boundary:

1. `hive usage enforce`: current session의 halt·threshold preflight
2. `hive run resume --dispatch-intent automatic`: exact run revision·role·brief 하나에
   one-time authorization 하나 발급
3. `hive loop checkpoint`: exact authorization locator·digest와 usage evidence의 graph
   revision 결합
4. `hive loop prepare`: current graph·전체 evidence·STATUS owner·active role·fresh
   capability·authorization의 optimistic 재검증
5. Host: exact prepared binding의 실제 실행과 authorization ID 단일 소비

Session window 우선, session 부재 시 exact single weekly fallback. History 위치:
`.hive/runtime/usage-history/<account-digest>.json`. Measurement/reset 역행, 동일 reset의
remaining 증가, malformed·tampered·symlink history: `usage_unknown` fail-closed.

Authorization 위치:
`.hive/runtime/dispatch-authorizations/<authorization-id>.json`. 동일 binding 재발급:
`already_issued`, exit `3`, brief 0개. Limited·unknown·expired permit: exit `3`, brief 0개.

Issued claim의 범위: Hive command의 같은 결과 재발급 차단. 이미 capture된 JSON의 Hive
외부 replay에 대한 cryptographic 방어는 범위 밖. Host dispatch boundary의 authorization
ID 단일 소비 필수.

## Prepare-only host dispatch

`hive loop prepare` 결과:

- `.hive/runs/<run-id>/graph/prepared/<usage-authorization-id>.json`의 durable record
- `prepared_only: true`, `spawned: false`
- graph revision·graph digest·dispatch kind·node 또는 steering·attempt·role·brief digest
  결합
- capability snapshot·fresh resolution digest, usage evidence ID, run dispatch
  authorization digest 결합
- publish 전후 `CURRENT.md`, `STATUS.md`, capability, evidence, authorization의 optimistic
  recheck
- publish 후 drift: 새 prepared record rollback
- 동일 exact request: idempotent no-op

Hive의 host process·model·subagent 실행 0건. Prepared record는 실행 명령이 아닌 exact
host-owned dispatch data.

## Hive-native orchestration 전환

현재 prepare-only graph 위의 default-off 확장 계획:

- Immutable event revision + `EVENT-CURRENT.toml` 단일 commit
- Exact target·expected head·control epoch·authenticated one-time authority
- Typed claim·launch·heartbeat·lookup·non-launch·cancel·result receipt
- `dispatch-uncertain`과 proof-gated safe reclaim
- Logical scheduler·lease fencing·budget·backpressure
- Team mailbox·barrier·shared-path lease와 multi-goal aggregation
- Cancel·status·recover·usage guard의 selected pointer·scheduler lock 독립 접근

정본 계획: [`../plans/active/native-iterative-execution.md`](../plans/active/native-iterative-execution.md).
Protocol: [`../plans/contracts/06-native-orchestration-state.md`](../plans/contracts/06-native-orchestration-state.md),
[`../plans/contracts/07-native-orchestration-workflows.md`](../plans/contracts/07-native-orchestration-workflows.md).

## Legacy external provenance

기존 pinned OMX/OMC owner의 foreign bytes·owner metadata: read-only provenance.
신규 workflow의 OMX/OMC 선택·command·runtime dependency 없음. Explicit migration은
별도 native run identity·staged generation·receipt·rollback locator 사용, 원본 변경 `0건`.

## 검증 계약

- 모든 action과 실패: `schemas/action-result.schema.json`
- Role, handoff, checkpoint, status, capability, dispatch brief와 loop request:
  Draft 2020-12 schema·format validation
- Source root·traversal·symlink·directory·FIFO·unreadable·oversized input의 bounded 거부
- Project와 fake HOME의 `.omx/.omc` foreign sentinel 불변
- Fresh-session recovery: transcript·SQLite 없이 tracked canonical artifact만 사용
- Automatic resume write 범위: Git-ignored Hive-owned usage history와 dispatch authorization
- Loop prepare write 범위: exact graph prepared record 하나; spawn·continuation 0건
- Ambient·selected pointer authority `0건`
- Provider API·credential·direct model/subagent process spawn `0건`
- Native scheduler activation 전 feasibility·ADR acceptance 필수
