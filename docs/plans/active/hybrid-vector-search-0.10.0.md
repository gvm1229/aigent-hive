# Hybrid vector search `0.10.0`

> Checklist owner: `KRG10-014`, `VEC10-*`
> 채택 방식: 측정 gate 통과 뒤 조건부 구현
> 실패 영향: FTS·native graph·Graphify code graph 일정 유지
> 연구 근거: [`vector-memory-0.10-feasibility-2026-08-22.md`](../../research/vector-memory-0.10-feasibility-2026-08-22.md)

## 목표

SQLite FTS의 exact 검색을 유지하면서 표현이 다른 의미·다국어 질문의 recall 보강. Vector:
후보 검색 계층, graph: 명시적 관계 계층, Markdown: 유일한 정본.

## Checklist

- [x] [KRG10-014] FTS·vector·hybrid 품질·속도·비용 비교와 exact engine·embedding 계약의 adopt|defer 결정 — 50,000 embedding build 10분 초과로 `defer`
- [x] [VEC10-001] 120개 gold query corpus: exact ID·날짜·수치·부정 30개, 한국어·영어 paraphrase 40개, cross-language 20개, relation·영향 30개 — `vector-gold-120.json`, deterministic generation 회귀
- [x] [VEC10-002] 현재 SQLite FTS·alias·tag의 Recall@10·MRR·warm/cold p95·returned byte 기준선 — semantic Recall@10 78.3%, warm p95 0.08ms
- [x] [VEC10-003] Local embedding 경계 결정: non-generative indexer, provider API·API key·prompt·text generation `0건`, model license·version·dimension·SHA-256 고정 — FastEmbed `0.8.0`, multilingual MiniLM, Apache-2.0, 384 dimension, model tree digest 고정
- [x] [VEC10-004] Qdrant Edge·sqlite-vec·SQLite-Vector의 exact version·license·Rust·세 운영체제·memory safety·storage·ANN maturity 조사 — Qdrant `0.8.0` beta, sqlite-vec `0.1.9` pre-1.0, SQLite-Vector `1.0.0` license 부적합
- [x] [VEC10-005] `tests/work/vector-research/` 격리 prototype: 같은 canonical chunk·embedding의 세 engine build·query·delete·rebuild — Dense quality와 동일 384-dimension storage engine 격리 실행
- [x] [VEC10-006] 50,000 chunk·100 collection benchmark와 semantic Recall@10·exact-fact 무회귀·scope filter·disk·RAM·build 비용 비교 — Dense semantic +15.0 points·exact 무회귀, engine p95 통과, full embedding build 실패
- [x] [VEC10-007] Hard gate 판정과 exact engine·embedding 선택 또는 `defer`; 통과 실패의 product dependency 추가 `0건` — `defer`, dependency `0건`
- [x] [VEC10-008] 통과 시 derived vector generation schema, model·engine receipt, chunk digest mapping과 scope별 물리 격리 — `not-applicable-by-gate`, 제품 schema 추가 `0건`
- [x] [VEC10-009] 승인형 local embedding sidecar preview·install·staging build·atomic activation·update·disable 계약 — `not-applicable-by-gate`, sidecar 추가 `0건`
- [x] [VEC10-010] FTS·dense vector·native relation·Graphify code 결과의 rank fusion·citation·matched-lane 표시 — `not-applicable-by-gate`, 기존 FTS·graph planner 유지
- [x] [VEC10-011] Canonical 변경의 incremental vector 갱신·full rebuild 동등성, stale model·dimension mismatch·손상 index fallback·rollback — `not-applicable-by-gate`, vector derived state `0건`
- [x] [VEC10-012] `.hivekb` vector·model 제외, destination rebuild, Windows x64·macOS arm64·Linux musl 공개 시험 수용 — `not-applicable-by-gate`, bundle·세 운영체제 제품 범위 추가 `0건`

## Hard gate

- Paraphrase·cross-language Recall@10: FTS 대비 `15 percentage points` 이상 향상과 `90%` 이상
- Exact ID·날짜·수치·부정 질문: FTS 결과 저하 `0건`
- Vector lookup p95: 50,000 chunk warm `50ms` 이하
- End-to-end query p95: warm `500ms`, cold `2s` 이하
- Reference Windows x64 CPU full build: `10분` 이하
- 100개 changed chunk incremental build: `30초` 이하
- Index·metadata: 50,000 chunk 기준 `512MiB` 이하, embedding model 별도 기록
- Shared·project-private·confidential 누출 `0건`
- Offline rebuild와 provider API·API key·background server·network `0건`
- Engine·model 부재·손상·mismatch의 FTS·graph 영향 `0건`

## Engine 후보

| 후보 | 조사 이유 | 초기 경계 |
| --- | --- | --- |
| Qdrant Edge | Embedded Rust·HNSW·offline·별도 server 없음 | Exact crate·license·세 운영체제 검증 전 미채택 |
| sqlite-vec | 기존 SQLite와 가까운 저장·query 경로 | ANN pre-release와 dynamic extension 경계 분리 |
| SQLite-Vector | In-process·cross-platform·SIMD claim | License·Rust packaging·filter·maturity 직접 검증 필요 |

Full Qdrant server·cloud: background service·network·credential 경계로 제외.

## 결과 계약

```text
matched_lanes: [fts, vector, markdown-graph, code-graph]
vector_model: <exact-id-and-digest>
vector_engine: <exact-id-and-digest>
score: <lane-local-score>
fusion_rank: <final-rank>
locator: <canonical-locator>
digest: <canonical-content-digest>
visibility: <scope>
```

Vector score만으로 사실 확정 금지. Canonical source·visibility·citation 확인 필수.
