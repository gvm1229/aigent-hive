# `0.10.1` 공개 시험

> Checklist owner: `REL101-*`
> 첫 공개 시험: `0.10.1-test.1`
> Stable baseline: `0.10.0`

## Checklist

- [ ] [REL101-001] `0.10.1` version·migration·release metadata와 product-byte gate 정합화
- [ ] [REL101-002] Rust workspace·strict Clippy·Python 전체 lane·문서·보안·rollback 검사
- [ ] [REL101-003] Candidate와 `0.10.1-test.1` 게시, npm `test`·GitHub prerelease 독립 확인
- [ ] [REL101-004] Windows x64·macOS arm64·Linux musl x64 공개 artifact 설치·upgrade 수용
- [ ] [REL101-005] Source·artifact digest·실행 host·통과·건너뜀·미증명 범위 기록

## 출시 경계

- npm `latest=0.10.0` 유지
- Stable `0.10.1` 후보·tag·publication·설치 제외
- 실제 DuckSoul apply 제외
