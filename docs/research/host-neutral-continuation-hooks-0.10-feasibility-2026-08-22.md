# Host-neutral 연속 실행·hook 조사

- 조사일: 2026-08-22
- Antigravity local 재검증: 2026-08-23
- 외부 사례: `oh-my-codex@3ad79a8a6fe6e95fdbb8c00e40716fffe4011ce2`
- 외부 source 사용: 구조·계약 비교만 수행, 코드·prompt 복사 `0 bytes`
- 로컬 host: Codex CLI `0.148.0`, Claude Code `2.1.163`, Antigravity CLI `1.1.18`
- 범위: Goal·task·Stop hook·closure gate·취소·복구

## 결론

채택 권고: **Goal 또는 host task를 실행 주체로 두고, Hive CLI closure gate와 선택형
Stop hook을 보조 안전장치로 결합**. Hook 단독 지속 실행과 무제한 Stop block은 미채택.

현재 Hive `checkpoint-reminder / Stop`의 neutral allow 계약은 그대로 유지. 새
`continue-active-run` capability는 `CON10-002–008`의 schema·consent·three-host 시험 통과
뒤에만 별도 추가.

## `oh-my-codex`에서 확인한 구조

### Ralph의 현재 상태

- `$ralph`: `0.21`에서 제거된 migration stub
- 후속 경로: `$ultragoal`
- 지속 주체: Codex Goal 하나의 aggregate objective
- 세부 진행: repository ledger의 개별 story
- 완료: 실제 artifact·시험·review evidence와 fresh `get_goal` snapshot 뒤
  `update_goal(status=complete)`
- Hook: goal state mutation 없음, reconciliation·continuation nudge만 수행

### 유용한 패턴

1. Host goal과 repository ledger의 소유권 분리
2. `active|blocked_on_user|complete|failed|cancelled` terminal 상태 구분
3. Completion audit의 checklist·verification evidence 필수
4. Session·thread·workspace binding 불일치의 block 금지
5. Foreign hook 보존과 관리 hook만 exact 제거
6. Stop hook timeout과 bounded state read
7. 취소·hook disable·stale recovery의 completion evidence 비의존
8. Hook의 hidden goal mutation 금지

### 미채택 패턴

- Legacy Ralph의 기본 `max_iterations: 50`
- Active flag만으로 계속 차단하는 경로
- Transcript 기반 복구를 일반 authority로 사용
- Watcher·tmux injection·별도 process runtime
- OMX/OMC state·command·prompt 재사용

## Host capability

| Host | 확인 결과 | `0.10.0` 판정 |
| --- | --- | --- |
| Codex | 로컬 `0.148.0`: `goals=stable`, `hooks=stable`, `plugin_hooks=removed`; `.codex/hooks.json` 형식과 `Stop` block 사례를 외부 repository에서 확인. 공식 OpenAI 공개 문서 검색에서는 hook schema 페이지 미발견 | Local-qualified, 공식 schema provenance 추가 확인 필요 |
| Claude Code | 공식 문서: `Stop` hook이 종료 차단 가능, `stop_hook_active` 제공, 진행 없는 연속 block 8회 뒤 host override, user interrupt에는 `Stop` 미발생 | Hook 지원 |
| Google Antigravity | 공식 문서: `.agents/hooks.json` 또는 user config, `Stop` 결과 `decision: continue`로 실행 loop 재진입. 로컬 `agy 1.1.18`의 `/hooks` JSON surface 성공, 등록 hook `0건`, token 사용 `0건` | Hook surface 확인, 실제 Stop 차단 시험 필요 |

Antigravity local evidence:

- Executable: user PATH의 `agy.exe`
- `agy --version`: `1.1.18`
- `agy -p "/hooks" --output-format json`: `status=SUCCESS`, `hooks=[]`, `num_turns=0`, token `0`
- Changelog: `1.1.17`의 단일 execution harness와 hook 일관성, `1.1.12`의 `/hooks` read-only print mode, `1.0.8`의 shared hook config 경로
- 현재 증명 범위: CLI·hook inspection surface
- 미증명 범위: 실제 `Stop decision=continue`, project-local config merge·disable·rollback

Claude 근거:

- [Hooks reference](https://code.claude.com/docs/en/hooks)
- [Hooks guide](https://code.claude.com/docs/en/hooks-guide)

Antigravity 근거:

- [Hooks](https://antigravity.google/docs/ide/hooks/)

Codex 참고:

- [Codex use cases](https://developers.openai.com/codex/use-cases)
- 외부 repository `docs/codex-native-hooks.md`, `src/config/codex-hooks.ts`,
  `src/scripts/codex-native-hook.ts`, `skills/ultragoal/SKILL.md`

## Provider-neutral 계약

### Canonical closure

```json
{
  "schema_version": 1,
  "run_id": "run-id",
  "run_revision": 12,
  "ready_for_final": false,
  "agent_owned": ["KRG10-001"],
  "awaiting_user_authority": [],
  "awaiting_external_evidence": [],
  "blocked": [],
  "excluded": ["REL10-005", "REL10-006", "REL10-007"],
  "closure_digest": "sha256:<64-lowercase-hex>"
}
```

- Markdown plan·run·evidence에서만 계산
- Transcript·hook payload·SQLite·host hidden state: 정본 제외
- `ready_for_final=true`: `agent_owned`가 비어 있고 현재 terminal state가 유효한 경우만
- Excluded item: exact 사용자 범위와 checklist ID 결합

### Stop adapter

Host별 wire 형식만 다르고 동일 closure decision 사용:

```text
if user_cancel_or_interrupt: allow
if session_or_workspace_mismatch: allow
if state_stale_or_malformed: allow + diagnostic
if terminal_or_blocked_on_user: allow
if ready_for_final: allow
if same run revision already nudged: allow
if consecutive blocks without progress >= 3: allow + resume-ready receipt
otherwise: one continuation nudge and block Stop
```

Bound:

- 같은 `run_revision + closure_digest`: block 1회
- 진행 없는 연속 block: 최대 3회
- 진행 증거: canonical run revision 증가만 인정
- Hook timeout: 30초 이하
- Hook에서 canonical state·Goal·task mutation `0건`
- User interrupt·cancel·disable·uninstall: completion gate보다 우선

## Hive 적용안

1. `hive run closure`: read-only closure 계산
2. `continue-active-run`: 기존 hook capability와 분리된 새 optional capability
3. 공통 continuation envelope와 Codex·Claude·Antigravity wire adapter 분리
4. Host capability의 exact event·decision semantics 검증
5. Preview에 event·path·command·digest·block bound·cancel 경로 표시
6. 사용자 승인 뒤 project-local non-clobber 설치
7. Stale·malformed·foreign·unsupported: artifact·질문·mutation `0건`
8. Goal/task 미지원 host: manual resume, watcher 대체 없음

## 채택 gate

- 세 host fixture의 동일 closure verdict
- Codex·Claude 실제 Stop 차단·정상 종료·cancel 검증
- Antigravity 실제 설치 host 증거 또는 default-off 미지원 판정
- 같은 revision의 두 번째 Stop block `0건`
- 진행 없는 연속 block 3회 초과 `0건`
- User cancel 뒤 Stop block `0건`
- Stale·malformed·foreign session Stop block `0건`
- Foreign host config byte 변경 `0건`
- Hook disable·uninstall 뒤 잔여 관리 entry `0건`
- Provider API·credential·transcript authority·background watcher `0건`

## 결정

- `CON10-001`: 조사 완료
- `CON10-002–008`: 조건부 구현·수용 필요
- 기존 neutral `Stop` 계약: 새 capability 채택 전 변경 금지
