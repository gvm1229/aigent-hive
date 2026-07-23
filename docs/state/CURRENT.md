# 현재 상태

- 기준 branch: `develop`
- product version: `0.4.0`
- plan revision: `1.9`
- 현재 milestone: Phase 3 완료
- 다음 milestone: Phase 4 role/run contract와 interoperability

## 현재 truth

Phase 1 결정적 setup renderer, Phase 2 canonical Markdown knowledge·disposable
SQLite index와 Phase 3 portable Skills·host projection이 구현됐다. Usage guard core와
optional CodexBar adapter는 Phase 5 선행 slice이며 실제 dispatch owner 연결과
live sensor qualification은 아직 완료되지 않았다.

Hive는 model-provider API, provider SDK, provider API key, model runtime, scheduler,
plan/Ralph/team clone을 소유하지 않는다.

## Phase 3 구현

### Skill catalog와 active routing

Discoverable built-in은 exact 6개다.

- `setup-harness`
- `hive-simple-question`
- `hive-prompt-refine`
- `hive-knowledge-capture`
- `hive-knowledge-query`
- `hive-knowledge-maintenance`

`hive-run-checkpoint`, `hive-run-resume`, `hive-role-handoff`,
`hive-judge-package`, `hive-update`, `hive-migrate`는 catalog-only future entry다.
Skill body가 없으며 active host discovery surface에 projection되지 않는다.

Routing request는 raw prompt가 아닌 normalized fact만 받는다. Active proof는 Skill
name, exact content digest, side-effect class, capability와 built-in source 또는
optional consent digest에 결합된다. Forged digest, unapproved optional Skill과
inactive Hive candidate는 fail closed하며 한 route는 Skill body를 최대 하나만 load한다.
Explicit Skill/direct answer, simple-question, compatible OMX/OMC, approved Hive Skill,
host-native 순서의 precedence를 적용한다.

Simple-question의 negative capability evidence는 normalized router의 empty capability와
추가 Skill body 0개 계약이다. OS syscall sandbox나 model behavior를 계측했다는
주장이 아니다. Description-based semantic match는 host가 소유하는 documented
interface이며 Hive는 raw prompt classifier나 `UserPromptSubmit` hook을 구현하지
않는다. Broader live-host match qualification은 Phase 4·7에 남아 있다.

### Prompt refinement

Normalized routing fact가 명시적인 prompt 작성·개선 intent를 표시할 때만
`hive-prompt-refine`를 선택한다.
`refine-only`가 기본이고 같은 요청에서 실행을 명시한 경우에만 `refine-and-run`을
허용한다. Original prompt는 immutable이며 must, must-not, scope, target output,
named tool/provider와 user authority를 보존한다. 필수 ambiguity는 한 번에 하나만
질문하고 나머지는 assumption 또는 placeholder로 남긴다.

`refine-only`는 project read/write, network, subagent, memory capture, run creation과
model execution을 허용하지 않는다. Provider를 지정하지 않은 result에는
Codex·Claude·Antigravity·OMX·OMC 전용 path나 command가 없다.

No-fabrication evidence는 structured assumption/unresolved item과 한-question
enforcement이며 model factuality를 판정하거나 보장하지 않는다. 이미 충분한 prompt는
exact character budget `max(original + 700, ceil(original × 1.5))`를 허용하고 한
character 초과를 거부하는 unit test로 검증했다.

### Host projection

- Codex: `.agents/skills/<skill>/SKILL.md`
- Antigravity: `.agents/skills/<skill>/SKILL.md`
- Claude Code: `.claude/skills/<skill>/SKILL.md`

각 host는 자기 exact discovery root에 implemented built-in 6개만 받는다. Host alias는
canonical Hive config에 저장하지 않는다. Antigravity projection byte 계약은
구현됐지만 broader live-host capability qualification은 Phase 4·7에 남아 있다.

Projection은 destination을 exclusive claim한 뒤 claimed bytes를 no-follow로 검증하고
destination-exclusive publication을 수행한다. Replace/delete로 밀려난 bytes는
same-directory quarantine에 보존한다. Rollback은 live published bytes를 다시
claim·검증하고 prior bytes를 exclusive republish한다. Foreign occupant는 overwrite나
delete하지 않으며 자동 복원이 안전하지 않으면 prior-byte recovery path를 diagnostic에
남긴다.

### Optional Skill과 fallback hook consent

Optional Skill은 name, immutable source, revision, content digest, 정렬된
requested/approved capabilities와 UTC-seconds approval의 RFC 8785 consent digest가
모두 일치해야 active proof와 projection을 얻는다. Approval metadata만 있거나 local
exact source digest가 다르면 inert다.

Fallback hook은 compatible OMX/OMC가 `absent`이고 exact capability, event,
project-local path, command, content digest와 consent digest를 승인한 경우에만
설치된다. 모든 non-Stop command는 fresh runtime evidence의 유일한 경로
`.hive/runtime/current-capability-resolution.json`을 사용한다.

- Setup은 runtime file과 `.hive/runtime/`를 만들거나 추적하지 않는다.
- Exact path의 non-symlink regular file과 60초 이하 freshness가 필요하다.
- Missing, stale, future, malformed, non-absent evidence는 approval·input을 읽기 전에
  exit `0`, `decision:allow`, `active:false`로 끝난다.
- Installed `.hive/config/capability-resolution.yml`은 live evidence를 대신하지 않는다.
- `Stop`은 runtime, approval, installed state, input과 tamper를 읽지 않는 neutral
  fast path다.
- External runtime이 감지되면 기존 fallback hook은 inert이고 reconfigure는
  Hive-owned hook artifact만 제거한다.

## Usage guard 선행 slice

Session window가 하나라도 있으면 session이 decision을 완전히 소유하며 weekly
snapshot의 duplicate·invalid·low/high 값은 decision에 관여하지 않는다. Session이
없을 때만 weekly를 fallback으로 선택한다. 선택된 window의
`remaining <= 10%`는 차단하며 missing, stale, scope mismatch는 `usage_unknown`으로
fail closed한다. 실제 automatic dispatch 직전 permit 요청·소비 연결은 Phase 5에
남아 있다.

## Fresh verification

Phase 3 milestone 동기화 후 fresh evidence:

- `cargo fmt --all --check`: PASS
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: PASS
- `cargo build --workspace --all-targets --all-features --locked`: PASS
- `cargo test --workspace --all-targets --all-features --locked`: Rust 107/107 PASS
- 전체 Python conformance: 313/313 PASS
  - Phase 1: 196/196
  - usage guard: 26/26
  - Phase 2: 22/22
  - Phase 3: 69/69
- Copier `9.17.0` default 3 host와 hostile fixture render/validation: PASS
- `hive --version`, Cargo metadata/lock, Copier harness template와 docs `0.4.0`
  parity: PASS
- `reuse lint`: 204/204 PASS
- `git diff --check`: PASS

CI는 Phase 1, usage guard, Phase 2와 Phase 3 corpus를 Ubuntu, macOS, Windows에서
실행하도록 동기화했다. 이 worktree의 remote native matrix는 push 전이므로 아직
완료 evidence로 표시하지 않는다.

## Version parity

다음 구현 표면은 `0.4.0`으로 동기화한다.

- root Cargo workspace와 Cargo.lock의 Hive package
- compiled `hive --version`
- Copier installed `.hive/config/harness.toml`
- README, PLAN, CURRENT와 version lifecycle ADR

`0.3.0 → 0.4.0`은 backward-compatible Phase 3 Skill/projection feature minor다.
Major는 변경하거나 추론하지 않았다.

## 남은 Phase 4–7

### Phase 4 — Role/run interoperability

- RoleProfile·Run parser와 fresh-session resume
- host-native subagent conformance
- run별 owner evidence pin, no-mid-run-switch와 OMX/OMC coexistence qualification

### Phase 5 — Usage guard와 judge

- dispatch owner와 one-shot usage permit 소비 연결
- CodexBar live account/window/freshness qualification
- clean-context judge package와 2/3, 3/3+human quorum

### Phase 6 — Update, migration와 release

- `hive-update`, same-major compatibility와 cross-major migration
- backup/restore/retention, crash recovery와 atomic update activation
- release bundle parity, GitHub Release packaging, signing과 install path

### Phase 7 — Public qualification

- macOS arm64/x86_64와 Windows x86_64 release qualification
- 세 live host base workflow와 host-native/OMX/OMC support matrix
- migration fault injection, supply-chain provenance와 release candidate
- 사용자가 exact `1.0.0`을 지시하기 전 stable major prepare 금지
