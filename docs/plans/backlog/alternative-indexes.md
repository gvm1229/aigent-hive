# Alternative vector indexes

- 상태: `researching`
- 마지막 검토일: 2026-08-22
- 관련 결정: [`ADR-0016`](../../decisions/ADR-0016-global-knowledge-rag.md)
- 활성 조사: [`hybrid-vector-search-0.10.0.md`](../active/hybrid-vector-search-0.10.0.md)

## 문제

FTS5와 선택형 Graphify로 충족하지 못하는 fuzzy recall·대규모 관계 검색 가능성.

## 기대 효과

측정된 검색 결손의 최소 보완.

## 현재 제외 이유

현재 bilingual recall·latency 기준 통과. Qdrant Edge·SQLite vector engine·local embedding의
품질·비용·세 운영체제 근거 미확정. `0.10.0` hard gate 통과 조합만 optional 구현.

## 승격 조건

재현 가능한 recall·latency 실패와 dependency·license·보안 검토 근거 확보.

Active 조사 실패: product dependency `0건`, 이 Backlog의 장기 후보 유지.
