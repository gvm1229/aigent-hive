# 현재 상태

- 기준 branch: `develop`
- product version: `0.5.0`
- plan revision: `1.10`
- 현재 milestone: Phase 4 완료
- 다음 milestone: Phase 5 usage dispatch integration과 judge quorum

## 현재 truth

Phase 1 결정적 setup renderer, Phase 2 canonical Markdown knowledge·disposable
SQLite index, Phase 3 portable Skills·host projection과 Phase 4 persistent role·durable
run recovery가 구현됐다.

Hive는 model-provider API, provider SDK, provider API key, model runtime, scheduler,
plan/Ralph/team clone을 소유하지 않는다. Model call, subagent spawn, retry와
continuation은 새 run에서 자동 resolve한 host-native, OMX 또는 OMC owner가 소유한다.

Usage guard core와 optional CodexBar adapter는 Phase 5 선행 slice다. Session window가
있으면 weekly보다 우선하고, host가 session limit을 노출하지 않을 때만 weekly를
fallback으로 사용한다. 실제 dispatch owner의 one-shot permit 소비와 live sensor
qualification은 아직 완료되지 않았다.

## Phase 4 구현

### Persistent role과 shared handoff

- `hive role validate`는 exact role file을 no-follow로 읽고 role schema, filename ID와
  runtime path를 검증한다. Project bytes를 수정하지 않는다.
- `.hive/team/roles/<role-id>.md`가 identity, definition, current assignment,
  handoff path와 exact Markdown body의 runtime 정본이다. Session/process가 role
  identity를 대신하지 않는다.
- `.hive/runs/<run-id>/HANDOFF.md`는 canonical JSON frontmatter의 `handoffs` map과
  exact `# Role handoffs\n` body를 가진 shared envelope다.
- `hive role handoff`는 observed assignment, path와 exact HANDOFF digest를 요구한다.
  Stale request는 write 0건이며 성공은 shared entry와 selected role assignment를
  optimistic two-file transaction으로 commit한다.
- Identical retry는 no-op이고 다른 role entry, role body와 foreign namespace는
  보존한다.

### Durable checkpoint와 evidence

- `hive run checkpoint`는 required criterion을 caller input이 아니라 existing
  `PLAN.md` checkbox에서 파생한다.
- `STATUS.md` revision은 exact optimistic counter이며 동일 request retry는
  byte-identical no-op, lost revision은 conflict다.
- Passed criterion은 exact
  `.hive/runs/<run-id>/evidence/<file>#sha256:<64-lowercase-hex>` locator를 요구한다.
  Missing 또는 tampered evidence는 status write와 resume를 막는다.
- Required criterion 하나라도 unchecked, failed 또는 unverified이면 `succeeded`를
  기록할 수 없다. 100개 중 99개 pass hostile fixture도 verification failure다.

### Immutable owner와 fresh-session resume

새 run owner는 사용자 선택 없이 fresh capability evidence에서 정한다.

| Detection | 새 run owner | Fallback hook |
| --- | --- | --- |
| Codex compatible `available` | OMX | 금지 |
| Claude compatible `available` | OMC | 금지 |
| `absent` | truthful host-native | exact preview와 explicit consent 후에만 가능 |
| `incompatible` | host-native; 부족한 capability는 `unsupported` | 금지 |
| `unknown` | host-native best-effort 또는 `unverified` | 금지 |

첫 checkpoint는 host, version, surface, external runtime, owner, full resolution JCS
digest와 subagent support를 함께 pin한다. Missing, incompatible, version 또는 evidence
drift에서 existing run owner를 바꾸지 않으며 write 없이 exit `3|4|5`로 중지한다.

`hive run resume`는 PLAN, STATUS, role body, shared handoff와 evidence만 읽는다.

- `executing|verifying` + `supported|best-effort`: role별 prepare-only brief,
  `spawned:false`
- `unsupported|unverified`: exit `4`, brief/spawn data 없음
- `blocked|usage-limited`: exit `3`, recovery data만 반환
- `resume-ready`: hidden transition 없이 recovery data만 반환
- terminal state: continuation 없이 durable result만 반환

SQLite, transcript, `.omx/.omc` state와 host-global config는 resume 정본이 아니다.

### Skills와 external coexistence

Discoverable implemented built-in은 exact 9개다.

- `setup-harness`
- `hive-simple-question`
- `hive-prompt-refine`
- `hive-knowledge-capture`
- `hive-knowledge-query`
- `hive-knowledge-maintenance`
- `hive-role-handoff`
- `hive-run-checkpoint`
- `hive-run-resume`

`hive-judge-package`, `hive-update`, `hive-migrate`는 catalog-only다.

Compatible OMX/OMC가 있어도 세 role/run data Skill은 canonical Hive state를
기록·검증·복구하기 위해 projection할 수 있다. 이 Skills는 OMX/OMC command,
plan/Ralph/team/retry/persistent loop 또는 lifecycle hook을 실행하지 않는다.
Project와 fake HOME의 `.omx/.omc` foreign bytes는 성공·실패 모두 불변이다.

## 격리 verifier remediation

Phase 4 최초 격리 adversarial verifier는 세 가지 completion blocker를 찾았다.

- 오래됐거나 미래 시각인 capability input을 checkpoint/resume가 수용할 수 있었다.
- 신규 file publish 뒤 cleanup failure에서 canonical destination이 남을 수 있었다.
- Shared `HANDOFF.md` timestamp가 shape만 검사해 불가능한 날짜·시각을 수용했다.

Remediation은 capability file의 no-follow regular preflight, exact opened-handle
metadata와 60초 freshness 검증을 parse·owner resolution·write·brief보다 먼저
적용했다. 신규 file cleanup failure는 temp handle의 exact inode이면 canonical
publish를 rollback하고, 다른 inode의 racer는 덮어쓰거나 삭제하지 않고 보존한다.
Shared handoff envelope와 모든 entry timestamp는 Draft 2020-12 `date-time` format
validation을 통과해야 한다. 후속 hostile hardening은 explicit input open을
nonblocking으로 만들고 preflight와 opened handle의 device/inode를 결속했으며,
same-handle post-read type·identity·size·mtime·byte-length stability를 요구한다.
아래 fresh evidence로 세 finding과 후속 race seam이 모두 닫혔다.

## Fresh verification

Phase 4 version/documentation/CI 동기화에서 실행한 fresh evidence:

- `cargo fmt --all --check`: PASS
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: PASS
- `cargo build --workspace --all-targets --all-features --locked`: PASS
- Explicit-input race targeted regressions: 3/3 PASS
- `cargo test -p hive-cli --all-targets --all-features --locked`: CLI 52/52 PASS
- `cargo test --workspace --all-targets --all-features --locked`: Rust 147/147 PASS
- 전체 Python conformance: 337/337 PASS
  - Phase 1: 196/196
  - usage guard: 26/26
  - Phase 2: 22/22
  - Phase 3: 71/71
  - Phase 4: 22/22
- Copier `9.17.0`: default host 3/3, hostile 1/1 PASS
- Non-absent fallback-hook Copier rejection: 3/3 PASS
- Draft 2020-12 schema meta-validation: 22/22 PASS
- `hive --version`: `hive 0.5.0`

CI는 Phase 1, usage guard, Phase 2, Phase 3와 Phase 4 corpus를 Ubuntu, macOS,
Windows에서 실행하도록 동기화했다. 이 worktree의 remote native matrix는 push
전이므로 완료 evidence로 표시하지 않는다. Local
`x86_64-pc-windows-msvc` cross-check는 Hive CLI compile 전에 bundled SQLite C
build가 Windows C SDK의 `stdlib.h`를 찾지 못해 중단됐으며 PASS로 표시하지 않는다.

## Version parity

다음 tracked/compiled 표면은 `0.5.0`으로 동기화한다.

- root Cargo workspace와 Cargo.lock의 Hive package
- compiled `hive --version`
- Copier/Rust installed `.hive/config/harness.toml`
- README, PLAN, CURRENT와 version lifecycle ADR

`0.4.0 → 0.5.0`은 backward-compatible Phase 4 role/run interoperability feature
minor다. 기존 같은-major supported contract를 깨뜨리지 않으며 major는 변경하거나
추론하지 않았다.

## 남은 Phase 5–7

### Phase 5 — Usage guard와 judge

- Dispatch owner와 one-shot usage permit 소비 연결
- CodexBar live account/window/freshness qualification
- Clean-context judge package와 2/3, 3/3+human quorum

### Phase 6 — Update, migration와 release

- `hive-update`, same-major compatibility와 cross-major migration
- Backup/restore/retention, crash recovery와 atomic update activation
- Release bundle parity, GitHub Release packaging, signing과 install path

### Phase 7 — Public qualification

- macOS arm64/x86_64와 Windows x86_64 release qualification
- 세 live host base workflow와 host-native/OMX/OMC support matrix
- Migration fault injection, supply-chain provenance와 release candidate
- 사용자가 exact `1.0.0`을 지시하기 전 stable major prepare 금지
