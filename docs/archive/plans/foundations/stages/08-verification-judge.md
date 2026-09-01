# Stage 8. Deterministic verification과 hostile judge

> 상태: implemented workflow reference; 이 stage 검토 시에만 load

#### 완성 후 동작

먼저 build, test, lint, schema, file ownership, link 검사처럼 결정론적 검증 수행. 그 뒤 사용자가 요청하거나 risk policy가 요구하면 독립 judge 실행.

Judge가 받는 context:

- 원래 목표와 acceptance
- artifact 또는 diff
- fresh verification evidence
- 알려진 제약

Judge가 받지 않는 context:

- task agent의 chain-of-thought 또는 reasoning
- task agent의 자기 점수·자기 칭찬
- 원하는 verdict를 암시하는 지시
- 다른 judge의 verdict

Risk tier:

| Tier | 예 | 판정 |
| --- | --- | --- |
| normal | 작은 문서·저위험 코드 | 요청 시 independent judge 1명 |
| elevated | cross-file architecture, migration, 보안 경계 | 3명 중 2명 PASS |
| critical | release signing, destructive migration, security-sensitive update | 3명 전원 PASS + human approval |

Judge는 `PASS`, `FAIL`, `INDETERMINATE`만 반환. `FAIL`은 재현 가능한 finding, `INDETERMINATE`는 부족한 evidence를 명시.

#### 구현

- host 또는 external runtime의 clean independent agent 사용; Hive CLI는 judge를
  실행하거나 spawn 금지
- 각 judge에 `schemas/judge-package.schema.json`을 따르는 동일 digest의 최소 context envelope 개별 전달
- verdict 전 exact package·criteria, requester, task agent, resolved owner와
  authenticated owner provenance, distinct slot/instance/eligibility tuple을
  `judge-assignment` JCS digest로 고정
- 각 결과는 `schemas/judge-verdict.schema.json`으로 검증
- verdict는 assignment digest, exact assigned tuple과 assignment 뒤 timestamp에 결합
- critical human approval은 모든 eligible verdict 뒤 별도 `judge-approval` JCS
  artifact로 고정하고 requester/task agent approval을 거부
- verdict 전에 다른 judge 결과 공개 금지
- quorum 계산은 deterministic code
- FAIL finding은 affected criterion/task에 연결
- `hive-judge-package`는 deterministic verification 뒤 read-only package 생성만
  수행하며 bounded knowledge retrieval, simple-question gate와 host-native 기본값 보존
- package, assignment, verdict와 approval은 target-contained target-relative path만
  bounded no-follow read하며 aggregate output에서 identity, slot, finding, digest와
  개별 verdict를 숨김
- owner, judge instance와 critical human approver는 consumer target 밖의
  agent-write-denied TOML trust root에 등록한 purpose-bound public key로 각각
  detached Ed25519 attestation을 검증
- Hive는 strict verification만 수행하고 private-key 생성·읽기·보관·signing은 외부
  authority가 소유. Target 내부 self-certified key, caller-supplied identity digest와
  공개 입력만으로 재계산 가능한 digest는 authentication evidence로 불인정
- unsigned v1 quorum은 diagnostic compatibility만 제공하고 completion-authorizing
  PASS를 반환 금지

#### 완료 조건

- [x] task agent가 자신의 결과를 최종 승인 금지
- [x] judge 간 verdict leakage 0회
- [x] 2/3와 3/3+human gate unit test
- [x] missing evidence는 PASS가 아닌 INDETERMINATE
- [x] verdict의 `package_digest`가 원본 judge package와 다르면 quorum 제외
- [x] provenance-bound critical human approval 없이 completion 불가
- [x] trusted owner/judge/human signature 또는 host attestation을 user/host-controlled
  trust root에 대해 검증
