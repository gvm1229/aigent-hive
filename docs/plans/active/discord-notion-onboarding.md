# Discord·Notion 연결과 usage-guard 알림 계획

> Checklist owner: `DNI-*`
> Target: 다음 독립 `0.9.0-test.N`
> Decision: [`ADR-0018`](../../decisions/ADR-0018-notion-wiki-backend.md)
> 선행 구현: [`TST9-*`](v0.9.0-test-finalization.md)
> 관련 setup: [`KST-*`](korean-setup-terminology.md)

## 목표

- 전역 setup 대화 안의 Markdown·Notion backend 선택과 실제 연결 검증
- 공식 Notion plugin/app 또는 hosted MCP의 host 소유 브라우저 OAuth 우선
- Discord webhook 생성·환경변수 설정·시험 알림의 대화형 안내
- 프로젝트·run·요청·진행 상태를 구분하는 usage-guard 중단 알림
- README와 함께 배포되는 접근 가능한 단일 HTML 시각 안내서
- Credential·원문 prompt·절대 경로의 기본 외부 전송 0건

## 제품 경계

### Notion

- 연결 우선순위: official host plugin/app → official hosted MCP → consented REST fallback
- OAuth·token·host-global 설정 소유자: Notion과 활성 host
- Hive 역할: backend 선택, host capability envelope 요청, receipt 검증, scope 확정,
  SQLite disclosure·동의, 연결 진단과 재개
- 브라우저 로그인: host가 공식 OAuth URL 또는 연결 UI 제공 시 사용자 동작으로 수행
- Hive 자체 OAuth callback server·token storage·provider API client 0건
- 지원 host surface 부재: `unsupported`와 host별 연결 절차 반환
- Notion mode의 canonical source·SQLite freshness·write-through: 기존 `TST9-004–012` 재사용

### Discord

- 목적지 기본값: 전역 incoming webhook 1개, 각 프로젝트가 상속
- 프로젝트 구분: payload의 안전한 project display name·project ID·run ID
- Project별 webhook override: 초기 범위 제외; secret·설정 복잡성 재평가 뒤 별도 결정
- 원문 prompt: 기본 제외, 정제된 요청 요약 기본값
- 원문 포함: 전역 명시적 opt-in·전송 직전 preview·secret redaction 통과 시에만 허용
- 절대 경로·credential·transcript·continuation token·source 내용 전송 0건
- Codex inbound continuation: 공식 session channel 전까지 `unsupported`
- Claude inbound continuation: 공식 Discord Channel plugin으로 위임

## 알림 계약

필수 구조화 필드:

- `project_display_name`
- `project_id`
- `run_id` 또는 `untracked`
- `request_summary`
- `current_phase`
- `completed_items`
- `total_items` 또는 `unknown`
- `last_checkpoint`
- `resume_hint`
- `host_scope`
- `usage_window`
- `remaining_percent`
- `halt_reason`
- `measured_at`
- `evidence_digest`

진행률 규칙:

- Canonical plan/run checklist 존재: `completed_items / total_items`와 계산된 백분율
- 분모 부재: 임의 백분율 금지, 현재 단계·완료 결과·남은 작업·checkpoint만 표시
- Usage 잔량과 작업 완료율의 별도 label·필드
- 새 halt transition 1회만 발송, 재시도·rate limit·실패 진단의 기존 notifier 계약 유지

## Checklist

### A. 계약·host 연결

- [x] [DNI-001] 기존 schema·CLI·Skill·research·test의 end-to-end gap 표와 ADR·threat model 갱신
- [ ] [DNI-002] `user-setup` catalog·schema에 Wiki backend, Notion scope·local index consent,
  Discord opt-in·prompt privacy mode의 typed migration 추가
- [ ] [DNI-003] Host plugin/app·hosted MCP·REST의 provider-neutral capability envelope와
  `connected|authorization-required|unsupported|invalid` receipt 계약 구현
- [ ] [DNI-004] Codex·Claude·Antigravity별 Notion 연결 탐지·공식 browser OAuth handoff·완료 후
  exact workspace/scope capability 재검증 구현

### B. 전역 setup 경험

- [ ] [DNI-005] `setup-hive` custom 흐름에 Markdown·Notion 선택, 연결 방식, scope,
  SQLite disclosure·동의를 한 질문씩 추가; expedited 기본 Markdown 유지
- [ ] [DNI-006] Notion 시험 read·create/update capability preview와 사용자 승인 후 제한된 시험,
  실패 시 설정 미활성·재개 가능한 진단 구현
- [ ] [DNI-007] Usage guard opt-in 뒤 Discord 설정 선택, webhook 생성 안내, 환경변수 이름 검증,
  secret 비저장과 선택형 시험 알림 구현
- [ ] [DNI-008] Reconfigure의 Notion 재인증·scope 변경·Discord on/off·webhook 교체와
  기존 설정 보존·dry-run·rollback 구현

### C. Runtime 연결

- [ ] [DNI-009] Notion mode의 매 user turn host freshness envelope 생성·receipt 검증·changed-only
  SQLite preflight를 `hive-knowledge-query` route에 연결
- [ ] [DNI-010] Agent-reviewed capture의 Notion canonical-first host write envelope·confirmed receipt·
  SQLite write-through를 `hive-knowledge-capture` route에 연결
- [ ] [DNI-011] Usage halt marker에 안전한 project identity·run ID·요청 요약·checkpoint reference를
  snapshot으로 결합하고 stale·cross-project binding 차단
- [ ] [DNI-012] Canonical plan/run state 기반 진행 상태 reducer와 분모 없는 작업의
  truthful unknown 표현 구현
- [ ] [DNI-013] Discord payload schema·human message에 project·request·progress·resume context 추가,
  raw prompt opt-in preview·redaction·size limit 구현

### D. 안내·검증·출시

- [ ] [DNI-014] 대화형 setup 문구와 README의 Discord·Notion 순서형 안내·host별 차이·실패 복구 추가
- [ ] [DNI-015] Release bundle의 self-contained HTML 시각 안내서와 `hive guide integrations --open`
  또는 exact local locator 구현; 외부 script·tracking·secret embedding 0건
- [ ] [DNI-016] Schema·Rust·Skill projection·static 계약과 fake webhook·fake host adapter의
  success·offline·OAuth expiry·429·scope drift·redaction 회귀 시험
- [ ] [DNI-017] 세 host setup fixture와 지원 host 실제 E2E, 두 프로젝트 동시 halt 구분,
  plan-backed·unplanned progress, Notion edit→다음 prompt freshness 검증
- [ ] [DNI-018] `KST-006`과 결합한 독립 numbered test candidate·publication·clean install 수용;
  npm `latest` 불변과 stable feature parity 확인

## 실행 순서

1. `DNI-001–004`: 현재 공식 host 문서 재검증, 계약·위협 모델·adapter envelope
2. `DNI-005–008`: 전역 setup·재설정·연결 시험
3. `DNI-009–013`: Notion turn integration·프로젝트별 Discord payload
4. `DNI-014–015`: README·대화·HTML 시각 안내
5. `DNI-016–017`: 회귀·host fixture·실제 E2E
6. `KST-006` + `DNI-018`: 동일 candidate의 한국어 setup·integration 수용

## DNI-001 증거

- 2026-08-07 Notion 공식 MCP 문서의 Codex·Claude Code·Antigravity hosted MCP·OAuth 절차 재확인
- Codex plugin/app의 workspace role·permission·action confirmation 소유와 Hive access grant 불가 재확인
- Claude Discord Channels research preview·실행 중 session 한정 재확인
- schema·receipt·SQLite·Discord·setup·안내 gap 표와 ADR 위협 모델 반영

## 완료 기준

- 처음 설치한 사용자의 대화만으로 Notion 또는 Discord 설정 완료
- Notion 공식 연결 가능 host의 browser OAuth 뒤 credential-free receipt 검증
- Notion 변경 뒤 다음 user prompt에서 stale SQLite 검색 0건
- 서로 다른 두 프로젝트의 Discord 중단 알림 식별 가능
- Plan-backed 진행률의 canonical checklist 계산, 임의 추정 백분율 0건
- 원문 prompt·credential·절대 경로의 동의 없는 Discord 전송 0건
- README·대화형 setup·HTML 안내의 동일 순서·용어·복구 절차
- 시험판·정식판 기능 차이 0건, 시험판 게시 뒤 npm `latest` mutation 0건

## 외부 중지 경계

- 실제 Notion workspace browser OAuth·scope 승인·write confirmation
- 실제 Discord server의 incoming webhook 생성·secret 환경변수 설정
- Claude subscription·Antigravity 공식 capability 부재 범위의 truthful unverified 또는 unsupported
- GitHub·npm numbered test publication authority
