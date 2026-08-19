# Graphify 지식 graph `0.10.0`

> Checklist owner: `GPH10-*`
> 기준 package: `graphifyy==0.9.47`

## 결정

- Markdown: 유일한 지식 정본
- SQLite: 직접 사실·본문 검색
- Graphify: 관계·경로·영향 범위의 선택형 파생 graph
- 조사 하드 게이트 통과 전 제품 의존성·설정·CLI 추가 금지

## 조사 Checklist

- [x] [GPH10-001] 유지보수·license·dependency·보안·출력 schema 조사 기록
- [x] [GPH10-002] 설치·추출 가용 증거 판정: Windows 실제 실행, Linux upstream CI, macOS·Markdown 미검증
- [x] [GPH10-003] source·project·global 각 10개 관계 질문 작성과 수용 중단 사유 기록
- [x] [GPH10-004] 구조 graph 반복 생성·증분 갱신·전체 재생성 동등성 검증
- [x] [GPH10-005] 성능·범위 격리·무변경·무자격 증명·network 경계 검증
- [x] [GPH10-006] 채택 또는 backlog 이관 결정과 근거 기록

## 조건부 제품 이관 기록

원래 `GPH10-007–016` 제품 범위: 하드 게이트 실패로 미착수. 버전 비종속
[`graphify-knowledge-graph.md`](../backlog/graphify-knowledge-graph.md)로 이전. 현재 완료율에서 제외.

## 하드 게이트

- 관계 질문 근거 경로 성공률 `>=90%`
- warm query p95 `<=500ms`, cold CLI query p95 `<=2s`
- canonical Markdown·collection registry 변경 `0건`
- critical·high finding, 범위 누출, API key 읽기, provider API 호출, query log `0건`
- Graphify 부재·손상·schema 불일치의 기존 FTS 영향 `0건`

하드 게이트 결과: 증분 동등성·visibility 격리 실패. 제품 통합 중단과 `0.10.0` 범위 재검토.
