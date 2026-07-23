# 현재 상태

- 기준일: 2026-07-23
- active plan: [`../plans/PLAN.md`](../plans/PLAN.md)
- plan revision: 1.5
- product version: `0.1.0` barebone source baseline
- phase: Phase 0 완료 — Phase 1 진입 준비
- Git: `develop`, 장기 branch는 `main`/`develop` 두 개만 사용
- remote: [`gvm1229/aigent-hive`](https://github.com/gvm1229/aigent-hive), default branch `main`

`0.1.0`은 Hive를 개발하기 위한 source scaffold와 contract baseline이다. 설치 가능한 실제 consumer harness, production renderer, host projection과 updater는 아직 구현되지 않았으며 지원되는 것으로 표시하지 않는다.

## 구현된 기반

- Hive 개발용 `AGENTS.md`와 `.agents/directives/`
- source/release/consumer 경계와 `hive-source.json`
- Rust workspace의 `hive-core`, `hive-cli` skeleton
- source-root guard와 `doctor/check-target`
- Copier 9.17.0 authoring·CI template
- 출하용 `setup-harness` Skill source
- setup/action/role/run/judge/capability machine contract schema
- persistent-role, knowledge-scope, optional-Skill consent projection
- setup-time canonical role materialization과 RFC 8785 Skill consent contract
- Markdown/SQLite, orchestration, Git workflow ADR
- 전체 source와 harness의 `Apache-2.0` REUSE 3.3 license contract
- Cargo workspace product version `0.1.0`
- `main/develop` 전용 Git 규칙과 cross-platform Rust·Copier/schema CI

## v1.5 확정 설계

- setup에서 `orchestration_layer`를 묻지 않고 Codex의 compatible OMX, Claude의 compatible OMC를 host-native보다 우선
- resolved owner는 run별 evidence digest와 함께 고정하며 mid-run hidden fallback 금지
- approved Skill은 narrow description으로 자동 선택하고 OMX/OMC capability를 Hive duplicate보다 우선
- prompt 작성·정제 전용 Skill 이름은 `hive-prompt-refine`; `refine-only`가 기본이며 hidden prompt rewrite 금지
- external capability가 conclusively absent일 때만 capability/event/path/digest를 설명한 뒤 optional fallback hook 승인 요청
- hook 거절은 정상 지원 상태이며, external capability detected/incompatible/unknown이면 Hive hook 설치 0개
- product version은 feature→minor, compatible quick bugfix→patch; major는 exact user instruction과 human confirmation 없이는 증가 금지

현재 Copier/setup schema/template에는 이전 `orchestration_layer` answer가 남아 있다. 이는 구현된 v0.1.0 baseline의 현재 상태이며 Phase 1에서 migration과 parity fixture를 포함해 제거한다.

## 검증 상태

- 임시 Rust stable 1.97.1 환경: `cargo fmt --check`, Clippy `-D warnings`, 4개 unit test PASS
- CLI smoke: `doctor`, 일반 target 허용, source root 거부 PASS
- 임시 Copier 9.17.0 환경: default 및 hostile-string/typed-answer render PASS
- setup answer schema, TOML/YAML parse와 role/scope/consent projection parity PASS
- invalid Codex+OMC 조합 staging 전 거부 PASS
- 7개 JSON Schema meta-validation과 대표 action/role/run/judge/capability instance PASS
- role materialization known-answer/idempotency와 Skill consent tamper fixture PASS
- Copier default·hostile render에서 `.hive/LICENSE-AIGENT-HIVE.txt` Apache 전문 일치와 consumer root license 불변 PASS
- root `LICENSE`, `harness/LICENSE`와 canonical Apache-2.0 전문의 byte parity PASS
- REUSE 6.2.0 lint: 74/74 file copyright·license mapping, missing/unused/bad license 0건 PASS
- `setup-harness` Skill validator PASS
- 적대적 scaffold 재검토: initial Git bootstrap APPROVE
- 적대적 v1.4 baseline plan 재검토: APPROVE
- v1.5 structural validation: local Markdown link 7개 문서 PASS, Stage 0~11 연속성 PASS, `hive-prompt-refine` naming/conditional hook consent contract PASS
- version parity: Cargo workspace, Cargo.lock, README와 CURRENT 모두 `0.1.0`
- v1.5 `git diff --check` PASS
- `develop` 구현 기준선 CI: Linux/macOS/Windows Rust와 Copier/schema conformance PASS ([run 29992480995](https://github.com/gvm1229/aigent-hive/actions/runs/29992480995))
- `main` merge 기준선 CI: Linux/macOS/Windows Rust와 Copier/schema conformance PASS ([run 29992271536](https://github.com/gvm1229/aigent-hive/actions/runs/29992271536))
- GitHub remote branch는 `main`과 `develop` 두 개, default는 `main`

## 다음 작업

1. Phase 1에서 `orchestration_layer` setup answer 제거와 answer migration
2. OMX/OMC capability resolver 및 fallback hook consent contract
3. `hive-render` crate, staging ownership과 shared marker conformance
4. Copier/Rust static parity와 role materializer parity
5. `hive setup --dry-run|apply|validate`
