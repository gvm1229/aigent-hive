# Hybrid vector search `0.10.0`

> Checklist owner: `KRG10-014`, `VEC10-*`, `VQR10-*`
> 작업 branch: `feature/0.10.0-vector-search`
> 채택 방식: 구현·격리 검증 후 측정 gate 통과한 조합만 공개 기능에 포함
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
- [ ] [VEC10-007] 기존 기준을 유지한 engine·embedding 채택 판정. 병렬 변환과 정적 임베딩 비교, 미통과 조합 공개 활성화 금지
- [ ] [VEC10-008] 파생 색인 세대·모델/engine 영수증·청크 digest·collection와 공개 범위별 물리 격리 구현
- [ ] [VEC10-009] 승인형 보조 환경 preview·설치·시간 제한 배치·원자 활성화·갱신·비활성화 구현
- [ ] [VEC10-010] FTS 보존, 의미 검색의 순위 결합·인용·검색 경로 표시. 벡터 점수만으로 사실 확정 금지
- [ ] [VEC10-011] 증분 갱신·중단 재개·전체 재생성 동등성, 오래된/손상된 색인의 FTS 복귀·rollback 구현
- [ ] [VEC10-012] Bundle 제외·다른 기기 재생성·세 운영체제 설치와 번호 공개 시험 수용

## 재검증 checklist

- [x] [VQR10-001] 특정 engine을 미리 채택하지 않고 embedding pipeline 최적화와 vector 재검증을 `0.10.0` 범위로 유지보수자 승인
- [x] [VQR10-002] 반복 50,000 corpus의 30 digest·5.75초와 고유 50,000 corpus의 50,000 digest·1,000 probe 분리 측정
- [x] [VQR10-003] MiniLM batch·4 ONNX thread 실제 측정과 FastEmbed 다국어 세 model의 크기·license·dimension 비교 — 가장 작은 model full build 실패
- [x] [VQR10-004] Digest cache·batch checkpoint·중단 재개·원자 pointer 구현, 100개 변경 7.20초·10개 추가/삭제 1.42초
- [x] [VQR10-005] 과거 임베딩 구간 측정: 같은 프로세스 p95 37.31ms·모델 생성/첫 변환 1회 643.45ms. 전체 검색·cold p95 통과 증거 제외
- [x] [VQR10-006] 50,000 실제 embedding prerequisite 실패로 Qdrant Edge·sqlite-vec 재비교 중단, 이전 engine 수치는 참고값만 유지
- [x] [VQR10-007] 여섯 scope의 physical research root 분리·제품 ANN 없음 확인, 실제 50,000 cross-scope ANN은 prerequisite 실패로 미실행
- [x] [VQR10-008] Research generation staging·checkpoint·atomic pointer·resume/one-shot digest 동등성, 미완료 generation 활성화 `0건`, FTS 변경 `0건`
- [x] [VQR10-009] Windows helper 수용 뒤 reference hard gate 실패로 macOS·Linux 후속 수용 중단, 제품 package `0건`
- [x] [VQR10-010] Hard gate 판정 `defer`: full build·세 platform 실패, optional adapter·product dependency `0건`

## 이전 gate 판정

### 실제 제품 경로의 추가 비교

- 실제 Wiki 30개·120문항: 초기 RRF는 80→90%로 미달, 점수 결합은 80→96.7%·정확 질문 100% 유지
- 공유 CPU 12개 작업의 고유 5만 변환 488.76초. 100개 모음 실제 CLI·운영체제 검증은 별도
- 별도 독립 질문으로 과도한 맞춤 여부 검증. 기존 품질·비용 기준 완화 금지
- 비교 정본: [제품 연결 재검증](../../research/vector-product-integration-2026-08-28.md)
- 기밀 승인·묶음 파일 제외·용량·세 운영체제 검증은 독립적으로 지속
- 성능 보강: collection별 최신성 분리, 조회의 반환 정본 검증, 지난 파생 세대의 안전한 용량 관리

- 2026-08-28 승인: 비벡터 수정 통합 뒤 전용 branch에서 벡터 구현 재개. 과거 `not-applicable` 완료 표시 취소
- 검색 품질·초기 생성 시간 기준의 임의 완화 금지. 통과 전 조사용 조합과 출하 조합 구분
- 실행·보안 계약: [벡터 구현 계약](../../architecture/vector-search.md)

- `0.10.0-test.1` 전 조사: Dense semantic Recall@10 `+15.0 points`, hybrid exact `100%`, 세 engine lookup 기준 통과
- 실패 지점: Windows x64의 naive 50,000 offline embedding build `600초` 초과
- Benchmark 한계: 30개 문서를 반복한 합성 scale에서 digest 중복 제거·resumable checkpoint·incremental build·query embedding 포함 p95 미검증
- 당시 `defer`와 product dependency `0건`은 유효한 과거 결정이며, 새 재검증 결과가 통과할 때까지 현재 제품 상태로 유지

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
- 합성 반복 corpus와 50,000 unique corpus 결과의 분리 해석
- Full build는 중단 뒤 재개 가능하고 완료 generation만 활성화

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
