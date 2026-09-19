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

## 명시적 확인의 구현 범위

- `4f1a1d52`: 초기화 중단의 재조회·프로세스 변경에 따른 자동 해제 수정, 표식 지문에 결합한 `acknowledge-reset` 추가
- 확인 후 보호 유지와 새 측정 필수. [사용 절차](../../guides/installed-usage-guard.md#quota-reset-guard)
- Windows Codex의 [사용량·투영 검사](../../../tests/results/runs/20260919T052700-196fb74d8558.md): 46개 중 43개 통과, POSIX 전용·링크 권한 조건 3개 제외
- 최종 [초기화·프로세스 변경·확인 재사용 회귀](../../../tests/results/runs/20260919T052928-e80b522a654f.md) 통과. 실제 공급자 할당량 변경·진행 중 추론 중단·15초 감시의 증명 제외
- `465debfe`·`5664b1d7`: 자동 재개·반복 작업 준비도 공통 세션 상태와 결합. 초기화 확인 후 자동 재개 회귀 통과
- 작업별 감시 제외·진행 중 중단·15초 감시는 후속 호스트 검증 대상. `RB104-004` 완료 승격 없음

## 초기화 감지의 작업별 제외 구현

- `RB104-004`의 독립 구현: `usage session`의 `disable-reset-guard`·`enable-reset-guard`, 해제 전용 확인 인자 필수
- 정확한 호스트·세션·프로세스에만 적용, 전역 설정과 잔여량 하한 보호 유지
- 이미 기록된 초기화 중단은 해제 불가. 기존 지문 확인과 새 측정 절차 유지
- 옛 제어 기록은 감지 활성 기본값, 새 세션·프로세스에 제외 권한 이전 금지
- 시험: 증가 감지만 제외, 낮은 잔여량·조회 오류의 차단 유지, 중단 우회 거부, 재활성화·세션 분리·자동 재개 결합
- 이 구현만으로 15초 감시·실제 추론 중단·전체 항목 완료 판정 금지
