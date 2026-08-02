# v0.9.0 model-routed custom subagent 계획

> 대상: `0.9.0`
> 상태: 실행 계획 활성, 구현 미착수
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
| `hive-independent-judge` | 독립 acceptance·security 판정 | `gpt-5.6-sol` / `max` | `claude-opus-4-8` / `max` | read-only |
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

- [ ] [MRA-007] 양쪽 host mapping·exact model/effort·trigger·negative route·permission·scope·digest 필수 typed schema
- [ ] [MRA-008] User/project layered scope·project precedence·collision·role/name normalization
- [ ] [MRA-009] Projection preview·명시적 동의·ownership ledger·non-clobber·recover 계약
- [ ] [MRA-010] Codex TOML projection·installed-version validation·fresh-session discovery
- [ ] [MRA-011] Claude Markdown projection·installed-version validation·environment/allowlist conflict detection
- [ ] [MRA-012] Capability preflight와 unsupported·silent fallback·unverified alias fail-closed
- [ ] [MRA-013] Exact role·model·effort·scope·definition digest runtime attestation receipt

### C. Built-in role·자동 route

- [ ] [MRA-014] `hive-routine-implementer` role·fixture·host projections
- [ ] [MRA-015] `hive-complex-implementer` role·fixture·host projections
- [ ] [MRA-016] Reserved `hive-independent-judge`·Sol Max/Claude exact profile·fresh read-only·shadow 거부
- [ ] [MRA-017] `hive-design-specialist` role·design task fixture
- [ ] [MRA-018] `hive-article-writer` role·article task fixture
- [ ] [MRA-019] `hive-research-specialist` role·citation/read-only fixture
- [ ] [MRA-020] Deterministic verifier evidence layer·model review authority `0건`·fixture
- [ ] [MRA-021] Spec→route→implement→verify→Judge→Ed25519 quorum→accept workflow·외부 signer 경계
- [ ] [MRA-022] `explicit|implicit` setup·자연어 변경·strict terminal Judge·일반 task false-positive exclusion

### D. On-demand 생성 Skill

- [ ] [MRA-023] `hive-custom-subagent-create` typed CLI·양쪽 projection·reserved Judge override 금지
- [ ] [MRA-024] Purpose-first recommendation과 signed host model catalog·capability 근거
- [ ] [MRA-025] `1 수락 | 2 수동 | 3 수정` decision state·재추천·digest lifecycle
- [ ] [MRA-026] 수동 설정의 이름·양쪽 exact model/effort·scope·permission·trigger field 검증
- [ ] [MRA-027] User/project preview·동의·apply·rollback·fresh-session activation
- [ ] [MRA-028] 생성 role의 auto-route registry 통합·reserved route 격리·reconfigure·disable·delete·update byte 보존

### E. Qualification·release gate

- [ ] [MRA-029] Schema·precedence·digest·추천 decision·Judge invocation setup state unit/property test
- [ ] [MRA-030] Collision·Judge shadow/downgrade·signer/model mismatch·symlink·stale host·fallback·missing receipt hostile test
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
