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
- Persistent OS advisory lock marker와 disposable index lifecycle 분리
- Shared reader·exclusive writer lock과 in-flight claim gap 관찰 0건
- Ambient SQLite target path 없는 in-memory build·검증과 pinned capability 기반
  recoverable two-phase CAS publication

## 결정과 기준선

- [x] [SLW-001] Canonical source Wiki를 tracked `llm-wiki/en/`·`llm-wiki/ko/`로
  고정하고 `omx_wiki/`·`.omx/wiki/`·consumer `.hive/knowledge/` 사용 금지
- [x] [SLW-002] OMX/OMC를 현재 compatibility dependency와 orchestration owner로만
  사용하되 OMX Wiki Skill은 제외하고, durable knowledge가 replaceable orchestrator보다
  오래 유지되도록 장기 host-native·provider-neutral 대체와 retirement 시 knowledge
  migration 0건 방향 기록
- [x] [SLW-003] Hive-owned Skill의 source↔consumer 양방향 reuse, shared canonical
  `harness/skills/`, exact source projection `.agents/skills/`, consent·conformance gate를
  `AGENTS.md`와 source architecture directive에 고정
- [x] [SLW-004] Consumer `hive-wiki` Markdown parser·lint·SQLite rebuild·query와
  `hive-knowledge-capture|maintenance|query`의 안전 계약을 reuse 대상으로 선정하고
  consumer layout·installed state·promotion scope 가정은 제외

## 구현

- [x] [SLW-005] `hive-wiki` core를 재사용하는 source-confined
  `hive source-wiki lint|index|query` adapter, `hive-source.json` identity와
  `llm-wiki/` 외 canonical mutation 차단
- [x] [SLW-006] Language, pair ID, topic slug, reciprocal counterpart, canonical source
  locator와 reviewed revision을 결합한 bilingual page schema와 exact pair validation
- [x] [SLW-007] `index.md`와 source overview·boundary·crate architecture·plugin lifecycle·
  knowledge·Skill routing·upgrade·usage host·security/release·workflow topic의
  영어·한국어 exact slug pair 작성
- [x] [SLW-008] Canonical Markdown만으로 ignored
  `.agents/work/source-wiki/index.sqlite3`를 재구축하고, persistent noncanonical
  `.index.lock`의 shared reader·exclusive writer OS advisory lock, reader의 in-flight
  claim gap 관찰 0건, ambient SQLite target path 없는 in-memory serialize·deserialize,
  pinned capability의 recoverable two-phase CAS, crash 뒤 missing live와 exact Hive-owned
  orphan claim·temporary의 explicit-rebuild-only cleanup, 삭제·corruption 뒤 logical
  query equivalence와 `query` fail-closed 검증
- [x] [SLW-009] Consumer knowledge Skill의 selected workflow를 재사용하는 source-only
  `hive-source-wiki` Skill, simple-question isolation과 explicit capture·maintenance intent
- [x] [SLW-010] Missing pair·mismatched source·broken cross-link·symlink·secret candidate·
  stale index hostile conformance, 영어·한국어 query와 clean-checkout rebuild PASS
- [x] [SLW-011] Material source task 종료 전 agent-reviewed bilingual task-fact autocapture:
  current authorized artifact와 요청만 대상, 결과·도구·기준·originating request 보존,
  idempotent current-truth 갱신, raw transcript·hook·runtime ingestion 없는 LumaDeck
  marketing deck 재개 record와 query 검증

구현 evidence:

- Canonical page: 영어 13개·한국어 13개, exact pair 13개
- Logical digest:
  `sha256:4102fd66d5cb57aad0837102643b209c62e845d931b4048fb990f8511c67f48e`
- `lint`: finding 0건, warning 0건
- 영어·한국어 text query: PASS
- Index 삭제 뒤 query: fail-closed exit `5`
- Rebuild equivalence: logical digest와 query 결과 일치. SQLite binary digest는
  invocation-local evidence이며 정본·clean-copy equivalence 기준이 아님
- Current targeted tests: `hive-wiki` 42/42, Source Wiki·static contract 66/66
- Full Python conformance: 556개 실행, 555 PASS, Windows `pwsh` 전용 1개 expected skip
- Git 제외: `.agents/work/source-wiki/index.sqlite3`, `.index.lock`

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
- Lock marker: ignored persistent noncanonical shared-reader·exclusive-writer coordination state
- Publication: pinned capability 기반 recoverable two-phase CAS
- Crash recovery: 다음 explicit rebuild만 exact regular Hive-owned claim·temporary 정리
- Query: missing·stale·corrupt·crash-interrupted index에서 implicit repair 없는 fail-closed
- Conflict: canonical source 우선, Wiki stale 처리
- External orchestration: current OMX/OMC 실행 보조, OMX Wiki Skill·Wiki authority 없음
- Retirement: OMX/OMC 제거 시 durable knowledge migration 0건
