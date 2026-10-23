# 0.14.0 사용량 보호의 선택과 기본값

> Plan version: 0.14.0
> Scope: product

## 항목

- [ ] [G140-001] 기본값·전체 스위치·모든 실행 경로의 일치
  - state: agent-owned; depends: P140-001
- [ ] [G140-002] 기존 사용자 선택·감시 전환·실제 상태 확인
  - state: agent-owned; depends: G140-001,P140-002

## G140-001 — 정책

| 새 설치 항목 | 기본값 |
| --- | --- |
| 현재 설정·사용량 수동 조회 | 제공 |
| 기존 정상 관측을 이용한 짧은 안내 | 활성·사용자 해제 가능, 알림 전용 추가 조회 없음 |
| 임계값·초기화·0%의 새 작업 차단 | 비활성 |
| 임계값·초기화의 실행 중 중단 | 비활성 |
| 조회 실패 중단 | 비활성, 별도 엄격 보호 선택 때만 사용 |
| 상시 감시·팝업·Discord·추가 센서 설치 | 비활성·개별 동의 |

- v2에 enforcement_enabled=false, warnings_enabled=true, stop_remaining_percent=20. preflight는threshold_enabled/reset_enabled/zero_quota_enabled=false·observation_failure=warn, midway는enabled와같은세원인false·observation_failure=warn. v1필드는호환읽기, v2저장은이중첩구조로정규화
- 전체 비활성이 프로젝트·세션·상속·과거 중단 기록보다 우선. 임계값 변경·새 세션·앱 재시작·후속 갱신으로 자동 활성 금지. 저장한 임계값은 보존, 사용자가 보호를 켤 때만 효력
- v1은 P140-002의 legacy 해석으로 현재 효과 보존. v2 전환 확인 전 키 누락을 새 기본 false로 간주하지 않는 경계
- 명시적 전체 비활성에서 `hive.usage-disabled`, exit0, `enforcement_enabled=false`, `authorizes_dispatch=false`. 센서·감시 등록·손상된 옛 halt/control 읽기·관측 기록 변경 없이 정책 판정 반환
- 소유: usage_control의 read_effective_config/enforce_captured/dispatch_preflight/verify_dispatch_preflight/current_dispatch_binding, run/loop의 허가 소비, usage_control/monitor와 usage_monitor의 공유 정책
- 비활성 검사 결과를 자동 실행 사전 검사의 정식 결과로 인정. 실제 dispatch는 기존 사용자 승인·소유·의존성·동일 세션/프로세스 검사로만 허가. 정책 지문에 전체 선택 포함, 검사와 실행 사이에 보호 변경 시 허가 재검증
- `enforced` 필드는 유지하되 실제 보호 비활성은false. 새 `policy_evaluated=true`·`enforcement_enabled=false`·`observation_required=false`로 검사 완료와 실제 보호의 의미 분리. 스킬은 무조건 enforced=true를 요구하는 계약 대신 유효 정책 평가와 정확한 실행 허가 확인
- 전체 활성에서도 센서 부재/오래된 값은 기본 경고. 엄격 보호 선택 시만 사용량 불명 차단. 정책 파일 위조·잘못된 권한 등 보안 검증 실패는 이 선택과 무관한 오류로 유지
- 검증: 하한 이하/초기화/0%/다른 구간0%/센서 부재/관측 만료/옛 halt 손상/새 대화/프로젝트 상향/AI 하위 작업/자동 loop. 비활성에서 센서 호출·사용량 이유 차단0건, 명시 활성에서 기존 회귀 통과

## G140-002 — 기존 설치 전환

- 진입: G140-001·P140-002. 현재 저장 선택과 실제 감시 상태를 구분해 CLI 표시, 이전 동의의 출처가 불명확하면 그대로 명시
- 기존 비활성은 보존. 활성/불명확 선택은 변경 효과 표시 후 유지/권장 해제 질문, 답변 뒤에만 적용. 비대화형·취소 시 기존 선택 유지, 다음 CLI의 해당 질문 대기
- 해제 선택: 기존 monitor disable의 소유 작업 검증·잠금·해제 사용, 실행 중인 정확한 감시 종료 확인. 연결 호환 검사나 정상 관측을 해제의 전제조건으로 요구하지 않는 구조
- 유지 선택: 기존 동작과 기록을 보존하는 버전 갱신. 상태 조회에서 의도/구성/실행/관측/제어를 별도 반환. 오래된 사건으로 새 대화 재중단 금지
- 해제 실패: 저장 의도와 실제 실행 불일치 표시·미완료 반환·정확한 복구 명령. 작업명 전체 삭제·호스트 앱 종료·감시 프로세스 이름 전체 종료 금지
- 전체 재활성은 해당 원인별 선택과 정확한 preview 동의 필요. 상시 감시는 별도 명시 동의. `--confirm`이나 빠른 설정 수락만으로 기존 외부 알림/센서 설치 추가 금지
- 지침: 현재 usage-guard·user-setup·실행 스킬·user_directives·소스07에 같은 조건 반영. 새 전역 전체 비활성은 옛 프로젝트 정책보다 우선, 타사 바이트와 역사 bases 불변
- 검증 명령: `python scripts/dev-check.py rust -- test -p hive-cli usage --locked`; 관련 Python 사용량 계약·보안·감시 시험. Windows 실제 예약 해제/종료, macOS/Linux에서 Windows 제어 호출0건
- 완료: 전후 자료 보존·해제 실패 복구·후속 재활성 보존·연속 두 번 update에서 정책 불변. 결제 차단 보장이나 실제 자연 초기화 미시험의 성공 주장 제외

## v1 유지·비대화형 이전의 정확한 대응

- T=기존일반보호, R=기존초기화선택, M=검증된감시선택. 일반enabled=false만으로전체비활성판정금지
- 전체보호true, preflight.threshold=T, reset=T AND R, zero=true, observation_failure=block
- midway.enabled=M, threshold=M AND T, reset=M, zero=M, observation_failure는M일때stop. 기존60초규칙보존
- 기존임계값/프로젝트상향값·같은유효결합의세션예외보존. 새경고는비대화형에서false·질문보류. 의도불명은legacy-unknown, 손상된감시선택은추측없이이전보류
- 사용자전체해제는모든차단·중단·상시감시에적용. 후속하네스갱신실패로옛감시자동재활성금지

## 경고 중복 억제

- 이미확보한유효관측만사용. 같은호스트/계정지문/구간/기간/원인/정책판의경고는한번, 낮은잔여량과0%는별도단계
- 기준초과정상관측두번또는새기간에서재설정. 연속조회실패는한번·정상관측후다음실패구간재설정, 프로그램재시작만으로반복금지
- warnings_enabled=false로해제. --quiet은선택안내만억제, 오류/필수질문/미완료유지. 갱신후AI도움문구는같은버전/설정에서반복금지
