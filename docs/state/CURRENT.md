# 현재 상태

- 기준 branch: `develop`
- product version: `0.7.0`
- plan revision: `1.49`
- 현재 milestone: Phase 7 qualification + source bilingual LLM Wiki `0.8.0`
- 현재 작업: `0.8.0` mandatory global onboarding·selected projection·shared index 계획;
  이후 Claude·signed publication qualification
- 외부 중지 경계: protected signing/publication credential, 실제 production publication,
  exact `1.0.0` 사용자 authority
- Plan load: compact `docs/plans/PLAN.md` + `docs/plans/phases/07-public-qualification.md`
  + `docs/plans/active/plugin-project-lifecycle.md`
  + `docs/plans/active/native-usage-sensor.md`
  + `docs/plans/active/user-onboarding-shared-index.md`
  + `docs/plans/active/source-llm-wiki.md`
  + `docs/plans/active/documentation-style.md` + `docs/plans/active/security-review.md`
- Plan completion: canonical checklist `184/208` 완료, `24`개 미완료, `88.5%`
- Native Goal routing: legacy `PLAN.md` checkbox 문구를 `phases/07-public-qualification.md`,
  `active/plugin-project-lifecycle.md`, `active/native-usage-sensor.md`,
  `active/user-onboarding-shared-index.md`, `active/source-llm-wiki.md`,
  `active/documentation-style.md`, `active/security-review.md`로 해석

## Global onboarding·shared index audit

Target: `0.8.0`

| 요청 범위 | `0.7.0` current truth |
| --- | --- |
| User install | 세 host direct operational install |
| Mandatory global setup | 없음 |
| Language·user profile·persona·multi-host | Global answer 없음 |
| Skill suite 선택 | User plugin built-in 전체 projection |
| Wiki opt-out | 없음; root/project Wiki seed 고정 |
| Usage guard 선택 | Project threshold 필수, setup 문서 기본 `10%` |
| User marker | `AIGENT-HIVE:USER:START|END` 구현 |
| User `.agents` | Generic user projection 없음 |
| Root knowledge | `~/.hive/knowledge` + disposable root SQLite 구현 |
| Project setup mode | Expedited/global inherit 없음 |
| Project type | Required project identity·domain profile 구현 |
| Project index | Project SQLite + root SQLite 독립 |

Accepted target:

- Minimal bootstrap 뒤 mandatory `setup-hive`
- Interface/Wiki language, user profile, persona, selected hosts·Skills
- Global Wiki default-on opt-out
- Usage guard opt-in, enabled 기본 threshold `20%`
- `expedited|custom` project setup, mode 무관 project kind 질문
- Project canonical Wiki + `~/.hive/index/hive.sqlite3` 단일 derived index
- Project SQLite 제거와 `0.7.0 → 0.8.0` non-destructive migration
- Decision:
  [`ADR-0012`](../decisions/ADR-0012-global-onboarding-shared-index.md)
- Active fragment:
  [`user-onboarding-shared-index.md`](../plans/active/user-onboarding-shared-index.md)

## Source bilingual LLM Wiki

- Canonical path: `llm-wiki/en/`, `llm-wiki/ko/`
- 금지 path: `omx_wiki/`, `.omx/wiki/`, source root의 consumer `.hive/knowledge/`
- Current OMX/OMC: replaceable compatibility dependency와 orchestration aid
- 장기 방향: host-native·provider-neutral capability 대체 뒤 OMX/OMC 제거
- Consumer reuse: `hive-wiki` core와 capture·maintenance·query 안전 계약
- Skill reuse: shared canonical `harness/skills/`, exact source `.agents/skills/` projection
- 현재 상태: 영어 11개·한국어 11개 page, exact pair 11개와 source-confined CLI·Skill 구현 완료
- Logical digest:
  `sha256:180247b1d5682d5b942385139f2fea2cd304a5ddb18b816b0c731902ab95af76`
- 검증: lint finding·warning 0건, 영어·한국어 query PASS, index 삭제 뒤 query
  fail-closed exit `5`, rebuild equivalence PASS
- SQLite binary digest는 invocation-local evidence이며 정본·clean-copy equivalence 기준이
  아님. Logical digest와 query 결과가 rebuild equivalence 기준
- Targeted tests: `hive-cli` 190/190, `hive-wiki` 27/27, Source Wiki Python 27/27
- OMX Wiki Skill 제외 이유·향후 OMX/OMC retirement 시 knowledge migration 0건:
  [`ADR-0011`](../decisions/ADR-0011-source-wiki-independence.md)
- Active fragment:
  [`source-llm-wiki.md`](../plans/active/source-llm-wiki.md)

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
- 잔여: 실제 Claude Pro/Max parity와 future Antigravity native fixture
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
- `setup-harness` 단일 질문 sequence, project/root canonical knowledge·독립 SQLite와
  explicit promotion
- Historical exact base, unmodified replace, modified local-priority three-way merge
- Durable journal, executable-mode backup, host-state compensation과 crash recovery
- Source `hive-prompt-refine` projection, explicit refine-only routing,
  모호성·핵심 세부 부족 prompt의 optional refine 제안

확인된 gap:

- Mandatory global onboarding·selected Skill·Wiki opt-out·usage opt-in
- Project expedited/custom mode와 user-root 단일 shared index
- 실제 Claude Code install/update E2E
- 실제 Claude Pro/Max quota usage parity
- macOS·Windows signing·notarization·publication

실제 current-host evidence:

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
- Canonical checklist `184/192`, active checklist ID 중복 0건
- Plan static contract 29/29, documentation style regression 18/18 PASS
- Human documentation inventory 1,698/1,698 review, finding 0건
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

남은 외부 범위:

- 실제 Claude app session E2E
- 실제 Claude Pro/Max qualified subscription usage sensor
- Public signed multi-platform release candidate

## Source 개발 usage safeguard

- Source-only `hive-usage-guard` Skill과 15초 native Codex app-server primary·CodexBar
  fallback-only watcher
- 현재 session threshold: remaining `20%` inclusive
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

Source Wiki targeted qualification PASS:

- `hive-cli` 190/190, `hive-wiki` 27/27, Source Wiki Python 27/27
- Rust workspace 415/415
- Canonical page 22개, bilingual pair 11개
- `lint` finding·warning 0건, 영어·한국어 query PASS
- Index 삭제 뒤 query fail-closed exit `5`, logical digest·query equivalence rebuild PASS
- Ignored index·persistent lock의 Git 추적 0건
- Full Python conformance 556개 실행, 555 PASS, Windows `pwsh` 전용 1개 expected skip

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

Remote qualification evidence:

- Qualified candidate commit: `28f1c366aa06a609b443724decc474cb7718ea8a`
- Native release runtime:
  [run `30205737619`](https://github.com/gvm1229/aigent-hive/actions/runs/30205737619),
  macOS arm64·Intel과 Windows x86_64 3/3 job PASS
- GitHub Actions CI:
  [run `30205737631`](https://github.com/gvm1229/aigent-hive/actions/runs/30205737631),
  7/7 job PASS
- P7-040: clean-clone full CI PASS

검증 경계:

- 로컬 Windows `pwsh` 부재에 따른 runtime test 1개 expected skip
- Direct installer의 같은 owner parent handle-pinning race

## 남은 external production gate

- macOS arm64/x86_64 Developer ID signing·notarization evidence
- Windows x86_64 Azure Artifact Signing evidence
- 실제 세 host base workflow와 capability matrix
- Protected GitHub attestation·TUF authorization·public release publication
- `0.8.x` signed release candidate qualification
- Exact `1.0.0` 사용자 지시 전 stable major preparation 금지

## 다음 action

1. UOS-001–004 global setup state·schema·catalog·CLI/Skill
2. UOS-005–010 user install·selected projection·Wiki·usage preference
3. UOS-011–015 project mode·shared index·migration
4. UOS-016 Codex·Antigravity local onboarding E2E
5. RPH-036·NUS-020·NUS-014·P7-013·P7-021 실제 Claude 포함 matrix
6. P7-018·020·037 signed release qualification
