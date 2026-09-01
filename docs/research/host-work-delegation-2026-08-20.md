# Host 작업 자동 분담 가능성

> 조사일: 2026-08-20
> 범위: 공식 문서·로컬 설치 상태의 읽기 전용 조사
> 판정: host 기능 존재, Hive 자동 분담 활성화 보류

## 조사 경계

- Model call·하위 agent 실행·host 설정 변경: `0건`
- Provider API·API key·직접 host process 실행: `0건`
- Hive 사용자 기능 활성화: `0건`
- 로컬 확인: version·인증 상태·명령 존재 여부만 확인

## 결론

| Host | 공식 하위 agent | 역할별 model·추론 | 중단·후속 작업 | Hive 활성화 판정 |
| --- | --- | --- | --- | --- |
| Codex | 지원 | 지원 | steer·stop·close 지원 | 부분 지원 |
| Claude Code | 지원 | 지원 | message·stop·resume 지원 | 설치본 미검증 |
| Antigravity | 지원 | model tier만 지원 | interrupt·kill·idle 재개 지원 | 로컬 미지원 |

- 판정 단위 분리: 세 host의 공식 기능과 Hive의 안전한 활성화
- 미확인: exact role·model·effort·definition digest의 외부 검증 가능 receipt

## Codex

공식 근거:

- [OpenAI Docs: Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents)
- 현재 Codex release: app·CLI·IDE에서 하위 agent 기본 제공
- 직접 요청 또는 적용 가능한 `AGENTS.md`·Skill 지침 기반 분담
- `.codex/agents/`·`~/.codex/agents/`의 역할별 정의
- `model`·`model_reasoning_effort`·`sandbox_mode` 설정
- 명시값 → `[agents]` 기본값 → parent 상속의 model·추론 수준 해석
- parent의 spawn·follow-up routing·wait·stop·close와 결과 취합

로컬 증거:

- Codex CLI: `0.148.0`
- 현재 desktop host: 하위 agent 도구 노출
- 실제 하위 agent 실행: 조사 범위 밖으로 미실행

결손:

- 요청값과 실제 사용값을 Hive 외부 signer가 검증할 stable receipt 부재
- role·definition digest·model·effort를 한 번에 결합한 host attestation 부재
- late result·중단 경쟁·exactly-once dispatch의 host 보증 부재

판정: host-native 분담 지원, Hive exact-attestation 활성화는 부분 지원.

## Claude Code

공식 근거:

- [Claude Code: custom subagents](https://code.claude.com/docs/en/sub-agents)
- `.claude/agents/`·`~/.claude/agents/`의 역할별 정의
- alias·full model ID·`inherit`와 역할별 `effort`
- 환경 변수 → invocation → frontmatter → parent 순서의 model 해석
- 허용 목록 차단 시 다른 model 대체와 interactive 경고
- foreground·background 실행, message·stop·resume, worktree 격리
- resume 뒤 대화·도구 결과·reasoning 보존

로컬 증거:

- Claude Code: `2.1.163`
- 인증 상태: `loggedIn=false`, `authMethod=none`
- 실제 model call·하위 agent 실행: 불가·미실행

Version 결손:

- model 해석 `inherit` 수정: `2.1.196`
- resume의 invocation model 보존: `2.1.211`
- 허용 목록 대체 최신 동작: `2.1.222`
- 설치본 `2.1.163`: 위 수정을 포함하지 않는 구형 상태

판정: 최신 공식 기능은 부분 지원, 현재 설치본 lifecycle·actual model 미검증.

## Antigravity

공식 근거:

- [Antigravity 2.0: Subagents](https://antigravity.google/docs/subagents)
- [Antigravity CLI: Background tasks & subagents](https://antigravity.google/docs/cli/subagents)
- [Antigravity CLI: `/agents`](https://antigravity.google/docs/cli/commands/agents)
- `invoke_subagent` 기반 비동기 실행과 여러 agent 동시 실행
- `.agents/agents/`·`~/.gemini/config/agents/`의 Markdown 정의
- workspace `inherit|branch|share`, model tier `inherit|flash|pro`
- `running|idle|killed`, message interrupt, idle auto-wake, 영구 kill
- parent permission·sandbox 상속과 승인 요청 전달

로컬 증거:

- `agy`·`antigravity` 명령 확인 결과: 없음
- 실제 lifecycle·model tier 확인: 미실행

결손:

- exact model ID·역할별 effort 설정 부재, `inherit|flash|pro` tier만 제공
- Hive exact model·effort attestation과 직접 대응 불가
- 로컬 설치·인증·현재 workspace projection 검증 부재

판정: 공식 host 기능 지원, 현재 환경 미지원, Hive exact-attestation 미지원.

## Hive 연결 가능성

현재 기반:

- Markdown·tracked TOML의 계획·역할·run 정본
- host가 소비하는 선언형 envelope
- `hive agent recommend|preview|apply|attest|activate|route`의 launch 없는 계약
- host 소유 model call·session·subagent process 경계

안전한 연결 후보:

1. Hive가 역할·작업·수락 기준·definition digest를 선언형 자료로 준비
2. 선택 host가 자신의 native agent 기능으로 작업 수행
3. 별도 signer가 actual role·model·effort·result digest를 결합한 receipt 생성
4. Hive가 trust root·freshness·exact match 확인 뒤 결과 승격

현재 중단 조건:

- Codex·Claude·Antigravity 공통 외부 signer receipt 부재
- Host별 stable task identity·cancel acknowledgement·late result 계약 부재
- Antigravity exact model·effort mapping 부재
- Claude 최신 인증 설치본의 fresh-session 증거 부재
- Codex fresh-session actual model 증거 부재

## 최종 분류

- Codex: `partial` — native 분담·model·effort·steer 지원, exact attestation 부재
- Claude Code 최신판: `partial` — native lifecycle 지원, signed receipt 부재
- Claude Code 설치본 `2.1.163`: `unverified` — 미인증·필수 수정 이전 version
- Antigravity 공식 기능: `partial` — native lifecycle 지원, tier-only model 계약
- Antigravity 현재 환경: `unsupported` — CLI 미설치
- `0.10.0` 실제 활성화: 금지 유지
- 다음 경로: [`host-work-delegation.md`](../plans/backlog/host-work-delegation.md)
