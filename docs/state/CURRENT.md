# 현재 상태

- 기준 branch: `develop`
- product version: `0.8.0`
- plan revision: `1.81`
- 현재 milestone: Phase 7 qualification + global onboarding·shared index `0.8.0`
- 현재 작업: npm exact `0.8.0|latest` 배포 완료, `develop` → `main` 반영
- 외부 중지 경계: GitHub Release·Git tag, protected signing/publication credential,
  exact `1.0.0` 사용자 authority
- Plan load: compact `docs/plans/PLAN.md` + `docs/plans/phases/07-public-qualification.md`
  + `docs/plans/active/plugin-project-lifecycle.md`
  + `docs/plans/active/native-usage-sensor.md`
  + `docs/plans/active/user-onboarding-shared-index.md`
  + `docs/plans/active/source-docs-wiki.md`
  + `docs/plans/active/windows-shell-install.md`
  + `docs/plans/active/documentation-style.md` + `docs/plans/active/security-review.md`
  + `docs/plans/active/docs-wiki-migration.md`
  + `docs/plans/active/release-0.8.0.md`
- Plan completion: canonical checklist `228/228` 완료, 미완료 `0`, `100%`
- Latest local Windows: Rust workspace 458개 실행·통과. Python 적합성 618개 발견 중
  576개 실제 실행·통과, 42개 미실행. 미실행 범위: 관리자 권한 없는 Windows의
  symbolic link 생성 제약 16개, POSIX·Unix 전용 동작 19개, macOS 전용 설치·서명
  동작 7개. 운영체제 판별: Windows. 미실행 42개: 이 컴퓨터에서 검증 완료로
  판단할 근거 불충분. PowerShell 5.1·7.6.4 installer와 `cmd.exe`
  bootstrap 계약은 이 Windows 컴퓨터에서 실제 실행·통과
- Latest native remote: exact `420e244`의 candidate run `30657669889`,
  macOS·Linux·Windows 5/5와 npm umbrella PASS
- Latest npm publication: run `30658188721`, exact `0.8.0` 여섯 package,
  `latest=0.8.0`, 기존 `test=0.8.0-test.1`, provenance PASS
- Actual Windows public install: npm·CMD clean install, repeat, pending receipt recovery,
  product·receipt `0.8.0`, npm·direct SHA-256
  `330f4e0c8da5b6347400b9b16a9f76b2fb4f94406a2eacfe8c641367ca344ef9`
- Native Goal routing: legacy `PLAN.md` checkbox 문구를 `phases/07-public-qualification.md`,
  `active/plugin-project-lifecycle.md`, `active/native-usage-sensor.md`,
  `active/user-onboarding-shared-index.md`, `active/source-docs-wiki.md`,
  `active/windows-shell-install.md`,
  `active/documentation-style.md`, `active/security-review.md`,
  `active/docs-wiki-migration.md`,
  `active/release-0.8.0.md`로 해석

## `docs/` Wiki 전환

- 결정: [`ADR-0014`](../decisions/ADR-0014-docs-wiki-architecture.md)
- Active fragment:
  [`docs-wiki-migration.md`](../plans/active/docs-wiki-migration.md)
- 유지: 간결한 English·Korean README와 빈 QA Contributors 표
- 복원 source: 간소화 직전 Git `README.md`
- 목표 구조: `docs/00-home.md`, `docs/01-index.md`, topic MOC,
  `docs/facts/{en,ko}` atomic pair
- 제거 완료: standalone source-Wiki directory를 tracked tree에서 제거
- 보존 원칙: valid knowledge 이동 우선, deprecated·incorrect·superseded knowledge만 제거
- AI directive: 간소화 전 durable claim inventory, replacement locator와 docs home·index
  도달성 확인, Git history recoverability 적용
- Human Wiki: `docs/00-home.md`, `docs/01-index.md`, topic MOC, product overview,
  development guide에서 간소화 직전 README knowledge 복원
- Atomic knowledge: `docs/facts/en`·`ko` 32개 exact pair, primary fact 1개,
  cross-link와 source digest

## Windows shell 설치 경계

- Consumer runtime: `hive.exe`·installed harness의 PowerShell dependency 없음
- Consumer direct install: Windows 기본 `powershell.exe` 5.1 지원
- Consumer `cmd.exe`: exact-version PowerShell 5.1 bootstrap 호출 명령 지원
- Consumer PowerShell 7: dependency·탐지 경고·설치 prompt 없음
- Source Windows: PowerShell 7.6.4 LTS 개발·release dependency
- Source dependency setup: exact command·package·scope preview, 명시적 동의,
  Microsoft 지원 installer 위임, Hive update·uninstall 없음
- Current host evidence: Windows PowerShell `5.1.26100.8875`, PowerShell `7.6.4`,
  Rust·Cargo `1.97.1`, Copier `9.17.0`, pip `26.1.2`
- Current implementation: `[IO.File]::Replace` atomic overwrite, shell-independent
  UTF-8, `cmd.exe` bootstrap, source dependency preview·동의·재검증
- Local evidence: Rust workspace 전체 PASS, Phase 6 Windows contract 21개 실행,
  platform 비대상 8개 expected skip와 나머지 PASS
- Active fragment:
  [`windows-shell-install.md`](../plans/active/windows-shell-install.md)
- Decision:
  [`ADR-0013`](../decisions/ADR-0013-0.8-release-scope.md)

## Global onboarding·shared index audit

Target: `0.8.0`

| 요청 범위 | 현재 구현 |
| --- | --- |
| User install | 세 host minimal bootstrap 뒤 mandatory global setup |
| Mandatory global setup | `setup-hive`와 user-scope setup CLI |
| Language·user profile·persona·multi-host | 첫 질문 language, 이후 선택 언어의 signed catalog one-question sequence |
| Update 확인 | Explicit opt-in daily check, offline 뒤 다음 host session 재시도 |
| Skill suite 선택 | Recommended 또는 individual, dependency closure preview |
| Wiki opt-out | Default-on, 언제든 disable/enable, Markdown 보존 |
| Usage guard 선택 | Explicit opt-in, enabled 기본 `20%`, fallback 별도 consent |
| User marker | `AIGENT-HIVE:USER:START|END` append·owned replace |
| User `.agents` | Provider-neutral directive·selected Skill projection |
| Root knowledge | `~/.hive/knowledge` + disposable root SQLite |
| Project setup mode | `expedited|custom`, 양쪽 모두 project kind 필수 |
| Project type | Required project identity·domain profile |
| Project index | User-root 단일 SQLite, project DB 생성 없음 |
| Initial global expedited | Language와 update-check consent 뒤 나머지 default 적용 |
| Project auto onboarding | Global 상속·canonical evidence·unresolved-only 질문 |
| Wiki task-fact capture | Wiki enabled material-task 종료 시 검토된 결과·도구·기준·요청 요약 자동 기록 |

완료 evidence:

- Global setup state·schema·catalog·selected projection
- Wiki disable의 Skill·operation 차단과 canonical Markdown 보존
- Usage consent의 runtime sensor·fallback 제어
- Project activation + root registry/index 연결 rollback
- Connected `0.7.0 → 0.8.0` preference 보존, unconnected setup-required fail-closed,
  legacy project DB cleanup과 거부 시 전체 install tree 무변경
- Codex·Antigravity expedited/custom connected matrix 4/4
- Initial expedited fixed defaults와 `auto-setup-harness` zero-question inference 구현
- Initial setup의 첫 질문 `English|한국어`, 이후 setup Skill 질문·preview와
  user directive·host guidance의 선택 언어 적용
- Initial setup update-check consent, 성공 확인 뒤 24시간 throttle, offline·malformed
  결과 무기록과 다음 host session retry, check-only no-install 구현
- Wiki disable 시 0건, enable 시 agent-reviewed bounded task-fact completion capture
- Auto Skill canonical·plugin·source·Codex·Claude projection parity
- 실제 Windows 11 x86_64 Codex user install·validate, recommended global setup,
  zero-question project auto onboarding, user-root shared index 재빌드·lint PASS
- Same-version repeat update·recover 뒤 user install·project harness 재검증 PASS
- Skill validator PASS, `hive-cli` 223/223와 version integration PASS,
  `hive-render` 59/59, Wiki·static contract 65/65 PASS
- Signed `0.8.0` release activation은 Phase 7 외부 gate
- Decision:
  [`ADR-0012`](../decisions/ADR-0012-global-onboarding-shared-index.md)
- Active fragment:
  [`user-onboarding-shared-index.md`](../plans/active/user-onboarding-shared-index.md)

## Source docs Wiki

- Canonical path: `docs/facts/en/`, `docs/facts/ko/`
- 금지 path: `omx_wiki/`, `.omx/wiki/`, source root의 consumer `.hive/knowledge/`
- Current OMX/OMC: replaceable compatibility dependency와 orchestration aid
- 장기 방향: host-native·provider-neutral capability 대체 뒤 OMX/OMC 제거
- Consumer reuse: `hive-wiki` core와 capture·maintenance·query 안전 계약
- Skill reuse: shared canonical `harness/skills/`, exact source `.agents/skills/` projection
- 현재 상태: 영어 37개·한국어 37개 atomic fact, exact pair 37개와 source-confined
  CLI·Skill·material-task completion capture 구현 완료
- Logical digest:
  `sha256:e9b5f4efc2ab464db3a07d4456004ad6d26f4bccdb458efc8ad8f8409a05d161`
- 검증: lint finding·warning 0건, 영어·한국어 query PASS, index 삭제 뒤 query
  fail-closed exit `5`, rebuild equivalence PASS
- SQLite binary digest는 invocation-local evidence이며 정본·clean-copy equivalence 기준이
  아님. Logical digest와 query 결과가 rebuild equivalence 기준
- Marketing deck 재개 record:
  [`aigent-hive-marketing-deck.md`](artifacts/aigent-hive-marketing-deck.md)
- LumaDeck 사용·생성 기준·초기 요청 요약:
  [`marketing-deck-record.md`](../facts/ko/marketing-deck-record.md)
- Current Wiki tests: `hive-wiki` 33/33, Source Wiki conformance 재검증 PASS
- OMX Wiki Skill 제외 이유·향후 OMX/OMC retirement 시 knowledge migration 0건:
  [`ADR-0011`](../decisions/ADR-0011-source-wiki-independence.md)
- Active fragment:
  [`source-docs-wiki.md`](../plans/active/source-docs-wiki.md)

## 세 host native usage sensor

- Codex: `codex-cli 0.145.0` app-server native primary와 process identity·bounded JSONL
  adapter 구현
- Claude Code: host-owned `/statusline` opt-in용 sanitized 5-hour·7-day capture 구현,
  `~/.claude/settings.json` mutation 0회, 실제 Pro/Max qualification 잔여
- Antigravity CLI `1.1.7`: native machine sensor `unsupported`, qualified CodexBar
  fallback 구현
- 실제 Antigravity fallback: CodexBar `0.45.2`, `default`·
  `antigravity-claude-gpt` provider-defined pool, threshold `10%`, selected window
  `multiple`, exit `0`, raw payload persistence 0건
- 세 provider 공통 CodexBar fallback-only, native limited 뒤 fallback 우회 0회
- CodexBar 미설치 notification·fixed command preview·explicit current-action consent 구현
- Deferred: 실제 Claude Pro/Max parity와 future Antigravity native fixture
- Active implementation fragment:
  [`native-usage-sensor.md`](../plans/active/native-usage-sensor.md)
- Decision:
  [`ADR-0010`](../decisions/ADR-0010-native-first-usage-sensors.md)

## 구현 완료 범위

| Phase | 완료 범위 |
| --- | --- |
| 1 | 결정적 setup, staging, ownership, conflict·rollback, host projection |
| 2 | canonical Markdown knowledge, disposable SQLite index, rebuild·suppression |
| 3 | portable Skill routing, simple-question isolation, prompt refinement, OMX/OMC precedence |
| 4 | persistent role, durable run, fresh-session recovery, owner continuity |
| 5 | subscription usage policy, one-shot dispatch authorization, authenticated judge quorum |
| 6 | verifier-only signed release, update·migration·backup·crash recovery, installer ownership |
| 7 local | shipping one-shot usage gate, 세 host projection, provenance verifier, fault injection |

제품 경계:

- Provider API·SDK·credential path 없음
- Model runtime, scheduler, plan/Ralph/team/persistent-loop clone 없음
- Source workspace, release bundle, installed consumer harness의 물리·논리 분리
- Release private key 생성·읽기·저장·signing 없음
- Canonical state: tracked Markdown·YAML·TOML
- SQLite: 삭제·재생성 가능한 local index
- OMX/OMC namespace와 host-global configuration의 Hive 소유권 없음

## User plugin·project lifecycle review

현재 구현:

- Codex·Claude Code·Antigravity native plugin package와 user-scope install/update.
  Antigravity는 Hive-owned source package와 `agy`-owned staging·registry를 분리
- User guidance marker append·own-block replace·foreign byte 보존
- Project `.agents/directives`, portable `.agents/skills`, Claude `.claude/skills` adapter
- `setup-harness` expedited/custom sequence, project canonical knowledge와
  user-root 단일 SQLite·explicit promotion
- Historical exact base, unmodified replace, modified local-priority three-way merge
- Durable journal, executable-mode backup, host-state compensation과 crash recovery
- Source `hive-prompt-refine` projection, explicit refine-only routing,
  모호성·핵심 세부 부족 prompt의 optional refine 제안

`0.8.0` npm 배포 gap:

- Interactive owner-aware `hive update`
- Linux x86_64·arm64 musl native build·install·runtime qualification
- `aigent-hive@0.8.0|latest` package family와 registry publication
- Unix·PowerShell·CMD npm-backed 직접 installer
- 5개 target SHA-256·GitHub artifact attestation·npm binary identity
- Exact product·npm `0.8.0` candidate qualification과 `latest` publication

Pre-1.0 비차단 deferred:

- 실제 Claude Code install/update E2E
- 실제 Claude Pro/Max quota usage parity
- macOS·Windows signing·notarization과 external TUF production authorization

실제 current-host evidence:

- Hive CLI: `~/.local/bin/hive`, `hive --version|-v|-V` 모두
  `hive 0.7.0 (released 2026-07-24)` 출력
- 기존 signed user harness에 현재 미서명 source를 덮어쓰는 setup preview는 ownership
  manifest 불일치로 exit `5`; 안전 경계 우회 0회
- Codex `0.145.0`: install→validate→update→validate PASS,
  `aigent-hive@aigent-hive` `0.7.0` enabled와 exact local source 확인
- Codex fresh ephemeral session: detail-poor ordinary request의 optional refine 제안,
  automatic rewrite 0회, safe read-only discovery 후 empty-workspace 중단
- Antigravity `agy 1.1.7`: support range `>=1.1.7 <1.2.0`, 기존
  directory-scan `0.7.0` migration dry-run→install→validate→repeat update→validate PASS
- Antigravity native discovery: `agy plugin list` import 등록 PASS, Hive source와
  host staging 16/16 exact path·byte parity, full-tree validation PASS,
  host staging의 Hive ledger ownership 0건
- Claude Code: executable·authenticated Pro/Max session 부재

결정:

- [`ADR-0009`](../decisions/ADR-0009-user-plugin-project-knowledge-boundary.md)
- Active implementation fragment:
  [`plugin-project-lifecycle.md`](../plans/active/plugin-project-lifecycle.md)

계획 evidence:

- Active fragment 8 KiB 제한 충족
- Canonical checklist `223/228`, active checklist ID 중복 0건
- Root English 159줄·Korean 155줄 README, 상호 language link와 빈 QA 표 PASS
- Plan static contract 29/29, documentation style regression 18/18 PASS
- Human documentation inventory 1,285/1,285 review, finding 0건
- Markdown link conformance PASS

## Phase 7 shipping usage gate

### 구현

- Built-in `hive-usage-guard` source, template mirror, 세 host projection과 active Skill
  ledger
- Typed CLI: `hive usage enforce|status|threshold|session`
- 새 automatic dispatch 직전 one-shot `enforce`; 일반 응답·manual·non-dispatch 호출 없음
- Exit `0`은 session-bound preflight-only; 별도 automatic resume의
  `enforced=true`, `outcome=authorized`, authorization ID 1개·brief 1개만 dispatch 허용
- Current halt 우선, exit `3`은 해당 dispatch 차단, session disable은 authorization 아님
- Host-scoped session digest:
  `SHA-256(primary_host || NUL || exact_session_id)`
- Current process ID 결합, 다른 host·session·process의 override·marker replay 거부
- Explicit current-session disable 확인 필수; enable·toggle과 새 session default-enable
- Current valid halt marker를 sensor보다 먼저 확인하고 반복 호출에서 sensor 재사용 금지
- Account digest 생략 시 qualified sensor의 unique account만 허용; 0개·복수 fail-closed
- Quota pool별 provider-defined window 단독 적용; cadence window는 session 우선,
  session 부재 시 weekly fallback; 모든 pool 통과 필수
- Allowed: exit `0`, halt marker 없음
- Limited·unknown: sanitized marker의 optimistic atomic publication, exit `3`
- Marker 내용: host scope, session digest, PID, decision, window, threshold, measured time,
  evidence digest, revision
- Raw account, raw session ID, CodexBar payload 저장·출력 없음
- Installed `primary_host`와 pinned run·capability host 불일치 차단; Codex app-server,
  Claude opt-in status-line capture, Antigravity truthful native unsupported를 구분하고
  allowlisted unavailable·unsupported·malformed에서만 CodexBar fallback
- Fallback hook, prompt rewrite, Skill activation, watcher, subagent, orchestration,
  Stop continuation 설치·실행 없음
- OMX/OMC cancellation 결과: 보조 evidence only; halt marker나 durable goal/task 상태
  대체 불가

### Local qualification evidence

- Rust workspace 390/390:
  `hive-cli` 185, `hive-core` 62, `hive-projection` 22, `hive-render` 51,
  `hive-update` 63, `hive-wiki` 7
- Python conformance 524개 실행, 523 PASS, Windows `pwsh` 전용 1개 expected skip
- Native hostile sensor·fallback, Phase 3 projection과 source usage guard regression PASS
- 독립 Antigravity code·test review: PASS, actionable finding 0건

Pre-1.0 비차단 deferred:

- 실제 Claude app session E2E
- 실제 Claude Pro/Max qualified subscription usage sensor
- macOS·Windows platform signing과 external TUF production authorization

## Source 개발 usage safeguard

- Source-only `hive-usage-guard` Skill과 15초 native Codex app-server primary·CodexBar
  fallback-only watcher
- 현재 session threshold: remaining `30%` inclusive
- Session window 우선, session 부재 시 weekly fallback
- Quota sensor unknown: 3초 뒤 1회 재시도, 반복 unknown은 observation 보존과
  `transient_unknown_ignored` 진행, confirmed-limited marker 유지
- 매 user turn과 tool·mutation·delegation·external write·push·final-answer 경계의 fresh
  `gate`
- Explicit current-session disable만 우회 허용; bare `continue`·`resume` 우회 해석 금지
- New session default-enable, raw account·session identifier 저장 없음
- Watcher의 Codex App process kill·signal과 `.omx/` 수정 금지

Source guard는 개발 workspace 전용. Shipping 제품은 watcher 없이 one-shot
`hive usage enforce` 사용.

## 사람용 문서 style

- Source directive: `.agents/directives/08-human-documentation-style.md`
- Consumer projection: `harness/template/AGENTS.md.jinja`, compiled renderer,
  `docs/guidance-schema.md`
- 대화 언어: 선택 언어로 질문·응답 전체 통일
- 한국어 대화: 고유명사·제품명·패키지명·명령어·코드 식별자·경로·스키마 키·정확한
  화면 문구·뚜렷한 한국어 대체어가 없는 용어만 영어 유지
- 영어 대화: 정확한 한국어 이름·문자열·인용문·사용자 보존 요청을 제외하고 영어로 통일
- 소비자 전역 지침: 한국어 선택 시 대체 가능한 일반 영어 단어의 한영 혼용 금지
- 한국어 설명문: 짧은 heading·bullet·table·checklist와 의미 중심 명사구 우선
- Declarative·conversational sentence-form과 기계적 nominalization 금지
- Exact bad/good 21쌍, authored callout·blockquote 적용, 비제한 규칙 명시
- Conversational imperative prompt sample은 path·line·reason·line digest allowlist로만 보존
- Exact external quote·UI prompt·protocol·fixture만 path·line·reason·line digest 예외
- Checker: `scripts/check-human-documentation-style.py`
- Regression: `tests/conformance/test_human_documentation_style.py`
- Independent semantic review PASS, residual finding 0건
- 최종 completion 조건: fresh inventory 전수 review, finding 0건, stale exception 0건,
  source/template/generated parity

## Phase 6 release·update truth

### Verifier-only trust

- Agent-write-denied public root에서 시작하는 TUF-compatible Ed25519 검증
- Offline root exact 2-of-3, role별 unique key material, duplicate·unassigned signature 거부
- Root rotation의 old+new threshold, expiry, version, rollback, target length·SHA-256 검증
- in-toto/SLSA source·builder·subject와 platform signing evidence semantic 검증
- Production publication에서 exact archive subject·target·candidate/source commit·Sigstore
  bundle 결합
- Product signing/private-key/downloader/provider-network API 없음

### Version·migration

- Compiled historical surface와 signed cumulative inventory의 독립 release classification
- Feature: exact next minor; compatible fix: exact next patch
- Same-major breaking change: major `0`에서도 거부
- Major target 자동 추론 없음; exact user target과 별도 confirmation 필수
- Signed metadata가 선택 가능한 compiled route:
  `same-major-render-v1|cross-major-system-representation-v1`
- Downloaded script·DLL·dylib·WASM·argv migration 실행 금지
- Supported `0.1.0`–`0.6.0` generation의 same-major dry-run·apply corpus
- Cross-major protected project/docs/preferences/Markdown과 foreign marker byte 보존

### Backup·activation·recovery

- Verification·classification·route selection·dry-run 전 target mutation 0건
- Changed owned path와 canonical config/team/run/knowledge의 self-digested backup
- SQLite/WAL/SHM/journal, runtime, backup, `.omx/.omc` 제외
- Durable journal과 exact dry-run plan/tree 기반 atomic activation
- Before/after digest에서만 rollback 또는 forward completion
- Concurrent third digest 보존과 conflict
- Canonical text에서 SQLite rebuild
- Exact 7일 초과 unreferenced backup만 재검증 후 정리

### Local evidence

- SEC-001·SEC-003 current regression: `hive-cli` 166/166,
  `hive-render` 51/51, `hive-update` 63/63
- SEC-001·SEC-003 strict Clippy와 독립 재review: PASS
- Phase 6 static·CLI conformance: 8/8
- Phase 4 run lifecycle: Rust 10/10, Python 29/29
- Upgrade/migration fault injection: activation failure, concurrent user edit, forged recovery,
  cross-major preservation PASS

## Version parity

다음 표면의 `0.7.0` 동기화:

- Root Cargo workspace와 Cargo.lock의 Hive packages
- Compiled `hive --version`
- Release manifest, migration table과 signed surface fixture
- Copier/Rust installed `.hive/config/harness.toml`
- Harness release metadata, README, PLAN, CURRENT와 version lifecycle ADR

`0.6.0 → 0.7.0`: signed release/update, safe migration·backup·recovery와 release
packaging을 추가한 backward-compatible feature minor. Major 변경·추론 없음.

## 현재 검증 상태

Global onboarding·shared index local qualification PASS:

- Strict Clippy all targets·all features와 format check PASS
- Rust workspace 477/477
- Python conformance 576개 발견 중 575개 실행·통과. Windows가 아닌 환경에서만
  가능한 `pwsh` 전용 검사 1개는 현재 Windows 환경에서 미실행
- Shared index 동일 입력 재실행 byte-exact no-op, `changed_paths=[]`
- Codex·Antigravity expedited/custom connected onboarding matrix 4/4
- 독립 final blocker review의 critical·high·medium·low finding 0건

Source docs Wiki targeted qualification PASS:

- `hive-wiki` 33/33, Source Wiki conformance 재검증 PASS
- Canonical fact 74개, bilingual pair 37개
- `lint` finding·warning 0건, 영어·한국어 query PASS
- Index 삭제 뒤 query fail-closed exit `5`, logical digest·query equivalence rebuild PASS
- Ignored index·persistent lock의 Git 추적 0건
- 당시 Full Python conformance 565개 발견 중 528개 실행·통과, 37개 미실행.
  미실행 범위는 현재 Windows에서 권한 없이 만들 수 없는 symbolic link와
  POSIX·macOS 전용 동작. 해당 동작의 Windows 검증 완료 근거로 사용 금지

### macOS Apple Silicon local release qualification historical CLEAR

- Host: Apple M2, macOS 26.5.2, native `arm64`
- Tested source: `ba798d8`
- 상태: historical evidence. Current candidate
  `28f1c366aa06a609b443724decc474cb7718ea8a` 재검증 필요
- Locked `aarch64-apple-darwin` release build·version·Mach-O architecture PASS
- Release strict Clippy·workspace strict Clippy·format PASS
- Rust workspace 236/236
- Deterministic release archive 2회 byte-identical
- Binary SHA-256:
  `914b684da0c28da1914121ffc43a7331828a11ef13ef7b1159adc05fe445eda3`
- Archive SHA-256:
  `bde2c886c6d475b4a1a564ba0df33eaa9b6fb4a1b49ca49a7f2a896aa586a54b`
- Actual archive direct-install fixture, ownership receipt, repeat install PASS
- Installed binary setup dry-run·apply·validate PASS
- Phase 6 release/update 15 PASS, Windows-only `pwsh` 1 skip
- Phase 1 setup 31/31
- Protected 경계: Developer ID signing·notarization·GitHub attestation 미실행
- Local signature observation: linker ad-hoc, `TeamIdentifier` 없음, Gatekeeper 거부

Current remote qualification evidence:

- Current native source:
  `baff938b99967b4830eee79daa6c4477a607f427`
- Native release runtime:
  [run `30581894132`](https://github.com/gvm1229/aigent-hive/actions/runs/30581894132),
  macOS arm64·Intel, Linux musl x86_64·arm64, Windows x86_64 5/5 job PASS
- Linux 두 target: locked release build, ELF architecture·static linkage, package layout,
  archive digest·실행, isolated Antigravity install lifecycle PASS
- P7-040 current clean-clone gate 충족
- P7-043 Linux x86_64·arm64 musl qualification 충족

검증 경계:

- Local Phase 6 계약: Windows 적용 대상 21개 실행·통과. macOS 전용 8개는 현재
  Windows 환경에서 실행 불가하여 미실행이며 macOS 동작은 이 결과로 미검증
- Direct installer의 같은 owner parent handle-pinning race

## `0.8.0` npm 배포 완료

- P7-044 public npm package family와 native smoke 완료
- P7-045 npm-backed Unix·PowerShell·CMD installer와 digest·owner receipt 검증 완료
- P7-049 설치 소유자 기반 대화형 `hive update` 완료
- P7-020 5개 target archive·npm tarball provenance 완료
- P7-018 exact `0.8.0` release candidate qualification 완료
- P7-037 GitHub Release 없이 npm `0.8.0|latest` publication·clean install 완료

배포 증거:

- Current source·npm product version `0.8.0`
- 사용자 지정 순서: npm `0.8.0` 배포 성공 뒤 `develop` → `main` 병합
- Candidate authority: PR·필수 상태 검사·삭제·강제 push 차단이 적용된 exact `develop`
- Current GitHub ruleset: `develop` 보호 활성, 우회 권한 없음
- Current publication environment: `release-publication` 필수 검토자 `gvm1229` 설정
  확인, 자기 배포 승인 차단 비활성
- Branch policy: `codex/release-0.8.0` 임시 branch 생성·push와 `develop` 대상 PR
  사용자 예외 승인 완료. `develop` 직접 push는 보호 규칙상 불가
- First candidate run `30633581092`: exact `develop` commit `1031ff0`, 5개 native
  target·6개 npm tarball PASS
- First publication run `30634201469`: `release-publication` 승인 PASS,
  `docs/releases/0.8.0.md` 누락으로 npm 게시 전 실패, npm publish 실행 0건
- Corrective branch: 제품 후보 출시 안내, 명확한 게시 선행 검사 오류, Codex process
  교체 뒤 source watcher 복구와 회귀 시험
- `release.yml`: protected exact `420e244`, run `30657669889` PASS
- `release-publish.yml`: GitHub Release 0건과 6개 package `latest` publication 계약으로
  전환. 성공한 정확한 `develop` 후보만 허용. 최초 등록은 명시적
  `bootstrap_with_token=true`·임시 `NPM_TOKEN`, 이후는 OIDC 전용
- Direct installer: exact `0.8.0` unpkg bootstrap, scoped npm tarball digest,
  native 제품·receipt `0.8.0`, polluted `PSModulePath` CMD 회귀 PASS
- GitHub repository environment `release-publication` 1개, 최초 등록용
  `NPM_TOKEN` secret 1개
- Local npm: Node.js `24.13.1`, npm `11.17.0`, registry 사용자 `gvm1229`
- Public registry: umbrella·5개 scoped package exact·latest `0.8.0`, test
  `0.8.0-test.1`
- Local Windows npm baseline: actual `0.8.0` binary의 platform·umbrella pack,
  isolated global install과 digest
  `a8bdb5d7dd42965ec6f4d2f1f334a4ee4184a7f659f09cb92caf794b96524b0d`
  byte identity PASS
- Interactive activation: npm `test` 확인, npm·direct owner 인증, 선택 언어 prompt,
  명시적 수락 뒤 exact adapter 실행·owner와 package version 재검증 PASS

Pre-1.0 비차단 deferred:

- macOS arm64/x86_64 Developer ID signing·notarization
- Windows Authenticode 또는 Azure Artifact Signing
- External TUF production authorization
- 실제 Claude subscription-backed install·usage parity
- GitHub normal release와 Git tag
- Exact `1.0.0` 사용자 지시 전 stable major preparation 금지

## 다음 action

1. npm 배포 성공 commit의 `develop` → `main` PR 병합
2. Trusted Publisher 설정 확인 뒤 임시 `NPM_TOKEN` 삭제
