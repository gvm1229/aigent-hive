# 0.11.1 원격 출시 검증 수정

> Plan version: 0.11.1
> Scope: product
> 다음 시험 대상: `0.11.1-test.5`

## 원인과 경계

- 00262b98 CI의 macOS 설치 순환 시험: 같은 프로세스·같은 초의 백업 폴더 이름 충돌
- Linux Rust 1.99 Clippy: 기존 hook 검증의 불필요한 closure 참조 6곳과 시험의 빈 값 assertion 표현 거절
- 0.11.1-test.2 후보 36904470719는 아직 미게시; 제품 수정 뒤 test.4으로 전환
- 안정판 0.11.1 승인과 승인된 공지 문구·지문 유지; 보호 규칙·소유권·복구 안전 조건 완화 제외

## 수용 기준

- [x] [RQP-001] 연속 설치·갱신의 독립 백업 경로와 이전 백업 보존
  - state: complete; evidence: repo:tests/results/runs/20261001T185332-c59edc040a31.md#sha256:984f554d19e59c5113144a36cc51aba8242306422cfb524e3fb189332c520070
- [x] [RQP-002] Rust 1.99 검사 호환과 hook 검증 동작 보존
  - state: complete; evidence: repo:tests/results/runs/20261001T185139-c5168eabc611.md#sha256:ee65157244569e7f1f2a0fc102421a00007d0fac488461b1c0633311dc21c06e

## 구현과 검증

1. RQP-001: user_install::apply_plan의 시간·PID 백업 이름에 OS 무작위 128비트 값을 추가. 기존 getrandom 의존성과 파일별 배타적 생성 재사용, 기존 백업 덮어쓰기·복구 형식 변경 제외. 기존 빠른 Claude 설치·재설치·갱신 시험에서 백업 경로의 구별과 이전 명세 보존 확인. Windows 관련 시험→전체 Rust·Python→새 번호 원격 수용, 실제 Claude 공개 파일 재검증.
2. RQP-002: hive-render::validate_revoked_hook_ownership의 map_err(&conflict) 6곳을 map_err(conflict)로 정정. Rust 1.99가 지적한 시험의 빈 값 확인만 길이의 assert_eq/assert_ne로 바꿔 실패 크기 표시. 원래 빈 값·값 존재 여부 조건과 모든 실행 검사 유지, lint 허용·시험 삭제 제외. hook 관련 시험·전체 Clippy, Linux Rust·macOS 원격 확인.

## 인계

- 완료된 수정과 근거를 test.4 제품 지문에 결합한 뒤 새 후보·게시·수용
- test.2 소스와 산출물은 새 제품의 성공 근거에서 제외, 게시·안정판 시험 통로 사용 금지

- 9658d4e5 Linux CI의 POSIX 전용 빈 값 확인을 같은 길이 조건으로 정정, 문체 한 곳 정정; test.3 미게시·test.4 사용

- fbae6016 Linux CI의 POSIX 다중 줄 빈 값 확인 추가 정정; 다음 후보는 원격 CI 전체 통과 뒤 test.5 생성
