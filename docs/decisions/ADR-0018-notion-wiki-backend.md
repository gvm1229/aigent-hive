# ADR-0018: Notion Wiki backend와 SQLite projection

- 상태: accepted, `0.10.0-test` 공개 보류
- 날짜: 2026-08-08
- Target: 첫 `0.10.0-test`
- 부분 대체: ADR-0003·ADR-0016의 consumer knowledge 정본 규칙
- 제외: Source `docs/` Wiki와 run·role·plan Markdown 정본

## 결정

### 출시 분리

- `0.9.0` test·stable의 user setup, CLI help, README, release note, bundled guide에서 Notion 노출 `0건`
- `0.9.0`의 user-visible Wiki backend: local Markdown 정본과 user-root SQLite projection
- Notion의 setup 선택·host 연결·freshness·write-through·사용자 문서 공개: 첫 `0.10.0-test`부터
- 현재 typed backend·SQLite engine·receipt validator는 `0.10` 구현 후보. 실제 host 연결 또는 사용자 지원 완료 주장 금지

### 상호 배타 backend

- `markdown`: 기존 user-root·project Markdown 정본과 user-root SQLite projection
- `notion`: 사용자 선택 Notion scope의 유일 정본과 user-root SQLite projection
- `0.10` user-scope backend 1개, project별 혼합 mode 0건
- 기존 `0.8.x` install의 기본 backend: `markdown`
- Mode 전환: explicit reconfigure·preview·검증·activation, product update와 결합 0건

### Notion mode

- Active local Wiki Markdown 생성·조회·기록 0건
- Notion page·database의 create·update 성공 뒤 durable write 완료 판정
- SQLite: Notion content chunk·metadata·FTS5 ranking의 삭제 가능 파생 색인
- SQLite local-content 저장 사실과 rebuild의 Notion 연결 의존성 setup disclosure
- 일반 RAG query: SQLite 단일 검색 owner
- Notion search: 초기 scope 선택·복구·누락 진단
- 매 user turn: remote revision freshness 확인과 changed-only fetch
- Webhook·양방향 Markdown sync·Notion AI 병렬 검색 0건
- `0.10` global setup의 `markdown|notion` 명시적 선택과 expedited 기본 `markdown`
- Notion 선택 시 official host plugin/app → hosted MCP → consented REST 연결 탐지
- Official host surface의 browser OAuth handoff와 완료 후 workspace·scope capability receipt 재검증
- Hive 자체 OAuth callback server·token 저장·host-global config mutation 0건
- 연결·scope·SQLite disclosure 질문, Notion 선택 동의에 포함한 제한 시험
- 연결 시험 실패 또는 설정 종료: 적용 전 답변·현재 질문·안전한 오류 요약만 Hive 관리 진행 기록에
  보존. OAuth token·webhook URL·원문 prompt·절대 경로 보존 0건
- 미완료 기록 재실행: `처음부터 전체 검토`, `선택 항목 검토`, `중단한 단계부터 계속` 제공. 계속 선택 시
  완료 답변과 연결 receipt 재검증 뒤 실패 또는 다음 단계부터 진행

### Capability·security

- 연결 우선순위: official host plugin/app → official hosted MCP → consented REST fallback
- Hive의 OAuth 대행·host-global config mutation·credential 저장 0건
- Selected root·database scope와 exact tool capability preview
- Partial fetch·unknown block·permission revoke·offline·rate limit: fresh index 승격 금지
- Notion content: untrusted data, embedded instruction authority 0건
- Remote canonical write 성공·SQLite publication 실패: dirty generation과 query fail-closed

### DNI-001 위협 모델 보강

| 위협 | 차단 계약 |
| --- | --- |
| Host config·OAuth token 탈취 또는 무단 변경 | Hive의 host config mutation·OAuth callback·token 저장 없음; host UI·CLI 소유 |
| Notion content의 prompt injection | remote content untrusted 처리, instruction authority·자동 Skill activation·외부 전송 없음 |
| Scope·workspace 또는 adapter drift | exact workspace·scope·read/create/update receipt 재검증, mismatch fail-closed |
| Discord의 prompt·secret·경로 유출 | raw prompt 기본 제외, opt-in preview·redaction 전송, credential·absolute path·transcript 0건 |
| Discord delivery failure·429 | guard 판정과 delivery 분리, bounded retry·진단, 자동 resume 없음 |
| Claude/Codex session 탈취 | Claude 공식 Channel plugin 위임, Codex 공식 compatible channel 전 `unsupported` |

### Discord

- 초기 범위: usage guard 중단의 optional outbound webhook 알림
- Claude inbound continuation: official Discord Channel plugin 위임
- Codex inbound continuation: official supported session channel 전까지 `unsupported`
- 전역 incoming webhook을 project가 상속하고 payload에서 안전한 project ID·run ID로 구분
- Payload의 정제된 요청 요약·canonical 진행 상태·checkpoint·재개 안내
- 원문 prompt 기본 제외; 전역 명시적 opt-in·전송 preview·redaction 뒤에만 제한적 포함
- Credential·transcript·절대 경로·continuation token의 외부 전송 0건
- 대화형 webhook 설정·환경변수 검증·시험 알림과 배포 bundle의 self-contained HTML 안내
- Discord 활성화 동의: webhook URL의 Hive 저장 없이 환경변수 이름만 저장하고 단일 시험 알림까지 포함
- 부분 설정 변경: 예시 문장 대신 전체 전역 설정과 하위 설정을 한 줄 목록으로 먼저 표시. 사용량 보호 하위에 Discord 알림·webhook 환경 변수 이름·요청 공개 범위 포함

### 구현 상태 구분

- `TST9-*`: typed backend, SQLite projection, capability receipt 검증, outbound notifier core
- `DNI-*`: 실제 host connection, global setup 대화, browser OAuth handoff, project-aware payload,
  README·HTML 안내와 end-to-end 수용
- 내부 core 완료를 사용자 연결 완료로 해석 금지

## 선택 근거

- Notion·Markdown 양방향 정본 동기화와 conflict policy 제거
- 현재 Hive FTS5·ranking·citation·collection schema 재사용
- Notion API search와 SQLite query의 이중 RAG 제거
- Notion 편집 경험과 local fast retrieval의 결합
- `0.10` 시험판·정식판 feature parity 유지

## 대안

| 대안 | 제외 근거 |
| --- | --- |
| Notion ↔ Markdown 양방향 sync | 정본 충돌·매 turn reconciliation·삭제 전파 복잡성 |
| Notion MCP live search only | 검색 품질·지연·rate limit·cross-project result 계약 drift |
| Notion + Notion AI + SQLite 병렬 검색 | 중복 비용·ranking authority 불명확 |
| Webhook 기반 변경 감지 | Active prompt boundary에 불필요한 daemon·public endpoint 부담 |

## 결과

- Notion mode의 offline Wiki availability 없음
- Notion mode SQLite rebuild의 network·authorization 의존
- Notion content의 로컬 SQLite copy와 explicit consent 필요
- Existing Markdown-mode portability·무네트워크 rebuild 보존
- Source Wiki·run·role·plan의 tracked Markdown authority 불변
- `0.9.0` 공개 범위: Markdown Wiki만
- 세부 후보: [`v0.10.0-notion-candidate.md`](../plans/active/v0.10.0-notion-candidate.md)
- Discord `0.9` 후속: [`discord-onboarding-v09.md`](../plans/active/discord-onboarding-v09.md)
- 근거: [`discord-notion-host-integrations.md`](../research/discord-notion-host-integrations.md)
