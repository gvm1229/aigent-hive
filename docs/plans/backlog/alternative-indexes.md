# Alternative indexes

- 상태: `idea`
- 마지막 검토일: 2026-08-20
- 관련 결정: [`ADR-0016`](../../decisions/ADR-0016-global-knowledge-rag.md)

## 문제

FTS5와 선택형 Graphify로 충족하지 못하는 fuzzy recall·대규모 관계 검색 가능성.

## 기대 효과

측정된 검색 결손의 최소 보완.

## 현재 제외 이유

현재 bilingual recall·latency 기준 통과. 추가 vector DB·cloud DB의 구체적 결손 증거 부재.

## 승격 조건

재현 가능한 recall·latency 실패와 dependency·license·보안 검토 근거 확보.
