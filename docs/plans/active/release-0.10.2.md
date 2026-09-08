# `0.10.2` 공개 시험과 안정판

> Checklist owner: `REL102-*`
> 첫 공개 시험: `0.10.2-test.1`
> 수용 공개 시험: `0.10.2-test.3`
> 공개 Stable: `0.10.2`

## Checklist

- [x] [REL102-001] version·migration·release metadata와 product-byte gate 정합화
- [x] [REL102-002] Rust workspace·strict Clippy·Python·문서·보안·rollback 검사
- [x] [REL102-003] Candidate와 최신 번호 공개 시험 게시, npm `test`·GitHub prerelease 독립 확인
- [x] [REL102-004] Windows x64·macOS arm64·Linux musl x64 공개 artifact 신규 설치·`0.9.5` 갱신 수용
- [x] [REL102-005] source·artifact digest·실행 host·통과·건너뜀·미증명 범위 기록
- [x] [REL102-006] accepted public test를 source·product digest·세 host 영수증에 결합
- [x] [REL102-007] `0.10.2` stable 문서·구독자 요약·승인 digest 정합화
- [x] [REL102-008] `develop → main` 통합과 stable publication
- [x] [REL102-009] npm `latest=0.10.2`·GitHub stable Release·독립 확인

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
- `test.1` 수용 뒤 Stable Skill 이력 제품 바이트 보완으로 제품 digest 변경, `test.2` 재수용 필요
- `test.2` 후보 `34185228729`, 게시 `34185931583`, 수용 `34186318745`
- `test.2` source `cfc0988cab0c599e4d347e1c6d72aab0851846ab`, 제품 digest `sha256:9cc1a609990d6dd5cfc04620a9feb29302becec13c73af92e5613e1d408896fe`
- `test.2` 수용 뒤 historical runtime 범위 보정으로 제품 digest 변경, `test.3` 재수용 필요
- `test.3` 후보 `34188057208`, 게시 복구 `34188994476`, 수용 `34189195317`
- `test.3` source `984221d93f0ce20660a81d1f568120d02272983c`, 제품 digest `sha256:468b576252174cc9082ccf2513b272a73b4040e46bce8193179a6e8943b0cafc`
- Windows x64·macOS arm64·Linux musl x64 공개 package 설치와 전역 갱신·질문 차단·재개·rollback 수용 성공
- 세 host 영수증 artifact: `korean-public-test-win32-x64`, `korean-public-test-darwin-arm64`, `korean-public-test-linux-x64`
- Linux와 Windows의 SQLite-capable Python 선택 단계: 조건 불충족에 따른 건너뜀. 각 host의 공개 binary 선택 vector 수용 단계: 성공. 실제 사용자 루트와 등록 프로젝트 설치: 미실행
- Stable 통합 PR `#54`, merge commit `059973ee8c71d396170bc4e41c6c7caa38127c`
- Stable 후보 `34190329196`, npm 첫 게시 `34199520577`, staged package 복구 확인 `34200229980`, 최종 복구 게시 `34200601474`
- npm 여섯 package: `0.10.2`, 각 `latest=0.10.2`, 각 `dist.integrity` 존재
- GitHub Release `v0.10.2`: 정식판, target `059973ee8c71d396170bc4e41c6c7caa38127c`, binary·npm·installer·attestation·integrity receipt 자산 공개
- annotated tag `v0.10.2`: Stable merge commit 지시 확인
