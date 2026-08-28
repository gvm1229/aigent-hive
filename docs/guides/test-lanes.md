# 시험 lane

## 정본

- Python module 대장: `tests/conformance/lanes.toml`
- 실행기: `python scripts/test-lanes.py --list|--lane <name>|--all`
- Rust 단위 시험: `cargo test --workspace --all-targets --all-features --locked`
- Rust 정적 검사: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`

모든 Python `test_*.py`: 하나의 lane 배정. 누락·중복·존재하지 않는 module: 실행 전 실패.
새 module: 대장 배정과 owner·contract·release gate 기록 필수. 기존 시험 삭제: 대체 coverage·baseline·owner 승인 전 금지.

## 목적별 위치

| 경로 | 목적 |
| --- | --- |
| `tests/conformance/documentation/` | 문서·개발 검증 |
| `tests/conformance/security/` | 소유권·비밀·사용량 보호 |
| `tests/conformance/contracts/` | CLI·setup·Skill·role·schema 계약 |
| `tests/conformance/integration/` | 설치·project·knowledge 수명주기 |
| `tests/conformance/release/` | package·update·publication |
| `tests/conformance/support/` | 공유 test helper |

## 분류와 Windows 기준 시간

| Lane | Owner | Contract | Release gate | Windows 시간 |
| --- | --- | --- | --- | ---: |
| documentation | documentation | 문서·개발 검증·개발 설치 | 예 | 1.47초 |
| security | platform-safety | guard·소유권 hostile | 예 | 89.41초 |
| contract | harness-core | CLI·setup·Skill·projection·role | 예 | 356.39초 |
| integration | consumer-lifecycle | user setup·knowledge·project lifecycle | 예 | 198.81초 |
| release | release-engineering | package·update·discovery | 예 | 39.59초 |

2026-08-20 Windows measured Python lane 합계: 685.67초. CI lane matrix의 Python test body
critical path: 356.39초, 순차 실행 대비 48.0% 단축 모델. checkout·dependency·CLI build 시간: 각 matrix job의
별도 기록 대상.

## Fixture 경계

- repository 안의 disposable consumer fixture: `tests/work/<suite>-<random>`
- `tests/work/`: Git ignore, 정상 종료 cleanup 대상
- host·filesystem hostile Rust unit fixture: OS temporary directory 유지, repository tree 생성 0건
- Run lifecycle: tracked test 인접 임시 디렉터리 생성 금지

## 결과와 정리

- 실행 기록: [`../../tests/results/README.md`](../../tests/results/README.md)
- 일반 실행: `python scripts/test-artifacts.py run --purpose "시험 목적" --path target/debug --command python scripts/test-lanes.py --lane <name>`
- 결과 기록의 Git 보존·명시 검토 뒤에만 `python scripts/test-artifacts.py cleanup --path <exact-path> --apply` 사용
- 진행 중·72시간 안의 구체적 재사용·실패 재현·원본 근거는 보존. 기간 만료만으로 자동 삭제 없음

## CI

- `rust`: format·Clippy·workspace Rust unit
- `conformance`: `os × lane` matrix, selected lane만 실행
- `copier`: Copier render·schema·문서 gate

설치 product usage guard 회귀: `contracts/test_usage_control.py`에서 configured project,
`hive-source.json` source workspace, non-Hive folder 대상 분류 검증.
