# `0.10.4-test.1` Quota Reset Guard

> 소유 항목: `RB104-001–004`
> 상태: 구현 중

- [x] [RB104-001] 같은 계정·quota pool·window의 잔량 증가를 초기화로 판정하는 core 계약과 단위 시험
- [x] [RB104-002] 전역 `quota_reset_guard_enabled` 기본값·구버전 설정 이관·스키마 반영
- [x] [RB104-003] `enforce` 경계의 기준 관측 저장과 `hive.usage-reset` 자동 실행 차단
- [ ] [RB104-004] 15초 host 감시, 진행 중인 작업 중단, 현재 작업별 opt-out·명시적 재개 계약

현재 Codex host contract에는 Hive가 진행 중인 추론을 중단하거나 작업에 결합한 15초 감시를 등록할
검증된 API가 없다. `RB104-004`는 이를 추정 구현하지 않으며, 지원 계약이 생긴 뒤에만 완료한다.
