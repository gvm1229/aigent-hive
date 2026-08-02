# v0.9.0 model-routed custom subagent 계획

> 대상: `0.9.0`
> 상태: 실행 계획 활성, 구현 미착수
> 결정: [`ADR-0019`](../../decisions/ADR-0019-hive-native-iterative-execution.md)
> 선행: `NAT-002–005`, 연계: `NAT-016`
> 지원 host: OpenAI Codex, Claude Code

## 목표와 경계

- Sol Advisor 역할 분리·검증의 clean-room 기능 동등성
- Task별 custom subagent·exact model·thinking level 자동 선택
- Codex·Claude mapping 필수 provider-neutral role 정본과 user/project scope
- Host-native agent discovery·dispatch 사용, Hive의 provider API·credential·직접 process spawn `0건`
- Antigravity 제외, 근거 없는 호환성 표기 금지

## Sol Advisor 기능 동등성

- Primary orchestrator: 요구 분석·분해·route·accept 소유
- Routine/complex implementer: 위험도 근거와 bounded envelope에 따른 exact-model worker
- Fresh reviewer: 구현 context와 분리된 read-only 검토
- Acceptance: role·model·effort·definition digest 불일치 또는 증명 부재 시 중지

## 초기 built-in role 후보

`MRA-004–005` 실제 검증 전 제안값. 활성 설정은 검증된 exact ID만 허용.

| Role | Task route | Codex 후보 | Claude 후보 | 기본 권한 |
| --- | --- | --- | --- | --- |
| `hive-routine-implementer` | 반복 구현·정형 수정 | `gpt-5.6-luna` / `max` | `claude-sonnet-5` / `high` | bounded write |
| `hive-complex-implementer` | 복잡 구현·architecture 연계 | `gpt-5.6-terra` / `max` | `claude-opus-4-8` / `max` | bounded write |
| `hive-fresh-reviewer` | 독립 code·security review | `gpt-5.6-sol` / `high` | `claude-opus-4-8` / `high` | read-only |
| `hive-design-specialist` | UX·visual·interaction 설계 | `gpt-5.6-sol` / `max` | `claude-opus-4-8` / `max` | read-only 기본 |
| `hive-article-writer` | article·guide·long-form 문서 | `gpt-5.6-sol` / `high` | `claude-sonnet-5` / `high` | bounded docs write |
| `hive-research-specialist` | 근거 수집·비교·synthesis | `gpt-5.6-terra` / `high` | `claude-sonnet-5` / `high` | read-only |
| `hive-verifier` | test evidence·claim 검증 | `gpt-5.6-sol` / `high` | `claude-opus-4-8` / `high` | read-only |

## 정본과 host projection

| Scope | Hive 정본 | Codex projection | Claude projection |
| --- | --- | --- | --- |
| User | `~/.hive/config/custom-subagents/<role-id>.toml` | `~/.codex/agents/hive-<role-id>.toml` | `~/.claude/agents/hive-<role-id>.md` |
| Project | `.hive/config/custom-subagents/<role-id>.toml` | `.codex/agents/hive-<role-id>.toml` | `.claude/agents/hive-<role-id>.md` |

- Project가 동일 user role보다 우선
- 이름·scope·host model/effort·permission·trigger·digest 정본화
- Exact path preview·명시적 동의 뒤 projection
- Hive ownership digest 일치 때만 replace, foreign byte 충돌 시 보존 후 중지

## 자동 호출 흐름

`request → semantic Skill·role description match → delegation benefit 판정 → exact role·host profile → installed definition preflight → host-native dispatch → runtime receipt → result → 필요 시 fresh review`

- Narrow description 기반 단일-owner route, 별도 classifier hook·중복 호출 금지
- Simple question·작은 단일 단계·unsupported model·receipt 부재·불명확 목적은 자동 호출 제외
- 구현은 routine/complex 뒤 verifier·fresh reviewer 연결, 생성 role도 같은 registry 사용

## `hive-custom-subagent-create` 대화 계약

1. 목적 질문 뒤 `이름·Codex model/effort·Claude model/effort·scope·권한·trigger` 추천
2. `1 추천 수락 | 2 거절 후 수동 설정 | 3 수정 요청`
3. `1`: preview·digest·동의, `2`: 모든 field 순차 질문, `3`: 변경 field만 질문·재추천
4. 양쪽 host 검증·canonical 저장·projection·registry 통합·fresh-session smoke receipt

- 양쪽 mapping 누락 완료 금지, reconfigure·delete는 owned projection만 대상

## 실행 checklist

### A. Feasibility·결정

- [ ] MRA-001 Sol Advisor의 orchestrator·routine·complex·review·attestation 기능 동등성 표와 clean-room 증거 확정
- [ ] MRA-002 Codex user/project agent schema·precedence·discovery·model/effort·runtime metadata matrix 검증
- [ ] MRA-003 Claude user/project agent schema·precedence·environment override·allowlist·fallback matrix 검증
- [ ] MRA-004 실제 Codex Luna·Terra·Sol install→fresh session→dispatch→attestation lifecycle spike
- [ ] MRA-005 실제 Claude exact model·effort install→fresh session→dispatch→override/fallback lifecycle spike
- [ ] MRA-006 Codex·Claude 한정, Antigravity unsupported, default-off, host-file consent의 ADR·security acceptance

### B. Canonical role·projection

- [ ] MRA-007 양쪽 host mapping·exact model/effort·trigger·negative route·permission·scope·digest 필수 typed schema
- [ ] MRA-008 User/project layered scope·project precedence·collision·role/name normalization
- [ ] MRA-009 Projection preview·명시적 동의·ownership ledger·non-clobber·recover 계약
- [ ] MRA-010 Codex TOML projection·installed-version validation·fresh-session discovery
- [ ] MRA-011 Claude Markdown projection·installed-version validation·environment/allowlist conflict detection
- [ ] MRA-012 Capability preflight와 unsupported·silent fallback·unverified alias fail-closed
- [ ] MRA-013 Exact role·model·effort·scope·definition digest runtime attestation receipt

### C. Built-in role·자동 route

- [ ] MRA-014 `hive-routine-implementer` role·fixture·host projections
- [ ] MRA-015 `hive-complex-implementer` role·fixture·host projections
- [ ] MRA-016 `hive-fresh-reviewer` role·read-only enforcement·fresh-context proof
- [ ] MRA-017 `hive-design-specialist` role·design task fixture
- [ ] MRA-018 `hive-article-writer` role·article task fixture
- [ ] MRA-019 `hive-research-specialist` role·citation/read-only fixture
- [ ] MRA-020 `hive-verifier` role·evidence/claim fixture
- [ ] MRA-021 Spec→route→implement→verify→fresh review→accept의 Sol Advisor 동등 workflow
- [ ] MRA-022 Skill·role description 기반 automatic semantic route와 false-positive exclusion·single-owner 규칙

### D. On-demand 생성 Skill

- [ ] MRA-023 Canonical `hive-custom-subagent-create`와 typed CLI·Codex·Claude Skill projection
- [ ] MRA-024 Purpose-first recommendation과 signed host model catalog·capability 근거
- [ ] MRA-025 `1 수락 | 2 수동 | 3 수정` decision state·재추천·digest lifecycle
- [ ] MRA-026 수동 설정의 이름·양쪽 exact model/effort·scope·permission·trigger field 검증
- [ ] MRA-027 User/project preview·동의·apply·rollback·fresh-session activation
- [ ] MRA-028 생성 role의 auto-route registry 통합과 reconfigure·disable·delete·update byte 보존

### E. Qualification·release gate

- [ ] MRA-029 Schema·precedence·digest·추천 decision state unit/property test
- [ ] MRA-030 Collision·symlink·stale host·unsupported effort·Claude override/fallback·missing receipt hostile test
- [ ] MRA-031 두 host·두 scope fresh-session E2E와 auto-call·model mismatch fail-closed
- [ ] MRA-032 사용자 guide·release note·bilingual fact·full Rust/Python/security/static gate와 `REL9-*` handoff

## Acceptance

- `MRA-001–032` evidence-backed 완료
- Codex·Claude 실제 fresh-session E2E의 exact role·model·effort attestation
- Positive route와 negative fixture false positive `0건`
- Custom role마다 Codex·Claude mapping 모두 존재
- User/project lifecycle·precedence 증명
- Silent fallback·mismatch·receipt 부재 결과 수용 `0건`
- Provider API·credential·직접 model/subagent process spawn `0건`
- Foreign·user-authored host config overwrite `0건`
- `MRA-*` 완료 전 `REL9-*` clean-clone·publication activation 금지

## 실행 순서

1. `MRA-001–006`과 `NAT-002–005` 공동 feasibility
2. `MRA-007–013` canonical role·projection·attestation
3. `MRA-014–022` built-in role·Sol Advisor 동등 auto-route
4. `MRA-023–028` on-demand 생성 Skill
5. `MRA-029–032` qualification·release handoff
6. `NAT-016`의 host capability adapter가 검증된 role profile·receipt 소비
