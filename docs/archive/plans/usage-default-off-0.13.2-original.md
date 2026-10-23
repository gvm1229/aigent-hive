# 0.13.2 사용량 중단의 기본 비활성화

> Plan version: 0.13.2
> Scope: product

## 요청·범위

- 2026-10-23 사용자 요청: 임계값·할당량 초기화에 따른 중간 중단과 새 작업 차단을 모든 Hive 사용자에게 기본 비활성화
- 현재는 조사·계획만. 구현·설정 변경·중단 시험·설치·게시 미실행
- 정식판 미공개 확인 시 대상0.13.2 유지, 다음 공개 시험0.13.2-test.3. 원래 출시 채팅의 진행 중 후보 취소와 출시 보류 승인
- 기존 통신 호환·0~99%·test.2 근거 보존. 변경 뒤 최종 수용으로 재사용 금지
- 적용 대상은 새 설치와 갱신을 마친 기존 사용자. 갱신하지 않은 옛 실행 파일·이미 읽힌 대화 지침의 원격 변경 제외

## 확인한 현재 동작

| 항목 | 현재 기본값·조건 | 근거 |
| --- | --- | --- |
| 임계값·초기화의 실행 중 중단 | 별도 감시 활성 동의 전 비활성. 활성 사용자 갱신은 활성 선택 보존 | `usage_monitor.rs::enable`, `upgrade.rs::upgrade_with` |
| 새 자동 작업의 임계값 차단 | 빠른 사용자 설정에서 활성·기본 잔여20% | `user_setup.rs`의 `expedited_defaults` |
| 새 자동 작업의 초기화 차단 | 누락값도 활성. 이전 관측과의 초기화 감지 시 해당 요청 차단 | `default_quota_reset_guard_enabled`, `migrate_legacy_missing_usage_threshold` |
| 일반 보호 해제 | 독립0% 검사·초기화 설정 잔존. 조회 불가도 작업 차단 가능 | `usage_control.rs::enforce_captured`, `effective_zero_quota_enabled` |

현재 보호 범위는 Hive 자동 실행 사전 검사와 선택형 Windows 감시. 모든 앱의 새 대화 금지와 구분.

## 선택한 설계

- 전역 `usage_guard.enforcement_enabled: bool` 추가, 누락·새 설치 기본값 `false`. 기존 `enabled`, 임계값, 초기화 선택은 하위 선호로 보존
- 전체 스위치가 꺼지면 임계값·초기화·독립0%·조회 실패에 따른 Hive 중단과 사전 차단 모두 비활성. 숨은 차단 경로 방지 목적
- 기존 사용자도 새 키가 없으면 비활성. 옛 `enabled=true`나 감시 동의만으로 새 전체 스위치 활성 금지. 재설정에서 명시적으로 켠 뒤에만 보호 재사용
- 전역 비활성이 프로젝트·세션·상속·감시 선택보다 우선. 임계값 변경·새 대화·재시작·다음 갱신으로 자동 활성 금지
- 꺼진 상태의 사전 검사 결과: `exit_code=0`, `code=hive.usage-disabled`, `data.enforcement_enabled=false`, `data.authorizes_dispatch=false`. 측정·감시 등록·중단 기록 갱신 없이 반환
- 사용량 검사 통과와 실제 작업 실행 허가는 별도. 작업 상태·의존성·보안·사용자 승인 검사는 그대로 유지. 공급자 자체 사용량 제한이나 결제 설정 변경 없음
- 전체 스위치 활성 상태에서만 기존 보호 동작 적용. Windows 감시는 추가로 기존 `preview`·지문 확인·중간 중단 동의 필요
- 기존 `user-setup` 재설정 사용. 상태 조회에 하위 선택과 실제 전체 활성 여부 구분

## 실행 항목

- [ ] [UD132-001] 전체 스위치·설정 기본값·기존 설정 호환
  - state: awaiting-user-authority; owner: user; reason: 계획 전용 요청으로 구현 제외
- [ ] [UD132-002] 사전 검사·감시의 비활성 경로
  - state: awaiting-user-authority; depends: UD132-001; owner: user; reason: 계획 전용 요청으로 구현 제외
- [ ] [UD132-003] 기존 감시 종료와 보존 갱신
  - state: awaiting-user-authority; depends: UD132-001,UD132-002; owner: user; reason: 계획 전용 요청으로 구현 제외
- [ ] [UD132-004] 지침·투영·회귀 검증
  - state: awaiting-user-authority; depends: UD132-001,UD132-002,UD132-003; owner: user; reason: 계획 전용 요청으로 구현 제외
- [ ] [UD132-005] 새 공개 시험 수용과 출시 재준비
  - state: awaiting-user-authority; depends: UD132-004; owner: user; reason: 현재 출시 보류와 계획 전용 경계 유지

## 구현 인계 순서

1. **UD132-001 / 설정 담당**: 구현 시작 지시 뒤 `crates/hive-cli/src/user_setup.rs`의 `UsageGuardPreferences`·설명·기존 설정 읽기와 `schemas/user-setup.schema.json` 수정. 새 bool은 누락 시 false, 잘못된 자료형 거부. `describe`의 빠른/맞춤 설정과 `harness/skills/user-setup/references/`에서 전체 비활성 제시. 기존 임계값·프로젝트 예외·연결 정보 보존. 시험: 누락/false/true, 이전0.13.1·test.2 입력, 재설정·재갱신의 사용자 선택 보존. 완료 기준은 저장·다시 읽기와 실제 정책 일치
2. **UD132-002 / 정책 담당**: `usage_control.rs::InstalledUsageConfig/read_effective_config/enforce_captured`와 `usage_control/monitor.rs::from_config`, `usage_monitor.rs`의 실행 진입·반복문 수정. 측정·과거 중단 자료 접근 전 전체 비활성 판정, 실행 중 재검사에서도 같은 스위치 사용. `effective_policy_digest`에 전체 선택 포함, 자동 실행 허가 재검사와 `run.rs`에서 비활성 결과 수용. 다른 허가 검사 유지. 시험: 하한 이하·초기화·0%·오래된 자료·센서 부재·손상된 옛 중단 자료·새 세션/프로젝트 모두 사용량 이유 차단0건. 꺼진 상태의 센서/감시 호출0건, true에서는 기존 차단 회귀 통과
3. **UD132-003 / 갱신 담당**: `user_install.rs::execute_command`, `usage_monitor/upgrade.rs::upgrade_existing/upgrade_with`, `usage_monitor.rs::disable` 재사용. 전체 비활성인 갱신은 옛 활성 선택 복원 대신 정확한 Hive 예약 작업 해제·감시 종료 확인·비활성 기록. 연결 호환 검사·새 관측 대기보다 해제 경로 우선. 기존 잠금·소유 확인 유지, 일치하지 않는 작업 변경 금지. 실패 시 미완료와 복구 경로 반환, 중단 실패를 비활성 성공으로 표시 금지. 재시도는 같은 상태로 수렴, 기록·설정·지식 보존, 복구 시 옛 감시 자동 재활성 금지. macOS/Linux는 Windows 작업 제어 호출0건
4. **UD132-004 / 검증 담당**: `harness/skills/usage-guard/`, `harness/directives/`, `user_directives.rs`, `copier.yml`, `hive-render/src/lib.rs`와 소스07 지침을 같은 계약으로 정렬. 전체 비활성의 전역 지침을 옛 프로젝트 지침보다 우선 적용. 현재 투영만 생성기로 갱신, 역사 원문 불변. 활성 보호에서만 소진 후 중단 적용. `cargo test -p hive-cli --locked`, `cargo test -p hive-render --locked`, `cargo test -p hive-core --locked`, 관련 Python 사용량 계약·보안·설정·갱신 검사 실행. 실패 입력 보존·해당 경로 재시험
5. **UD132-005 / 출시 담당**: 기존 test.2·취소 후보를 새 제품의 합격 근거로 사용 금지. 필수 전체 검사→develop의 정확 변경 검사→test.3 공개→Windows/macOS/Linux 새 설치·0.13.1/test.2 갱신 수용. Windows에서는 이전 활성/비활성/불완전 감시 갱신과 예약 제거·실제 감시 종료·앱 생존 확인. 새 범위의 출시 문서·공지 문구 재작성, 새 원문 전송 승인은 별도. 정식 게시·로컬 설치는 계획 전용 보류 해제 후 기존 버전 승인과 최신 범위를 대조해 진행

## 완료·실패 기준

- 새 설치 및 지원 이전판 갱신 후 기본 중간 중단0건·새 자동 작업의 사용량 차단0건, 명시 재활성 뒤 기존 보호 회귀 통과
- 시험은 격리 입력 우선. 실제 채팅 중단 필요 시 기존 승인 시험 대상만 사용, 계정 고의 소진·앱 종료 제외
- 실제 세 운영체제 수용 전 범용 동작 입증 주장 금지. 공개 파일·기록 보존·재시도 결과까지 완료 근거에 포함
- 현재 단계의 완료는 기본값 조사·출시 보류 확인·본 계획 저장·문서 검사. 제품 기능 완료와 구분
