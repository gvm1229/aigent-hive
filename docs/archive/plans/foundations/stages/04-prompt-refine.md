# Stage 4. `hive-prompt-refine`

> 상태: implemented workflow reference; 이 stage 검토 시에만 load

#### 완성 후 동작

`RefinePrompt` 또는 `hive-prompt-refine`의 자동 선택 조건: prompt 작성·개선·구조화, 모호한 brief의 copy-ready prompt 변환. 이름이나 역사적 별칭을 직접 말할 필요는 없지만, 일반 질문과 일반 작업 prompt를 몰래 가로채거나 재작성 금지.

기본 mode는 `refine-only`다. Refined prompt를 실행하는 `refine-and-run`은 사용자가 같은 요청에서 명시한 경우에만 허용.

Flow:

1. 원문을 immutable input으로 보존하고 primary intent, target audience/agent와 원하는 결과를 추출
2. project-scoped prompt일 때만 허용된 read-only context에서 실제 path, command와 제약을 확인
3. objective, scope, inputs, constraints, acceptance, output shape와 stop condition의 ambiguity를 평가
4. 결과를 바꾸는 필수 정보가 없으면 한 번에 한 질문만 수행
5. 복잡한 요구사항 정제가 필요하면 Hive-native interview workflow 또는 host question surface 사용
6. 사용자가 답하지 않은 비필수 항목은 명시적인 assumption 또는 placeholder로 남기고 사실을 창작 금지
7. provider-neutral prompt를 기본 생성하고 사용자가 target host를 지정했을 때만 Codex/Claude/Antigravity syntax로 얇게 projection
8. intent summary, assumptions/unresolved items와 copy-ready refined prompt를 분리해 반환
9. `refine-only`에서는 model execution, project mutation, subagent spawn, memory capture와 run 생성 0회
10. 사용자가 저장을 요청한 prompt만 지정한 tracked Markdown 또는 active run artifact에 기록

Refined prompt 최소 구조:

```text
Goal
Context and grounded inputs
Required workflow
Constraints and prohibited actions
Acceptance and verification
Output contract
Stop, blocker, and escalation conditions
```

이미 충분히 구체적인 prompt는 불필요하게 장문화 금지. 원문의 tone, authority boundary, must/must-not와 명시된 tool/provider 선택을 보존하며, 개선 전후 meaning drift를 검사.

#### 구현

- `harness/skills/hive-prompt-refine/SKILL.md`에 narrow trigger와 `refine-only` default 정의
- provider-neutral `PromptRefinement` input/result schema와 normalized fixture 추가
- intent·constraint·acceptance preservation을 structural field와 text locator로 검증
- optional project grounding은 read-only capability로 분리하고 simple-question path와 memory ingest에서 격리
- OMX/OMC analysis·interview 기능은 NAT-002 clean-room 재평가 대상, 신규 external invocation 없음
- prompt artifact에 original/refined text를 함께 저장할 때 secret scan과 explicit save consent 적용

#### 완료 조건

- [x] “이 prompt를 개선해줘”, “agent에게 줄 prompt를 만들어줘” fixture에서 이름 언급 없이 `hive-prompt-refine` 선택
- [x] 일반 질문·일반 coding request에서 automatic prompt rewrite 0회
- [x] `refine-only`에서 project write, subagent, run creation과 memory capture 0회
- [x] 원문 must/must-not, scope, target output과 user authority가 refined prompt에 손실 없이 존재
- [x] 필수 ambiguity 질문은 한 번에 하나이며 답하지 않은 항목을 fabricated fact로 채우기 금지
- [x] provider를 지정하지 않은 fixture에 provider-specific command·path 0개
- [x] 이미 충분한 prompt는 exact normalized character-growth budget을 초과 금지
- [x] `refine-and-run`은 명시 intent 없이 activation 불가
