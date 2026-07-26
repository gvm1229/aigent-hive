# 제품·배포 결정

기준일: 2026-07-26

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
| runtime 관찰 | active host capability metadata, side-effect-free public `--version`, pinned-qualified usage sensor의 fixed-argv·JSON-RPC read만 허용; foreign state·provider credential read 금지 |
| 재구현 금지 | Hive plan·Ralph·team·provider session engine 없음 |
| Skill | active projection은 구현 완료 built-in 13개; optional은 이름·source·revision·content digest·권한의 개별 수동 승인 후에만 사용 |
| prompt refine | `hive-prompt-refine`; 명시적인 prompt 작성·정제 intent에서만 자동 선택, `refine-only` 기본, hidden rewrite 금지 |
| fallback hooks | OMX/OMC가 conclusively absent이고 사용자가 capability/event/path/digest를 승인한 경우에만 project-local data-integrity hook 허용 |
| 사용량 | Codex app-server JSON-RPC, Claude Code status-line JSON capture, 향후 qualified Antigravity structured surface를 native primary로 사용; CodexBar는 세 provider 모두 explicit-consent fallback-only; installed `usage_stop_remaining_percent`가 권위값; session 절대 우선, session 부재 시에만 weekly fallback |
| Claude sensor ownership | Plugin executable만 제공; user가 Claude host의 `/statusline`으로 opt-in하며 Hive의 `~/.claude/settings.json` mutation 없음, existing status line non-clobber |
| Antigravity sensor truth | Official structured surface 확인 전 `native=unsupported`; interactive TUI·private LSP/HTTP·credential·browser state parsing 금지 |
| sensor fallback 설치 | Active-host native sensor 불가와 CodexBar 미설치 때만 필요성·대상·command preview 제공 후 current action explicit consent 요청; 수락 시 supported package manager 사용, 거절 시 core 유지와 automatic dispatch fail-closed |
| dispatch replay | Hive는 같은 authorization 재발급을 거부하지만 capture된 JSON의 외부 replay는 차단하지 못함; host/orchestration owner가 authorization ID를 한 번만 소비 |
| judge | verdict 전 digest-bound assignment, exact roster/slot/instance/evidence/timestamp, requester/task-agent 배제, verdict 후 별도 human approval; elevated 2/3, critical 3/3+human |
| judge 신뢰 | consumer target 밖의 agent-write-denied TOML public-key trust root, purpose-bound detached Ed25519 signature와 aggregate-only output; Hive는 strict verification만 수행하고 private-key custody/signing은 외부 authority가 소유 |
| release 신뢰 | TUF 1.0.31-compatible offline root 2-of-3와 분리된 targets/snapshot/timestamp, 전역 unique role key, strict Ed25519 verification, old+new root rotation, semantic in-toto/SLSA·platform evidence; signing/private key는 Hive 밖의 external authority |
| backup | update 전 canonical config/team/run/knowledge와 changed path snapshot, SQLite/runtime/backup/foreign orchestration 제외, exact 7일 경계 이후 validated unreferenced backup만 정리 |
| 저장소 | 비기밀 canonical source와 data는 Git 추적, runtime/cache/SQLite 제외 |
| 배포 정본 | GitHub Releases |
| host projection | Codex·Antigravity `.agents/skills`, Claude `.claude/skills`; exact Hive Skill file만 관리하고 foreign byte 보존 |
| role/run | shared role HANDOFF, PLAN-derived criterion, exact evidence locator, immutable owner pin, sensor-independent manual과 one-role usage-guarded automatic no-spawn resume |
| 현재 버전 | Phase 6 verifier-only signed release와 safe update milestone `0.7.0`; root Cargo workspace version이 정본 |
| 버전 증가 | feature는 원칙적으로 `Y`, compatible quick bugfix는 `Z`; `X`는 exact target을 사용자가 명시하고 human confirmation한 경우에만 |
| 호환성 | major `0`을 포함해 같은 major만 non-breaking upgrade 보장 |
| cross-major | 사전 경고, 자동 migration, project/docs/preferences 보존, SQLite rebuild |
| release workflow | OS-signed candidate build, offline GitHub Sigstore bundle·platform evidence, external TUF authorization과 public publication 분리; candidate는 tag/release 권한 없음 |
| install ownership | fixed official URL+archive allowlist+OS signature를 통과한 direct receipt만 Hive-owned; Homebrew/WinGet binary는 package manager 소유, Hive의 덮어쓰기·manager 실행 금지 |
| Git | `main` 안정, `develop` 일반 개발; `develop → main` PR |
| 라이선스 | CLI/source, `harness/**`와 생성된 Hive 소유 material 모두 `Apache-2.0` |

## 미확정 항목

- Antigravity의 official machine-readable structured quota surface
- Apple/Azure/external TUF signer credential이 provision된 첫 production release 실행

Judge identity authentication은
[`ADR-0007`](ADR-0007-ed25519-judge-trust.md)의 external protected trust root와
detached Ed25519 attestation으로 확정. Hive는 judge private key를 소유하지
않으며 release artifact signing key 선택과도 분리. Release authorization은
[`ADR-0008`](ADR-0008-verifier-only-tuf-updates.md)의 verifier-only TUF/Ed25519
contract로 확정. Production signing/publication은 protected external credential이
실제 provision 전 완료 표시 금지.
