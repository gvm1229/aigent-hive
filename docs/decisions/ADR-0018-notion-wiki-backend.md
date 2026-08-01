# ADR-0018: Notion Wiki backend와 SQLite projection

- 상태: accepted
- 날짜: 2026-08-02
- Target: `0.9.0-test`
- 부분 대체: ADR-0003·ADR-0016의 consumer knowledge 정본 규칙
- 제외: Source `docs/` Wiki와 run·role·plan Markdown 정본

## 결정

### 상호 배타 backend

- `markdown`: 기존 user-root·project Markdown 정본과 user-root SQLite projection
- `notion`: 사용자 선택 Notion scope의 유일 정본과 user-root SQLite projection
- User-scope backend 1개, project별 혼합 mode 0건
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

### Capability·security

- 연결 우선순위: official host plugin/app → official hosted MCP → consented REST fallback
- Hive의 OAuth 대행·host-global config mutation·credential 저장 0건
- Selected root·database scope와 exact tool capability preview
- Partial fetch·unknown block·permission revoke·offline·rate limit: fresh index 승격 금지
- Notion content: untrusted data, embedded instruction authority 0건
- Remote canonical write 성공·SQLite publication 실패: dirty generation과 query fail-closed

### Discord

- 초기 범위: usage guard 중단의 optional outbound webhook 알림
- Claude inbound continuation: official Discord Channel plugin 위임
- Codex inbound continuation: official supported session channel 전까지 `unsupported`
- Webhook payload의 raw prompt·transcript·credential·continuation token 0건

## 선택 근거

- Notion·Markdown 양방향 정본 동기화와 conflict policy 제거
- 현재 Hive FTS5·ranking·citation·collection schema 재사용
- Notion API search와 SQLite query의 이중 RAG 제거
- Notion 편집 경험과 local fast retrieval의 결합
- 시험판·정식판 feature parity 유지

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
- 세부 실행: [`v0.9.0-test-finalization.md`](../plans/active/v0.9.0-test-finalization.md)
- 근거: [`discord-notion-host-integrations.md`](../research/discord-notion-host-integrations.md)
