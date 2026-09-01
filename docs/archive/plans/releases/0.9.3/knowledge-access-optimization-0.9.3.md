# `0.9.3` knowledge access·자동 최적화 계획

> Checklist owner: `KBA93-*`
> 대상: `0.9.3`
> 선행: `KPX-008–018`, `SKI93-001–003`
> 결정: 기본 검색 격리 유지, 명시 source access 허용, index-time automatic promotion

## 사용자 의도

- Project A의 ordinary automatic lookup: Project B의 project-private knowledge 혼입 없음
- 사용자가 Project B 또는 unique collection alias를 명시: A에서 B collection의 knowledge 직접 조회
- 일반화 가능한 knowledge: 조회 시점이 아니라 scan·rescan·maintain 시점에 Hive가 자동 판정·승격
- 사용자: claim별 promote preview·confirmation·babysitting 부담 없음

## 수락 기준

- [x] [KBA93-001] `auto` retrieval은 current collection·user-root·shared만 사용하고 foreign project-private collection `0건` — `retrieval_enforces_scopes_confidentiality_and_detached_state` PASS, PortareFolium auto retrieval의 source-private hit `0건`
- [x] [KBA93-002] explicit `project:<id>`·`collection:<id-or-alias>` retrieval은 foreign project-private collection을 direct query하며 user-root 또는 unrelated collection 결과 혼입 `0건` — PortareFolium `collection:aigent-hive` direct retrieval의 source collection-only receipt
- [x] [KBA93-003] unknown·ambiguous project 또는 collection reference fail-closed, confidential collection은 exact current-action authorization 유지 — existing collection ambiguity·confidential authorization regression과 focused `hive-wiki` 111 PASS
- [x] [KBA93-004] scan·rescan·maintain의 reviewed safe-general claim은 user interruption 없이 root shared claim으로 atomic promotion; private·personal·secret·credential·ambiguous·conflicting claim 자동 승격 `0건` — source scan apply의 reviewed 19 claims, safe-general 2 claims automatic user-root promotion; candidate type·applicability·credential gate 추가
- [x] [KBA93-005] promotion provenance·source digest·deduplication·contradiction·supersede·source invalidation 및 auto-query/direct-query E2E 검증 — atomic promotion·replay·source invalidation regression PASS, source generation 110 receipt, direct retrieval은 promotion pipeline 호출 없음

## 동작 모델

```text
ordinary request in A → auto → A private + user-root + shared
"use Project B knowledge" → explicit B → B collection only
scan / rescan / maintain → reviewed safe-general classifier → user-root shared claim
```

`shared`: verified general knowledge 전용. B의 전체 table 공개 아님. Explicit B query:
source collection 직접 조회, promotion pipeline 호출 대상 아님.

## 안전 경계

- explicit source: registry의 stable ID·unique alias만 허용, name inference·ambiguous alias fallback 없음
- non-confidential project-private: explicit source query만 허용
- confidential: existing query-bound one-time authorization 필수
- automatic promotion: deterministic policy와 reviewed evidence 모두 충족한 claim만 허용
- user-root promotion: source collection·review ID·source digest·classification receipt 보존
- scan target·foreign source bytes·raw query·runtime state mutation 없음

## 실행 순서

1. scope SQL과 result contract를 exact collection-only로 보정
2. Skill routing에서 explicit project/collection intent를 unique registry reference로 해석
3. index-time automatic promotion policy·atomic transaction·maintenance rescan 연결
4. privacy·ambiguity·confidential·deduplication hostile matrix와 real source collection E2E
