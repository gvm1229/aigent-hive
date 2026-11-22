# 사용량 보호의 날짜별 승인

에이전트 지침 [07](../../.agents/directives/07-installed-usage-guard.md)에서 옮긴 날짜별 승인 기록.

## 2026-10-23 실제 시험 승인

- 지정한 Windows 대화 2개에서 초기화·5%·0% 중단, 앱 유지, 직접 재개 확인
- 합성 관측값은 소스 디버그 고정 자료로 한정. 실패·선택 동의·60초 공백 유지, 의도적 소진 금지

## 2026-11-10 `hive` 실행 불가 세션

- 사용자 결정: Hive를 고치는 작업이 Hive 실행에 의존하지 않도록, `hive`를 실행할 수 없는 소스 세션은 사전 확인을 건너뛰고 작업 진행
- 조건: 미실행 사실 보고, 사용자 루트 임의 생성 금지

## 2026-11-21 측정 없음은 경고 후 계속(OV160-013)

- 유지관리자 결정: 보호 대상 한도의 측정이 없거나 오래되면 차단 대신 exit 0 v2 `hive.usage-observation-warning`. 빠진 한도·원인·해결 명령 표시, 중단 기록·"작업 중단" 알림 없음
- 차단 유지: 실제 측정값이 기준 이하, 0% 보호, 초기화 보호, 측정값 형식·세션·중복 오류. 한 한도가 빠져도 측정된 다른 한도가 기준 이하이면 차단
- 저장된 `preflight.observation_failure = "block"`은 형식 오류 같은 무결성 실패에만 적용
- Claude 상태 표시줄 연결: `hive setup --scope user --section usage --claude-statusline`이 미리보기와 확인 지문 뒤에만 `~/.claude/settings.json`의 `statusLine` 항목 하나만 변경. 기존 명령은 이어 실행, 되돌리기 기록 보존. 사용자 동의에 따른 제품 동작이고 05 지침의 전역 설정 금지에 대한 유일한 예외. 에이전트의 실제 호스트 파일 수정은 계속 금지

## 2026-11-21 멈춤 기준 96% 사용

- 유지관리자 결정: 주간 87% 사용 시점에 멈춤 기준을 96% 사용(남은 4%)으로 변경 후 작업 계속
- 적용: `hive usage threshold --user-root <home> --remaining-percent 4`(전역, 이전 10). 현재 기준 값은 남은 비율 하나라 Codex에도 남은 4%로 적용
- 후속: 호스트별 지표 방향(Claude 사용 비율, Codex 남은 비율) 구분은 새 0.16.0 계획에서 처리

## 2026-11-23 현재 세션 보호 끄기

- 유지관리자 원문: "Disable the usage guard for this session only while proceeding. You are authorized to do so if needed."
- 적용: 조율 중인 Claude 세션 연결 하나에 `hive usage session --action disable --confirm-session-disable`. 전역 남은 4% 기준과 0% 보호 유지
- 한계: 다음 세션·다른 프로젝트에 전파 없음
