# Vector search와 파일 기반 memory 검토

> 조사일: 2026-08-22
> 대상: Hive `0.9.5`, Anthropic memory tool·Managed Agents memory, Qdrant·SQLite vector 후보
> 판정: Vector DB의 즉시 확정 대신 품질·속도 hard gate 통과 뒤 조건부 `0.10.0` 구현

## Hive 현재 상태

- 검색 엔진: SQLite FTS5 `MATCH`·BM25
- Embedding·dense vector·HNSW: 없음
- 50,000 chunk·100 collection 검증: warm p95 약 `0.134ms`, fresh p95 약 `170ms`
- Vector 도입 목적: raw 속도보다 paraphrase·다국어 semantic recall

## Anthropic 공식 확인

### Memory tool

- Claude: `/memories`의 `view|create|str_replace|insert|delete|rename` 요청
- 사용자 application: 실제 storage 작업과 path confinement 소유
- Backend: local files·database·cloud storage·encrypted files 선택 가능
- 결론: vector DB 폐기 선언 아님, storage-agnostic file-operation interface

공식 문서: [Anthropic memory tool](https://platform.claude.com/docs/en/agents-and-tools/tool-use/memory-tool)

### Managed Agents memory store

- Workspace-scoped text document collection
- Session sandbox에 directory로 mount
- 작은 focused file 권장
- 변경마다 immutable version과 audit·rollback·redaction
- Read-only·read-write access

공식 문서: [Managed Agents memory](https://platform.claude.com/docs/en/managed-agents/memory)

### Dreams

- 기존 memory store와 1–100개 session transcript의 비동기 model 검토
- 중복·모순·stale entry 정리와 별도 output store
- Input store 불변
- Research preview·provider model 실행 필요

공식 문서: [Anthropic Dreams](https://platform.claude.com/docs/en/managed-agents/dreams)

Hive 적용 판정: 파일 정본·version·감사·만료 pattern은 현재 architecture와 일치. Anthropic API,
Managed Agents, Dreams 직접 연동은 provider API·credential·model runtime 경계로 제외.

## 검색 역할 비교

| 계층 | 강점 | 약점 |
| --- | --- | --- |
| Markdown file memory | 사람 검토·수정·version·감사·삭제 | 대규모 semantic nearest-neighbor 검색 |
| SQLite FTS | Exact term·ID·날짜·수치·매우 낮은 latency | 표현이 다른 paraphrase·cross-language recall |
| Dense vector | Semantic similarity·다국어 후보 검색 | 정확한 부정·수치·관계·출처 확정 |
| Relationship graph | 명시적 edge·다단계 path·영향 범위 | 표현 유사성 후보 검색 |

권장: FTS·vector·graph의 hybrid retrieval. Vector의 Markdown 정본 대체 금지.

## Qdrant 공식 확인

- Dense vector의 semantic similarity와 HNSW approximate nearest-neighbor
- Exact keyword와 semantic query를 결합한 hybrid search 권장
- Qdrant Edge: embedded Rust·in-process·offline·별도 server·network 없음
- Embedding neural encoder: vector DB와 별도 필수 구성 요소

공식 문서:

- [Vector search](https://qdrant.tech/documentation/overview/vector-search/)
- [Hybrid search](https://qdrant.tech/documentation/search/text-search/hybrid-search/)
- [Qdrant Edge](https://qdrant.tech/documentation/edge/edge-quickstart/)

## 구현 긴장

Hive 현재 금지: provider API·credential·직접 generative model runtime. Semantic vector에는
embedding model 필요. 조건부 해법: 사용자 승인형 non-generative local embedding indexer를
별도 파생 index boundary로 정의하고 exact model·license·digest·resource budget 고정.

이 boundary 자체가 acceptance 실패 시 vector product implementation 중단.

## 권장 순서

1. FTS gold baseline
2. Embedding architecture·candidate review
3. Qdrant Edge·SQLite engine 격리 prototype
4. Quality·latency·disk·RAM·scope benchmark
5. Hard gate `adopt|defer`
6. 통과 조합만 optional hybrid adapter 구현
7. Upgrade·rollback·세 운영체제 공개 시험
