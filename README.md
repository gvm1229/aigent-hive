# Aigent Hive

[![Rust](https://img.shields.io/badge/Rust-stable-000000?logo=rust&logoColor=white)](rust-toolchain.toml)
[![Cargo](https://img.shields.io/badge/Cargo-workspace-CB4B16?logo=rust&logoColor=white)](Cargo.toml)
[![Python](https://img.shields.io/badge/Python-3.13-3776AB?logo=python&logoColor=white)](.github/workflows/ci.yml)
[![Copier](https://img.shields.io/badge/Copier-9.17.0-5C4EE5)](copier.yml)
[![JSON Schema](https://img.shields.io/badge/JSON%20Schema-2020--12-000000?logo=json&logoColor=white)](schemas/)
[![SQLite](https://img.shields.io/badge/SQLite-planned-003B57?logo=sqlite&logoColor=white)](docs/plans/PLAN.md)
[![GitHub Actions](https://img.shields.io/badge/GitHub%20Actions-CI-2088FF?logo=githubactions&logoColor=white)](.github/workflows/ci.yml)
[![Platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey)](docs/plans/PLAN.md)
[![License](https://img.shields.io/badge/license-Apache--2.0-green)](LICENSE)

Aigent Hive는 Codex, Claude Code, Gemini Antigravity 같은 구독형 agent host 위에 설치할 **provider-neutral 로컬 agent harness**를 개발하는 source workspace다. 프로젝트마다 일관된 setup, 역할·지식·run 상태, 안전한 update와 검증 계약을 제공하되 모델 실행과 orchestration은 host 또는 사용자가 선택한 OMX·OMC에 맡긴다.

Hive는 모델 API나 provider SDK를 사용하지 않으며 API key를 요청하거나 저장하지 않는다.

## 아키텍처

Hive는 다음 세 artifact를 분리한다.

1. **Source workspace** — 이 저장소의 Rust source, template, schema, test와 개발 지침
2. **Release bundle** — 재현 가능한 CLI binary, versioned template·projection과 검증 metadata
3. **Installed harness** — 독립된 소비자 프로젝트에 생성되는 `.hive/`, `AGENTS.md`와 host projection

정본 데이터는 Git으로 추적 가능한 Markdown·YAML·TOML에 둔다. SQLite는 향후 로컬 검색용 파생 index로만 사용하며, 삭제 후에도 정본 파일만으로 재생성할 수 있어야 한다. Copier는 template authoring과 CI parity 검증에만 사용하고, 배포된 Rust CLI와 소비자 harness는 Python이나 Copier에 의존하지 않는다.

## 기술 스택

| 기술 | 사용 방식 |
| --- | --- |
| Rust stable, Edition 2021 | cross-platform CLI, setup/update 안전 경계와 결정적 projection |
| Cargo workspace | `hive-core`와 `hive-cli` 빌드·테스트·lint |
| Markdown | 계획, 지식, 지속형 역할, run 상태와 사람이 읽는 지침의 정본 |
| YAML·TOML | setup 답변, typed 설정, 승인 ledger와 ownership manifest |
| JSON Schema Draft 2020-12 | action, role, run, judge, capability 등 provider-neutral machine contract |
| Copier 9.17.0 + Jinja templates | authoring-time template와 Rust renderer parity fixture |
| Python 3.13 | CI의 Copier·schema conformance test 전용 |
| SQLite | 계획된 재생성 가능 로컬 FTS index; 정본이나 Git 추적 대상이 아님 |
| GitHub Actions | Linux·macOS·Windows Rust 검증과 Copier/schema conformance |
| GitHub Releases | 향후 signed CLI와 release bundle 배포 대상 |

지원 대상은 macOS와 Windows CLI다. Codex·Claude Code·Gemini Antigravity는 host adapter 대상이며, OMX와 OMC는 선택적 orchestration layer다. 어느 host나 orchestration plugin도 Hive의 필수 build dependency가 아니다.

## 의존성

현재 Rust runtime은 외부 crate를 사용하지 않는다. `hive-cli`는 workspace 내부의 `hive-core`에만 의존한다.

| 범위 | 고정 의존성 | 목적 |
| --- | --- | --- |
| Rust toolchain | `stable`, `rustfmt`, `clippy` | build, format, lint와 unit test |
| Template authoring·CI | `copier==9.17.0` | template render와 fixture parity |
| Schema test | `jsonschema[format]==4.25.1` | JSON Schema meta/instance 검증 |
| YAML test | `PyYAML==6.0.2` | setup·projection fixture 검증 |
| CI | `actions/checkout@v7.0.1` | repository checkout; commit SHA로 고정 |
| CI | `actions/setup-python@v7.0.0` | Python 3.13 환경; commit SHA로 고정 |
| CI | `dtolnay/rust-toolchain` | Rust stable 환경; commit SHA로 고정 |

정확한 CI pin은 [`.github/workflows/ci.yml`](.github/workflows/ci.yml), template pin은 [`copier.yml`](copier.yml), Rust 구성은 [`rust-toolchain.toml`](rust-toolchain.toml)에서 관리한다.

## 저장소 구조

```text
.
├── crates/
│   ├── hive-core/       # provider-neutral invariant와 target safety
│   └── hive-cli/        # `hive` CLI entry point
├── harness/             # 출하할 template, Skill, profile, projection source
├── schemas/             # provider-neutral JSON Schema contract
├── tests/               # Copier fixture와 conformance test
├── LICENSE              # GitHub가 감지하는 Apache-2.0 전문
├── LICENSES/            # REUSE용 Apache-2.0 canonical 전문
├── REUSE.toml           # file-scope license mapping
├── docs/
│   ├── architecture/    # 현재 설계
│   ├── decisions/       # ADR와 제품 결정
│   ├── plans/PLAN.md    # 유일한 active plan
│   └── state/CURRENT.md # evidence-backed 현재 handoff
├── .agents/             # Hive source 개발 전용 지침; 출하 대상 아님
└── .github/workflows/   # cross-platform CI
```

## 개발과 검증

```bash
cargo build --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p hive-cli -- doctor
cargo run -p hive-cli -- check-target /path/to/consumer-project
```

CI는 Copier default·hostile fixture, 잘못된 host/orchestrator 조합, schema instance, 지속형 role materialization, optional Skill consent 변조도 함께 검증한다.

## Git workflow

- `main`: 보호된 안정 기준. Pull Request와 필수 CI를 통과해야 하며 삭제와 force push를 금지한다.
- `develop`: 일반 개발과 통합

장기 branch는 이 둘만 사용하며, 일반 변경은 `develop`에서 진행한 뒤 Pull Request로 `main`에 반영한다.

## 현재 상태

Phase 0 source scaffold와 contract baseline이 완료되었다. 다음 단계는 Rust renderer, staging·ownership 검증과 `hive setup --dry-run|apply|validate` 구현이다. 자세한 내용은 [`docs/plans/PLAN.md`](docs/plans/PLAN.md)와 [`docs/state/CURRENT.md`](docs/state/CURRENT.md)를 참고한다.

## 라이선스

Aigent Hive의 CLI, source, 출하 template과 생성된 Hive 소유 material은 모두 [`Apache-2.0`](LICENSE)으로 배포한다. 상용·비공개 제품에서 사용·수정·배포할 수 있으며, 저작권·라이선스 고지와 변경 사항을 보존해야 한다. Apache-2.0의 명시적 특허 허여와 방어 조항도 적용된다.

소비자 프로젝트의 기존 source, 문서, 설정과 data는 Aigent Hive가 재라이선스하지 않는다. 생성된 harness에는 동일한 Apache-2.0 전문이 `.hive/LICENSE-AIGENT-HIVE.txt`로 포함된다. 자세한 적용 범위는 [`docs/licensing.md`](docs/licensing.md)와 [`REUSE.toml`](REUSE.toml)에 정의한다.
