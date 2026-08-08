# Source 개발·검증

## 역할 구분

| Surface | Dependency |
| --- | --- |
| Consumer `hive` runtime | Native Rust binary |
| npm install channel | Node.js·npm |
| Direct install channel | Unix shell 또는 Windows PowerShell 5.1·`cmd.exe` |
| Source development | Rust stable, Python 3.13 conformance environment |
| Windows source workflow | PowerShell 7 LTS |
| Template authoring·parity | Copier 9.17.0 |

Consumer runtime의 Python·Copier·Node.js·PowerShell 7 dependency 없음.

## 기술 stack

| 기술 | 사용 범위 |
| --- | --- |
| Rust stable, Edition 2021 | Cross-platform CLI와 safety boundary |
| Cargo workspace | 6개 crate build·test·lint |
| Markdown | Knowledge·plan·role·run·human guidance 정본 |
| YAML·TOML | Setup answer·typed config·approval·ownership |
| JSON Schema 2020-12 | Provider-neutral machine contract |
| SQLite FTS5 | 재생성 가능한 local search projection |
| Python 3.13 | Schema·Copier·black-box conformance |
| Copier 9.17.0 | Authoring-time template parity |
| GitHub Actions | Linux·macOS·Windows CI와 release qualification |

## Rust dependency

정확한 pin 정본: [`Cargo.toml`](../../Cargo.toml)과 [`Cargo.lock`](../../Cargo.lock).

| Dependency | 역할 |
| --- | --- |
| `cap-std`, `cap-primitives`, `cap-fs-ext` `4.0.2` | Filesystem capability·no-follow identity |
| `tempfile` `3.27.0` | 충돌 방지 staging·activation temp |
| `rusqlite` `0.40.1` + bundled | Portable FTS5·serialized disposable index |
| `jsonschema` `0.48.5` | Setup·capability·hook contract |
| `serde` family | Typed config·projection |
| `serde_json_canonicalizer` `0.3.2` | RFC 8785 evidence bytes |
| `sha2` `0.11.0` | Content·identity digest |
| `ed25519-dalek` `3.0.0` | Strict detached signature verification only |

Model provider SDK·network model client dependency 없음.

## Python·template dependency

정확한 pin 정본:

- [`requirements-conformance.txt`](../../requirements-conformance.txt)
- [`copier.yml`](../../copier.yml)
- [`rust-toolchain.toml`](../../rust-toolchain.toml)

| Dependency | 역할 |
| --- | --- |
| `copier==9.17.0` | Template fixture |
| `jsonschema[format]==4.25.1` | Schema meta·instance validation |
| `PyYAML==6.0.2` | YAML fixture |
| Rust `stable` + `rustfmt` + `clippy` | Build·format·lint·test |

## 빠른 검증

```console
python scripts/dev-check.py rust
python scripts/dev-check.py python
python scripts/dev-check.py pre-push
python scripts/dev-check.py rust test -p hive-core
python scripts/dev-check.py python tests.conformance.test_phase4_contracts -v
```

`dev-check.py`: caller environment mutation 없는 wrapper. Rust toolchain은 `PATH`와
rustup stable에서 탐색. Python conformance는 `uv` isolated environment와 exact
requirements 사용.

## 직접 명령

```console
cargo build --workspace --all-targets --all-features --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
python -m unittest discover -s tests/conformance -v
```

## Local developer binary

공개 npm 시험판 없이 현재 source를 확인할 때:

```console
scripts/dev-install.sh --sandbox
./target/release/hive -v
```

출력은 공개 시험판과 구분되는 `AIgent Hive vX.Y.Z-dev · local developer build (built
YYYY-MM-DD)` 형식. 기존 전역 `hive`를 이 개발 binary로 임시 교체하려면
`scripts/dev-install.sh --global`, 이전 실행 파일 복구는 `scripts/dev-install.sh --rollback`을
사용. 세 mode 모두 `~/.hive`의 preference·knowledge·index와 user directive·Skill, project
`.hive` 생성·변경·삭제 금지.

## CLI 진단 예시

```console
cargo run -p hive-cli -- doctor
cargo run -p hive-cli -- check-target /path/to/consumer-project
cargo run -p hive-cli -- index rebuild --target /path/to/consumer-project --output json
cargo run -p hive-cli -- knowledge query --target /path/to/consumer-project --text "search terms" --output json
```

## Verification tier

1. Work loop: 변경 crate와 직접 관련 Python test
2. Pre-commit: nearest black-box·schema·static regression
3. Pre-push: full Rust workspace와 full Python conformance
4. Release: clean clone, 모든 target, installer·recovery·provenance·publication gate

Git 절차: [Commit rule](commit-rules.md) · [Branch rule](branching-rules.md).
