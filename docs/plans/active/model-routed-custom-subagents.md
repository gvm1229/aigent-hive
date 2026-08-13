# v0.9.3 model-routed custom subagent 계획

> 대상: `0.9.3`
> 상태: current-tree evidence 재조정·구현 진행
> 결정: [`ADR-0019`](../../decisions/ADR-0019-hive-native-iterative-execution.md)
> 선행: `NAT-002–005`, 연계: `NAT-016`
> 지원 host: OpenAI Codex, Claude Code

## 목표와 경계

- Sol Advisor 역할 분리·검증 clean-room 동등성과 task별 exact-model 자동 선택
- Codex·Claude 필수 mapping의 neutral role 정본과 user/project scope
- Host-native discovery·dispatch, Hive provider API·credential·직접 process spawn `0건`

## 초기 built-in role 후보

`MRA-004–005` 검증 전 후보. 검증된 exact ID만 활성화.

| Role | Task route | Codex 후보 | Claude 후보 | 기본 권한 |
| --- | --- | --- | --- | --- |
| `hive-routine-implementer` | 반복 구현·정형 수정 | `gpt-5.6-luna` / `max` | `claude-sonnet-5` / `high` | bounded write |
| `hive-complex-implementer` | 복잡 구현·architecture 연계 | `gpt-5.6-terra` / `max` | `claude-opus-4-8` / `max` | bounded write |
| `hive-independent-judge` | 독립 acceptance·security 판정 | `gpt-5.6-sol` / `high` | `claude-opus-4-8` / `max` | read-only |
| `hive-design-specialist` | UX·visual·interaction 설계 | `gpt-5.6-sol` / `max` | `claude-opus-4-8` / `max` | read-only 기본 |
| `hive-article-writer` | article·guide·long-form 문서 | `gpt-5.6-sol` / `high` | `claude-sonnet-5` / `high` | bounded docs write |
| `hive-research-specialist` | 근거 수집·비교·synthesis | `gpt-5.6-terra` / `high` | `claude-sonnet-5` / `high` | read-only |

## 정본과 projection

- User: `~/.hive/config/custom-subagents/`; project: `.hive/config/custom-subagents/`
- Codex `.codex/agents/*.toml`, Claude `.claude/agents/*.md` projection
- 일반 role은 project 우선. Reserved Judge는 user-scope authoritative, project shadow 금지
- Scope·model/effort·permission·trigger·digest 정본, preview·동의·owned replace·foreign-byte 보존

## 자동 호출 흐름

`request → semantic route → exact profile → dispatch·receipt → evidence → 정책별 Judge·quorum → accept`

- Narrow description 단일-owner route, classifier hook·중복 호출 금지
- Simple·작은 단일 단계·unsupported·receipt 부재·불명확 목적 자동 호출 제외
- Deterministic verifier는 evidence 수집 layer이며 model review authority 없음

## Judge 호출 정책

- User setup `judge_invocation = explicit|implicit`; custom setup 질문, 자연어 변경은 typed setup preview·apply
- `explicit`: iterative·team·multi-goal criterion/goal terminal gate만. `implicit`: strict + material-risk route
- 단순 질문·read-only·format-only·결정적 실패 제외. Tick·heartbeat·retry별 호출 금지
- Existing·noninteractive 기본 `explicit`; 새 custom setup은 추론 금지
- Dispatch 전 usage guard. Strict gate 제한은 성공 우회 없이 pending·usage-limited
- Agent는 verdict만 생성. 외부 signer가 private key를 소유하고 Hive는 bound signature·quorum 검증

## `hive-custom-subagent-create` 대화 계약

1. 목적 뒤 `이름·양쪽 model/effort·scope·권한·trigger` 추천
2. `1 추천 수락 | 2 거절 후 수동 설정 | 3 수정 요청`
3. `1`: preview·digest·동의, `2`: 모든 field 순차 질문, `3`: 변경 field만 질문·재추천
4. 양쪽 검증·canonical 저장·projection·registry·fresh-session receipt

- 양쪽 mapping 누락 완료 금지, reconfigure·delete는 owned projection만 대상

## 실행 checklist

### A. Feasibility·결정

- [ ] [MRA-001] Sol Advisor의 orchestrator·routine·complex·review·attestation 기능 동등성 표와 clean-room 증거 확정
- [ ] [MRA-002] Codex user/project agent schema·precedence·discovery·model/effort·runtime metadata matrix 검증
- [ ] [MRA-003] Claude user/project agent schema·precedence·environment override·allowlist·fallback matrix 검증
- [ ] [MRA-004] 실제 Codex Luna·Terra·Sol install→fresh session→dispatch→attestation lifecycle spike
- [ ] [MRA-005] 실제 Claude exact model·effort install→fresh session→dispatch→override/fallback lifecycle spike
- [ ] [MRA-006] Codex·Claude 한정, Antigravity unsupported, default-off, host-file consent의 ADR·security acceptance

### B. Canonical role·projection

- [x] [MRA-007] 양쪽 host mapping·exact model/effort·trigger·negative route·permission·scope·digest 필수 typed schema
  - Evidence: commit `ed21b87`, closed profile schema·JCS definition digest·floating alias 거부와
    Codex TOML·Claude Markdown deterministic rendering 시험
- [x] [MRA-008] User/project layered scope·project precedence·collision·role/name normalization
  - Evidence: commit `ed21b87`, project precedence·same-scope collision·reserved user Judge shadow 거부
- [x] [MRA-009] Projection preview·명시적 동의·ownership ledger·non-clobber·recover 계약
  - Evidence: commits `47dd972`·`8fdbba4`·`88bc18f`, `hive agent preview|apply|validate|remove`의
    exact digest 동의·Hive ledger·foreign byte 거부·no-follow parent claim·동일 요청 재시도 수렴,
    Rust custom-agent CLI 12 PASS
- [ ] [MRA-010] Codex TOML projection·installed-version validation·fresh-session discovery
- [ ] [MRA-011] Claude Markdown projection·installed-version validation·environment/allowlist conflict detection
- [x] [MRA-012] Capability preflight와 unsupported·silent fallback·unverified alias fail-closed
  - Evidence: commits `1a37c0a`·`8fdbba4`·`3344b5d`, exact host·최소 version·13개 required
    lifecycle capability와 fresh-session evidence가 `supported`일 때만 허용, runtime attestation
    model·effort·digest mismatch 거부, Rust core·CLI focused PASS
- [x] [MRA-013] Exact role·model·effort·scope·definition digest runtime attestation receipt
  - Evidence: commit `ed21b87`, closed attestation schema와 exact host·role·scope·model·effort·
    definition digest 결합, silent fallback mismatch 거부 시험

### C. Built-in role·자동 route

- [x] [MRA-014] `hive-routine-implementer` role·fixture·host projections
- [x] [MRA-015] `hive-complex-implementer` role·fixture·host projections
- [x] [MRA-016] Reserved `hive-independent-judge`·Sol High/Claude exact profile·fresh read-only·shadow 거부
- [x] [MRA-017] `hive-design-specialist` role·design task fixture
- [x] [MRA-018] `hive-article-writer` role·article task fixture
- [x] [MRA-019] `hive-research-specialist` role·citation/read-only fixture
  - Evidence: commit `ed21b87`, 6개 canonical JSON fixture의 closed-schema 검증과 양쪽 host
    projection, Judge user-only·read-only·reserved enforcement
- [x] [MRA-020] Deterministic verifier evidence layer·model review authority `0건`·fixture
  - Evidence: commits `09700a0`·`b6679a6`·`ed21b87`, evidence reducer와 terminal Judge gate를
    분리하고 model attestation을 verdict provenance로만 처리
- [ ] [MRA-021] Spec→route→implement→verify→Judge→Ed25519 quorum→accept workflow·외부 signer 경계
- [x] [MRA-022] `explicit|implicit` setup·자연어 변경·strict terminal Judge·일반 task false-positive exclusion
  - Evidence: `JudgeInvocationPolicy`의 closed explicit·implicit 정책, strict terminal gate와 material-risk 허용 행렬, simple·read-only·format·scheduler·heartbeat·retry·deterministic failure·unsupported host 거부. user-setup persisted round-trip와 prompt projection, core 6·CLI 42 focused PASS

### D. On-demand 생성 Skill

- [x] [MRA-023] `hive-custom-subagent-create` typed CLI·양쪽 projection·reserved Judge override 금지
  - Evidence: commits `7272cba`·`b8b9a30`·`8fdbba4`, closed creation request·양쪽 projection·
    exact decision digest·reserved Judge reject·product Skill projection, Rust custom-agent CLI 12 PASS
- [ ] [MRA-024] Purpose-first recommendation과 signed host model catalog·capability 근거
  - 구현 증거: `804fc80`·`1b7b2de`, 외부 보호 catalog·분리 attestation·trust root의 exact mapping 검증,
    위조 서명·mapping 누락 거부, custom-agent CLI 13·core 6·Copier/Rust parity 22·strict Clippy 통과
  - 완료 전제: 실제 발급 catalog와 Codex·Claude fresh-session capability 수용
- [x] [MRA-025] `1 수락 | 2 수동 | 3 수정` decision state·재추천·digest lifecycle
  - Evidence: commit `e659d87`, `accept`의 prior 없음, `manual|revise`의 prior digest·동일 scope·
    exact prior request 검증, `--previous-request` 재추천, custom-agent CLI 14·Copier/Rust parity 22·strict Clippy 통과
- [x] [MRA-026] 수동 설정의 이름·양쪽 exact model/effort·scope·permission·trigger field 검증
  - Evidence: commit `7272cba`, closed JSON schema의 양쪽 host mapping·exact model/effort·
    scope·permission·trigger required field와 incomplete mapping hostile test PASS
- [ ] [MRA-027] User/project preview·동의·apply·rollback·fresh-session activation
- [x] [MRA-028] 생성 role의 auto-route registry 통합·reserved route 격리·reconfigure·disable·delete·update byte 보존
  - Evidence: commits `6d13681`·`758c168`·`8fdbba4`·`88bc18f`, user/project precedence·reserved
    Judge isolation·disable·owned-only update/delete·foreign byte 보존·interrupted deletion retry
    convergence, Rust custom-agent CLI 12 PASS

### E. Qualification·release gate

- [x] [MRA-029] Schema·precedence·digest·추천 decision·Judge invocation setup state unit/property test
  - Evidence: commits `3344b5d`·`a86e9e4`, 13개 capability fail-closed table regression,
    creation decision lineage·scope·digest tests, Judge `explicit|implicit` schema·default·persisted
    round-trip·describe contract tests, focused user-setup 41 PASS·custom-agent core 6 PASS·strict
    Clippy PASS
- [x] [MRA-030] Collision·Judge shadow/downgrade·signer/model mismatch·symlink·stale host·fallback·missing receipt hostile test
  - Evidence: commit `271ad81`, duplicate role·trigger collision, missing receipt no-mutation,
    reserved Judge shadow/downgrade·forged signer·model fallback·unsupported capability·foreign
    symlink hostile regression. custom-agent core 7 PASS·CLI 15 PASS·strict Clippy PASS
- [ ] [MRA-031] 두 host·두 scope fresh-session E2E와 두 Judge mode·strict workflow 강제 gate·model mismatch fail-closed
- [ ] [MRA-032] 사용자 guide·release note·bilingual fact·full Rust/Python/security/static gate와 `REL9-*` handoff

## Acceptance

- `MRA-001–032` evidence-backed 완료
- Codex·Claude 실제 fresh-session E2E의 exact role·model·effort attestation
- Positive route와 negative fixture false positive `0건`
- Custom role마다 Codex·Claude mapping 모두 존재
- Strict workflow terminal acceptance의 authenticated Judge 누락 `0건`
- Explicit mode 일반 task 자동 Judge `0건`, implicit negative fixture false positive `0건`
- User/project lifecycle·precedence 증명
- Silent fallback·mismatch·receipt 부재 결과 수용 `0건`
- Provider API·credential·직접 model/subagent process spawn `0건`
- Foreign·user-authored host config overwrite `0건`
- `MRA-*` 완료 전 `REL9-*` clean-clone·publication activation 금지

## 실행 순서

1. `MRA-001–006`과 `NAT-002–005` 공동 feasibility
2. `MRA-007–013` canonical role·projection·attestation
3. `MRA-014–022` built-in role·Judge policy·Sol Advisor 동등 auto-route
4. `MRA-023–028` on-demand 생성 Skill
5. `MRA-029–032` qualification·release handoff
6. `NAT-016`의 host capability adapter가 검증된 role profile·receipt 소비
