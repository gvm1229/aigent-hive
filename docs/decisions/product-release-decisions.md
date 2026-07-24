# 제품·배포 결정

기준일: 2026-07-24

| 영역 | 결정 |
| --- | --- |
| 구현 언어 | Rust stable, Cargo workspace |
| 제품 형태 | macOS·Windows CLI-first, 별도 GUI 없음 |
| 실행 경계 | 사용자가 로그인한 정액제 subscription host 위에서만 동작 |
| API | model-provider API 호출·SDK·API key 전부 금지 |
| 소스와 출하 | Hive source, release bundle, consumer harness 분리 |
| setup template | Copier 9.17.0을 authoring·CI에 사용, 소비자 runtime dependency 금지 |
| canonical data | 지식·role·run은 Markdown, setup/config/approval은 tracked YAML/TOML, Raw는 허용된 source object |
| SQLite | 삭제 가능한 FTS·tag·link index, Git 제외, 무네트워크 재구축 |
| orchestration | Codex의 compatible OMX, Claude의 compatible OMC를 우선하고 `absent|incompatible|unknown`이면 truthful host native가 소유; setup 선택지와 mid-run switch 없음 |
| runtime 관찰 | active host capability metadata, side-effect-free public `--version`, pinned-qualified usage sensor의 fixed-argv read만 허용; foreign state read 금지 |
| 재구현 금지 | Hive plan·Ralph·team·provider session engine 없음 |
| Skill | exact 10개 implemented built-in을 active projection; update/migrate 2개는 catalog-only; optional은 이름·source·revision·content digest·권한의 개별 수동 승인 후에만 사용 |
| prompt refine | `hive-prompt-refine`; 명시적인 prompt 작성·정제 intent에서만 자동 선택, `refine-only` 기본, hidden rewrite 금지 |
| fallback hooks | OMX/OMC가 conclusively absent이고 사용자가 capability/event/path/digest를 승인한 경우에만 project-local data-integrity hook 허용 |
| 사용량 | installed `usage_stop_remaining_percent`가 권위값; session 절대 우선, session 부재 시에만 weekly fallback; ignored Hive runtime의 prior snapshot으로 monotonicity 검증; exact run revision·active role·brief당 authorization 1개 |
| dispatch replay | Hive는 같은 authorization 재발급을 거부하지만 capture된 JSON의 외부 replay는 차단하지 못함; host/orchestration owner가 authorization ID를 한 번만 소비 |
| judge | verdict 전 digest-bound assignment, exact roster/slot/instance/evidence/timestamp, requester/task-agent 배제, verdict 후 별도 human approval; elevated 2/3, critical 3/3+human |
| judge 신뢰 | consumer target 밖의 agent-write-denied TOML public-key trust root, purpose-bound detached Ed25519 signature와 aggregate-only output; Hive는 strict verification만 수행하고 private-key custody/signing은 외부 authority가 소유 |
| backup | update 전 생성, 최대 7일, Git 제외 |
| 저장소 | 비기밀 canonical source와 data는 Git 추적, runtime/cache/SQLite 제외 |
| 배포 정본 | GitHub Releases |
| host projection | Codex·Antigravity `.agents/skills`, Claude `.claude/skills`; exact Hive Skill file만 관리하고 foreign byte 보존 |
| role/run | shared role HANDOFF, PLAN-derived criterion, exact evidence locator, immutable owner pin, sensor-independent manual과 one-role usage-guarded automatic no-spawn resume |
| 현재 버전 | 마지막 완료 milestone인 Phase 5 usage guard와 authenticated judge quorum `0.6.0`; root Cargo workspace version이 정본 |
| 버전 증가 | feature는 원칙적으로 `Y`, compatible quick bugfix는 `Z`; `X`는 exact target을 사용자가 명시하고 human confirmation한 경우에만 |
| 호환성 | major `0`을 포함해 같은 major만 non-breaking upgrade 보장 |
| cross-major | 사전 경고, 자동 migration, project/docs/preferences 보존, SQLite rebuild |
| Git | `main` 안정, `develop` 일반 개발; `develop → main` PR |
| 라이선스 | CLI/source, `harness/**`와 생성된 Hive 소유 material 모두 `Apache-2.0` |

## 미확정 항목

- Release artifact용 Rust signing library와 정확한 release-key custody 구현
- host별 subscription usage sensor
- Homebrew·WinGet 배포 자동화의 첫 release 범위

Judge identity authentication은
[`ADR-0007`](ADR-0007-ed25519-judge-trust.md)의 external protected trust root와
detached Ed25519 attestation으로 확정했다. Hive는 judge private key를 소유하지
않으며 release artifact signing key 선택과도 분리한다. 다른 미확정 항목은 지원된
것으로 표시하지 않는다.
