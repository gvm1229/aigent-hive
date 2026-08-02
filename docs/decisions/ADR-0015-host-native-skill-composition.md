# ADR-0015: v0.9 host-native Skill 조합

- 상태: implemented baseline; orchestration non-goal superseded by proposed ADR-0019
- 날짜: 2026-08-01
- 대상: `0.9.0`
- 대체 대상: [`ADR-0004`](ADR-0004-orchestration-ownership.md)
- 후속 정책: [`ADR-0019`](ADR-0019-hive-native-iterative-execution.md)

## 배경

ADR-0004의 Codex→OMX·Claude Code→OMC 우선순위는 외부 호환 계층에 실행 소유권을
부여하는 `0.8.x` 기준. v0.9 목표는 호스트 자체 subagent·goal·hook 기능과 Hive의
provider-neutral Skill·Markdown 상태 계약 조합. 외부 계층 없이도 같은 지식·역할·실행
상태를 유지하는 제품 경계 필요.

## 구현 기준선

- 새 run의 기본 실행 소유자: 검증된 host-native capability
- Hive 소유 범위: Skill 조합, DAG·상태 전이 계약, `.hive/` Markdown 정본,
  evidence 검증, checkpoint·resume·role handoff
- Host 소유 범위: model 실행, subagent 생성, goal 유지, 허용된 hook event 전달
- Capability 판정: `supported|best-effort|unsupported|unverified`
- 필수 capability의 `unsupported|unverified`: 명시적 `host_capability_unsupported`
  차단, 다른 runtime 자동 전환·유사 기능 생성 금지
- OMX·OMC: 당시 사용자가 별도로 선택 가능한 외부 호환 계층. 신규 workflow 경로는 ADR-0019에서 제거
- 기존 `0.8.x` run: 고정된 owner 보존. v0.9 새 run과 명시적 migration에만 새 결정 적용

## Graph engineering 경계

- `hive-loop-engineering`: host-native capability를 조합하는 얇은 Skill
- DAG node·edge, cycle detection, bounded retry, evidence-based transition,
  `blocked|failed|complete` terminal outcome, dynamic steering의 typed 계약
- 매 node·retry·steering dispatch 전 `hive-usage-guard` gate
- 독립 verification role과 evidence locator 없는 success edge 금지
- 기존 `hive-run-checkpoint`, `hive-run-resume`, `hive-role-handoff` 재사용
- 구현 완료 기준선: scheduler·model runtime·session daemon·provider API client·tmux·`omx|omc` command dependency 0개
- 당시 범위: Stop hook continuation·Ralph·team/swarm runtime 제외
- 후속 범위: provider-neutral scheduler·iterative·team·multi-goal 구현 허용, provider runtime·direct process spawn 금지
- Hook 사용 범위: host-native checkpoint·무결성 알림. 실행 재호출·계속 여부 결정 금지

## Wiki와 Skill 조합

- `hive-wiki`: 기존 knowledge capture·query·maintenance와 source-wiki의 얇은 통합 진입점
- 공개 동사: `add|query|lint|list|read|delete|refresh`
- Keyword·tag·category query, 안정 category taxonomy, `[[wikilink]]`, 검토 기반 quick-add
- Markdown 정본·SQLite 파생·secret safety·current-truth 유지
- `omx_wiki` path·`omx` command·raw session 자동 수집 금지
- 초기 suite: `hive-loop-engineering`, `hive-wiki`, `ai-slop-cleaner`,
  `best-practice-research`와 기존 필수 Hive Skill
- 중복 기능: 하나의 정본 구현 또는 얇은 router로 통합
- `ai-slop-cleaner`: 회귀 시험 우선, 동작 보존, fallback 분류, smell별 pass,
  변경 파일 한정과 pass별 품질 gate
- `best-practice-research`: 읽기 전용 bounded 연구, 공식·upstream 우선, 날짜·version,
  저장소 사실 분리, 최소 evidence set, 인용과 명시적 handoff
- 전체 OMX·OMC Skill·adapter: 기능 영역·owner·격차·`adopt|merge|exclude` 근거표 작성
- Hive 승격 조건: 비중복, 사용자 승인, license·보안·source↔consumer conformance 검토

## 기준선 수락 조건

- v0.9 active plan의 `V9-*` 완료
- 세 host capability matrix와 unsupported 결과 검증
- Cycle·retry·evidence·steering·usage gate·independent verification 회귀 검증
- Wiki 동사·taxonomy·link·query·quick-add·delete 안전 경계 검증
- Consumer projection의 tmux·scheduler·Stop continuation·외부 namespace dependency 0건
- v0.9 지침·계약·routing·projection·test·fact의 host-native 기본값과
  OMX/OMC 자동 우선권·`omx|omc` command dependency 0건

## 효력

ADR-0004는 `0.8.x`와 기존 run의 역사적 계약으로 유지. 이 결정은 v0.9 새 run과
명시적 migration에 적용한 data-only baseline. Scheduler·Ralph·team 비구현 정책은
ADR-0019의 사용자 결정으로 superseded. 기존 run owner·foreign bytes 불변.
