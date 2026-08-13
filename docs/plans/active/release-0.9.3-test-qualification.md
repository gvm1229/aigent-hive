# `0.9.3` 시험·정식 출시 계획

> Checklist owner: `REL93-*`
> 대상: `0.9.3`
> 선행: `VAL93-001–004`, `OPT93-001–005`
> 정식판 전제: numbered public test 수용, protected `main` 검토, stable publication 환경 승인
> 실행 graph: [`v0.9.3-release-loop.graph.md`](v0.9.3-release-loop.graph.md)

## 출시 원칙

- stable 채널: 시험 수단 사용 금지
- source·package·설치·복구·성능의 최초 증거: exact numbered public test만 인정
- test 수용 뒤 제품 byte 변경: affected acceptance 재개방, 다음 번호 test 재수행
- 문서·계획·state만 바뀐 동일 tree: 제품 시험 재게시 금지
- `N10-002–011`: `0.10.0-test` 후보 유지, `0.9.3` 범위 제외

## Checklist

- [ ] [REL93-011] exact candidate에서 numbered public `0.9.3-test.N` build·publish·artifact attestation·clean install 수용
- [ ] [REL93-012] test 수용 뒤 protected `main` 통합과 exact stable candidate 생성
- [ ] [REL93-013] protected `main` 검토와 `release-publication` 환경 승인 뒤 GitHub Release·npm `latest=0.9.3` 연속 게시
- [ ] [REL93-014] 이 Windows public stable 설치·version·release date·validate·public docs 최종 확인
- [ ] [REL93-015] stable 수용 뒤 reviewed `knowledge-import`로 source architecture·intent·decision·fact 색인

## 수락 기준

- `VAL93-001–004`·`OPT93-001–005`·`REL93-011–015` evidence-backed 완료
- stable 출시 전 agent-owned checklist `0건`
- 정식판 전 최초 제품 동작·포장·설치·복구 증명 `0건`
- `0.10.0-test` 제외: `N10-002–011`만, `PLAN.md`의 target 명시 유지
