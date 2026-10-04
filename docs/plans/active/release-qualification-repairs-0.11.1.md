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

- [x] [RQP-003] 재실행된 공개 수용의 유효 산출물 집합 검증
  - state: complete; evidence: repo:tests/results/legacy/11df02a60451ecd446fc.md#sha256:3b2b7eebd31e48614b0520e910de6666f37a6dfe0e95a3e26f969d839ab1426e

- [x] [RQP-004] 후보·게시 전 실제 버전 문서와 출시 설명의 선행 검사
  - state: complete; evidence: repo:tests/results/stable-preflight-20261005.md#sha256:5e9fde28c528f46616263060596caca63397aa2306c4ed552bc34db6aa664afe

## 구현과 검증

1. RQP-001: user_install::apply_plan의 시간·PID 백업 이름에 OS 무작위 128비트 값을 추가. 기존 getrandom 의존성과 파일별 배타적 생성 재사용, 기존 백업 덮어쓰기·복구 형식 변경 제외. 기존 빠른 Claude 설치·재설치·갱신 시험에서 백업 경로의 구별과 이전 명세 보존 확인. Windows 관련 시험→전체 Rust·Python→새 번호 원격 수용, 실제 Claude 공개 파일 재검증.
2. RQP-002: hive-render::validate_revoked_hook_ownership의 map_err(&conflict) 6곳을 map_err(conflict)로 정정. Rust 1.99가 지적한 시험의 빈 값 확인만 길이의 assert_eq/assert_ne로 바꿔 실패 크기 표시. 원래 빈 값·값 존재 여부 조건과 모든 실행 검사 유지, lint 허용·시험 삭제 제외. hook 관련 시험·전체 Clippy, Linux Rust·macOS 원격 확인.

## 인계

- 완료된 수정과 근거를 test.4 제품 지문에 결합한 뒤 새 후보·게시·수용
- test.2 소스와 산출물은 새 제품의 성공 근거에서 제외, 게시·안정판 시험 통로 사용 금지

- 9658d4e5 Linux CI의 POSIX 전용 빈 값 확인을 같은 길이 조건으로 정정, 문체 한 곳 정정; test.3 미게시·test.4 사용

- fbae6016 Linux CI의 POSIX 다중 줄 빈 값 확인 추가 정정; 다음 후보는 원격 CI 전체 통과 뒤 test.5 생성

## 승격 산출물 검사

- 원인: 수용 실행 37220067876의 macOS 재실행으로 동일 이름 산출물 두 개, 기존 원시 이름 목록 비교의 거절
- 변경: `.github/workflows/release.yml`에서 만료되지 않은 산출물의 이름 집합 생성, 정확한 세 운영체제 집합 비교 유지. 실행 성공·브랜치·소스·패키지 연결의 기존 검사는 그대로 유지
- 검사: `test_public_test_acceptance.py`에서 실제 워크플로 명령을 실행, 중복 허용·필수 이름 누락·추가 유효 이름·필수 자료 만료·잘못된 JSON의 거절 확인
- 범위: 출시 자동화만 변경, 수용한 제품 지문 불변. 공개 시험 재생성·이전 실패 자료 삭제 없음
- 완료: Windows·Codex 관련 검사 17개와 실제 GitHub 산출물 목록 대조 통과, 제품 지문 불변 확인. 정확한 소스 CI와 main 승격 검사는 정식 게시 전 필수 단계
- [관련 검사](../../../tests/results/runs/20261004T190420-234110b62b67.md)·[실제 네 산출물의 유효 이름 집합](../../../tests/results/legacy/11df02a60451ecd446fc.md), 필수 누락·추가·만료·잘못된 JSON 거절

## 게시 전 문서 검사 보강

- 두 문서 실패의 공통 원인: 합성 자료 검사와 실제 출시 문서 검사의 차이. 첫 후보의 버전 줄 형식, 첫 게시 37230948639의 영어 VERIFY-02 25단어 초과를 후보/게시 단계에서 뒤늦게 발견
- 소유 파일: `test_release_notes.py`에서 현재 Cargo 버전의 실제 출시 설명과 `check-release-version.sh`의 실제 Python 검증부 실행. 문서 형식·잠금 버전·배포 표본·이전 원본의 바이트 검사, 새 로컬 Rust 빌드 제외
- 공개 문구: VERIFY-02의 의미·Windows 범위·미실행 한계를 유지한 20단어 설명, 승인된 구독자 문구 불변
- 완료: 실제 문서 검사·길이 위반 회귀·배포 묶음 인증·안정판 스킬 이력·승인 공지 검토 완료. 원격 비밀 값은 읽지 않고 환경의 이름만 확인, 실제 전송 검증은 게시 절차에서 수행
- 같은 조건의 재실행·검사 완화 금지. 문서 수정의 새 소스·정확한 CI 후 후보 생성, 게시 성공 전 출시 완료 주장 제외

- [완료 근거](../../../tests/results/stable-preflight-20261005.md): 실제 문서·버전·공지와 후보 12개 증명 검증, 신규 후보와 실제 전송의 별도 확인 유지
