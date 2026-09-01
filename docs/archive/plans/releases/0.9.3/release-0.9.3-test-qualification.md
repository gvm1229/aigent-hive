# `0.9.3` 시험·정식 출시 계획

> Checklist owner: `REL93-*`
> 대상: `0.9.3`
> 선행: `VAL93-001–004`, `OPT93-001–005`, `SKI93-001–003`, `KBA93-001–005`
> 정식판 전제: qualified public test 수용, protected `main` 검토, stable publication 환경 승인
> 실행 graph: [`v0.9.3-release-loop.graph.md`](v0.9.3-release-loop.graph.md)

## 출시 원칙

- stable 채널: 시험 수단 사용 금지
- source·package·설치·복구·성능의 최초 증거: exact qualified public test만 인정
- test 수용 뒤 제품 byte 변경: affected acceptance 재개방, 다음 qualified test 재수행
- 문서·계획·state만 바뀐 동일 tree: 제품 시험 재게시 금지
- `N10-002–011`: `0.10.0-test` 후보 유지, `0.9.3` 범위 제외

## Checklist

- [x] [REL93-011] exact candidate `114817677e83aae535bd1f8b47518bf9b6745432`에서 public `0.9.3-test` build·publish·artifact attestation·Windows clean install·`0.9.2 → 0.9.3-test` upgrade·pending receipt recovery 수용
- [x] [REL93-012] PR #30의 protected `main` 통합 `e4c5b5eec8b24b0e4815f515eb26bda66e99c73c`과 exact stable candidate run `31741922500` 생성
- [x] [REL93-013] stable candidate 수용 뒤 publication run `31742946016`으로 GitHub Release·npm `latest=0.9.3` 연속 게시
- [x] [REL93-014] 이 Windows x64 public stable direct install·global npm install·preserving user reinstall·version·release date·`install --validate`·public docs 확인
- [x] [REL93-015] stable installed Hive에서 PortareFolium target의 explicit `collection:aigent-hive` source knowledge retrieval·canonical citation 재확인

## 수락 기준

- `VAL93-001–004`·`OPT93-001–005`·`REL93-011–015` evidence-backed 완료
- stable 출시 전 agent-owned checklist `0건`
- 정식판 전 최초 제품 동작·포장·설치·복구 증명 `0건`
- `0.10.0-test` 제외: `N10-002–011`만, `PLAN.md`의 target 명시 유지

## REL93-011 evidence

- Candidate workflow: [run 31737049684](https://github.com/gvm1229/aigent-hive/actions/runs/31737049684), macOS arm64·macOS x64·Linux x64/arm64·Windows x64 PASS
- Public test: [GitHub Release `v0.9.3-test`](https://github.com/gvm1229/aigent-hive/releases/tag/v0.9.3-test), npm `test=0.9.3-test`, `latest=0.9.2` 유지
- Local Windows x64: isolated direct install `0.9.2`, public test upgrade, matching pending receipt recovery, `AIgent Hive v0.9.3-test · developer test build (released 2026-08-14)`, pending receipt `0건`
- Performance: release RAG 50,000 chunks cold p95 `170.8967 ms`, warm p95 `0.1452 ms`; bundle 100 collections·50,000 chunks export p95 `1008.4492 ms`, import/rebuild p95 `3713.0145 ms`

## Stable evidence

- Protected integration: [PR #30](https://github.com/gvm1229/aigent-hive/pull/30), current CI names로 갱신한 `Protect main` rule과 merge commit `e4c5b5e`
- Stable candidate: [run 31741922500](https://github.com/gvm1229/aigent-hive/actions/runs/31741922500), 5 native targets·npm umbrella·direct installers·integrity bundle PASS
- Publication: [run 31742946016](https://github.com/gvm1229/aigent-hive/actions/runs/31742946016), [GitHub Release `v0.9.3`](https://github.com/gvm1229/aigent-hive/releases/tag/v0.9.3), npm `latest=0.9.3`
- Actual Windows x64: direct installer receipt `0.9.3`, global npm `0.9.3`, preserving user reinstall과 `hive install --validate` PASS, `AIgent Hive v0.9.3 (released 2026-08-14)`
- Installed retrieval: PortareFolium에서 `collection:aigent-hive`를 명시해 source-private citation 5건 반환, 자동 cross-project 혼입 없음
