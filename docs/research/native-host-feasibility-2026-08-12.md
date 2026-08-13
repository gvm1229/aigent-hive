# Native host·custom subagent feasibility

> 기준일: 2026-08-12
> 범위: `NAT-002–005`, `MRA-001–006`
> 상태: schema·공식 계약·로컬 발견 검증 완료, 실제 fresh-session dispatch·attestation 수용 대기

## 근거

- Codex 공식 문서: [Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents)
- Claude Code 공식 문서: [Create custom subagents](https://code.claude.com/docs/en/sub-agents)
- 로컬 Codex: `codex-cli 0.147.0`, project `.codex/agents/*.toml` profile의 fresh ephemeral
  `--profile` 발견·실행 확인. 이 경로는 profile configuration load 증거이며 parent→child native
  delegation 또는 host-signed runtime receipt 증거와 분리
- 로컬 Claude Code: `2.1.163`, `--agents`·`--agent`·`--model`·`--effort`·권한 mode·fallback option
  확인. `claude auth status`는 `loggedIn=false`; fresh-session model 실행·attestation `0회`
- Typed fixture: `tests/fixtures/native-orchestration/*-capability.json`

## Capability matrix

| Capability | Codex | Claude Code | Antigravity |
| --- | --- | --- | --- |
| User/project agent 발견 | 지원 | 지원 | 미검증 |
| Exact model·effort pin | 지원 | 지원 | 미검증 |
| Native dispatch·result | 지원 | 지원 | 미검증 |
| Thread/task lookup·cancel | 지원 | 지원 | 미지원 판정 |
| Hive idempotency key | 미지원 | 미지원 | 미지원 |
| Exact role·model·effort·digest 서명 receipt | 부분 | 부분 | 미지원 |
| Fresh-session profile 발견 | 확인 | 인증 부재 | 제외 |
| Fresh-session child dispatch·attestation 수용 | 대기 | 인증 부재 | 제외 |

`partial`: host의 native 상태·result는 존재하지만 Hive typed receipt·서명·정의 digest 결합 부재.
`unsupported`: 공개·로컬 증거로 필수 계약 부재. 숨은 fallback 금지.

## Host 계약 판정

### Codex

- User scope `~/.codex/agents/*.toml`, project scope `.codex/agents/*.toml`
- 필수 `name|description|developer_instructions`, 선택 `model|model_reasoning_effort|sandbox_mode`
- Project·Skill 지침의 delegation 요청과 native agent thread·wait·steer·interrupt 지원
- Parent permission override 우선, custom agent `sandbox_mode`만 더 좁은 권한에 사용
- Hive 보완 필요: dispatch 전 authority, dispatch 뒤 exact definition digest·actual model receipt

### Claude Code

- User scope `~/.claude/agents/*.md`, project scope `.claude/agents/*.md`
- Managed→session `--agents`→project→user→plugin precedence
- 필수 `name|description`, 선택 `tools|disallowedTools|model|effort|permissionMode|maxTurns`
- Agent ID·resume·result·foreground/background·cancel과 fresh context 지원
- Model allowlist·fallback substitution 가능성. Actual model 검증 없는 성공 수용 금지
- Installed `2.1.163`: current 문서의 일부 후속 lifecycle fix보다 이전. 실제 수용 전 update 또는
  현재 version 한정 negative test 필요

### Antigravity

- Qualified custom subagent·receipt·cancel evidence 부재
- `0.9.2` custom subagent 기능의 명시적 `unsupported`, hidden fallback `0건`

## Sol Advisor clean-room 동등성

| 기능 | Hive·host owner | 수용 기준 |
| --- | --- | --- |
| Primary orchestration | Parent host session | Provider API·직접 process spawn `0건` |
| Routine implementation | Bounded role profile | Exact model·effort·scope·digest receipt |
| Complex implementation | Bounded role profile | 별도 narrow route·권한·negative fixture |
| Independent review | Reserved read-only Judge | Project shadow·self-approval·downgrade 거부 |
| Attestation | Host result + external Ed25519 signer | Assignment·artifact·role·runtime 결합 검증 |

외부 prompt·runtime code·namespace 복사 `0 bytes`. 기능 이름과 공개 capability만 clean-room 입력.

## Feasibility 결론

- Data-only cooperative lifecycle: 구현 가능
- Host model·session 직접 소유: 금지 유지
- Exactly-once automatic dispatch: host idempotency proof 부재로 주장 금지
- Ack 유실: `dispatch-uncertain` 정지와 authenticated `non-launch-proof` 전 자동 reclaim 금지
- Codex·Claude activation: default-off 유지, 두 host fresh-session lifecycle 뒤 별도 결정
- Antigravity activation: unsupported

## 남은 실제 증거

1. Codex exact Luna·Terra·Sol role의 fresh-session dispatch·result·runtime metadata
2. Claude exact model·effort의 fresh-session dispatch와 fallback·override negative case
3. Host result를 Hive typed receipt로 변환하는 cooperative CLI 수용
4. Cancel·late result·resume·wrong role/model mismatch 수용

위 증거 전 `NAT-004–005`, `MRA-004–006`과 runtime activation 완료 처리 금지.
