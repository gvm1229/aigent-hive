# 현재 상태

- 기준 branch: `develop`
- product version: `0.3.0`
- plan revision: `1.8`
- 현재 milestone: Phase 2 완료
- 다음 milestone: Phase 3 portable Skills와 host projection

## 현재 truth

Phase 1 결정적 setup renderer와 Phase 2 canonical Markdown knowledge·disposable
SQLite index가 구현됐다. Usage guard core와 optional CodexBar adapter는 Phase 5
선행 slice로 유지된다.

아직 지원 완료로 표시하지 않는 범위:

- automatic approved-Skill routing과 `hive-prompt-refine`
- Codex·Claude·Antigravity host projection
- durable run/role resume interoperability
- dispatch owner와 usage permit 소비 연결
- hostile judge quorum
- signed update, migration과 release

## Phase 2 구현

### Canonical knowledge

- `.hive/knowledge/Raw/**`: 5 MiB 이하 비기밀 source의 SHA-256 content-addressed immutable revision
- `.hive/knowledge/Wiki/<id>.md`: typed YAML frontmatter와 Markdown body
- `.hive/knowledge/Schema/schema.md`: kind, tag, alias, link, source와 contradiction contract
- `.hive/knowledge/suppression.yml`: fingerprint, source locator, reason, replacement, timestamp만 저장
- `OPENAI_API_KEY`, bearer/token prefix, private-key PEM과 password/secret assignment는 Raw write 전에 거부
- `deprecated|superseded|archived` active page 거부
- orphan, broken link, missing citation, invalid contradiction, alias collision과 stale index lint
- canonical mutation 전 project-local lock 획득으로 parallel extraction 결과의 serial integration
- 같은 Wiki page 재수집은 기존 source·tag·alias·link·contradiction을 보존·합집합하며 concurrent ingest도 source를 잃지 않음
- Raw/Wiki write 뒤 stale/index 단계 실패가 나면 canonical tree와 derived state를 operation 이전 byte로 rollback

### Disposable index

`crates/hive-wiki`가 `.hive/index/hive.sqlite3`에 다음 derived row를 투영한다.

- FTS5 summary/body/alias/tag
- tag와 alias
- backlink와 explicit/inline Wiki link
- Raw source와 contradiction graph
- canonical page·Raw content hash
- page/raw count와 deterministic logical digest

`hive index rebuild`는 canonical source를 scan하고 같은 index directory의 exclusive
temp DB에 전체 projection을 생성한다. Page count와 logical digest를 검증한 뒤
active DB를 교체하며 `.stale`을 제거한다. SQLite byte hash 동일성은 요구하지 않는다.

`hive knowledge query`는 read-only open 전 canonical logical digest와 stale marker를
검증한다. DB 삭제 후 rebuild한 query result와 logical rows는 동일하다.

### CLI

```text
hive knowledge ingest
hive knowledge query
hive knowledge lint
hive knowledge delete
hive knowledge suppress
hive index rebuild
```

모든 command는 schema-valid `ActionResult` JSON, stable exit class, changed path와
logical digest evidence를 반환한다. Provider API, model call, network SDK와 credential
path는 없다.

## Dependency 결정

`rusqlite 0.40.1`을 `bundled` feature로 exact pin했다. Rust 표준 library에는 SQLite,
FTS5와 safe prepared binding이 없고 직접 C ABI 구현은 `unsafe_code = "forbid"`와
cross-platform 재현성을 해친다. `rusqlite` MIT와 bundled SQLite public-domain
license는 Apache-2.0 배포와 양립한다.

상세 근거: [`../research/rusqlite-sqlite-index.md`](../research/rusqlite-sqlite-index.md)

## Fresh verification

- `cargo fmt --all --check`: PASS
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: PASS
- `cargo build --workspace`: PASS
- `cargo test --workspace --all-targets --all-features`: Rust 73개 PASS
  - `hive-cli` 31
  - `hive-core` 19
  - `hive-render` 20
  - `hive-wiki` 3
- `python -m unittest discover -s tests/conformance -t . -p 'test_phase1_*.py' -v`: 192/192 PASS
- `python -m unittest tests.conformance.test_phase2_wiki -v`: 22/22 PASS
- Phase 2 corpus:
  - immutable Raw revision과 credential/size gate; misleading `example` substring이 있는 실제-looking secret도 거부
  - prepared Wiki ingest와 deprecated rejection
  - FTS5/tag/alias/link/source/content-hash row
  - DB 삭제/rebuild logical digest와 query equivalence
  - direct Markdown change stale detection
  - orphan/broken link/missing citation/contradiction lint
  - page+unreferenced Raw delete와 deleted prose 비보존
  - suppressed fingerprint re-ingest 거부
  - parallel extraction + serial integration no-lost-update
  - same-page sequential/concurrent source accumulation no-lost-update
  - canonical write 뒤 injected failure의 exact-tree rollback
  - 기존 empty index directory까지 보존하는 rollback
  - Raw locator filename과 실제 content digest mismatch 거부
  - suppression reason을 stable enum code로 제한해 deleted prose 비보존
  - SQLite와 stale marker symlink를 no-follow로 거부하고 외부 target byte 보존
  - decoy key-name 뒤 실제 assignment를 포함한 Raw, prepared Wiki와 수동 content-addressed Raw의 likely credential도 activation/rebuild 전 거부
  - traversal·filename/digest mismatch Raw locator를 canonical write 전에 거부
  - active Wiki/Raw fingerprint 또는 locator와 겹치는 direct suppression·rebuild 거부
  - consumer target 위쪽 symlink ancestor를 entrypoint에서 거부하고 resolved 외부 tree write 0건
  - standalone rebuild가 stale marker를 제거할 때 top-level `changed_paths`에 marker와 DB를 모두 보고
- Phase 1 renderer/Copier static tree parity: PASS
- Phase 2 runtime ignore + parity targeted corpus: 7/7 PASS
- default Copier render + full schema/contract validation: PASS
- `reuse lint`: 149/149 file licensing PASS
- provider/model/network SDK dependency audit: 0개
- local Markdown link resolution: PASS
- `git diff --check`: PASS
- live Hive usage gate: threshold `10%`, `hive.usage-allowed`, sanitized evidence
  `sha256:82b05fbf9184a8d92ce51e9f3bc8c98904a6d4fbf1b5beea121022eb9c7fc363`
- usage precedence conformance: 300분 session window가 있으면 weekly보다 우선하고,
  session이 없을 때만 weekly를 fallback으로 사용하며 exact `10%`는 차단
- isolated Phase 2 release verifier: hostile matrix와 `changed_paths` schema reproduction PASS
- committed Phase 1·usage guard native CI run `30027682535`: Ubuntu/macOS/Windows와
  Copier 전 job PASS; Windows `.exe`/`.cmd`/`.bat` CodexBar discovery 포함

## Version parity

다음 표면은 `0.3.0`으로 동기화한다.

- root Cargo workspace와 Cargo.lock의 Hive package
- compiled `hive --version`
- Copier installed `.hive/config/harness.toml`
- README, PLAN, CURRENT와 version lifecycle ADR

## 다음 작업

1. Phase 3 `hive-simple-question` isolation
2. `hive-prompt-refine` refine-only/refine-and-run contract
3. approved-only minimal Skill routing과 OMX/OMC precedence
4. Codex·Claude·Antigravity thin projection

로컬 macOS에서 `x86_64-pc-windows-msvc` check를 시도했으나 bundled SQLite C
compile에 필요한 Windows SDK `stdlib.h`가 없어 `libsqlite3-sys` build script에서
중지했다. Rust-only Phase 1 Windows target은 이전 gate에서 통과했으며, bundled
SQLite와 Phase 2 native behavior는 변경을 push한 뒤 원격 `windows-latest` matrix로
확인해야 한다.
