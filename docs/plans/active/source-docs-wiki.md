# Source `docs/` Wiki 계획

> Checklist owner: `SLW-*`
> Load condition: source Wiki, bilingual atomic fact, source↔consumer Skill reuse
> Storage: `docs/facts/en/`, `docs/facts/ko/`

## 목표

- Human topic document와 bilingual atomic fact의 단일 `docs/` graph
- 동일 topic slug의 English·Korean exact pair
- 한 pair당 primary fact 1개
- Consumer knowledge core·Skill 안전 계약 재사용
- OMX·OMC namespace와 runtime state 독립
- Clean checkout 기반 SQLite 무네트워크 재구축
- Persistent advisory lock과 disposable index 분리

## 결정과 구현

- [x] [SLW-001] Canonical source fact를 tracked `docs/facts/en/`·`docs/facts/ko/`로
  고정하고 `omx_wiki/`·`.omx/wiki/`·consumer `.hive/knowledge/` 사용 금지
- [x] [SLW-002] OMX·OMC를 durable Wiki authority에서 제외하고 신규 workflow dependency
  없이 legacy foreign provenance만 보존
- [x] [SLW-003] Hive-owned Skill의 source↔consumer reuse와
  `harness/skills/` canonical·`.agents/skills/` source projection 계약
- [x] [SLW-004] `hive-wiki` Markdown parser·lint·SQLite rebuild·query와 knowledge
  Skill의 secret·review·current-truth safety primitive 재사용
- [x] [SLW-005] Source-confined `hive source-wiki lint|index|query`, source marker와
  `docs/facts/` 외 fact mutation 차단
- [x] [SLW-006] Language, pair ID, topic slug, reciprocal counterpart, source digest,
  reviewed revision과 one-H1·no-subsection·800-byte atomic body schema
- [x] [SLW-007] Source purpose·boundary·crate·onboarding·knowledge·routing·role·run·
  usage·judge·release·update·workflow의 27개 English·Korean fact pair
- [x] [SLW-008] Canonical fact만 사용하는 ignored SQLite rebuild, persistent
  shared-reader·exclusive-writer lock, in-memory build·검증과 recoverable two-phase CAS
- [x] [SLW-009] Source-only `hive-source-wiki` Skill, simple-question isolation과
  explicit capture·maintenance intent
- [x] [SLW-010] Missing pair·mismatched source·broken link·symlink·secret·stale index
  hostile conformance와 English·Korean clean-checkout rebuild
- [x] [SLW-011] Material source task의 agent-reviewed bilingual task-fact completion
  capture, raw transcript·hook·runtime ingestion 0건
- [ ] [SLW-012] `hive-source.json` source root의 automatic knowledge 조회를
  `hive source-wiki query`로 고정하고 consumer `hive knowledge retrieve` 호출 금지,
  source·consumer route의 static regression

## Authority

- Current source·schema·test: 최우선
- Human topic document·ADR: current explanation·decision
- Atomic fact: reviewed retrieval projection
- SQLite: ignored derived index
- Lock: ignored noncanonical coordination marker
- Query: missing·stale·corrupt index에서 fail-closed
- Legacy external provenance: Wiki authority 없음
