# Stage 0. 진입 라우팅

> 상태: implemented workflow reference; 이 stage 검토 시에만 load

#### 완성 후 동작

사용자 요청을 다음 명시 action으로 분리:

- `SetupHarness`
- `AnswerSimpleQuestion`
- `RefinePrompt`
- `RunWork`
- `ResumeWork`
- `VerifyWork`
- `IngestKnowledge`
- `QueryKnowledge`
- `UpdateHarness`

자동 intent 추정은 보조 신호만 사용. 명시 action과 충돌하면 명시 action 우선.

Skill resolution은 action 분류 뒤 다음 precedence를 사용:

1. 사용자가 명시한 Skill 또는 명시한 direct/plain-answer 지시
2. simple-question gate
3. active host가 노출한 compatible OMX/OMC Skill
4. 승인되어 host discovery surface에 projection된 Hive 고유 Skill
5. host-native direct capability

Semantic routing은 Skill의 좁은 `description`과 generated `AGENTS.md`의 compact mapping으로 수행. Hive가 별도 `UserPromptSubmit` classifier hook을 생성 금지. Catalog가 크더라도 사용자가 승인한 Skill만 active projection에 들어가며, 한 task에는 필요한 최소 Skill 집합만 본문을 load.

#### 구현

- host projection은 action을 Hive namespace로 노출
- Rust CLI는 동일 action name과 `--output json`을 제공하고 `schemas/action-result.schema.json`을 준수
- action마다 허용 read/write/subagent/network capability 선언
- Skill catalog entry마다 `provided_by`, `superseded_by_external`, invocation intent와 side-effect class 선언
- unsupported action은 write 전에 종료

CLI process exit 의미:

| Exit | 의미 | `ActionResult.status` |
| ---: | --- | --- |
| `0` | 요청한 action 성공 | `success` |
| `2` | 잘못된 입력, schema 위반 또는 source-target 거부 | `blocked` 또는 `error` |
| `3` | 안전·승인 blocker 또는 ownership conflict | `blocked` 또는 `conflict` |
| `4` | 선택 host/runtime에서 capability 미지원 | `unsupported` |
| `5` | action 실행 후 필수 검증 실패 | `verification-failed` |
| `10` | 예기치 않은 내부 오류 | `error` |

`--output json`에서는 비정상 종료도 가능한 범위에서 schema-valid JSON 한 개만 stdout에 기록하고 human diagnostic은 stderr로 분리. Write 전 실패는 `changed_paths: []`, 모든 evidence는 locator와 `sha256` digest를 포함. 문자열 `code`는 exit number와 별개인 안정된 `hive.*` domain code.

#### 완료 조건

- [x] 세 host에서 같은 logical action이 같은 contract로 resolve
- [x] unknown action이 project write 또는 agent spawn 없이 실패
- [x] host별 alias가 core state에 저장 금지
- [x] investigate 요청에서 available OMX/OMC `analyze`가 Hive duplicate 대비 우선
- [x] approved Hive-only Skill은 이름을 직접 말하지 않아도 matching description에서 자동 선택
- [x] simple-question fixture에서 Skill description 외 추가 Skill body load 0개
- [x] explicit `plain answer`/`no workflow` 요청이 automatic Skill invocation에 우선
- [x] 모든 exit class가 schema-valid `ActionResult`와 정확히 대응
