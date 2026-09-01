# ADR-0016: v0.9 전역 knowledge RAG

- 상태: accepted
- 날짜: 2026-08-01
- 대상: `0.9.0`
- 부분 대체: [`ADR-0012`](ADR-0012-global-onboarding-shared-index.md)의 query·capture·
  data portability, Stage 3 simple-question memory isolation
- 후속 부분 대체: Consumer Notion mode의 canonical source·rebuild는
  [`ADR-0018`](ADR-0018-notion-wiki-backend.md)

## 현재 기반과 gap

`0.8.0` 기반: user-root canonical Markdown, 등록 project Wiki, 단일 SQLite,
FTS5·tag·alias·BM25와 visibility-aware shared query.

미충족 영역:

- Simple-question path의 memory·Wiki 접근 전면 차단
- Project knowledge가 필요하다는 explicit intent에서만 query
- Material task completion에 한정된 autocapture
- 사용자 선호·일반 fact의 mandatory user-root write 부재
- Page metadata 중심 result와 query-time canonical source 전체 검증
- Canonical knowledge의 portable export·import 부재
- 선택 source 1개 외 directory bulk scan·claim review 부재

## 결정

### Mandatory memory

- Global Wiki enabled 상태의 모든 user turn에 agent-reviewed durable-memory gate
- 명시적 reusable fact·preference·workflow: extra prompt 없는 canonical write
- Exact 발화 대신 bounded normalized fact·scope·provenance·digest 기록
- Current-truth upsert·supersede·dedup·contradiction 계약
- Unsafe·ambiguous·ephemeral input: write 0건과 이유 표시
- Raw transcript·complete conversation·hook·tool output·runtime state 저장 금지

Operational enforcement:

- User·project guidance 모두에 every-turn 판정, exact `hive knowledge remember`, canonical receipt 규칙 포함
- Localized Skill description도 every-turn mandatory route 의미 보존
- Project setup 부재: user-global fact의 `user-root` 기록 유지, project-specific scope 추론·자동 등록 금지
- 선택 host의 사용자 범위 지침은 설치 직후 모든 folder에 적용. Project setup·Hive harness·project marker는
  user-root capture의 전제 조건 아님
- Expected-byte install validation과 별도로 command·receipt·Wiki disabled negative semantic assertion 필수
- Manual Skill 호출 없는 fresh-session write·recall 수용 전 기능 완료·stable publication 주장 금지

### Retrieval before routing

- 질문·research·knowledge-dependent task: routing 전 bounded retrieval 1회
- Single owner: 기존 `knowledge-recall`; 새 find Skill 생성 금지
- Automatic default top 5·byte budget, explicit query만 확대
- Retrieval 종료 뒤 next Skill sequential handoff, 동시 Skill body load 최대 1개
- No-hit: 기존 simple 또는 task route 유지
- Hit: canonical chunk·locator·digest 기반 답변, retrieved fact와 inference 분리
- Wiki disabled: retrieval·capture·index repair 0건
- Retrieved instruction·command: untrusted data, 실행·Skill activation·권한 확대 0건

### Scope와 visibility

- `auto`: current project + user-root + shared projects
- 미등록 current target의 `auto`: user-root + shared projects. Project-private 지식 포함 금지
- `global`: user-root + shared projects
- `project:<id>`: 사용자가 명시한 등록 project private·shared + user-root
- `collection:<id>`: 등록 또는 detached collection의 승인된 범위 + user-root
- `all-visible`: user-root + shared projects
- `confidential`: current project 또는 별도 current-action 승인
- Unknown·ambiguous project identity: private retrieval·write fail-closed
- Background daemon·raw prompt recorder 없이 foreground agent-reviewed capture만 허용

### Derived RAG index

- Canonical authority: Markdown·tracked config·registration ledger
- SQLite authority: disposable search·ranking·chunk projection만 허용
- Schema: collection, document, deterministic chunk, claim, FTS5, tag·alias·link·source·replacement,
  generation·dirty journal
- Canonical-first write-through, exact dirty-set recovery, full-scan 없는 query snapshot
- Missing·stale·corrupt state의 derived-only bounded repair
- SQLite schema 전면 교체 허용, canonical Markdown migration 금지
- Vector projection: measured bilingual recall deficiency와 dependency·license·security review 이후
- Vector boundary: local·disposable·offline, provider API·credential 0건

### Portable bundle·collection

- Machine 간 이식: SQLite가 아닌 `.hivekb` canonical bundle
- Bundle: deterministic ZIP, versioned manifest, relative payload path, SHA-256 전체 목록
- 제외: SQLite·WAL·SHM·journal·runtime·lock·absolute path·credential·confidential content
- Hash: 내부 무결성만 입증, source authenticity·transport secrecy claim 없음
- Import: dry-run·staging·conflict·backup·atomic activation 뒤 destination index rebuild
- Stable `collection_id` row 사용, directory별 table 생성 0건
- 다른 machine의 project: detached collection, explicit local mapping 전 private auto-query 제외
- Wiki disabled 상태의 explicit export·import 허용, enable·index·retrieval 상태 변경 없음

### Directory scan

- 새 `knowledge-import`: explicit bulk inventory·claim review Skill
- Git tracked-first, optional untracked nonignored, non-Git allowlist와 size·count budget
- Claim kind와 assertion status로 project-specific decision·observation도 안전하게 보존
- Dependency 존재와 successful convention 분리; version·revision·test/build evidence 필수
- Project collection upsert 뒤 reusable candidate 2차 scan
- Root 승격은 기존 `hive knowledge promote`의 redaction·dedup·contradiction·consent 재사용
- Scan content와 retrieved text의 prompt instruction authority 0건

## 답변 품질

- Citation-ready chunk·project·visibility·locator·digest·score 반환
- Conflict·stale·insufficient evidence의 명시적 no-answer
- Bilingual paraphrase `recall@5 >= 90%`
- 50,000 chunk warm `p95 <= 100ms`, cold `p95 <= 500ms`

## 효력

ADR-0012의 Markdown 정본·user-root 단일 SQLite·visibility·opt-out 유지.
v0.9 구현에서 query·capture·simple-question isolation·data portability·scan 확대.
세부 실행 정본: [`v0.9.0-global-knowledge-rag.md`](../archive/plans/releases/0.9.0/v0.9.0-global-knowledge-rag.md).
Bundle·scan 정본:
[`v0.9.0-knowledge-portability-scan.md`](../archive/plans/releases/0.9.0/v0.9.0-knowledge-portability-scan.md).
