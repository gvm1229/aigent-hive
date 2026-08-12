# 시험 lane

## 정본

- Python module 대장: `tests/conformance/lanes.toml`
- 실행기: `python scripts/test-lanes.py --list|--lane <name>|--all`
- Rust 단위 시험: `cargo test --workspace --all-targets --all-features --locked`
- Rust 정적 검사: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`

모든 Python `test_*.py`: 하나의 lane 배정. 누락·중복·존재하지 않는 module: 실행 전 실패.
새 module: 대장 배정과 owner·contract·release gate 기록 필수. 기존 시험 삭제: 대체 coverage·baseline·owner 승인 전 금지.

## 분류와 Windows 기준 시간

| Lane | Owner | Contract | Release gate | Windows 시간 |
| --- | --- | --- | --- | ---: |
| documentation | documentation | 문서·개발 검증·개발 설치 | 예 | 1.01초 |
| security | platform-safety | guard·소유권 hostile | 예 | 60.81초 |
| contract | harness-core | CLI·setup·Skill·projection·role | 예 | 262.03초 |
| integration | consumer-lifecycle | user setup·knowledge·project lifecycle | 예 | 141.19초 |
| release | release-engineering | package·update·discovery | 예 | 12.73초 |

2026-08-11 Windows measured Python lane 합계: 477.77초. CI `os × lane` matrix의 Python test body
critical path: 262.03초, 순차 실행 대비 45.2% 단축 모델. checkout·dependency·CLI build 시간: 각 matrix job의
별도 기록 대상.

## Fixture 경계

- repository 안의 disposable consumer fixture: `tests/work/<suite>-<random>`
- `tests/work/`: Git ignore, 정상 종료 cleanup 대상
- host·filesystem hostile Rust unit fixture: OS temporary directory 유지, repository tree 생성 0건
- Phase 4: `tests/hive-phase4-*` 생성 금지

## CI

- `rust`: format·Clippy·workspace Rust unit
- `conformance`: `os × lane` matrix, selected lane만 실행
- `copier`: Copier render·schema·문서 gate

설치 product usage guard 회귀: `test_phase7_usage_control.py`에서 configured project,
`hive-source.json` source workspace, non-Hive folder 대상 분류 검증.
