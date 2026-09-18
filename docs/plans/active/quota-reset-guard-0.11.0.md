# `0.11.0-test.1` Quota Reset Guard

> Plan version: 0.11.0
> Scope: product

> 소유 항목: `RB104-001–004`
> 상태: 구현 중

- 범위 결정: 미출시 `0.10.4` 변경을 `0.11.0`으로 통합, 별도 `0.10.4` 출시 제외
- 기존 구현: `1286c75e`, 이름 정리: `41fda8bc`, 검증 기록: `87b84f42`
- 추적 보존: `RB104-*` 식별자 유지, 기존 구현 3개는 실패 기록을 대체하는 새 Windows Rust 회귀로 재검증
- 소유 결정: [ADR-0023](../../decisions/ADR-0023-foundation-refactor.md), 호스트 확인: [RFH-001](refactor-context-hosts.md)

- [x] [RB104-001] 같은 계정·quota pool·window의 잔량 증가를 초기화로 판정하는 core 계약과 단위 시험
  - state: complete; evidence: repo:tests/results/runs/20260918T164022-1065e14ef4f2.md#sha256:64ac7576d2c87ef31693741eaa88dda42ef35fcbff1cf8c4e140be0d3efc66a0
- [x] [RB104-002] 전역 `quota_reset_guard_enabled` 기본값·구버전 설정 이관·스키마 반영
  - state: complete; evidence: repo:tests/results/runs/20260918T164022-1065e14ef4f2.md#sha256:64ac7576d2c87ef31693741eaa88dda42ef35fcbff1cf8c4e140be0d3efc66a0
- [x] [RB104-003] `enforce` 경계의 기준 관측 저장과 `hive.usage-reset` 자동 실행 차단
  - state: complete; evidence: repo:tests/results/runs/20260918T164022-1065e14ef4f2.md#sha256:64ac7576d2c87ef31693741eaa88dda42ef35fcbff1cf8c4e140be0d3efc66a0
- [ ] [RB104-004] 15초 host 감시, 진행 중인 작업 중단, 현재 작업별 opt-out·명시적 재개 계약
  - state: awaiting-external-evidence; depends: RFH-001; owner: 호스트 계약; reason: 15초 감시와 진행 중 추론 중단의 지원 경로 미검증

기존 검토의 한계: Hive에서 진행 중 추론 중단·작업별 15초 감시에 사용할 검증된 Codex API 부재.
`RFH-001`에서 현재 계약 재확인 후 지원 경로만 구현; 미지원 유지 시 `RB104-004` 미완료 유지와
정확한 제한 보고. 이 항목의 출시 제외는 유지보수자의 별도 범위 결정 필요.
