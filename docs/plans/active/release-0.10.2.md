# `0.10.2` 공개 시험과 안정판

> Checklist owner: `REL102-*`
> 첫 공개 시험: `0.10.2-test.1`
> Stable baseline: `0.10.1`

## Checklist

- [x] [REL102-001] version·migration·release metadata와 product-byte gate 정합화
- [x] [REL102-002] Rust workspace·strict Clippy·Python·문서·보안·rollback 검사
- [ ] [REL102-003] Candidate와 `0.10.2-test.1` 게시, npm `test`·GitHub prerelease 독립 확인
- [ ] [REL102-004] Windows x64·macOS arm64·Linux musl x64 공개 artifact 신규 설치·`0.9.5` 갱신 수용
- [ ] [REL102-005] source·artifact digest·실행 host·통과·건너뜀·미증명 범위 기록
- [ ] [REL102-006] accepted public test를 source·product digest·세 host 영수증에 결합
- [ ] [REL102-007] `0.10.2` stable 문서·구독자 요약·승인 digest 정합화
- [ ] [REL102-008] `develop → main` 통합과 stable publication
- [ ] [REL102-009] npm `latest=0.10.2`·GitHub stable Release·독립 확인

## 출시 경계

- 유지보수자가 stable `0.10.2`의 `main` 통합·tag·npm·GitHub 공개를 승인
- 실제 사용자 루트와 등록 프로젝트 설치는 제외
- product 또는 installer bytes 변경 뒤에는 다음 번호 공개 시험으로 영향 범위 재수용
