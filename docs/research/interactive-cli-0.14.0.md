# 대화형 CLI·설정 사용량 조사 근거

## 범위와 한계

- 조회일2026-10-23. 실행 환경Windows·Codex. 로컬 소스와 공식 자료의 읽기 전용 대조, 제품 구현·설치·실제 모델 비교 시험 없음
- [구현 인계](../plans/0.14.0-implementation.md). 추천과 확인 사실 구분, 미표기 문서 갱신일의 추정 없음

## 확인한 외부 근거

| 자료 | 날짜·적용 범위 | 확인 사항 |
| --- | --- | --- |
| [Hermes 설정 이전](https://github.com/NousResearch/hermes-agent/blob/v0.22.5/hermes_cli/update_cmd_config.py#L134-L241) | v0.22.5 공개2026-10-23, 조회2026-10-23 | 새 설정 질문·형식 변경만 있을 때 자동 이전·비대화형 분리 |
| [Hermes 새 완료 프로세스](https://github.com/NousResearch/hermes-agent/blob/v0.22.5/hermes_cli/_update_takeover.py#L184-L200) | 같은 공개 버전·조회일 | 새 코드의 완료 단계를 별도 프로세스로 실행. Hive 호환성의 직접 증명 아님 |
| [Hermes 제거](https://hermes-agent.nousresearch.com/docs/getting-started/updating#uninstalling) | 날짜 미표기, 조회2026-10-23 | 미리보기와 자료 포함 삭제의 구분. Hive에는 보존형만 채택 |
| [Debian Policy](https://www.debian.org/doc/debian-policy/ch-files.html#behavior) | 문서4.7.4.1, 조회2026-10-23 | 사용자 변경 보존·설정 이전의 재실행 안전. Debian 규칙과 일반 CLI 권고 구분 |
| [CLI Guidelines](https://clig.dev/#interactivity) | 날짜 미표기, 조회2026-10-23 | 대화형 터미널에서만 질문, 비대화형 입력 경로 제공 |
| [Windows AttachConsole](https://learn.microsoft.com/en-us/windows/console/attachconsole)·[콘솔 핸들](https://learn.microsoft.com/en-us/windows/console/console-handles) | 핸들 문서2021-12-30, 조회2026-10-23 | 부모 콘솔 연결·리다이렉션과 별도 입출력. 실제 Hive 부모/PTY에서 추가 시험 필요 |
| [Linux tty](https://man7.org/linux/man-pages/man4/tty.4.html) | 현행 문서, 조회2026-10-23 | /dev/tty는 현재 제어 터미널. 터미널 없는 자동화의 질문 수단 아님 |
| [Git 설정 출처](https://git-scm.com/docs/git-config#Documentation/git-config.txt---show-origin) | 날짜 미표기, 조회2026-10-23 | 값의 출처/범위 표시. 사용자 동의 이력 필드는 Hive 설계 제안 |
| [Codex 속도](https://learn.chatgpt.com/docs/agent-configuration/speed) | 날짜 미표기, 조회2026-10-23 | 지원 모델 Fast의 포함 구독 사용량2.5배, CLI 실행 속도와 별개 |
| [Codex 사용량](https://learn.chatgpt.com/docs/pricing) | 날짜 미표기, 조회2026-10-23 | 모델·문맥·추론·도구·검색·캐시 영향, 고정 메시지 수/설정 비용 보장 아님 |

## 저장소 관찰

- 공개0.13.1 update는 이미 core 교체 뒤 새 사용자 apply/validate 호출. 기존 부모의 입력null·출력pipe·10분 제한과 Windows CREATE_NO_WINDOW가 대화 연결 검증 대상
- 기존 갱신에 진행 확인 존재, --confirm은 이를 생략. 부족한 기능은 설정별 질문과 상세 변경 결과 전달
- ChildActionResult의 action/status/exit_code만 읽는 구조. 새 하위 data 추가만으로 첫 갱신 안내가 자동 표시된다는 주장 불가
- 기존 UsageGuardPreferences에 값별 의도 출처 없음. true만으로 직접 선택/빠른 기본값 구분 불가
- user-setup workflow13,187byte·질문 문서5,104byte·schema7,175byte·catalog3,607byte. 전체가 항상 모델에 입력된다는 주장은 제외, 제한 조회와 왕복 감축 후보로만 사용
- 모델 호출0인 기존 합성/호스트 콜백 검사는 일반 사용자의 실제 AI 설정 비용 증거가 아님

## 설계 판단

- 질문과 설정 처리를 알고리즘으로 통합해 AI의 반복 질문·전체 답안 재작성 축소. 원문 로그 수집이나 더 많은 모델 호출을 계측 전제로 추가하지 않는 방향
- 실제 공개0.13.1→새 후보의 첫 갱신을 초기 검증 관문으로 선정. 소형 모델 사용량·정확성·세 운영체제의 대화형 수용 전 제품 완료 주장 없음
