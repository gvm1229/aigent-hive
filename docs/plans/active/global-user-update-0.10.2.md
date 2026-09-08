# `0.10.2` 전역 사용자 설치 자동 갱신

> Checklist owner: `GUU102-*`
> 재현 환경: Windows Codex, 인증된 `0.9.5–0.10.1` user install fixture
> 소비자 적용: release 전용 격리 대상만 허용

## 목표

- bare `hive update`가 실행 파일과 Hive 소유 전역 사용자 상태를 수렴
- 새 질문은 일반 Hive 작업을 차단하고 답변 뒤 전역 transaction을 자동 재개
- npm은 실행 파일만 설치하며, 최초 호스트 선택과 최소 투영은 `hive update`가 담당

## Checklist

- [x] [GUU102-001] `0.10.2` version·release compatibility·historical user inventory 등록
- [x] [GUU102-002] 전역 update 상태와 versioned question catalog·answer ledger 도입
- [x] [GUU102-003] 인증된 `0.9.5–0.10.1` 설정·Skill rename/merge 이관
- [x] [GUU102-004] `hive update`의 최신-version reconciliation·backup·apply·validate transaction
- [x] [GUU102-005] 미응답 질문의 최소 `setup-required` host projection과 CLI 차단
- [x] [GUU102-006] claim·answer·마지막 답 이후 digest-bound 자동 resume
- [x] [GUU102-007] 최초 npm 설치의 대화형 호스트 선택과 같은-version recovery
- [x] [GUU102-008] 변조·foreign bytes·symlink·중단 transaction 무변경/rollback 회귀
- [x] [GUU102-009] Rust·Python integration 및 세 host fixture lifecycle
- [x] [GUU102-010] README·Skill·bilingual Source Wiki 현재 truth 정합화

## 완료 근거

- `v0.10.1` 공개 사용자 플러그인 62개 파일 동결과 세 호스트 적용·manifest 검증 통과
- Windows에서 Rust workspace, strict Clippy, 계약 466건, 통합 94건, 문서 87건, 보안 103건, 출시 119건 실행
- Source Wiki 174쪽 오류·경고 없는 lint 통과
- Windows에서 실행할 수 없는 POSIX·symlink·macOS 항목은 공개 시험의 해당 운영체제 수용 범위로 유지

## 수용 기준

- 프로젝트 registry와 project harness 접근·변경 `0건`
- 질문 없는 갱신은 사용자 직접 `install`·`setup` 명령 없이 완료
- 질문 있는 갱신은 답변 전 일반 작업 거부, 취소·무응답은 계속 대기
- `yes`·`no` 답변 모두 전역 설치를 재개하고 선택 기능의 실제 변경은 별도 안전 흐름 유지
- 실패한 전역 갱신은 같은 `hive update`로 recover 가능
