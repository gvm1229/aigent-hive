# Source bilingual LLM Wiki 계획

> Checklist owner: `SLW-*`
> Load condition: source Wiki, bilingual knowledge, source↔consumer Skill reuse
> Storage: `llm-wiki/en/`, `llm-wiki/ko/`

## 목표

- Hive source 전용 영어·한국어 LLM Wiki
- 동일 topic slug의 언어별 별도 Markdown
- Consumer knowledge core·Skill 안전 계약 재사용
- OMX/OMC namespace와 runtime state 독립
- Clean checkout 기반 SQLite 무네트워크 재구축

## 결정과 기준선

- [x] [SLW-001] Canonical source Wiki를 tracked `llm-wiki/en/`·`llm-wiki/ko/`로
  고정하고 `omx_wiki/`·`.omx/wiki/`·consumer `.hive/knowledge/` 사용 금지
- [x] [SLW-002] OMX/OMC를 현재 compatibility dependency와 orchestration owner로만
  사용하고 장기적으로 host-native·provider-neutral surface로 대체 후 제거하는 방향 기록
- [x] [SLW-003] Hive-owned Skill의 source↔consumer 양방향 reuse, shared canonical
  `harness/skills/`, exact source projection `.agents/skills/`, consent·conformance gate를
  `AGENTS.md`와 source architecture directive에 고정
- [x] [SLW-004] Consumer `hive-wiki` Markdown parser·lint·SQLite rebuild·query와
  `hive-knowledge-capture|maintenance|query`의 안전 계약을 reuse 대상으로 선정하고
  consumer layout·installed state·promotion scope 가정은 제외

## 구현

- [ ] [SLW-005] `hive-wiki` core를 재사용하는 source-confined
  `hive source-wiki lint|index|query` adapter, `hive-source.json` identity와
  `llm-wiki/` 외 canonical mutation 차단
- [ ] [SLW-006] Language, pair ID, topic slug, reciprocal counterpart, canonical source
  locator와 reviewed revision을 결합한 bilingual page schema와 exact pair validation
- [ ] [SLW-007] `index.md`와 source overview·boundary·crate architecture·plugin lifecycle·
  knowledge·Skill routing·upgrade·usage host·security/release·workflow topic의
  영어·한국어 exact slug pair 작성
- [ ] [SLW-008] Canonical Markdown만으로 ignored
  `.agents/work/source-wiki/index.sqlite3`를 재구축하고 삭제·corruption 뒤
  logical query equivalence 검증
- [ ] [SLW-009] Consumer knowledge Skill의 selected workflow를 재사용하는 source-only
  `hive-source-wiki` Skill, simple-question isolation과 explicit capture·maintenance intent
- [ ] [SLW-010] Missing pair·mismatched source·broken cross-link·symlink·secret candidate·
  stale index hostile conformance, 영어·한국어 query와 clean-checkout rebuild PASS

## Reuse 판정

| Consumer 자산 | Source 적용 |
| --- | --- |
| `hive-wiki` Markdown parser·lint | 직접 재사용 |
| Disposable SQLite FTS·tag·link index | Source-owned ignored path로 재사용 |
| `hive-knowledge-capture` review·secret boundary | Source Skill workflow로 재사용 |
| `hive-knowledge-maintenance` lint·rebuild·suppression boundary | Source Skill workflow로 재사용 |
| `hive-knowledge-query` bounded read-only query | Source Skill workflow로 재사용 |
| `hive-knowledge-promote` project→user scope | 초기 source Wiki 범위 제외 |
| Consumer `.hive/` layout·setup binding | 재사용 금지 |

## Authority

- Canonical source: `AGENTS.md`, directives, architecture, ADR, plan, state, tracked source
- Wiki: LLM retrieval·onboarding용 reviewed projection
- SQLite: ignored derived index
- Conflict: canonical source 우선, Wiki stale 처리
- External orchestration: current execution aid, Wiki authority 없음
