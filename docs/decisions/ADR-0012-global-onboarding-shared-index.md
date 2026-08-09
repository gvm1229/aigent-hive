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
- User profile: 사용자 이해용 전역 기본 맥락. signed release catalog의 복수 context와 optional
  사용자 설명 동시 선택·보존. workflow·작업 우선순위·project별 구현 방식 결정 금지
- Agent persona: signed release catalog의 strict, balanced, friendly와 `custom`
- Active host: `codex|claude|antigravity` 복수 선택
- Skill selection: 모든 built-in Skill 기본 활성화. 변경은 Skill별 on/off; profile·recommended suite
  결합 없음. existing recommended closure 변경은 preview·명시 approval 이후
  - typed user config: `all|individual`; legacy `recommended`는 저장된 config validate·preview
    migration 전용
  - project setup recommendation: global catalog와 분리된 project-only catalog
- Wiki: 기본 `enabled`, 명시적 opt-out
- Usage guard: 명시적 opt-in, enabled 상태의 global default remaining threshold `20%`,
  등록 project별 더 보수적인 early-stop override

### Setup scope routing

- Global·user-scope preference, language, host, Skill, Wiki, persona, usage guard 요청: `setup-hive`
- Project·repository·folder·path의 local harness 요청: `setup-harness`
- Bare Hive setup·reconfigure 요청: global user-scope 우선, ambient working directory inspection 없음
- Global·project 동시 요청: global setup 완료 뒤 project setup의 별도 사용자 확인
- Numbered test release의 user projection update: exact authenticated predecessor inventory만 허용
- `0.9.0-test.3` Codex host inventory: frozen `setup-hive` digest와 current selected projection의
  exact inventory 조합만 predecessor 인증
- Unknown·변조 predecessor manifest: preview·apply 차단과 foreign byte 보존

### User projection

- Provider-neutral generated projection: `~/.agents/directives/`,
  `~/.agents/skills/`
- Canonical preference·consent: `~/.hive/config/`
- Host별 native plugin package: 선택 host와 선택 Skill만 projection하는 thin adapter
- Global guidance: active host instruction file의 exact `AIGENT-HIVE:USER` marker
- Foreign byte와 third-party marker 보존
- Setup bootstrap·reconfigure Skill: 항상 설치
- 선택 Skill dependency closure: preview와 사용자 승인 필수
- Authenticated historical base와 live local·incoming digest 비교
- Schema-1 `0.7.0`: saved preference digest·지원 legacy path inventory·모든 recorded live digest
  일치 시에만 base 인증. later `agents/openai.yaml` metadata는 신규 파일로 추가하고 schema 2 full base 기록
- `local == base`: incoming exact replacement
- Disjoint local·incoming text 변경: 양쪽 hunk 결합
- Overlap: local hunk 보존·omitted incoming hunk preview
- Schema-2 전 legacy local edit: fabricated base·자동 merge 없이 conflict와 active byte 보존
- Missing·unauthenticated base: active bytes 불변·conflict

### Skill identity amendment (`0.9.0`)

- Host-facing invocation: `aigent-hive:<short-name>`
- 이름 변경 범위: source·consumer Hive-owned Skill 합집합. 승인 이름 정본: [`docs/skills.md`](../skills.md)
- Shared workflow도 source·consumer active ID 분리. Related name family는 유지하고 exact collision 금지
- Source-only Skill의 `hive-*` 예외 없음. Consumer host만 `aigent-hive:` namespace 부여
- Consumer built-in Skill: short action-oriented IDs만 신규 projection·catalog·preview에 출력
- Legacy `hive-*`, `setup-hive`, `setup-harness`, `ai-slop-cleaner`,
  `best-practice-research`: saved selection migration 입력 전용
- Migration result: `configure`, `setup-project`, `record-knowledge`,
  `import-repository-knowledge`, `clean-ai-slop`, `research-practices` 등 current public ID
- Rename ledger: scope별 source·consumer retired ID → current ID canonical mapping. saved selection
  migration, dependency closure, source routing, collision reservation에 공통 사용. 삭제 권한 없음
- Retired projection cleanup: frozen release inventory 또는 installed ownership manifest의 release byte
  ·ownership proof 일치 때만 삭제. 변조·unknown·foreign path는 write 0건 conflict. Future rename: ledger와
  authenticated historical-base cleanup regression 동시 추가
- `en|ko` global interface language: Hive-owned user projection의 display name, short description,
  `SKILL.md` frontmatter description에 적용
- Workflow body: provider-neutral English contract 유지
- Historical release inventory: frozen byte·old ID 보존, current rename 대상 제외

### Global setup UX

- Initial setup: interface language 질문 우선
- 질문 전 signed CLI 확인 필수. Windows의 ambient `PATH`에 `hive`가 없으면 `Get-Command`,
  `where.exe`, `npm prefix -g`로 npm-owned `hive.cmd` absolute path를 찾아 version·ownership 확인.
  확인 전 질문·answer 저장 시작 금지
- Signed CLI는 `hive setup --scope user --describe --output json`으로 embedded schema,
  canonical answer example, localized question contract, built-in Skill catalog·digest를 읽기 전용 제공.
  Agent의 binary byte·npm tree 검색과 setup field·Skill ID 추측 금지
- Reconfigure: 부분 preference 변경 또는 전체 setup 재검토 선택 우선
- 모든 완료 질문 뒤 non-secret partial answer·next step 저장. webhook URL·raw prompt 저장 금지.
  작업 answer는 OS temp의 session별 단일 파일로 제한하고 success·failure·cancel에 cleanup
- Refresh 필요 상태: authenticated Hive-only install과 saved-answer user projection은 preview 뒤
  자동 apply·revalidate. 별도 review-only yes/no 질문 없음
- 명시 global setup 요청은 safe temp write·dry-run·conflict 없는 built-in apply를 포함. 별도 질문은
  conflict·third-party Skill·external install·secret access·destructive action처럼 권한이 달라질 때만 사용
- Internal path·digest·projection 용어: 기본 안내 제외, 요청 시 diagnostic 제공
- 한국어 대화: `Skill`, `Wiki`, host·product name, command, path, schema key, Skill ID는 exact
  term 유지. 일반 설명만 한국어화하며 `Skill → 기술` 같은 일반명사 직역 금지
- 한국어 setup 질문: canonical `setup-hive`의 exact sample·용어표와 source-to-projection
  regression으로 관리

### Source developer binary

- `scripts/dev-install.sh --sandbox`: source-local `product-dev` binary만 build
- `--global`: active `hive` executable만 backup 뒤 atomic replacement; canonical user data 무변경
- `--rollback`: developer binary digest가 아직 active target과 일치할 때만 saved executable 복구
- `product-dev` version output은 local developer build로 표기하며 npm public `product-test[.N]`
  release identity 미사용
- Local `product-dev` binary: internally reproducible prior manifest와 live managed byte 일치 시
  developer-only three-way base 허용. Public stable·test binary는 signed historical base 부재 시
  계속 fail-closed

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
- Enabled global threshold 기본값: remaining `20%`
- Project override: registered project별 선택값, global보다 낮은 값 거부
- Effective threshold: `max(global, project override)`; global `20%`, web `50%`, game `30%`의
  effective threshold: web `50%`, game `30%`
- Global guard disable: project override와 무관하게 guard 비활성화
- User-facing control: product `usage-guard` 하나. source guard는 같은 policy resolver를 쓰는
  internal enforcement adapter이며 별도 active source Skill 아님
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
- 현재 project의 workflow·기술 선택·delivery constraint·작업 우선순위: project scope 전용
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
