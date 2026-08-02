# ADR-0012: Global onboarding과 shared knowledge index

- 상태: accepted
- 날짜: 2026-07-28
- Target: `0.8.0`
- 부분 대체: ADR-0009의 direct operational install, 전체 built-in Skill projection,
  project/root 독립 SQLite

## 결정

### User onboarding

| 단계 | 계약 |
| --- | --- |
| User install | Host-native minimal bootstrap package와 Hive marker 설치 |
| 설치 직후 | `setup-required`; global setup 외 operational Hive route 차단 |
| Global setup | `setup-hive` Skill + `hive setup --scope user` typed CLI |
| Reconfigure | 동일 setup workflow의 기존 answer preview·변경·검증 |
| 정본 | `~/.hive/config/user-setup.yml` |

Global setup answer:

- Interface language: `en|ko`
- Wiki language: `en|ko|both`
- User profile: signed release catalog의 web developer, game developer,
  non-developer와 `custom`
- Agent persona: signed release catalog의 strict, balanced, friendly와 `custom`
- Active host: `codex|claude|antigravity` 복수 선택
- Skill selection: recommended suite 또는 개별 선택
- Wiki: 기본 `enabled`, 명시적 opt-out
- Usage guard: 명시적 opt-in, enabled 상태의 기본 remaining threshold `20%`

### User projection

- Provider-neutral generated projection: `~/.agents/directives/`,
  `~/.agents/skills/`
- Canonical preference·consent: `~/.hive/config/`
- Host별 native plugin package: 선택 host와 선택 Skill만 projection하는 thin adapter
- Global guidance: active host instruction file의 exact `AIGENT-HIVE:USER` marker
- Foreign byte와 third-party marker 보존
- Setup bootstrap·reconfigure Skill: 항상 설치
- 선택 Skill dependency closure: preview와 사용자 승인 필수

### Wiki lifecycle

- Global Wiki 기본 활성화
- Setup 중 opt-out, 이후 setup rerun 또는 명백한 agent request로 disable·enable
- Disable: capture·query·automatic retrieval·index refresh 중지
- Existing canonical Markdown: 기본 보존
- Canonical Markdown 삭제: 별도 explicit destructive action
- Re-enable: preserved Markdown 기반 index rebuild
- Enabled completion gate: material work의 agent-reviewed task fact 자동 capture
- 기본 task fact: bounded outcome, tool 또는 project, criteria, originating request summary
- Exact request: explicit retention intent와 safety review 필요
- Raw transcript·hook payload·tool output·hidden prompt·runtime state ingestion 금지

### Usage guard

- Global setup의 explicit opt-in
- Enabled threshold 기본값: remaining `20%`
- Qualified native sensor 우선
- CodexBar: native unavailable·unsupported·malformed 상태의 fallback-only
- CodexBar 설치: 필요성·고정 command preview·current-action consent 이후
- Claude status-line integration: host-owned opt-in과 non-clobber 유지
- Guard opt-out: Hive core·Wiki·project setup 사용 가능

### Project setup

| Mode | 질문 |
| --- | --- |
| `expedited` | Global preference 상속 + 필수 project kind |
| `custom` | 필수 project kind + language·Wiki language·persona·Skill override |

공통 계약:

- Project kind 질문 생략 금지
- Project별 `AGENTS.md`, `.agents/`, canonical `.hive/knowledge/`
- Global Wiki disable 상태의 project Wiki: disable 상속
- Custom project Wiki opt-in: global Wiki enable 또는 동일 action의 global re-enable 필요

### Shared SQLite

- 유일한 product knowledge DB: `~/.hive/index/hive.sqlite3`
- Index source: user-root canonical Wiki + 등록된 project canonical Wiki
- Project registration ledger: `~/.hive/config/projects.yml`
- Project `.hive/index/hive.sqlite3`: `0.8.0` 이후 생성 금지
- Durable fact의 SQLite-only 저장 금지
- Clean canonical Markdown + registration ledger 기반 무네트워크 rebuild
- 모든 row의 source project, page ID, language, digest, visibility provenance
- Current project query: own project 전체 + cross-project visible knowledge
- Other-project query: confidential·project-private row 제외
- Project removal·move·stale source: explicit scan 결과와 deterministic rebuild
- Root promotion: user-root canonical Markdown으로 durable 공유가 필요한 경우 유지

## Compatibility

- `0.7.0` user install의 host·guidance·knowledge 보존
- `0.7.0` project canonical Markdown 보존
- Existing project SQLite: migration 입력 제외 후 삭제 가능한 derived artifact
- First `0.8.0` update: global setup review 필요 상태와 non-destructive migration preview
- Setup 미완료 상태의 knowledge 삭제·host uninstall·project mutation 없음

## 결과

- 설치와 개인화 분리
- User preference 기반 일관된 multi-project harness
- Project Wiki의 단일 user-scope search index
- Cross-project retrieval과 canonical data 경계 분리
- CodexBar fallback-only 결정 유지
- Feature minor target `0.8.0`; planning change의 즉시 version mutation 없음
