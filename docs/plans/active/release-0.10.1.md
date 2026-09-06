# `0.10.1` 공개 시험

> Checklist owner: `REL101-*`
> 첫 공개 시험: `0.10.1-test.1`
> Stable baseline: `0.10.0`

## Checklist

- [x] [REL101-001] `0.10.1` version·migration·release metadata와 product-byte gate 정합화
- [x] [REL101-002] Rust workspace·strict Clippy·Python 전체 lane·문서·보안·rollback 검사
- [x] [REL101-003] Candidate와 `0.10.1-test.1` 게시, npm `test`·GitHub prerelease 독립 확인
- [x] [REL101-004] Windows x64·macOS arm64·Linux musl x64 공개 artifact 설치·upgrade 수용
- [x] [REL101-005] Source·artifact digest·실행 host·통과·건너뜀·미증명 범위 기록
- [x] [REL101-006] accepted public test를 source·product digest·세 host 영수증에 묶는 stable promotion mode 구현
- [x] [REL101-007] `0.10.1` stable 공개 문서·구독자 요약·승인 digest 정합화
- [ ] [REL101-008] `develop → main` 통합과 accepted-test promotion candidate·stable publication
- [ ] [REL101-009] npm `latest=0.10.1`·GitHub stable Release·설치 갱신 독립 확인

## 출시 경계

- npm `latest=0.10.0` 유지
- Stable `0.10.1`은 2026-09-06 유지보수자 승인에 따라 accepted-test promotion으로 진행
- 실제 DuckSoul apply 제외
