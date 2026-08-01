# prompt refine 자동 routing 수정 계획

> Checklist owner: `PRF-*`
> Target: `0.9.0`
> 상태: 계획 확정, 구현 미착수
> Decision: [`ADR-0009`](../../decisions/ADR-0009-user-plugin-project-knowledge-boundary.md)

## 목표

- Material ambiguity가 있는 ordinary work의 `hive-prompt-refine` 자동 선택
- Explicit invocation과 automatic route의 `refine-only` 기본값
- Refined prompt 제시 직후 `awaiting-approval` 정지
- 사용자 교정 또는 승인 전 project read·tool·write·execution 0건
- Simple question·editless question·clear work의 기존 routing 보존
- Prompt 분류용 hook·provider API·hidden rewrite 0건

## 확인된 원인

| 현상 | 현재 계약·구현 | 판정 |
| --- | --- | --- |
| 자동 호출 없음 | Skill description·catalog·source/consumer guidance의 explicit-only 규칙 | 현재 정책과 일치, 사용자 기대와 불일치 |
| 모호한 ordinary work | `prompt_quality`가 `refine_suggestion=true`만 생성 | 자동 Skill 선택 경로 부재 |
| 명시적 `$hive-prompt-refine` 뒤 즉시 실행 | 명령 뒤 imperative payload를 실행 승인으로 오해 | 호출 문법·정지 상태 경계 결함 |
| `refine-and-run` | `explicit_run_intent` 없는 요청을 typed router가 차단 | 낮은 계층 안전 계약 존재 |
| Simple question | 독립 route와 zero-capability 계약 | 보존 필요 |

핵심 원인: 자동 routing 자체의 runtime 고장보다 explicit-only 제품 정책과 host
guidance의 일관된 적용. 수정 대상: 정책·Skill discovery metadata·typed route·승인 lifecycle.

## 목표 routing

| 입력 | 결과 |
| --- | --- |
| `$hive-prompt-refine <payload>` | `refine-only` → refined prompt → `awaiting-approval` |
| 자연어 prompt 작성·개선 요청 | `refine-only` → `awaiting-approval` |
| Materially ambiguous ordinary work | 자동 `hive-prompt-refine` → `refine-only` → `awaiting-approval` |
| 충분히 명확한 ordinary work | 기존 host-native·selected Skill 실행 |
| Simple·editless question | 기존 isolated answer route |
| 다음 turn의 교정 | 같은 intent의 재정제 → 새 digest → `awaiting-approval` |
| 다음 turn의 명시적 승인 | exact refined prompt digest 결합 → host-owned execution |
| `$hive-prompt-refine --run <payload>` | same-request 명시 승인일 때만 refine-and-run |

Imperative payload 자체, `do it`, urgency, autonomy, “complete the task” 문구는 실행 승인
근거에서 제외. `--run` 또는 refined prompt를 특정한 후속 승인만 실행 권한으로 인정.

## Material ambiguity 기준

다음 항목 누락만으로 자동 정제 금지. 둘 이상의 합리적 실행이 아래 결과를 실질적으로
달리 만들 때만 자동 route:

- 목표 또는 성공 상태
- 수정·조사·배포 대상 범위
- 허용된 side effect·외부 mutation·credential 경계
- 핵심 제약·금지 조건
- acceptance evidence 또는 요구 출력 형태

맞춤법, 짧은 문장, 선호 tone, 안전한 repository discovery로 해결 가능한 locator 부족은
단독 trigger에서 제외. Usage guard·setup-required·explicit Skill·simple/editless question의
기존 우선순위 유지.

## 승인 lifecycle

```text
candidate prompt
  → refine-only validation
  → refined prompt + digest
  → awaiting-approval
  ├─ correction → new refined prompt + new digest
  └─ explicit approval of exact digest → host-owned execution
```

- `awaiting-approval`: project read, network, subagent, memory capture, run 생성, model task
  execution, file write 전부 false
- 승인 대상: exact refined prompt bytes·preservation envelope·target host·mode digest
- Stale approval: 최신 digest 불일치로 차단
- 수정 요청: 이전 승인·digest 무효화
- 실행 owner: host-native 기본값 또는 사용자가 이미 명시 선택한 compatible owner
- 승인 대기 상태의 raw prompt durable capture: 0건

## 구현 대상

- Policy: `.agents/directives/01-behavior.md`, consumer `AGENTS.md` template,
  `docs/guidance-schema.md`, product decision
- Skill: `harness/skills/hive-prompt-refine/`, exact source projection,
  bundled plugin projection, catalog·Codex metadata
- Typed routing: `crates/hive-projection`, `crates/hive-cli`, routing·refinement schema
- Tests: route fixture, Rust unit, Phase 3 static·routing, fresh-session host projection
- Documentation: ADR·CURRENT·release notes·bilingual atomic fact
- Frozen `0.7.0|0.8.0` historical Skill bytes: mutation 금지

## Checklist

### A. Contract

- [ ] [PRF-001] Explicit-only·suggestion-only current truth와 typed router baseline fixture 고정
- [ ] [PRF-002] `$hive-prompt-refine` refine-only 기본, `--run` same-request 승인,
  imperative payload 비승인 문법 확정
- [ ] [PRF-003] Material ambiguity 판정과 usage·setup·explicit Skill·simple/editless·clear
  work precedence 확정
- [ ] [PRF-004] Refined prompt digest·`awaiting-approval|authorized`·stale approval·correction
  lifecycle schema 확정

### B. Implementation

- [ ] [PRF-005] Source·consumer directive, Skill description·body, catalog·metadata의 automatic
  refine-only discovery와 mandatory stop 반영
- [ ] [PRF-006] Normalized `prompt_quality`의 suggestion-only 분기를 automatic Skill route로
  전환하고 approval envelope·CLI validation 구현
- [ ] [PRF-007] Canonical Skill의 source·Codex·Claude·Antigravity projection parity와 frozen
  historical base 불변 검증

### C. Verification

- [ ] [PRF-008] Ambiguous work 자동 선택과 clear work·simple/editless negative route fixture
- [ ] [PRF-009] Explicit command + imperative payload의 refined prompt 이후 tool·read·write·run
  0건과 `awaiting-approval` 검증
- [ ] [PRF-010] 후속 exact 승인 실행, correction 재정제, stale·generic approval 차단 E2E
- [ ] [PRF-011] 세 host fresh-session 자동 discovery·single-body load·prompt-classifier hook 0건 검증
- [ ] [PRF-012] Full Rust·Python 적합성, 문서 style·projection·release note·bilingual fact 완료

## 완료 기준

- 모호한 ordinary work fixture의 selected Skill: `hive-prompt-refine`
- Refined prompt 반환 turn의 side effect·project read·execution: 0건
- 사용자 승인 전 자동 continuation: 0건
- Exact 후속 승인 뒤 실행 handoff: 1건
- Simple·editless·clear work false-positive: 0건
- Source·consumer·세 host projection drift: 0건
- Prompt classification hook·provider API·raw prompt durable capture: 0건
