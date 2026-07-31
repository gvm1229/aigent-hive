# ADR-0016: v0.9 전역 knowledge RAG

- 상태: accepted
- 날짜: 2026-08-01
- 대상: `0.9.0`
- 부분 대체: [`ADR-0012`](ADR-0012-global-onboarding-shared-index.md)의 query·capture,
  Stage 3 simple-question memory isolation

## 현재 기반과 gap

`0.8.0` 기반: user-root canonical Markdown, 등록 project Wiki, 단일 SQLite,
FTS5·tag·alias·BM25와 visibility-aware shared query.

미충족 영역:

- Simple-question path의 memory·Wiki 접근 전면 차단
- Project knowledge가 필요하다는 explicit intent에서만 query
- Material task completion에 한정된 autocapture
- 사용자 선호·일반 fact의 mandatory user-root write 부재
- Page metadata 중심 result와 query-time canonical source 전체 검증

## 결정

### Mandatory memory

- Global Wiki enabled 상태의 모든 user turn에 agent-reviewed durable-memory gate
- 명시적 reusable fact·preference·workflow: extra prompt 없는 canonical write
- Exact 발화 대신 bounded normalized fact·scope·provenance·digest 기록
- Current-truth upsert·supersede·dedup·contradiction 계약
- Unsafe·ambiguous·ephemeral input: write 0건과 이유 표시
- Raw transcript·complete conversation·hook·tool output·runtime state 저장 금지

### Retrieval before routing

- 모든 사용자 질문: simple-question 판정 전 bounded user-root retrieval 1회
- No-hit: 기존 simple 또는 task route 유지
- Hit: canonical chunk·locator·digest 기반 답변, retrieved fact와 inference 분리
- Wiki disabled: retrieval·capture·index repair 0건

### Scope와 visibility

- `auto`: current project + user-root + shared projects
- `global`: user-root + shared projects
- `project:<id>`: 사용자가 명시한 등록 project private·shared + user-root
- `all-visible`: user-root + shared projects
- `confidential`: current project 또는 별도 current-action 승인
- Unknown·ambiguous project identity: private retrieval·write fail-closed

### Derived RAG index

- Canonical authority: Markdown·tracked config·registration ledger
- SQLite authority: disposable search·ranking·chunk projection만 허용
- Schema: document, deterministic chunk, FTS5, tag·alias·link·source·replacement,
  generation·dirty journal
- Canonical-first write-through, exact dirty-set recovery, full-scan 없는 query snapshot
- Missing·stale·corrupt state의 derived-only bounded repair
- SQLite schema 전면 교체 허용, canonical Markdown migration 금지
- Vector projection: measured bilingual recall deficiency와 dependency·license·security review 이후
- Vector boundary: local·disposable·offline, provider API·credential 0건

## 답변 품질

- Citation-ready chunk·project·visibility·locator·digest·score 반환
- Conflict·stale·insufficient evidence의 명시적 no-answer
- Bilingual paraphrase `recall@5 >= 90%`
- 50,000 chunk warm `p95 <= 100ms`, cold `p95 <= 500ms`

## 효력

ADR-0012의 Markdown 정본·user-root 단일 SQLite·visibility·opt-out 유지.
v0.9 구현에서 query·capture·simple-question isolation만 이 결정으로 확대.
세부 실행 정본: [`v0.9.0-global-knowledge-rag.md`](../plans/active/v0.9.0-global-knowledge-rag.md).
