# Graphify 전면 지식 graph

- 상태: `blocked`
- 마지막 검토일: 2026-08-22
- 관련 조사: [`graphify-0.10-feasibility.md`](../../research/graphify-0.10-feasibility.md)
- 제한 채택: [`knowledge-relationship-graph-0.10.0.md`](../active/knowledge-relationship-graph-0.10.0.md)

## 문제

Markdown 사실 검색만으로 찾기 어려운 code·문서·지식의 다단계 관계와 영향 범위 탐색 필요.

## 기대 효과

- 명시적 node·edge 기반 영향 분석
- AST 관계의 `EXTRACTED` 근거
- source·project·global knowledge의 관계 탐색

## 현재 제외 이유

- Graphify code-only full-rebuild adapter는 `0.10.0` active scope로 분리
- `0.9.47` 증분 갱신과 전체 재생성 graph 불일치
- 단일 upstream global graph의 collection visibility 격리 부재
- Markdown 의미 추출의 host-owned provider-neutral adapter 부재
- macOS·Linux·50,000 chunk 검증 부재

## 선행 조건

- Upstream 또는 Hive adapter의 증분·전체 정규화 동등성
- shared·project-private·confidential collection별 분리 graph
- provider API·API key 없는 host-owned 의미 추출 receipt
- safe pip·exact dependency lock·세 운영체제 설치
- 30개 관계 질문과 50,000 chunk 성능 수용

## 승격 조건

위 선행 조건 전체와 canonical Markdown·SQLite 무변경 upgrade·rollback 증거 확보.

이 Backlog 항목: Graphify가 Markdown·global·private·confidential 관계 전체를 소유하는
확장 범위. Active code-only adapter와 별도.
