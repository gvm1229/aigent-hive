## Response language contract

The selected interface language applies to every setup question, warning, summary, and recovery
result. A user message in another language does not by itself change the saved interface
language.

When the selected interface language is English, use ASD-STE100 Simplified Technical English.
Use short direct sentences, concrete verbs, and one main instruction, condition, result, or
warning per sentence. Use an approved dictionary word when known. Do not use idiom, figurative
language, casual filler, vague pronouns, stacked clauses, or unnecessary synonyms.

When the selected interface language is Korean, use Korean vocabulary and Korean sentence
structure. Keep English only for proper nouns, product or package names, commands, code
identifiers, paths, schema keys, exact UI labels, and terms without a clear Korean equivalent. Do
not insert replaceable English general nouns, mixed Korean-English compounds, or an English
parenthetical after an unambiguous Korean term. Translate meaning rather than English word order.
Keep an English literal only when the user must enter, select, search, or distinguish that exact
literal.

Do not write `benign한 source claim ID의 credential 오인`,
`safe한 default 적용`, `global setting을 update`, `fallback으로 처리`, or
`사용자 설정(user configuration) 확인`. Write `일반 원본 지식 항목 식별자의 비밀 값 오인 방지`,
`안전한 기본값 적용`, `전역 설정 갱신`, `대체 경로 처리`, or
`사용자 설정 확인` instead. Do not use English to look technical, shorten an ordinary Korean
word, or add emphasis. These examples are mandatory patterns, not a closed list.

## Korean interaction contract

When the selected interface language is Korean, retain product terms and identifiers exactly as
`Aigent Hive`, `Skill`, `Wiki`, `Codex`, `Claude`, `Antigravity`, `CodexBar`, commands,
paths, schema keys, Skill IDs, and versions. Do not translate `Skill` as `기술`.

Use these host-independent question patterns, one question at a time:

1. `사용 언어 선택: English 또는 한국어.`
2. `사용자 기본 맥락 선택. 복수 선택 가능. 프로젝트 작업 흐름·우선순위 결정 없음.`
   - `웹 개발`: 웹 애플리케이션 관련 배경 또는 관심사
   - `게임 개발`: 게임 관련 배경 또는 관심사
   - `일반 지식 작업`: 소프트웨어 개발 외 배경 또는 관심사
3. `추가 배경·관심사·선호 입력. 없으면 건너뛰기.`
4. `Skill 선택 방식 선택.`
   - `모든 내장 Skill 사용`
   - `개별 내장 Skill 선택`
5. `Judge 호출 정책 선택: explicit (권장) 또는 implicit. explicit: 반복·팀·다중 목표의 최종 수용만. implicit: 엄격한 중대한 위험 경로 추가. 단순 질문·읽기 전용·형식 전용·결정적 실패·tick·heartbeat·retry 제외.`
6. `Wiki 저장 위치: 이 컴퓨터의 Markdown 파일. Obsidian 같은 앱 열기 가능.`
7. `사용량 보호 선택: 활성화 (권장) 또는 비활성화. 신속 설정은 남은 사용량 20%에서 중지.`
8. `사용량 한도 도달 시 Discord 알림 수신 여부.`
   - `아니요`
   - `예, 시험 알림도 보내기`
9. `Discord webhook URL 저장: 환경 변수. 예: HIVE_DISCORD_WEBHOOK_URL. URL 자체 대신 환경 변수 이름 사용. Hive 시험 알림으로 연결 확인.`
10. `Discord 알림 항목·순서 선택. 기본값: 남은 사용량, 프로젝트, 요청, 진행 상태, 호스트, 계속하기. 시험 알림: 실제 알림과 같은 형식, 첫 줄 시험 안내 추가.`

For Korean partial reconfiguration without a named setting, show this catalog before asking which
setting to change. An explicitly named setting/value skips the catalog and repeated choice:

`변경할 전역 설정 1개 선택. 아래: 전체 변경 가능 설정.`

1. **인터페이스 언어** — 이후 Hive 질문과 요약에 사용할 언어
   - 선택: `English` 또는 `한국어`
2. **일일 업데이트 확인** — 24시간에 한 번 업데이트 존재 여부만 확인하며 자동 설치 없음
   - 선택: 켜기 또는 끄기
3. **Wiki** — 이 컴퓨터의 Markdown 지식 Wiki와 작성 언어
   - 사용 여부: 켜기 또는 끄기. 끄더라도 기존 Markdown 보존
   - 작성 언어: `en`, `ko`, 또는 `both`
4. **사용자 기본 맥락** — Hive가 사용자의 배경과 관심사를 이해하기 위한 정보. 프로젝트 작업 흐름·우선순위 결정 없음
   - 맥락: `web-developer`, `game-developer`, `non-developer` 중 복수 선택 가능
   - 추가 설명: 선택 사항인 한 줄 배경·관심사·선호
5. **에이전트 페르소나** — Hive 지원 작업의 기본 대화 방식
   - 선택: `strict`, `balanced`, `friendly`, 또는 `custom`
   - 사용자 지정 설명: `custom` 선택 때만 필수
  6. **사용할 호스트** — 전역 Hive 설정을 적용할 subscription host
   - 호스트: `codex`, `claude`, `antigravity` 중 하나 이상
7. **Judge 호출 정책** — 독립 수용 검토 호출 기준
   - 선택: `explicit` 또는 `implicit`
   - `explicit`: 반복·팀·다중 목표의 최종 수용만
   - `implicit`: `explicit` 경로와 엄격한 중대한 위험 경로. 단순 질문·읽기 전용·형식 전용·결정적 실패·tick·heartbeat·retry 제외
8. **내장 Skill** — 활성화할 내장 Hive Skill
   - 선택 방식: 모든 내장 Skill 사용 또는 개별 내장 Skill 선택
   - 개별 선택: 각 Skill의 켜기·끄기. 필수 `user-setup`는 계속 활성화
9. **사용량 보호** — 남은 사용량이 정한 기준에 도달할 때 새 Hive 작업 중지
   - 사용 여부: `활성화 (권장)` 또는 `비활성화`
   - 중단 기준: 남은 사용량 `1`%부터 `99`%까지
   - Discord 사용량 알림: 켜기 또는 끄기. 사용량 보호를 켠 경우에만 선택 가능하며 Hive에서 Discord로 보내는 알림만 지원
   - Discord webhook 환경 변수: `HIVE_DISCORD_WEBHOOK_URL` 같은 대문자 환경 변수 이름. Hive의 URL 자체 저장 금지
   - Discord 요청 공개 범위: 기본 `summary` 또는 preview·redaction 뒤 명시적으로 선택한 `raw-prompt`
   - Discord 알림 형식: 안전한 항목의 포함 여부와 순서. `remaining-usage`, `project`, `request`, `progress`, `host`, `resume`, `measured-at`, `evidence` 중 선택
   - Discord 알림 언어: 인터페이스 언어 기준. 시험 알림: 실제 중단 알림과 같은 항목·순서·언어. 첫 줄: 시험 안내.

When no setting was named, ask for one numbered parent setting or named child setting. Do not replace this catalog with a
single examples-only sentence.

Do not describe a global user context as a role that prioritizes web, game, non-development, or
any other project workflow. Project setup alone determines project-specific workflow, technical
choices, constraints, and delivery priorities.
