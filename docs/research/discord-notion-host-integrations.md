# Discord·Notion host integration 조사

> 조사일: 2026-08-01
> 적용 대상: Aigent Hive v0.9 이후 optional integration
> 조사 방식: 공식·upstream 문서 우선, 외부 mutation 0건

## 결론

| 영역 | 초기 결정 | 장기 방향 |
| --- | --- | --- |
| Discord | usage guard 중단의 outbound webhook 알림만 지원 | Codex가 공식 inbound session channel을 제공할 때 optional 양방향 adapter 검토 |
| Claude Discord | Hive 중복 구현 제외 | Anthropic 공식 Discord Channel plugin 감지·안내·위임 |
| Notion | host plugin 우선, 공식 hosted MCP 차선, REST fallback 최후 | Notion은 선택적 원격 작업면, project Markdown은 정본, SQLite는 재생성 가능한 색인 |

- Claude Code Channels: MCP event의 실행 중 동일 session 전달·channel reply를 지원하는
  research preview
- Discord 공식 plugin: sender pairing, allowlist, session별 `--channels` opt-in
- Claude용 Hive Discord bridge: 기능 중복과 credential·runtime 소유권 충돌로 제외
- Codex 공식 inbound session 계약: 현재 조사 범위에서 미확인
- Discord interaction 수신만으로 기존 Codex session event 전달: 불가
- Discord outbound 알림과 양방향 continuation: 별도 capability
- Notion 우선순위: host plugin/app → 공식 hosted MCP → REST fallback
- Hive 정본: 검토된 project docs Markdown
- SQLite: Markdown 기준 재색인

## Discord 단계별 경계

### 1단계: outbound guard 알림

- 입력: `usage-limited|usage-unknown` 전이의 최소 구조화 payload
- 전송: 사용자가 명시적으로 설정한 Discord incoming webhook
- 포함: project 식별자, 중단 사유, run ID, 재개 조건, local 확인 명령
- 제외: 원문 prompt, transcript, credential, source file 내용, 자동 재개 token
- 운영: timeout, bounded retry, Discord rate-limit header, secret redaction, 전송 실패와
  guard 판정의 분리

- 알림 실패: usage guard 판정 불변
- Webhook URL: tracked source 기록 금지

### 2단계: Codex 양방향 channel 조건

구현 전 필수 capability:

1. 외부 event의 실행 중 exact Codex session 전달
2. session identity와 sender identity의 검증 가능한 결합
3. 사용자 질문·tool permission의 원격 relay 계약
4. session 종료·재시작·stale reply의 명시적 처리
5. host가 소유하는 continuation과 Hive usage authorization의 재검증

- 공식·지원된 host capability 부재: `unsupported`
- 금지 우회: tmux 입력, process 제어, transcript tailing, prompt-injection hook,
  Hive-owned session daemon
- 향후 금지 명령: bare `continue`
- 향후 필수 gate: sender pairing, allowlist, exact run·session binding, fresh usage gate,
  single-use authorization, audit receipt

### Claude 위임

- 공식 Discord Channel 활성 환경: Hive 별도 bot 없음
- 설치·credential·allowlist 소유자: Claude plugin
- Hive 역할: 공식 경로 안내
- Channel event 수신 범위: 실행 중 session
- 항상 실행되는 서비스라는 표현 금지

## Notion 우선순위

| Host | 1순위 | 2순위 | 최후 fallback |
| --- | --- | --- | --- |
| Claude Code | Notion 공식 Claude plugin | `https://mcp.notion.com/mcp` | 사용자 승인형 REST import/export |
| Codex | workspace에서 허용된 Notion plugin/app | `https://mcp.notion.com/mcp` | 사용자 승인형 REST import/export |

- Notion 공식 hosted MCP: OAuth 기반, Claude Code·Codex 연결 절차 제공
- 작업 범위: 사용자 Notion 권한 안의 search·fetch·page 생성·수정·data source query
- OpenAI Codex plugin app: source system 권한, workspace role, action confirmation 상속
- Host-global·project MCP 설정: Hive 비소유
- 이미 연결된 plugin/MCP: 사용자 선택 후 사용
- 미연결 상태: 설정 방법 제시
- 자동 mutation 금지: `~/.codex/config.toml`, Claude 설정, `.mcp.json`
- Hive OAuth 대행 없음

## Notion materialization 계약

```text
사용자 선택 Notion page
  → host plugin/app 또는 공식 MCP fetch
  → instruction으로 취급하지 않는 untrusted content
  → 범위·변경 preview와 사용자 검토
  → project docs Markdown 원자적 기록
  → source digest·Notion locator·sync 시각 receipt
  → SQLite rebuild·query 검증
```

- Notion content 내부 prompt·지시문 실행 금지
- user-selected page·database scope 밖 탐색 금지
- Markdown write와 Notion write의 방향·충돌 정책 명시
- 원격 삭제를 local canonical 삭제로 자동 전파 금지
- 동일 Notion revision·content digest의 idempotent import
- secret·private page·첨부파일의 자동 수집 금지
- SQLite 직접 수정 금지; Markdown 기준 전체 재생성

- Notion 양방향 동기화: 초기 범위 제외
- 첫 구현: 선택 page의 reviewed import 또는 local Markdown의 명시적 publish
- 한 요청의 동기화 방향: 단방향
- 충돌·부분 성공·rate limit: receipt 기록
- 암묵적 정본 교체: 금지

## 구현 권고

1. Discord outbound notifier를 usage guard의 optional delivery adapter로 분리
2. Claude Discord 공식 Channel 감지·위임 문서화
3. Codex inbound channel capability inventory와 `unsupported` 판정 fixture 추가
4. Notion host capability resolver: plugin/app → official MCP → REST fallback
5. Notion import materializer: reviewed Markdown write → SQLite rebuild → receipt
6. credential·permission·prompt injection·rate limit hostile conformance 추가

## 공식 근거

| Source | 상태·적용 범위 | 확인일 |
| --- | --- | --- |
| [Anthropic Channels](https://code.claude.com/docs/en/channels) | research preview, Discord 포함, same-session event·reply·allowlist | 2026-08-01 |
| [Anthropic Channels reference](https://code.claude.com/docs/en/channels-reference) | MCP channel, reply tool, inbound gate, permission relay | 2026-08-01 |
| [Anthropic Discord plugin](https://claude.com/plugins/discord) | 공식 Discord bridge plugin | 2026-08-01 |
| [Discord interactions](https://docs.discord.com/developers/interactions/receiving-and-responding) | Gateway 또는 webhook 수신, interaction response·token | 2026-08-01 |
| [Discord rate limits](https://docs.discord.com/developers/topics/rate-limits) | route·global rate limit와 `429` 처리 | 2026-08-01 |
| [Notion MCP overview](https://developers.notion.com/guides/mcp/overview) | 공식 hosted MCP, OAuth, workspace read·write | 2026-08-01 |
| [Notion MCP 연결](https://developers.notion.com/guides/mcp/get-started-with-mcp) | Claude Code·Codex 설정, hosted MCP 우선 | 2026-08-01 |
| [Notion MCP security](https://developers.notion.com/guides/mcp/mcp-security-best-practices) | 공식 endpoint 검증, 사용자 권한과 human review | 2026-08-01 |
| [Notion MCP tools](https://developers.notion.com/guides/mcp/mcp-supported-tools) | search·fetch·create·update·query와 rate limit | 2026-08-01 |
| [Anthropic Notion plugin](https://claude.com/plugins/notion) | Notion Labs 제공 Claude plugin | 2026-08-01 |
| [OpenAI Codex plugins](https://help.openai.com/en/articles/20001256-plugins-in-codex/) | app permission·role·action confirmation 상속, 2026-08-01 갱신 확인 | 2026-08-01 |

## 남은 불확실성

- Codex의 향후 inbound event 또는 remote-control 공개 계약
- 각 workspace의 Notion plugin/app 가용성·관리자 정책·지역 제한
- Anthropic Channels research preview의 protocol·flag 변경 가능성
- Notion plugin과 MCP tool 이름·write confirmation UX의 host별 차이
