# 0.14.0 첫 갱신 연결·대화·완료 처리

> Plan version: 0.14.0
> Scope: product

## 항목

- [ ] [U140-001] 공개0.13.1에서 시작한 첫 대화형 갱신 연결
  - state: agent-owned; depends: S140-001
- [ ] [U140-002] 새 갱신 조정기·질문·보존·복구 통합
  - state: agent-owned; depends: U140-001,P140-002,G140-002
- [ ] [U140-003] 상세 결과·설정 안내·호환 출력
  - state: agent-owned; depends: U140-002

## 현재 확인 사실과 선택

- 공개0.13.1의 `update_activation.rs::update_flow_with_projection_channel`은 프로그램 갱신 뒤 새 실행 파일의 사용자 apply/validate 호출. 새 코드 진입은 첫 갱신에도 존재
- `usage.rs::SystemCommandRunner`는 하위 입력 null·출력 pipe, Windows CREATE_NO_WINDOW 사용. 부모의 시간 제한10분, ChildActionResult는 action/status/exit_code만 해석
- 선택: 이 기존 새 실행 파일 진입점에 한정한 구버전 연결층 추가. 새 부모에서는 터미널 상속을 정식으로 지원. 설치 순서 재배치만으로 해결했다는 주장 금지
- package postinstall의 대화·셸 설정 삽입·사용자에게 다른 버전 선행 설치 요구·첫 질문을 다음 버전으로 연기하는 대안 제외

## U140-001 — 첫 연결

1. 실제 공개0.13.1 npm/직접 설치 실행 파일을 합성 사용자 루트에 준비. 버전·원본 지문 고정, 시험을 위한 구버전 재컴파일 금지. incoming 코드만 새 후보 사용
2. `user_install.rs::execute_command`의 apply 경계에서 현재 부모가 소유·버전·실행 인자가 확인된 옛 Hive update인지 검사. 공개 0.13.1과 공개 0.13.2-test.1/2만 첫 호환 목록. 임의 JSON 호출·일반 setup 호출의 진입 금지
3. 원래 사용자가 터미널에서 시작한 사실을 확인한 경우에만 전용 터미널 연결. 구버전 --confirm은 입력 모드를 확실히 증명하지 못하면 비대화형 처리. POSIX는 /dev/tty, Windows는 해당 부모 콘솔 AttachConsole과 CONIN$/CONOUT$ 사용. 기존 stdout/stderr pipe는 별도 핸들로 보존, 부모에게 정확한 JSON만 반환
4. 원래 입력 상태가 확인되지 않으면 non-interactive 정책. 임의 부모 콘솔 탐색·새 창 생성·GUI 조작·자동 yes 주입 금지. 사용자에게 미응답 질문과 재개 경로를 정확히 보고
5. 언어·현재 설정·필요 질문을 공통 엔진에서 처리. 최종 확인 뒤 기존 apply/validate 흐름 연결. 새 코드 진입과 실제 질문·답변·저장을 각각 증거로 기록
6. 구버전 부모600초보다 짧은 내부 기한540초. 질문 수집은420초까지, 이후 미응답 저장·적용 시작 금지. 적용 진입은 답변·preview·입력 지문 확정과 잔여120초 이상일 때만 허용. 적용 최대90초, 정리·기록 최대30초. 외부 명령은 남은 기한 이하로 제한, 모든 쓰기 전 복구 기록 저장
7. 기한 부족·중단은 정확한 pending과 재개 명령 반환. 같은0.14.0 update에서 남은 단계만 재개, 새 버전 대기 없음. 새 부모에서는 답변 대기와 작업 실행 기한 분리. 기한 직전 답변·느린 적용·부모 종료에서 자료 보존을 검증하기 전 출시 금지

- 실패 복구: 연결 불가·취소·시간 초과 시 선호/감시 선택 변경 없음, 이미 갱신한 core와 미완료 설정 단계를 구분. 강제 재설치·사용자 자료 삭제 대신 진행 재개
- 첫 검증: 실제 터미널의 질문·답변과 기계 JSON 분리, 한글 입력, 취소/EOF,420초 질문 기한·적용 예산 부족, 재개, npm/직접 설치. 지원 OS에서 실패 시 U140-001 미완료와 출시 차단, 후속 버전으로 조용한 우회 금지
- 산출물: 새 `scripts/qualify-interactive-cli.py`의 legacy-upgrade 시나리오, 구버전/후보 지문·OS·질문 ID·답변 분류·결과 코드. 원문 세션·비밀 수집 없음

## U140-002 — 정식 조정기

- 소유: update_activation, user_install, user_setup의 호출 경계와 기존 install/update journal. 본체 교체/질문/설정 이전/하네스 갱신/검증을 서로 구분된 단계로 분리
- 흐름: 읽기 전용 설치 확인→갱신 승인→core 획득/검증/활성→새 버전 완료기 실행→언어/선택→변경 미리보기→설정/감시/하네스 적용→검증→요약
- 새 부모의 완료기는 터미널 모드와 구조화 모드를 명시해 실행. 사용자 입력 대기 시간은 일반 명령 시간 제한에서 제외, 네트워크·파일·감시 확인의 기존 제한 유지
- 재실행 시 registry 조회 결과의 최신 여부와 무관하게 미완료 journal 먼저 확인. 이미 갱신된 core의 재설치 없이 남은 단계만 재개. 같은 설정 질문의 중복·다른 진행의 중첩 방지
- 소유 잠금은 실제 변경 단계에만 유지. 질문 시 원본 지문 보유, 적용 직전 재대조. 다른 세션 변경 발견 시 영향받은 질문과 preview만 갱신
- 비대화형은 기존 선택 보존·새 기능 비활성·optional pending. 필요한 언어/필수값 부재만 needs-input. optional pending만으로 업데이트 실패 또는 일반 작업 차단 금지
- 사용자 전역 하네스 갱신과 등록 프로젝트 파일 갱신 구분. 프로젝트는 별도 명시한 target의 setup/project upgrade 사용
- 실패 시 core 변경/설정 변경/감시 상태를 각각 기록. 파괴적 전체 rollback 대신 마지막 검증 가능한 단계 복구. 감시 종료 실패를 보호 비활성 성공으로 표시 금지

## U140-003 — 결과와 안내

- 기존 action/status/exit_code 보존, `data`에 `from_version`, `to_version`, `steps`, `settings_changes`, `pending_questions`, `effective_usage_policy`, `monitor_state`, `resume_command` 추가
- 부모의 ChildActionResult·ProjectionRefresher를 확장해 새 상세 결과 전달. 검증되지 않은 설정값을 실제 효과로 표시 금지
- 사람 요약: 설치 결과·이번 변경·보존 항목·현재 중단/차단/감시 상태·필요한 재개. 이미 비활성이면 유지로 표시, 변경 없이 해제 표시 금지
- 선택형 AI 설정 검토 문구는 선택 언어로 제공, 기본 짧은 안내와 상세 도움 분리. 모델/속도 추천은 호스트별 확인된 안내만. JSON에는 설명문 혼입 금지
- 구버전 부모가 새 필드를 읽지 못해도 전용 터미널 요약과 기존 JSON 성공 계약으로 첫 갱신 완료. pending/failure는 구버전에도 nonzero 반환
- 검증: `python scripts/dev-check.py rust -- test -p hive-cli update_activation --locked`, 사용자 설치 회귀와 첫 갱신 시나리오. 완료/보류/실패/재개별 출력·단계 지문·실제 파일 대조

- 0.14.0 --confirm은 소프트웨어 승인만 생략. --non-interactive/JSON은 질문 없음, --interactive는 실제 터미널 필수, 충돌 옵션 거부. 구버전 자체의 최초 문구 유지, 새 설정 첫 질문부터 선택 언어 적용
