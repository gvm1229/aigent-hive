# `0.10.0` 시험 구조 재편

> Checklist owner: `TST10-*`

## 목표

- phase·version 중심 이름을 기능 목적 중심 구조로 전환
- 안정성 회귀 보존
- 실제 측정에 근거한 실행 시간 개선

## Checklist

- [ ] [TST10-001] Python·Rust 시험과 fixture의 현재 참조·소유 lane·실행 시간 목록 작성
- [ ] [TST10-002] Python 시험을 `documentation|security|contracts|integration|release|support`로 이동
- [ ] [TST10-003] Fixture를 `setup|knowledge|skills|run|judge|usage|release|native-orchestration`로 이동
- [ ] [TST10-004] Rust `v09_*` qualification 파일을 현재 기능 이름으로 변경
- [ ] [TST10-005] source·script·workflow·문서의 모든 이전 경로 갱신
- [ ] [TST10-006] 하위 package 재귀 발견과 lane별 단일 소유 검증
- [ ] [TST10-007] lane·module 실행 시간 보고와 변경 경로 기반 로컬 선택 추가
- [ ] [TST10-008] 진짜 미사용·폐기 또는 동등한 빠른 대체가 입증된 시험만 제거
- [ ] [TST10-009] 전체 Python 적합성·Rust workspace·지원 운영체제 CI 계약 통과

## 삭제 기준

- 현재 source·지원 version·출시 계약 참조 없음
- 고유 실패 조건 없음
- 동등한 작은 대체 시험 존재
- 전체 시험과 보호 범위 동등성 확인

## 수락 기준

- 모든 `test_*.py`: 정확히 한 lane 소유
- Phase 이름의 시험·fixture 디렉터리 `0건`
- 기존 안정성 회귀의 근거 없는 삭제 `0건`
