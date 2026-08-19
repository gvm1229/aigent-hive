# 7. Hive-native orchestration workflow 계약

## Team

- Immutable mailbox message ID·sender·recipient·sequence·digest
- Duplicate message no-op, conflicting bytes quarantine
- Message size·count·retention 상한과 acknowledged sequence
- Barrier membership revision, quorum, timeout, failed-lane semantics
- Sender authority와 lane별 action capability
- Shared path canonicalization: casefold·Unicode·symlink·parent-child overlap 검사
- Stable lock ordering과 crashed-lane lease recovery
- Parent cancel의 child lane fan-out, late result quarantine
- Executor와 verifier role 분리

## Multi-goal

- Criterion 단일 ownership과 parent-child provenance
- Aggregation: `AND|OR|quorum`
- Terminal lattice: `pending < executing < verifying < complete`, 독립
  `blocked|failed|cancelled|quarantined`
- Partial success·failure의 parent 전파 규칙
- Decomposition 변경의 user·planner authority와 새 event revision
- Parent budget reservation, child allocation, unused refund
- Nested team cancel·rollback·evidence 보존
- Goal 완료: 모든 required criterion의 verified evidence 충족

## CLI surface

```text
hive orchestration status --target <project-root> --run <id> --output json
hive orchestration plan --target <project-root> --run <id> --request <json> ...
hive orchestration dispatch --target <project-root> --run <id> --action <id> ...
hive orchestration receipt --target <project-root> --receipt <json> ...
hive orchestration cancel --target <project-root> --run <id> --reason <text> ...
hive orchestration recover --target <project-root> --run <id> ...
hive orchestration authority issue|revoke --target <project-root> ...
hive orchestration migrate --target <project-root> --from-run <legacy-id> --dry-run|--apply|--recover ...
```

모든 mutation 명령 공통 입력:

```text
--expected-head <sequence:sha256|none>
--control-epoch <n>
--authority <locator#sha256>
--request-digest <sha256>
```

## Legacy migration

- 원본 external owner·foreign bytes: read-only provenance
- Dry run: validated source inventory, unsupported fields, new native run preview, mutation `0건`
- Apply: 별도 staging generation과 새로운 native run ID
- Receipt: source owner digest, source locator digest, target generation, migrated/rejected field map
- Recovery: `RECOVERY.toml` locator와 exact staged generation
- Commit: target event head CAS 뒤 `MIGRATION.md` provenance publish
- Partial publish: target generation rollback 또는 forward recovery; source byte 변경 `0건`
- In-place owner 변경 금지

## Host feasibility gate

세 host별 측정 항목:

| Capability | 판정 |
| --- | --- |
| Envelope consume 직전 head·epoch 재검증 | `supported|best-effort|unsupported|unverified` |
| Idempotency key single-consume | 동일 |
| Claim·launch ack·heartbeat·result receipt | 동일 |
| Exact lookup·non-launch proof | 동일 |
| Cancel request·cancel ack | 동일 |
| Native task identity·qualified provenance | 동일 |

- 최소 한 host의 complete lifecycle 실제 proof
- 나머지 host의 fixture + truthful capability matrix
- Complete lifecycle proof 부재: ADR acceptance·runtime activation 중지
- Host API가 아닌 Skill-produced declarative envelope 소비
- Host-global configuration 자동 mutation `0건`

## Skill suite

| Skill | 역할 |
| --- | --- |
| `hive-iterative-execution` | Criterion loop, retry budget, verification, steering, completion |
| `hive-team-execution` | Lane·mailbox·barrier·shared-path lease·cancel |
| `hive-multi-goal` | Goal decomposition·aggregation·budget·nested execution |
| `hive-loop-engineering` | Existing graph authoring·validation의 thin compatibility route |
| Planning | `ralph-loop` 뒤 `iterative-execution`의 criterion·receipt·terminal Judge 경로 |
| Review·QA | `package-review` 뒤 `iterative-execution`의 독립 검증·terminal Judge 경로 |
| Research | `research-best-practices`의 근거 handoff 뒤 동일 evidence·terminal Judge 경로 |
| Performance | 명시 measurement criterion의 `iterative-execution` 경로 |

기능 inventory 제외 사유: `unsafe|provider-specific|redundant|non-useful`만 허용.
`ownership-collision` 단독 제외 금지.

- Strict terminal Judge: `explicit|implicit` 설정과 무관한 planning·review·QA·research·performance
  criterion의 reserved independent Judge·외부 signature 요구
- `implicit`: 그 밖의 strict material-risk route 추가 허용

## 검증

### Unit·property

- Reducer determinism·illegal transition
- Event publish·head CAS 모든 crash point
- Two-scheduler CAS race와 fencing
- Cancel-vs-claim·cancel-vs-consume permutation
- Ack loss·duplicate result·late receipt·receipt conflict
- Clock rollback·lease expiry·safe reclaim
- Team mailbox·barrier membership random sequence
- Multi-goal aggregation·budget·cancel fan-out random sequence

### Integration·E2E

- Wrong selected pointer + exact active target
- Stop 100회 + canonical mutation `0건`
- Pointer mismatch 중 cancel·guard disable/restore·recover 접근
- Claim 뒤 host silence → `dispatch-uncertain`
- Non-launch proof 없는 automatic reclaim `0건`
- Legacy migration dry-run·partial apply·recover·rollback
- Three-host capability별 exact claim table

### Security·observability

- Authority forgery·expiry·revocation·nonce replay
- Target-contained trust root·writable issuer 거부
- Event sequence·head digest·control epoch·lease·receipt·quarantine metric
- Raw prompt·transcript·credential·provider payload persistence `0건`
- Provider SDK·endpoint·credential locator·process spawn static finding `0건`

## Activation·rollback

- 초기 feature flag: `off`
- Shadow reducer·receipt validation 우선
- Host별 capability claim 범위 밖 실행 금지
- Schema·event generation backward read 지원
- Activation 실패: data-only graph·manual recovery 경로 유지
- OMX·OMC 신규 dependency 복귀 금지
- Legacy foreign state 삭제 금지, ordinary Git history와 migration provenance 보존

## 완료 중단점

- Active checklist `NAT-*` 전체 완료
- ADR-0019 `accepted`
- 21개 이상의 measurable acceptance와 hostile/property/E2E 통과
- Clean clone·full Rust·full Python·세 host projection 검증
- Provider boundary·authority·cancel 독립성 finding `0건`
- 신규 workflow의 OMX·OMC functional dependency `0건`
