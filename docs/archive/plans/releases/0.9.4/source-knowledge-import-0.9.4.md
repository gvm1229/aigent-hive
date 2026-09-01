# `0.9.4` source knowledge scan 정합성 계획

> Checklist owner: `SKI94-*`
> 대상: `0.9.4` patch
> 선행: published `0.9.3` stable immutable
> 요청: source의 architecture·intent·decision·fact를 설치 Hive의 private collection에 안전하게 기록하고 명시 질의 가능 상태 유지

## 문제

동일한 digest-bound reviewed claim 파일에서 `hive knowledge scan --candidates`는 store-level
credential 검증 없이 성공하지만 `--apply`는 뒤늦게 실패 가능. 실제 benign source fact
`source-fact-v0-9-skill-suite-plan`이 이 경로로 거부됐으며 오류 문구는 claim이 아니라 raw source를 지목.

## 수락 기준

- [x] [SKI94-001] candidate와 apply는 registry·index mutation 전 동일한 store-level validation을 실행하며 identical review의 security acceptance 결과 차이 `0건` — `knowledge::tests::scan_apply_rejects_credentials_before_registry_or_index_mutation` PASS
- [x] [SKI94-002] benign source fact·claim identifier·canonical claim serialization은 credential false-positive 없이 수용하고 실제 credential fixture는 candidate와 apply 모두 claim-bound diagnostic으로 거부 — `store::tests::scan_claim_human_review_id_is_not_misclassified_as_a_credential` PASS, Wiki 112·CLI 376 tests와 strict Clippy PASS
- [x] [SKI94-003] current-truth source fact·architecture·decision review를 source-private `aigent-hive` collection에 apply, automatic shared promotion `0건`, external consumer target의 explicit `collection:aigent-hive` retrieval 확인 — source build candidate·apply 61 claims, collection `collection-d9f7…129c5`, generation 115, PortareFolium direct retrieval 10 hits

## 경계

- `0.9.3` GitHub Release·npm stable artifact·published test artifact 변경 없음
- source Markdown은 정본, user-root Markdown collection과 SQLite는 direct retrieval용 파생·재생성 가능 상태
- current collection의 ordinary `auto` retrieval에 source-private result 혼입 없음
- raw source·runtime state·credential·historical claim: 무차별 수집 대상 제외
- scan 대상 source root mutation 없음

## 실행 순서

1. exact benign claim과 actual credential fixture로 candidate/apply parity regression 재현
2. shared validation과 claim-bound diagnostic 보정, canonical rendering false-positive 확인
3. current-truth source review 재생성·apply, source Wiki lint와 explicit retrieval 확인
