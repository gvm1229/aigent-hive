# `0.9.3` 시험·정식 출시 계획

> Checklist owner: `REL93-*`
> 대상: `0.9.3`
> 선행: `NAT-001–024`, `MRA-001–032`
> 정식판 전제: numbered public test 수용, protected `main` 검토, stable publication 환경 승인
> 실행 graph: [`v0.9.3-release-loop.graph.md`](v0.9.3-release-loop.graph.md)

## 출시 원칙

- stable 채널: 시험 수단 사용 금지
- source·package·설치·복구·성능의 최초 증거: exact numbered public test만 인정
- test 수용 뒤 제품 byte 변경: affected acceptance 재개방, 다음 번호 test 재수행
- 문서·계획·state만 바뀐 동일 tree: 제품 시험 재게시 금지
- `N10-002–011`: `0.10.0-test` 후보 유지, `0.9.3` 범위 제외

## Checklist

- [ ] [REL93-001] `NAT-001–024` 현재 tree·직접 시험·fresh host evidence 재조정 완료
- [ ] [REL93-002] `MRA-001–032` 현재 tree·직접 시험·fresh host evidence 재조정 완료
- [ ] [REL93-003] provider API·credential·direct model/subagent process·OMX/OMC 신규 runtime static gate `0건`
- [ ] [REL93-004] Rust workspace·format·strict Clippy·Python conformance·security review 통과
- [ ] [REL93-005] Windows x64 clean install·upgrade·preserving uninstall/reinstall·validate·doctor 수용
- [ ] [REL93-006] Codex·Claude host-native profile projection·fresh-session discovery·exact attestation 수용
- [ ] [REL93-007] native loop graph initialize·validate·checkpoint·recover, hostile receipt·cancel·race 수용
- [ ] [REL93-008] team mailbox·barrier·shared-path lease·multi-goal budget·terminal Judge 수용
- [ ] [REL93-009] custom role preview·explicit consent·apply·rollback·foreign host file 보존 수용
- [ ] [REL93-010] README·설치 안내·plugin metadata·npm README·release note·bilingual fact 최신화
- [ ] [REL93-011] exact candidate에서 numbered public `0.9.3-test.N` build·publish·artifact attestation·clean install 수용
- [ ] [REL93-012] test 수용 뒤 protected `main` 통합과 exact stable candidate 생성
- [ ] [REL93-013] protected `main` 검토와 `release-publication` 환경 승인 뒤 GitHub Release·npm `latest=0.9.3` 연속 게시
- [ ] [REL93-014] 이 Windows public stable 설치·version·release date·validate·public docs 최종 확인

## 수락 기준

- `NAT-001–024`·`MRA-001–032`·`REL93-001–014` evidence-backed 완료
- stable 출시 전 agent-owned checklist `0건`
- 정식판 전 최초 제품 동작·포장·설치·복구 증명 `0건`
- `0.10.0-test` 제외: `N10-002–011`만, `PLAN.md`의 target 명시 유지
