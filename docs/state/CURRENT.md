# 현재 상태

- 기준일: 2026-07-24
- active plan: [`../plans/PLAN.md`](../plans/PLAN.md)
- plan revision: 1.6
- product version: `0.2.0`
- phase: Phase 1 local completion gate 완료; Phase 5 usage guard core/adapter 선행 slice 완료
- Git: `develop`, 장기 branch는 `main`/`develop` 두 개만 사용
- remote: [`gvm1229/aigent-hive`](https://github.com/gvm1229/aigent-hive), default branch `main`

현재 worktree의 product version은 `0.2.0`이다. Phase 1의 local completion checklist와 conformance gate는 완료됐다. 갱신된 변경의 원격 `windows-latest` runtime matrix는 `develop` push 뒤 확인해야 한다. Markdown knowledge ingest/query와 SQLite index, portable Skill/host projection, durable run, judge와 update/release는 아직 지원된 것으로 표시하지 않는다.

사용자 요청에 따라 Phase 5의 usage guard core/adapter를 순서 밖 선행 구현했다. 이는 다음 automatic dispatch용 provider-neutral 판단 경계이며 이미 시작된 host turn을 강제 종료하지 않는다.

## 완료된 구현

- Rust workspace의 `hive-core`, `hive-render`, `hive-cli`
- `hive setup --dry-run|--apply|--validate --output json`
- source-root guard, staging render, manifest ownership과 symlink/path escape 방어
- target file을 읽기 전 symlink ancestor를 거부하는 no-follow 경계
- `canonical-data-protected` byte 보존과 ownership class별 write/delete 제한
- `--dry-run`의 target sibling 전체 staging render·exact byte/installed contract validation과 target write 0회
- 충돌하지 않는 임의 이름의 exclusive staging/activation temp와 transaction rollback
- setup 시작 시 ambient parent에서 no-follow로 pin한 target capability에서 protected/marker/role/revocation render read와 changed-path 계산
- 같은 target-root handle 아래 staging 이후 snapshot·mkdir·replace/delete·rollback·post-validation
- exact-pinned `cap-fs-ext 4.0.2`의 stable root/child/file no-follow open과 mutation 전후 handle-derived device/inode 비교로 ambient retarget·symlink ancestor 교체 차단
- rollback 자체가 실패하면 exit `10`, `hive.activation-rollback-failed`로 별도 보고
- Windows remove-before-persist 교체의 이전 destination backup/복원과 backup residue 제거
- shared `AGENTS.md`의 exact `AIGENT-HIVE` marker merge와 외부 bytes 보존
- Copier/Rust static tree parity와 setup answer migration
- role seed materialization, idempotency, definition conflict와 승인된 reconfigure 보존
- optional Skill RFC 8785 consent 검증과 tamper 차단
- evidence-derived `available|absent|incompatible|unknown` capability resolution
- Codex의 compatible OMX, Claude의 compatible OMC 우선과 Antigravity/부재 host-native resolution
- full capability resolution object를 결합하는 evidence digest
- external capability가 conclusively `absent`일 때만 가능한 fallback hook consent·projection
- hook descriptor content digest, approval consent digest와 activation-time 재검증
- 승인된 non-Stop hook의 유효한 capability/event별 typed input 처리와 bounded data-integrity action
- malformed·unsafe non-Stop input과 실행 오류의 exit `0`, `active:false`, neutral allow
- 미승인·변조·non-absent non-Stop hook의 inactive allow와 input 선행 read 차단
- 모든 `Stop` 상태의 exit `0`, `active:false`, neutral allow
- 재설정 dry-run의 hook revoke preview와 apply의 installed approval/evidence·exact ledger/descriptor 재검증 후 제거
- version, answer, capability, ledger, hook, role, protected seed와 marker를 아우르고 revoked known hook descriptor 잔존도 거부하는 installed validation
- unknown top-level action의 schema-valid `UnknownAction` JSON과 project write 0회
- consumer `.hive/.gitignore`의 SQLite/WAL/SHM와 backup 제외
- Cargo/CLI/Copier installed harness/README/CURRENT product version `0.2.0`

## Usage guard 선행 slice

- `UsageSnapshot` schema와 provider-neutral policy evaluator
- session `300m` 우선, session limit 미제공 시 weekly `10080m` fallback
- 기본 `remaining <= 10%` inclusive block과 `1..99` setup override
- missing, stale, version/account/source/window mismatch와 역행 값을 typed `usage_unknown`으로 fail-closed
- 최대 5초, non-cloneable, 1회 소비 `UsagePermit`
- optional CodexBar `0.45.2`의 fixed argv, no-shell, bounded `usage` JSON adapter
- raw account를 출력·저장하지 않고 `sha256:` digest로만 account scope 비교
- `hive usage check --account-digest <sha256:...> --output json`
- allow/block/unknown 모두 consumer project write 0회

CodexBar `0.45.2` app/CLI와 실제 account binding은 이 machine에서 확인했다. 현재 OpenAI가 five-hour session limit을 제공하지 않아 local CLI snapshot은 weekly `10080m`만 노출하며, policy는 session이 돌아오면 이를 우선하고 그 전에는 weekly를 fallback으로 사용한다. OMX·host-native dispatch owner에 연결된 자동 permit 소비는 아직 완료되지 않았다.

## Phase 1 completion review

`schema_version: 999`인 JSON-parseable cross-major role fixture를 setup shadow render의 실제 role schema gate에 연결했다. Candidate는 `hive.setup-conflict`로 거부되고 `changed_paths` 0개와 active tree 전체 byte 불변을 보존한다. 기존 테스트의 `harness_version` 변조 조기 실패는 제거했다. 이는 Phase 1 candidate safety gate이며 Phase 6의 실제 cross-major update route·transform·activation 지원을 뜻하지 않는다.

## Phase 1 계약

- Orchestration owner를 setup preference로 묻지 않는다. Active host evidence에서 자동 resolve한다.
- `available`은 matching compatible evidence로 external owner를 선택한다.
- `absent`는 host catalog와 public executable 양쪽의 explicit absent evidence가 필요하다.
- `incompatible|unknown`은 host-native로 resolve하지만 fallback hook을 제안·설치하지 않는다.
- Fallback hook은 exact capability, event, path, command와 digest를 승인받은 data-integrity guard뿐이다.
- Hook 거절은 setup 성공 상태다. Hook은 prompt classification/rewrite, Skill activation, orchestration, memory ingest 또는 continuation을 수행하지 않는다.
- Hook descriptor는 `{schema_version, capability, event, path, command}` RFC 8785 JCS bytes와 LF이며 content digest가 설치 bytes를 결합한다.
- Consent digest는 `consent_digest`를 제외한 approval payload 전체를 결합한다.
- 승인된 non-Stop hook만 input을 읽고 capability/event별 typed 검사를 수행한다. malformed·unsafe input이나 실행 오류, 미승인·변조·non-absent hook은 inactive neutral allow다.
- `Stop` hook은 승인·변조·malformed/non-absent 상태와 관계없이 authorization과 input을 읽지 않는 `active:false` neutral allow이며 continuation loop를 만들지 않는다.
- Hook 승인을 철회한 setup은 installed setup answer, absent resolution evidence, consent digest, exact ledger/descriptor bytes를 다시 검증한 Hive-owned artifact만 제거하고 인접 user/foreign bytes를 보존한다.

상세 계약은 [`../architecture/hook-consent.md`](../architecture/hook-consent.md), consumer marker ownership은 [`../guidance-schema.md`](../guidance-schema.md)를 따른다.

## Fresh verification — usage guard slice

- `cargo fmt --all --check`: PASS
- `cargo test -p hive-core`: 17/17 PASS
- `cargo test -p hive-cli`: 31/31 PASS
- `cargo test --workspace`: 63/63 PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `python -m unittest tests.conformance.test_usage_guard -v`: 26/26 PASS
- live CodexBar `0.45.2` check: session 미제공, weekly remaining `51%`, threshold `10%`, allow
- Homebrew `/opt/homebrew/bin/codexbar` symlink를 실제 helper executable로 resolve하는 unit regression: PASS
- UsageSnapshot schema meta-validation과 ActionResult schema validation: PASS
- Copier 9.17.0 default Codex render의 setup answers/TOML threshold `10`: PASS
- allow `57%`, exact boundary `10%`, one-window `8%`, missing/version/timeout/malformed/process/row/account/window/stale hostile cases: PASS
- allow/block/unknown consumer project write 0회와 raw account 비노출: PASS
- isolated adversarial verifier: safeguard slice PASS, blocker 0개
- `git diff --check`: PASS

## Phase 1 verification

### Fresh verification — target capability activation

- `cargo fmt --all --check`: PASS
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: PASS
- `cargo build --workspace`: PASS
- `cargo test --workspace --all-targets --all-features`: Rust test 68개 PASS — `hive-cli` 31, `hive-core` 18, `hive-render` 19
- `python -m unittest discover -s tests/conformance -p 'test_phase1*.py' -v`: Phase 1 conformance 192개 PASS
- stable `x86_64-pc-windows-msvc` workspace all-target/all-feature `cargo check`: PASS
- stable `x86_64-pc-windows-msvc` `hive-render` strict Clippy: PASS
- target-root handle open 직후 ambient target retarget과 managed ancestor symlink swap을 각각 주입한 deterministic hostile regression: 둘 다 conflict로 차단하고 retarget 시 pinned target rollback, 외부 sentinel byte 불변 PASS
- initial pin 뒤 ambient target을 교체해도 protected render bytes는 pinned tree에서만 읽는 regression: PASS
- Windows-style capability-relative destination backup 교체 success/failure: 이전 bytes 복원과 backup residue 0개 PASS
- role/parity targeted corpus 20/20 PASS; JSON parse 성공·`schema_version: 999` schema conflict·harness version 조기 실패 부재·active tree 전체 byte 불변 PASS
- isolated adversarial verifier: implementation·stable Windows cross-compile PASS; remote native-Windows runtime gate만 미확인
- `reuse lint`: 158/158 file licensing PASS
- `git diff --check`: PASS

### 이전 completion-review verification

- `cargo fmt --all --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo build --workspace`: PASS
- `cargo test --workspace --all-targets --all-features`: Rust test 65개 PASS — `hive-cli` 31, `hive-core` 18, `hive-render` 16
- `python -m unittest discover -s tests/conformance -t . -p 'test_phase1_*.py' -v`: Phase 1 conformance 192개 PASS
- Copier 9.17.0 default render: Codex, Claude, Antigravity 각각 schema/render validation PASS
- Copier 9.17.0 hostile-string/typed-answer render: schema/render validation PASS
- `available|incompatible|unknown` all-host negative hook validation: hook artifact·command 0개 PASS
- 4-hook와 no-hook Copier/Rust normalized static tree parity: PASS
- Hook consent/event/path/capability/content digest tamper와 installed descriptor tamper: activation 0회 PASS
- 철회 전 ledger/descriptor tamper: target mutation 0회와 exact tree 보존 PASS
- malformed·unsafe non-Stop input과 typed execution 오류: exit `0`, inactive neutral allow PASS
- dry-run sibling staging success/failure injection: target write 0회, staging cleanup과 exact validation PASS
- revoked known Hive descriptor without ledger: installed `--validate` exit `5`, target tree 불변 PASS
- Windows-style destination backup 교체 success/failure: 이전 bytes 복원과 backup residue 0개 PASS
- approved non-Stop의 typed protection/update/invalidation/checkpoint 동작: PASS
- approved, unapproved, malformed, tampered와 non-absent `Stop`: recursive neutral allow PASS
- ownership-class 재설정 보존, target no-follow FIFO, mid-activation failure rollback, rollback-failure code, hook revoke와 complete installed validate: PASS
- project/host-global foreign namespace FIFO no-read·checksum 보존, no-consent projection 0개와 approved hook 인접 foreign entry digest 보존: PASS
- canonical Markdown/YAML/TOML/role/run Git visibility와 SQLite/SQLite3의 WAL/SHM·backup 전용 ignore 의미: PASS
- `hive --version`: `hive 0.2.0`
- Cargo direct dependency audit: model provider, model API와 network SDK 0개
- `reuse lint`: 150/150 file licensing PASS
- `git diff --check`: PASS

CI workflow는 Linux·macOS·Windows에서 Phase 1과 usage guard Python corpus를 실행하도록 구성했다. Local stable Rust 1.97.1에서 `x86_64-pc-windows-msvc` workspace all-target/all-feature `cargo check`와 `hive-render` strict Clippy가 PASS했다. 원격 `windows-latest` matrix는 이 변경이 `develop`에 push된 뒤 확인해야 한다. ReFS의 128-bit file identifier를 `cap-fs-ext`가 64-bit inode로 표현하는 제한은 Windows matrix에서 계속 감시할 known risk다.

Stage 2 완료 조건은 9/9다. 실제 source-version route, role transform과 cross-major migration activation은 Phase 6 범위로 계속 미지원이다.

## 다음 작업

1. CodexBar dispatch owner integration
2. Phase 2 Wiki/frontmatter schema와 Markdown ingest/query/lint
3. suppression/delete, stale-reference, FTS5/tag/link index와 deterministic rebuild
4. logical digest, query equivalence와 parallel extraction/serial integration fixture

Phase 3의 `hive-prompt-refine`, automatic approved-Skill routing과 host projection은 Phase 2 완료 뒤 진행한다. `refine-only` 기본, explicit `refine-and-run`, meaning preservation과 hidden rewrite 금지 계약은 그대로 유지한다.
