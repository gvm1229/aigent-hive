# 설치본 usage guard의 source 적용

Source 개발도 설치된 `hive usage`를 단일 정본으로 사용. 별도 Python gate, 15초 watcher,
source-local threshold·halt state 없음.

## 대상 분류

| 대상 | 사용량 보호 | Threshold |
| --- | --- | --- |
| `.hive/config/harness.toml` 보유 Hive project | 활성 | `max(global, project)` |
| `hive-source.json` 보유 Aigent Hive source | 활성 | global |
| 자체 `AGENTS.md`만 보유한 project | 비활성 | 없음 |
| 빈 folder | 비활성 | 없음 |

Non-Hive target의 `status`, `enforce`, threshold 변경, session override, halt marker와 runtime
file 생성 없음. Project threshold 요청의 global 변경 재해석 금지.

## Hive 기능과 guard 분리

Non-Hive의 guard 비활성은 Hive 전체 비활성과 별개. `quick-answer`, prompt 개선, user-root
지식처럼 project state가 불필요한 Skill은 setup·usage preflight 없이 사용. Project plan,
run, file ownership이 필요한 workflow만 한 번의 명확한 project 활성화 승인 대상으로 분리.
승인 뒤 capability 확인과 run bootstrap은 해당 workflow가 자동 처리하며 usage guard 오류로
내부 전제 노출 금지.

## Source task preflight

Source task 시작 시 active host의 정확한 session ID·process ID 사용:

```text
hive usage enforce --target <source-root> --host <active-host> --session-id <current-session-id> --process-id <current-process-id> --user-root <user-root> --output json
```

Task당 one-shot 확인 1회. Tool·mutation·push·최종 응답 전 반복 gate와 background watcher 없음.
Exit `3`, `hive.usage-limited`, `hive.usage-unknown`: guard control·동의한 fallback recovery 외
작업 중단.

## Quota Reset Guard

- `0.11.0` 개발 범위: `usage_guard.quota_reset_guard_enabled` 기본값 `true`
- 같은 세션·계정·사용량 묶음·집계 구간의 유효한 두 측정에서 잔량 증가 시 `hive.usage-reset` 자동 실행 차단
- 첫 측정: 비교 기준만 기록. 초기화 시각 예약·외부 소식 추적 제외
- 비교 시점: `enforce`의 새 측정. 진행 중 추론의 15초 감시·호스트 프로세스 중단은 미지원
- 제한 표시: `status`의 `quota_reset_guard_monitoring=false`. 공개 설치본의 지원 여부와 개발 구현 구분
- 0.12.1의 초기화 중단은 해당 요청 한 번만 적용. 다음 사용자 요청은 새 관측으로 재판정, 확인 명령·재개 잠금 없음

현재 작업의 초기화 감지만 제외하는 개발 명령:

```text
hive usage session --target <hive-target> --host <active-host> --session-id <current-session-id> --process-id <current-process-id> --user-root <user-root> --action disable-reset-guard --confirm-reset-guard-disable --output json
```

- 복원: 확인 인자 없이 `--action enable-reset-guard`. 두 동작 뒤 같은 연결 정보로 `enforce` 필수
- 전역 설정·잔여량 하한 보호 유지. 낮거나 알 수 없는 잔여량의 차단 유지
- 초기화 표식은 과거 관측 기록, 새 요청의 사용량 하한·0% 확인 유지
- 다른 호스트·세션·프로세스에 제외 권한 이전 금지. 일반 보호의 켜기·끄기와 별개 설정
- 공개 설치본의 도움말에서 지원 여부 확인. 15초 감시·진행 중 작업 중단의 구현 증명과 구분

## Threshold 변경

명시적 global 변경:

```text
hive usage threshold --user-root <user-root> --remaining-percent <1..99> --output json
```

설정 완료 Hive project의 조기 중지 변경:

```text
hive usage threshold --target <project-root> --remaining-percent <1..99> --output json
```

Global `20%`, project `40%`: 적용값 `40%`. Non-Hive target의 project `40%` 요청: mutation
`0건`, 적용 threshold 없음.

## Session control

현재 binding만 비활성화:

```text
hive usage session --target <hive-target> --host <active-host> --session-id <current-session-id> --process-id <current-process-id> --user-root <user-root> --action disable --confirm-session-disable --output json
```

재활성화:

```text
hive usage session --target <hive-target> --host <active-host> --session-id <current-session-id> --process-id <current-process-id> --user-root <user-root> --action enable --output json
```

새 session 기본값: 활성. Raw account·session ID 저장 없음. Native sensor 우선, CodexBar는
별도 동의를 받은 failure-only fallback. Provider credential·API 호출·OMX/OMC·host process signal
경로 없음.

## 자동 재개와 반복 작업 준비

- `0.11.0` 자동 `run resume`: 현재 호스트의 `--session-id`, 양수 `--process-id`, 실행할 `--role` 필수. 수동 복구의 기존 읽기 전용 동작 유지
- `--user-root`에 연결된 사용자 루트를 전달하여 전역·프로젝트 정책 함께 적용. 계정 지문 생략은 센서가 계정 하나만 식별한 경우에 한정
- 자동 재개 내부의 공통 사용량 검사 1회. 같은 실행을 위한 선행 `usage enforce` 중복 호출 제외
- 허가 기록 직전 정책·세션 제어·중단 기록 지문 재확인. 이후 `loop` 준비와 결과 등록도 같은 결합 확인, 변경 시 거부
- 반복 작업의 사용량 근거에 `user_root`를 선택적으로 전달 가능. 전역 정책을 사용해 발급한 허가는 같은 사용자 루트로 검증 필수
- 새 허가 기록 형식 2와 기존 형식 1 판독 지원. 이미 발급한 같은 작업의 허가 재발급 금지
- 초기화 차단 뒤 정확한 중단 지문의 명시적 확인과 새 측정 필수. 실행 중 모델 중단·15초 감시는 이 명령의 보장 범위 밖

Windows Codex에서 [명령·사용량 검사](../../tests/results/runs/20260919T082740-2745afbd4d3d.md)
66개 통과, POSIX 전용 4개 제외. [CLI 단위 시험](../../tests/results/runs/20260919T083543-4aa35fb79366.md)
462개 통과, 별도 실행 환경 1개 제외. 합성 입력의 명령 결과와 상태 보존 증거이며 실제 호스트 실행·다른 운영체제 증명 제외.

## 0.12.0 기본 활성 소진 보호

- 목적: 구독 잔여량 0%에서 Hive 경유 새 자동 작업 차단, 유료 크레딧으로의 자동 진행 위험 감소
- 기본 활성: 전역·현재 대화의 일반 보호 해제 뒤에도 검사 유지. 전달된 세션·주간·제공자 모음 중 하나라도 0%면 `hive.usage-quota-exhausted`, 종료 코드 3
- 누락·손상·만료·다른 계정: 안전한 판단 불가로 차단. 충분한 다른 창의 잔여량이나 유료 크레딧으로 소진 창 대체 금지
- 차단 기록과 새 측정: 기존 초기화 보호의 저장·원자적 교체 경로 재사용. 단순 재개·초기화 확인으로 소진 해제 불가
- 잔량 회복: 새 측정 뒤 소진 차단 해제 가능. 일반 초기화 보호 활성 시 증가 감지의 별도 확인 절차 유지
- 현재 대화 한정 제외: 정확한 호스트·대화·프로세스의 아래 별도 명령. 새 대화·프로세스 기본 활성

```text
hive usage session --target <hive-target> --host <active-host> --session-id <current-session-id> --process-id <current-process-id> --user-root <user-root> --action disable-zero-quota-guard --confirm-zero-quota-guard-disable --output json
```

- 복원: 확인 인자 없는 `--action enable-zero-quota-guard`, 두 명령 뒤 같은 연결의 `enforce` 필수
- 제외 뒤 일반 하한 보호 유지. 두 보호를 각각 명시적으로 해제한 경우만 측정 우회·크레딧 사용 가능성
- Windows·Codex 개발 검증: Rust 사용량102개·공개 명령28개 통과, Windows 링크 시험1개 제외. 합성 측정·파일·제어 결합의 증명
- 실제 계정의 0% 소진·유료 전환 시험 미실행, 계정 비용 발생 시험 제외. 지원 중단 연결 없는 진행 중 호스트 호출·Hive 밖 호출의 크레딧 사용 방지 보장 제외
- 0.12.0 검증 당시 설치 0.11.1과 개발판 구분. 도움말의 새 명령 지원 확인 선행, 사용자 설치 승인 별도

## Windows 실행 중 세션 중단

- 공개0.13.0의 선택형 기능, 개발0.13.1에서 기존 하한·조회·갱신 보정. Windows 11 x64·Codex 26.1013.4894.0 범위, 다른 앱 버전·운영체제·원격·클라우드 미지원
- 실제 초기화 또는 정상 조회 공백 60초 때 감지 당시 작업에 한 번 중단 요청. 같은 사건으로 사용자 재개·새 입력·앱 재시작 차단 없음
- 시작 간격 15초·조회 제한 5초·실패 재시도 2초. 연속 실패당 한 번 중단, 정상 조회 회복 후 다음 실패 구간 감지
- 앱 전체 종료·일시정지·호스트 프로세스 신호 없음. 현재 사용자·공식 패키지·생성 시각·보유 핸들 확인
- 앱 연결로 대화 소유·실행 상태 확인, 응답의 문장 저장 없음. 사건 때 중단 요청 우선·5초 확인, 제어 연결 재시작의 과거 요청 재전송 억제
- 사건은 원인·확인 결과의 기록. 보호 잠금·해제·확인 명령 없음, 사용자의 Codex 재개·새 메시지 허용. 이전 작업 자동 재개 없음
- 공식 로고의 [안내 창](../research/usage-stop-notice-0.12.1.md)은 이유·상태와 닫기만 제공. 초기화와 조회 실패의 서로 다른 설명, 확인 실패의 성공 표시 금지
- 기존 하한·0% 보호 유지. 서버 처리·과금 종료 시각과 소비 상한 보장 제외

```text
hive usage monitor preview --user-root <user-root> --output json
hive usage monitor enable --user-root <user-root> --confirm <preview-digest> --confirm-session-interruption --output json
hive usage monitor status --user-root <user-root> --output json
hive usage monitor disable --user-root <user-root> --output json
```

- 설치 전 미리보기의 설정·상태·고정 실행 파일·로그인 예약 경로 확인. enable은 정확한 미리보기 지문과 버전별 설치 승인 필수
- status의 정상 관측·제어 연결·미확인 대화 부재와 마지막 중단 결과를 구분. 지난 사건의 존재는 새 실행 차단 조건에서 제외
- disable은 소유 예약만 제거, 사건과 고정 실행 파일 보존. 관리된 개발 설치 원복은 별도 백업 경로
- [공개0.13.0 수용](../research/public-test-0.13.0.md)·[이전 개발 설치 영수증](../../tests/results/session-stop-one-shot-installation-0.12.1.json). 개발판 수용과 현재 사용자 감시의 실제 정상 여부 구분

## 개발0.13.1 보정

- 중간 감시를 켠 경우에만 기존 하한을 직접 중단에 연결. 최초 관측이 하한 이하인 실행도 대상, 소유 대화의 위치·부모·정책 확인 선행
- 확인된 같은 하한 사건의 직접 재개·새 메시지 허용. 다른 대화·자동 실행과 0% 보호로 허용 전달 금지
- 지속 연결 실패 시 새 프로세스의 보조 조회. 어느 경로든 유효한 관측이면 정상, 연속 60초 기준 유지. 새 외부 도구·자격 증명 수집 없음
- `status`에서 설정 활성 여부와 실제 제어부 생존·버전·관측 경로·실패·전환 상태를 따로 확인
- 갱신 시 기존 감시 선택·기록 보존, 고정 실행 파일·소유 예약 작업 교체와 정상 관측 확인. 실패 시 미완료 표시, 데이터 삭제 재설치 금지
- Codex에 “중간 중단 감시를 꺼 줘” 요청: 기존 `disable` 대행과 `status` 확인. 프로젝트 밖에서도 사용 가능, 사용자 PowerShell 입력 불필요. 일반 하한 보호 해제와 구분
- [현재 검증](../research/usage-guard-0.13.1.md): 30분 조회·자동 회귀 완료, 현재 앱의 새 긴 추론·셸 중단과 공개 시험·설치 수용은 별도
