# 과거 Codex 재개 차단의 원리와 재현

## 목적과 현재 제품의 경계

- 목적: 2026-10-18 Windows에서 재개한 작업까지 다시 멈췄던 방식의 보존, 미래의 별도 기능 검토를 위한 재현 자료
- 0.12.1-dev 검증본과 0.13.0 정식판: 감지 당시 작업에 한 차례 중단 요청만 전달. 재개 잠금·확인·해제 명령 없음
- 이번 문서 작업: 역사 코드 읽기·기존 실측 대조만 수행. 과거 감시기 활성화·추가 실제 중단 없음

## 재개 버튼을 눌러도 멈춘 이유

사용자의 재개·새 메시지 → Codex의 작업 시작 → 별도 감시기의 반복 중단 요청 → 시작한 작업의 재중단.
버튼 비활성화가 아니라 이 반복으로 발생한 재개 차단 효과.
여기에 새 Hive 작업의 시작 전 검사도 같은 중단 기록을 읽어 실행 거부.

| 경로 | 작동 지점 | 효과 |
| --- | --- | --- |
| 반복 세션 중단 | Codex 앱의 대화 제어 연결 | 재개·새 입력으로 시작한 작업도 다시 중단 |
| Hive 시작 전 검사 | Hive 명령의 작업 허가 검사 | 해당 명령의 실행 거부 |
| 공통 사건 기록 | 사용자 루트의 감시 상태 파일 | 새 대화·프로세스·앱 재시작 뒤에도 중단 이유 유지 |

Codex 앱이 원래 제공하는 사용자 중단 경로 사용. 운영체제 일시 정지·앱 종료·모델에 보내는 중지 지시·제공자 API 호출 없음.

## 재현할 코드의 정확한 위치

- 기준: `cd0f8ac6e4d2973133728bf6b1885c024741fead`, 공식 로고 적용 뒤·재개 잠금 제거 전
- 관련 실제 중단 검증 기준: `85d4374a`, 실행 파일 SHA-256 `dc26bf4ad24105bcbb2f4b55c7cb1ad121abdbe3e0e2f8834034b7edffbc5fc3`
- 잠금 제거: `ae5455ff7ca9b37bdbd3b599027b9f35f8f6f83c`
- 첫 재개 시 이미 처리한 초기화의 재차 감지 방지: `b0c1d256b12b9444fc19fdab166cf0e984a8ac5a`

역사 코드의 읽기 전용 조회 명령. 현재 작업 폴더의 파일 교체 없음.

```powershell
git show cd0f8ac6:crates/hive-cli/src/usage_monitor.rs
git show cd0f8ac6:crates/hive-cli/src/usage_monitor/state.rs
git show cd0f8ac6:crates/hive-cli/src/usage_monitor/desktop.ps1
git show cd0f8ac6:scripts/qualify-session-adapter.py
git show ae5455ff -- crates/hive-cli/src/usage_monitor.rs crates/hive-cli/src/usage_monitor/state.rs crates/hive-cli/src/usage_monitor/desktop.ps1
```

## 상태와 제어의 결합

1. `MonitorState::halt(reason, now)`: 사건 부재 시에만 `generation` 증가와 `incident` 생성.
   사건 지문 `digest`에 세대·원인·시각·계정 정보 반영. 기존 사건 존재 시 새 사건 생성 없이 유지
2. 상태 경로: 사용자 루트의 `.hive/runtime/usage-monitor/state.json`, 설정은 같은 폴더의 `config.json`.
   사용자 루트 잠금 안에서 원자적 저장. 사건은 화면 표시뿐 아니라 계속 중단할지를 결정하는 제어 상태
3. 독립 예약 작업의 `controller_loop`: 사용량 조회 작업과 제어 반복문 분리.
   조회 작업의 응답 정지에도 제어·보호 시한 처리 지속
4. `dispatch_halt`: 사건별 최초 `desktop.stop` 전달. 사건이 남으면 앱 연결의 `$stop` 유지.
   Rust의 매번 stop 전송이 아니라 PowerShell 제어부의 반복 요청
5. `Advance-Stop`: 각 대화의 `lastStop` 이후 1,000ms 이상 경과 시 재중단 요청.
   `Register-Target`: 새 대화·실행 추가, 앞선 확인이 끝난 뒤 시작한 실행에도 새 5초 확인 구간 설정 가능
6. `check_dispatch`: `incident.is_some()`이면 새 Hive 실행 거부.
   감시 비활성화 뒤에도 사건이 남으면 이 조건 유지. 감시 활성 중에는 관측·심장박동의 최신성,
   제어 연결·중단 해제 여부·알 수 없는 대화 수까지 추가 검사.
   `enforce_captured`와 최종 `verify_dispatch_preflight`에서 공통 검사 호출

알림 창 닫기·앱 재시작은 사건 삭제 동작과 별개. 따라서 계속 유지된 재개 차단.
다만 정상 감시기·지원 앱의 관측 결과로 한정. 감시기를 중단한 공격자까지 막는 보안 경계나 모든 재시작 경로의 보장 제외.

## Codex가 실제로 중단을 받은 경로

Windows 11 x64·Codex `26.1013.4894.0` 대상. `desktop.ps1`에서 `codex-ipc` 연결 등록과 로컬 대화 소유자 탐색.
메시지 형식: UTF-8 JSON 앞에 4바이트 작은 자리 우선 길이 정보 추가.

| 호출 | 버전 | 용도 |
| --- | --- | --- |
| `initialize` | 0 | `clientType=aigent-hive-usage-monitor`로 제어 연결 등록 |
| `thread-owner-discovery` | 1 | `hostId=local`, `conversationId`에 해당하는 앱 소유자 확인 |
| `thread-follower-interrupt-turn` | 3 | `{conversationId, mode:"user-stop"}`로 사용자의 중단과 같은 경로 요청 |

앱 내부의 `turn/interrupt`와 목표 정지 처리로 연결. `expectedTurnId`에 고정하지 않아 같은 대화의 다음 실행도 반복 중단 대상.
실행 이력은 필요할 때만 구독. 실제 실행의 `interrupted`와 목표가 `active`가 아닌 상태를 함께 확인.
이력 구독 해제 뒤에도 `$stop`이 남으면 1초 반복 요청 지속. 모든 Codex 버전에서 유지되는 공개 API 보장 제외.

## 미래의 제한된 재현 절차

현재 실행 지시와 별개의 미래 실험 절차. 별도 승인·소유한 시험 대화·격리된 소스 사본 전제.
현재 설치·사용자 루트·예약 작업을 재현 대상으로 사용 금지. 전체 감시기 대신 과거 단일 대화 검사기로 범위 제한.
원본 작업 폴더 checkout·현재 제품의 잠금 복원 없이, 별도 사본에서 위 기준 커밋의 파일 유지.

1. 앱 버전과 시험 대화 ID 확인. 다른 버전이면 실험 중단 후 새 제어 검증 계획
2. 외부 터미널과 유한한 유지 시간 준비. 시험 프로그램 종료 뒤에도 앱·저장 파일을 보존하는 복구 경로 확인
3. 시험 대화에서 소유한 작업 실행. 과거 검사기의 다음 인수 사용.
   검사기는 `WindowsApp`으로 PID·생성 시각을 함께 확인하고 명시한 시험 대화 하나로 제어 제한

```text
python scripts/qualify-session-adapter.py --thread <시험-대화-UUID> --app-pid <현재-앱-PID> --app-created-ticks <현재-생성-UTC-ticks> --output <격리된-결과.json> --interrupt --hold-seconds 20
```

4. 최초 중단 뒤 20초 유지 구간 안에서 그 시험 대화만 직접 재개.
   최초·다음 실행의 중단을 각각 관찰. `confirmed_stop`, 서로 다른 `scope_digest`의 `subsequent_stops`,
   원래 앱 생성 시각·표시 창 생존 기록. 다른 대화 실행·시험 범위 확대 제외
5. 유지 구간 종료 시 검사기의 `clear` 전달과 `control_stopping=false` 확인.
   `subscriptions_released`·`app_preserved`가 참인지 대조. 자동 작업 재개 없음

위 절차의 증명 범위는 단일 대화 반복 중단. 공통 상태의 디스크 지속성·전체 감시기·새 Hive 명령 차단의 증명 제외.
전체 결합 실험은 별도 사용자 계정/격리 Windows, 시험용 사용자 설정, 사건 주입·외부 복구를 갖춘 추가 승인 계획 필요.
현재 전역 상태 직접 수정·현재 예약 작업의 과거 실행 파일 교체 절차는 제외.

## 과거 전체 감시기의 정상 해제 절차

과거 빌드 전용 명령. 0.12.1-dev 최종 검증본과 0.13.0 정식판에서 제외한 인터페이스.

```text
hive usage monitor status --user-root <시험용-user-root> --output json
hive usage monitor acknowledge --user-root <동일한-시험용-user-root> --incident <현재-incident.digest> --output json
```

- 외부 터미널에서 정확 사건 지문 지정. 잘못된 지문 거부
- `acknowledge`: 상태 잠금을 잡은 채 새 정상 사용량 조회. 실패 시 사건 유지, 성공 시 새 기준 관측과 사건 삭제.
  오프라인 강제 해제 명령이 아닌 정상 관측 전제의 해제
- 활성 제어부: 사건 부재를 보고 `desktop.clear()` 전송. 명령은 최대 3초 동안
  `control_stopping=false`를 기다리고 `controller_release_verified`로 결과 구분
- 사건 삭제와 제어 해제 확인은 별개. 제어 해제 확인이 거짓이면 재개 성공으로 보고 금지.
  성공 뒤에도 `auto_resume=false`, 사용자의 별도 행동으로만 재개
- 별도 세션 기준값의 과거 `acknowledge-reset --confirm-reset`과 공통 사건 해제는 다른 기능.
  한 명령을 다른 명령의 대체로 사용 금지

조회 고장 시 정상 해제가 불가능한 운영 불편까지 드러난 과거 설계. 현재 제품에서는 제거.
비상 복구 방법으로 앱 종료 사용 금지.

## 실측 근거와 한계

- [단일 대화](../../tests/results/session-adapter-latch-windows-0.12.1.json): 최초 중단 확인 313ms,
  같은 사건 중 새 실행 확인 136ms, 앱 유지·구독 해제 확인. 모델 추론 자체의 시작 증명 제외
- [전체 구성요소 수용](../../tests/results/session-stop-component-qualification-0.12.1.json): 실제 네 실행 중단 283~306ms와 후속 읽기 전용 확인 결합.
  원래 전체 자동 확인 실패를 성공으로 바꾸지 않은 기록
- [사건 확인](../../tests/results/session-stop-acknowledgement-0.12.1.json): 격리한 비활성 사건의 잘못된 지문 거부·새 관측·정확 사건 해제·자동 재개 없음.
  활성 제어부 해제의 실측 제외
- 실제 제공자 초기화·서버 처리/과금의 동시 종료·다른 운영체제·원격·클라우드 중단 미증명

현재 동작·설치의 정본: [세션 중단 설명](session-stop-0.12.1.md)과
[한 차례 중단 수용](../../tests/results/session-stop-one-shot-qualification-0.12.1.json).
