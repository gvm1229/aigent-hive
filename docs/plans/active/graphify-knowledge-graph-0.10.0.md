# Graphify 지식 graph `0.10.0`

> Checklist owner: `GPH10-*`
> 기준 package: `graphifyy==0.9.47`

## 결정

- Markdown: 유일한 지식 정본
- SQLite: 직접 사실·본문 검색
- Graphify: 관계·경로·영향 범위의 선택형 파생 graph
- 조사 하드 게이트 통과 전 제품 의존성·설정·CLI 추가 금지

## 조사 Checklist

- [ ] [GPH10-001] 유지보수·license·dependency·보안·출력 schema 조사 기록
- [ ] [GPH10-002] Windows·macOS·Linux 격리 설치와 code·Markdown 추출 가능성 검증
- [ ] [GPH10-003] source·project·global 각 10개 관계 질문과 직접 사실 기준선 작성
- [ ] [GPH10-004] 구조 graph 반복 생성·증분 갱신·전체 재생성 동등성 검증
- [ ] [GPH10-005] 성능·범위 격리·무변경·무자격 증명·network 경계 검증
- [ ] [GPH10-006] 채택 또는 backlog 이관 결정과 근거 기록

## 조건부 제품 Checklist

- [ ] [GPH10-007] 승인형 exact-version Python 보조 환경의 preview·설치·검증·disable 계약
- [ ] [GPH10-008] source·project·global·collection별 graph manifest와 Hive 소유 저장 경계
- [ ] [GPH10-009] `hive knowledge graph preview|enable|status|rebuild|query|disable` 구현
- [ ] [GPH10-010] `hive source-wiki graph status|rebuild|query` 구현
- [ ] [GPH10-011] `knowledge-recall`·`knowledge-maintain`의 FTS·graph 역할 분리
- [ ] [GPH10-012] `EXTRACTED|INFERRED|AMBIGUOUS` 근거·canonical citation 결과 계약
- [ ] [GPH10-013] private·confidential collection별 격리 opt-in과 cross-scope 누출 차단
- [ ] [GPH10-014] pre-`0.10.0` canonical byte·FTS 결과 보존 upgrade·rollback
- [ ] [GPH10-015] `.hivekb`의 graph·보조 환경·cache 제외와 destination rebuild
- [ ] [GPH10-016] 세 운영체제 clean install·upgrade·failure recovery 수용

## 하드 게이트

- 관계 질문 근거 경로 성공률 `>=90%`
- warm query p95 `<=500ms`, cold CLI query p95 `<=2s`
- canonical Markdown·collection registry 변경 `0건`
- critical·high finding, 범위 누출, API key 읽기, provider API 호출, query log `0건`
- Graphify 부재·손상·schema 불일치의 기존 FTS 영향 `0건`

하드 게이트 실패: `GPH10-007–016` 미착수, 후보 backlog 이전, `0.10.0` 범위 재검토.
