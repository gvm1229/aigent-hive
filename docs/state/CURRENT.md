# 현재 상태

- 기준 branch: `develop`
- product version: `0.6.0`
- plan revision: `1.13`
- 현재 milestone: Phase 5 usage guard와 authenticated Ed25519 judge quorum 완료
- 다음 milestone: Phase 6 update·migration·release `0.7.0`

## 현재 truth

Phase 1 결정적 setup renderer, Phase 2 canonical Markdown knowledge·disposable
SQLite index, Phase 3 portable Skills·host projection과 Phase 4 persistent
role·durable run recovery와 Phase 5 usage-guarded automatic
resume와 clean-context quorum에 external protected trust root 기반 detached Ed25519
authentication을 구현했다. 전체 workspace, Python conformance, schema/Copier와
isolated adversarial verification을 통과해 `0.6.0` 완료 범위로 확정했다.

Hive는 model-provider API, provider SDK, provider API key, model runtime, scheduler,
plan/Ralph/team clone을 소유하지 않는다. Model call, subagent spawn, retry,
continuation과 independent judge invocation은 새 run에서 자동 resolve한 host-native,
OMX 또는 OMC owner가 소유한다.

## Phase 5 완료 구현 snapshot

### CodexBar qualification과 window precedence

- Optional CodexBar `0.45.2` executable을 한 번 resolve하고 executable identity를
  version/usage read 전후에 고정한다.
- Version은 exact `0.45.2`만 허용한다. Usage command는 shell 없이 fixed argv로
  bounded 실행하며 stdout/stderr 각각 1 MiB, version 5초, usage 1분 제한을 둔다.
- Account는 raw identity를 출력하지 않고 caller-supplied SHA-256 digest로 선택한다.
  Missing, mismatch, duplicate account, provider/source/identity mismatch와 row error는
  `usage_unknown`이다.
- Snapshot은 `usage.updatedAt` 기준 60초 TTL을 사용한다. Stale 또는 60초를 넘는
  future sample은 `usage_unknown`이다.
- 300분 session window가 있으면 weekly가 더 낮거나 malformed 또는 duplicate여도
  session만 선택한다. Session이 없을 때만 exact 단일 10080분 weekly window를
  fallback으로 사용한다.
- Automatic resume는 installed `.hive/config/harness.toml`의 root
  `usage_stop_remaining_percent`를 권위값으로 사용한다. Missing, malformed,
  duplicate 또는 symlink config는 fail-closed하고 optional CLI threshold가 설치값과
  다르면 거부한다.
- 선택된 window의 `remaining <= installed threshold`는 inclusive block이며 default
  installed threshold는 `10%`다.

### Run-resume one-shot dispatch authorization

`hive run resume`은 durable PLAN/STATUS, immutable owner continuity, role, shared
handoff와 evidence를 먼저 검증한다.

- 기본값과 `--dispatch-intent manual`은 CodexBar를 읽지 않고 기존 prepare-only
  recovery를 유지하며 runtime history/authorization을 읽거나 쓰지 않는다.
  `usage_guard.enforced=false`, `outcome=not_requested`로 표시한다.
- `--dispatch-intent automatic`은 `--account-digest sha256:<64-lowercase-hex>`를
  필수로 요구하고 selected active `--role` 하나에 대해서만 평가한다.
- Prior selected snapshot은 Git에서 제외된
  `.hive/runtime/usage-history/<account-key>.json`에 bounded·integrity-bound
  record로만 저장한다. 이후 measurement/reset timestamp 역행과 같은 reset의
  remaining 증가는 `usage_unknown`이며 malformed, tampered 또는 symlink history도
  fail-closed한다.
- Fresh snapshot이 policy를 통과하면 one-shot permit을 dispatch brief 준비 closure
  직전에 현재 시각으로 소비한다. Exact run revision·role·brief digest로 결정되는
  authorization ID 하나와 sanitized snapshot digest, selected window를 포함한 brief
  하나만 반환하고 issued claim을 Git에서 제외된
  `.hive/runtime/dispatch-authorizations/`에 기록한다.
- 같은 binding의 재요청은 sensor를 다시 읽지 않고 `already_issued`, exit `3`,
  `dispatch_briefs: []`로 끝난다. Limited, unknown 또는 expired permit도 exit `3`,
  brief 0개와 recovery data만 반환한다.
- Hive는 같은 authorization의 재발급은 거부하지만 이미 capture된 JSON의 Hive 밖
  replay를 막지 못한다. 실제 host/orchestration owner가 authorization ID를 dispatch
  boundary에서 한 번만 소비해야 한다.
- Manual은 project write 0건이고 automatic의 write scope는 위 두 Hive-owned ignored
  runtime directory로 제한된다. 모두 `spawned:false`이며 Hive는 model, subagent 또는
  resolved owner process를 실행하지 않는다.

### Clean-context judge와 quorum

- `hive judge package`는 goal, acceptance criteria, exact artifact/evidence digest
  reference와 known constraint만 포함한 read-only package를 만든다.
- Package는 `package_digest`를 제외한 모든 field의 RFC 8785 JCS bytes에 대한
  SHA-256으로 결합한다. Referenced target file은 no-follow bounded read와 exact
  digest 검증을 통과해야 한다.
- Unknown field, task-agent reasoning/chain-of-thought, self-score/self-praise, desired
  verdict, prior judge verdict와 허용 field 안에 숨긴 verdict-leading text를 거부한다.
- Verdict 전 `judge-assignment`는 exact subject/package/criteria, requester, task
  agent, resolved owner와 authenticated owner provenance, distinct
  slot/judge-instance/eligibility-evidence tuple, timestamp를 JCS digest로 고정한다.
  Requester와 task agent는 roster에 들어갈 수 없다.
- Final verdict는 assignment digest와 exact tuple을 참조하고 assignment 뒤 timestamp를
  가져야 한다. Duplicate, unknown, mismatched 또는 early tuple은 제외한다.
- Critical human approval은 모든 eligible verdict 뒤 별도 `judge-approval` artifact로
  고정하며 exact assignment/package/subject/criteria, approver, `APPROVE`, timestamp와
  JCS digest를 검증한다. Requester와 task agent는 approver가 될 수 없다.
- Normal은 명시 요청 시 독립 1명, elevated는 assigned 3명 중 2명 PASS, critical은
  assigned 3명 전원 PASS와 valid approval이 필요하다. Missing evidence 또는
  authenticated owner provenance는 PASS가 아니라 `INDETERMINATE`다.
- Package, assignment, verdict와 approval은 target 안의 target-relative file만 bounded
  no-follow read한다.
- Quorum result는 aggregate count/status만 반환하며 judge identity, individual
  verdict, slot, finding, digest와 statement를 다른 judge 또는 aggregate output에
  노출하지 않는다.
- `hive-judge-package`가 exact 10번째 implemented read-only built-in Skill로
  projection됐다. 이 Skill은 package만 준비하고 judge 실행, verdict 생성, quorum
  계산 또는 completion 승인을 소유하지 않는다.
- `judge-quorum-request` v2는 assignment, 각 verdict와 approval의 detached
  attestation을 요구한다. Attestation은 exact artifact JCS digest, trust-root ID,
  principal, key ID와 artifact kind를 domain-separated Ed25519 signature로 결합한다.
- Public key는 consumer target 밖의 agent-write-denied TOML trust root에서만 읽는다.
  Unix/macOS는 root-owned non-writable single-link file과 replace-safe ancestor,
  Windows는 reparse 거부와 current-token mutation right별 `ACCESS_DENIED`를 요구한다.
- Owner, judge와 human key purpose를 분리하고 trust root 전체의 duplicate public key,
  owner/judge/approver key 재사용, revoked/out-of-window key를 거부한다.
- Unsigned v1은 diagnostic compatibility만 제공하고 항상
  `authenticated:false`, `INDETERMINATE`로 끝난다.
- Hive는 strict verification만 수행하며 private key 생성·읽기·보관·signing은 외부
  authority가 소유한다. Signature가 보장하지 않는 판단의 진실성, human biometric
  presence와 전역 replay 한계는
  [`judge-trust-boundary.md`](../architecture/judge-trust-boundary.md)에 정의한다.

### Phase 5 authentication 결정과 현재 gate

Independent adversarial review에서 task agent가 다른 `task_agent_id`, 세 judge ID,
가짜 human ID와 모든 공개 JCS digest를 함께 만들어 structural `PASS`를 구성할 수
있음을 재현했다. 사용자는 선택 1인 user/admin-controlled public-key trust root와
Hive 밖의 private-key custody를 승인했다. `ADR-0007`로 이 결정을 고정하고
`test_self_authored_identity_forgery_cannot_authorize_critical_pass`의 expected-failure
marker를 제거했다. 현재 test는 attacker가 caller-writable trust root를 제출해도
command가 blocked되고 PASS가 나오지 않음을 확인한다.

Public-only known-answer vector와 signed critical 3/3+human chain은 core에서 정상
PASS를 검증한다. 로컬 user process는 의도적으로 protected trust root를 만들 권한이
없어 CLI success path를 우회하지 않는다. Native Windows ACL implementation과
remote CI는 milestone commit 뒤 platform matrix evidence로 별도 확인해야 한다.

## Skills와 external coexistence

Discoverable implemented built-in은 exact 10개다.

- `setup-harness`
- `hive-simple-question`
- `hive-prompt-refine`
- `hive-knowledge-capture`
- `hive-knowledge-query`
- `hive-knowledge-maintenance`
- `hive-role-handoff`
- `hive-run-checkpoint`
- `hive-run-resume`
- `hive-judge-package`

`hive-update`, `hive-migrate`는 catalog-only다.

Compatible OMX/OMC가 있어도 role/run/judge data Skill은 canonical Hive state와
검증 package를 기록·검증·복구하기 위해 projection할 수 있다. 이 Skills는 OMX/OMC
command, plan/Ralph/team/retry/persistent loop 또는 lifecycle hook을 실행하지 않는다.
Project와 fake HOME의 `.omx/.omc` foreign bytes는 성공·실패 모두 불변이다.

## Fresh verification

Phase 5 완료 gate에서 확인한 fresh evidence:

- `cargo fmt --all --check`: PASS
- `cargo build --locked --workspace --all-targets --all-features`: PASS
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`: PASS
- `cargo test --locked --workspace --all-targets --all-features`: 176/176 PASS
- `python3 -m unittest discover -s tests/conformance -p 'test_*.py'`: 399/399 PASS
- Copier `9.17.0` Codex·Claude·Antigravity default와 hostile render +
  `tests/validate_scaffold.py`: 4/4 PASS
- compiled `hive --version`: `0.6.0`
- isolated adversarial Ed25519 gate: PASS
- `git diff --check`: PASS

Remote native Windows trust-root execution과 live host/human signer identity는 local
evidence로 추론하지 않는다.

## Version parity

다음 tracked/compiled 표면은 마지막 완료 milestone `0.6.0`으로 동기화한다.

- root Cargo workspace와 Cargo.lock의 Hive package
- compiled `hive --version`
- Copier/Rust installed `.hive/config/harness.toml`
- README, PLAN, CURRENT와 version lifecycle ADR

`0.5.0 → 0.6.0`은 Phase 5 usage guard, automatic resume dispatch authorization과
authenticated judge quorum completion gate를 모두 충족한 backward-compatible
feature minor다. Major는 변경하거나 추론하지 않았다.

## 다음 action

`0.6.0` parity를 재검증하고 milestone commit을 `origin/develop`에 push한 뒤 Phase 6
update·migration·release를 시작한다.

## 남은 Phase 6–7

### Phase 6 — Update, migration와 release

- `hive-update`, same-major compatibility와 cross-major migration
- Backup/restore/retention, crash recovery와 atomic update activation
- Release bundle parity, GitHub Release packaging, signing과 install path

### Phase 7 — Public qualification

- macOS arm64/x86_64와 Windows x86_64 release qualification
- 세 live host base workflow와 host-native/OMX/OMC support matrix
- Migration fault injection, supply-chain provenance와 release candidate
- 사용자가 exact `1.0.0`을 지시하기 전 stable major prepare 금지
