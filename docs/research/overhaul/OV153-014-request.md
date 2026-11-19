# OV153-014 요청 — 호스트별 5시간·주간 사용량 보호 설정

- 기준일: 2026-11-19. 유지관리자 local 세션 요청(원문 영어, 요지 번역)
- 환경: Aigent Hive 0.15.2, Windows 11, Claude 구독(5시간 한도 있음). 주간 사용량 42% 사용으로 여유, 5시간 한도는 소진 추정

## 보고

- local 세션 시작 사전 확인 `hive usage enforce --host claude` → exit 3, `hive.usage-unknown`, `selected_window=unknown`, `snapshots: []`. 같은 묶음 재확인도 동일
- `hive usage probe-native --host claude` → `hive.usage-native-probe-deferred`. Claude 측정은 호스트 세션의 기록(`hive usage capture`)에 의존, 이 PC의 Claude 설정에 기록 경로 없음
- `hive usage session --action disable --confirm-session-disable` 뒤에도 `enforce`는 다시 `usage-unknown` 차단 기록 작성(`status`의 `guard_enabled=false`). 세션 해제가 `enforce` 결과에 미반영
- 유지관리자 진단: 사용량 보호는 5시간·주간 두 한도를 전제로 했지만 개발 중 쓰던 Codex 구독에 5시간 한도가 없어 주간 한도 위주로 설계. 5시간 한도가 있는 구독(Claude) 사용자 흐름 고려 부족

## 범위(0.15.3 필수)

- U1 설정 질문: 사용량 보호를 켤 때 선택한 호스트마다 "이 구독에 5시간 한도가 있는지"를 묻고, 있으면 보호 대상 A 5시간만 / B 주간만 / C 둘 다 중 선택. 없으면 주간 기준만 질문
- U2 임계값: A·B는 해당 한도의 남은 비율 임계값 1개, C는 5시간·주간 각각의 임계값을 따로 질문
- U3 호스트별 반복: 질문 묶음(U1·U2)은 선택한 호스트 수만큼 정확히 한 번씩. 호스트 1개면 1회, 여러 개면 호스트마다 1회. 제목에 호스트 이름 표시
- U4 저장·평가: 선택은 호스트별로 저장(기존 v1·v2 설정 읽기 호환, 기존 단일 임계값은 모든 호스트의 기존 동작으로 해석). 평가는 그 호스트에서 보호 대상으로 고른 한도만 확인. 보호 대상이 아닌 한도의 소진·측정 없음은 차단 사유가 아님
- U5 측정 없음의 표시: 보호 대상 한도의 측정이 없으면 `usage-unknown`이 어느 한도의 측정이 없는지와 해결 경로(세션 기록 설정·세션 해제·대체 측정)를 문구와 `next_action`에 표시
- U6 세션 해제 반영: `hive usage session --action disable` 뒤 같은 묶음의 `enforce`는 차단 기록을 새로 쓰지 않고 해제 상태(`hive.usage-session-disabled` 또는 v2 비활성 결과)를 반환
- U7 명확성: CLI 설정 화면(OV153-012 형식), `--describe`, 사용자 설정 Skill·`usage-guard` Skill·`aither` 안내에서 같은 질문 순서와 뜻. 에이전트가 사용자를 도울 때도 호스트별 질문 규칙 유지
- U8 시험: 호스트 1개·3개 질문 횟수, A·B·C 저장·평가, 5시간 소진이지만 주간만 보호 시 허용, 이전 설정 읽기, 세션 해제 뒤 `enforce`, 측정 없음 문구
