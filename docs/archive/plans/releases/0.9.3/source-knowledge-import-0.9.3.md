# `0.9.3` source knowledge import 계획

> Checklist owner: `SKI93-*`
> 대상: `0.9.3`
> 선행: `KPX-008–018`
> 요청: 설치 Hive가 source repository의 architecture·intent·decision·fact 질의에 재사용 가능한 검토 지식 보유
> access: default auto 범위 외부, explicit `collection:aigent-hive` direct retrieval 허용

## 문제

`hive knowledge scan`의 consumer project용 generic path guard: source workspace의 tracked
`.agents/` 경로에서 foreign namespace 오류 중단. source Wiki: 별도 SQLite index 전용,
설치 Hive의 user-root collection 생성 경로 부재.

## 수락 기준

- [x] [SKI93-001] Git source scan의 foreign host namespace: content read·claim·target mutation 없이 stable skip reason 기록 — `knowledge_scan` focused 20 PASS, source inventory의 `.agents/` 93개 foreign-host-namespace receipt
- [x] [SKI93-002] Windows Git root 경로 정규화·exact `safe.directory`와 tracked source 문서·manifest·구현 evidence의 inventory·digest-bound reviewed claim 검증·user-root collection 등록 — Windows source inventory `sha256:61fa…4d50`, included 824·skipped 236, collection `collection-d9f7…129c5`
- [x] [SKI93-003] 전체 source repository의 reviewed architecture·intent·decision·fact collection apply, canonical Markdown·derived index receipt와 installed retrieval 확인 — reviewed claim 19개 apply, source-private direct retrieval과 user-root automatic promotion receipt generation 110

## 경계

- host namespace의 foreign bytes 읽기·해석·실행 없음
- source target mutation 없음, collection identity에 basename·absolute path 사용 없음
- raw transcript·runtime state·secret·credential·generated·binary·license 콘텐츠 제외
- source Wiki는 source 정본 검사, source scan collection은 direct retrieval용 별도 canonical knowledge
- source claim의 user-root shared promotion은 `KBA93-*`의 index-time policy만 소유

## 실행 순서

1. scanner가 foreign host namespace를 fail-closed skip 처리하도록 보정
2. regression과 digest-bound scan contract 검증
3. inventory 확인, bounded source locator 검토, reviewed claims validate·apply
4. installed Hive retrieval receipt 확인
