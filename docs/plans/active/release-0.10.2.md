# `0.10.2` 공개 시험과 안정판

> Checklist owner: `REL102-*`
> 첫 공개 시험: `0.10.2-test.1`
> Stable baseline: `0.10.1`

## Checklist

- [x] [REL102-001] version·migration·release metadata와 product-byte gate 정합화
- [x] [REL102-002] Rust workspace·strict Clippy·Python·문서·보안·rollback 검사
- [x] [REL102-003] Candidate와 `0.10.2-test.1` 게시, npm `test`·GitHub prerelease 독립 확인
- [x] [REL102-004] Windows x64·macOS arm64·Linux musl x64 공개 artifact 신규 설치·`0.9.5` 갱신 수용
- [x] [REL102-005] source·artifact digest·실행 host·통과·건너뜀·미증명 범위 기록
- [x] [REL102-006] accepted public test를 source·product digest·세 host 영수증에 결합
- [x] [REL102-007] `0.10.2` stable 문서·구독자 요약·승인 digest 정합화
- [ ] [REL102-008] `develop → main` 통합과 stable publication
- [ ] [REL102-009] npm `latest=0.10.2`·GitHub stable Release·독립 확인

## 출시 경계

- 유지보수자가 stable `0.10.2`의 `main` 통합·tag·npm·GitHub 공개를 승인
- 실제 사용자 루트와 등록 프로젝트 설치는 제외
- product 또는 installer bytes 변경 뒤에는 다음 번호 공개 시험으로 영향 범위 재수용

## 공개 수용 근거

- 후보 실행 `34149429486`, 게시 복구 실행 `34150609316`, 수용 실행 `34150815006`
- 공개 tag `v0.10.2-test.1`, source `05526842f10d288feb19a5da8db363d0bfe0ab59`
- 제품 tree digest `sha256:fb7eab520803250ac4989c89617703f7bb3acab541536ebcbee9a6e67e7950a2`
- Windows x64, macOS arm64, Linux musl x64 영수증 artifact 3개
- 승인 구독자 안내 digest `sha256:ab8a9fac5f277c80d819e7bf3e698d5f443484f8c690eeed6f504d6b8e8f6112`
