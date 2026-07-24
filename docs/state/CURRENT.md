# 현재 상태

- 기준 branch: `develop`
- product version: `0.7.0`
- plan revision: `1.32`
- 현재 milestone: Phase 7 public qualification `0.8.0`
- 현재 작업: direct installer 안내 교정, 전체 재검증, 최종 독립 review,
  `develop` push
- 외부 중지 경계: protected signing/publication credential, 실제 production publication,
  exact `1.0.0` 사용자 authority
- Plan load: compact `docs/plans/PLAN.md` + `docs/plans/phases/07-public-qualification.md`
  + `docs/plans/active/documentation-style.md` + `docs/plans/active/security-review.md`
- Plan completion: canonical checklist `110/119` 완료, `9`개 미완료, `92.4%`
- Native Goal routing: legacy `PLAN.md` checkbox 문구를 `phases/07-public-qualification.md`,
  `active/documentation-style.md`, `active/security-review.md`로 해석

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
- Session window 우선, session 부재 시에만 weekly fallback
- Allowed: exit `0`, halt marker 없음
- Limited·unknown: sanitized marker의 optimistic atomic publication, exit `3`
- Marker 내용: host scope, session digest, PID, decision, window, threshold, measured time,
  evidence digest, revision
- Raw account, raw session ID, CodexBar payload 저장·출력 없음
- Installed `primary_host`와 pinned run·capability host 불일치 차단; Non-Codex host는 sensor 호출 없이 `usage-unknown` fail-closed
- Fallback hook, prompt rewrite, Skill activation, watcher, subagent, orchestration,
  Stop continuation 설치·실행 없음
- OMX/OMC cancellation 결과: 보조 evidence only; halt marker나 durable goal/task 상태
  대체 불가

### Local qualification evidence

- `hive-cli`: 67/67
- `hive-render`: 39/39
- Phase 7 usage-control conformance: 16/16
- Phase 3 static·projection conformance: 45/45
- Source usage guard: 17/17
- 독립 shipping gate verifier: PASS, actionable finding 0건

남은 외부 범위:

- 실제 Codex·Claude·Gemini Antigravity session E2E
- Codex 외 qualified subscription usage sensor
- Public signed multi-platform release candidate

## Source 개발 usage safeguard

- Source-only `hive-usage-guard` Skill과 15초 CodexBar watcher
- 현재 session threshold: remaining `20%` inclusive
- Session window 우선, session 부재 시 weekly fallback
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

- SEC-001·SEC-003 staged snapshot: `hive-cli` 68/68, `hive-render` 44/44,
  `hive-update` 42/42
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

Fresh targeted PASS:

- `cargo fmt --all -- --check`
- `cargo test -p hive-cli --all-targets`: 68/68
- `cargo test -p hive-render --all-targets`: 44/44
- `cargo test -p hive-update --all-targets`: 42/42
- `cargo clippy -p hive-cli -p hive-render -p hive-update --all-targets -- -D warnings`
- Phase 3 static·projection: 45/45
- Phase 6 update: 8/8
- Phase 7 usage control: 16/16
- Source usage guard·human style regression: 35/35
- `scripts/check-release-version.sh 0.7.0`

진행 중:

- Direct installer publication asset 안내 교정
- Workspace build·strict Clippy·full Rust test
- Full Python conformance·Copier/hostile scaffold validation
- Workflow YAML·shell·secret·diff 검증
- 독립 code review·architecture review·completion verification
- Commit·push 뒤 exact SHA의 clean-clone remote CI

## 남은 external production gate

- macOS arm64/x86_64 Developer ID signing·notarization evidence
- Windows x86_64 Azure Artifact Signing evidence
- 실제 세 host base workflow와 capability matrix
- Protected GitHub attestation·TUF authorization·public release publication
- `0.8.x` signed release candidate qualification
- Exact `1.0.0` 사용자 지시 전 stable major preparation 금지

## 다음 action

1. Direct installer publication asset 안내 교정
2. 전체 local qualification 실행
3. 독립 최종 review finding 교정
4. PLAN·CURRENT 최종 evidence 반영
5. `develop` commit·push
6. Exact pushed SHA의 GitHub Actions 확인
