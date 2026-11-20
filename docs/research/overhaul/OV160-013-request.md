# OV160-013 요청 — Claude 사용량 측정 없음의 원인 해결과 경고 후 계속

- 날짜: 2026-11-21, local 세션(Windows 11, Claude 데스크톱 앱 Code 탭, 설치 `AIgent Hive v0.15.3`)
- 계기: 0.16.0 local 확인 시작의 07 사전 확인 `hive usage enforce --host claude`가 exit 3 `hive.usage-unknown`(`missing_windows=["session","weekly"]`). 같은 차단이 0.15.3 local 세션([OV153-010](OV153-010.md))에도 발생
- 유지관리자 요구(2026-11-21): 이번 출시에서 근본 해결. 측정이 없을 때 동작은 "경고 후 계속" 선택
- 이 세션 진행: 사용자 선택에 따라 이 세션만 `hive usage session --action disable --confirm-session-disable`(`hive.usage-session-disabled`), 재확인 `hive.usage-session-bypassed`. OV153-014 U6 수정 확인

## 원인

1. 연결 없음: Claude 측정의 유일한 경로는 상태 표시줄(`statusLine`) 입력의 `rate_limits`. 플러그인은 `bin/hive-claude-usage-capture`만 제공, `statusLine` 연결 없음. 이 PC `~/.claude/settings.json`에 `statusLine` 없음 → 측정 기록 0건
2. 소스 폴더 거부: `capture_claude`(`crates/hive-cli/src/usage_control.rs`)가 `read_installed_config`로 `.hive/config/harness.toml`을 요구. `hive-source.json`만 있는 소스 폴더는 오류. 감싼 스크립트의 `|| :`가 오류를 숨김
3. 기록 위치 불일치 가능: 소스 폴더 정책은 사용자 루트의 대상 지문 실행 상태를 쓰지만 기록은 대상 폴더 `.hive/runtime/usage-guard/claude/`에 씀. 읽는 쪽 위치와 일치하는지 미확인
4. 짧은 유효 시간: 기록 유효 시간 120초. 상태 표시줄은 화면 재표시 때만 실행, 그래서 첫 응답 전·긴 명령 중 빈 측정
5. 데스크톱 앱: Code 탭이 `statusLine` 명령을 실행하는지 미확인

## Discord 알림 결함

- 측정 없음에도 소진용 작업 중단 양식 사용
- 요청·진행 칸의 출처는 실행 `PLAN.md`뿐. 세션 시작 확인에서는 항상 빈칸
- 호스트 줄의 `unknown`은 `selected_window` 내부 값 그대로
- 계속 진행 요청 안내는 같은 확인의 재실행과 같은 차단 반복으로 연결
- 빠진 한도, 원인(연결 없음·기록 거부·오래됨), 해결 명령 미표시

## 수용 기준(0.16.0-test.2)

- U1 경고 후 계속: 보호 대상 한도의 측정이 없으면 차단·중단 기록 없이 exit 0 `hive.usage-observation-warning`(v2), 빠진 한도·원인·해결 명령 포함. 실제 측정값이 기준 이하일 때만 차단. 0% 보호와 초기화 보호 유지
- U2 소스 폴더 기록: `hive-source.json` 대상의 기록 허용, 기록·읽기 위치를 사용자 루트 대상 지문 실행 상태로 통일. 일반 프로젝트 동작 유지
- U3 오류 표시: 기록 실패를 숨기지 않고 사유를 실행 상태에 남김. `usage status`·경고에 마지막 실패 사유 표시. 상태 표시줄 출력 보존
- U4 연결 설정: `hive setup --scope user --section usage`가 동의 뒤 `statusLine` 연결. 기존 상태 표시줄은 보존해 이어 실행, 되돌리기 제공. 미리보기와 확인 지문 필수
- U5 알림: 측정 없음 전용 문구(경고, 빠진 한도, 원인, 해결 명령). 맴도는 "계속 진행" 안내 제거. `selected_window` 한국어 표시. 요청 칸은 실행 계획이 없으면 그 사실을 짧게 표시
- U6 데스크톱 앱: Code 탭의 `statusLine` 실행 여부 확인·기록. 미실행이면 한계와 대안을 출시 노트와 설정 안내에 표시
- 확인: 단위·통합 시험(측정 없음 경고, 소스 폴더 기록·읽기 왕복, 기록 실패 사유, 기존 상태 표시줄 보존, 알림 스냅샷), 세 OS 공개 시험, 이 PC 실제 확인은 OV160-012에 추가
