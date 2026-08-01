# 제품·배포 결정

기준일: 2026-08-01

| 영역 | 결정 |
| --- | --- |
| 구현 언어 | Rust stable, Cargo workspace |
| 제품 형태 | macOS·Linux·Windows CLI-first, 별도 GUI 없음 |
| 실행 경계 | 사용자가 로그인한 정액제 subscription host 위에서만 동작 |
| API | model-provider API 호출·SDK·API key 전부 금지 |
| 소스와 출하 | Hive source, release bundle, consumer harness 분리 |
| setup template | Copier 9.17.0을 authoring·CI에 사용, 소비자 runtime dependency 금지 |
| canonical data | 지식·role·run은 Markdown, setup/config/approval은 tracked YAML/TOML, Raw는 허용된 source object |
| SQLite | `0.8.0` target은 `~/.hive/index/hive.sqlite3` 단일 projection; user Wiki + 등록 project Wiki 기반 무네트워크 rebuild, project DB 없음 |
| v0.9 knowledge RAG | 모든 질문의 simple-question 이전 bounded retrieval, named project scope, durable user fact·preference·workflow mandatory write, citation-ready chunk·score·locator 반환. Markdown 정본 유지, SQLite는 incremental FTS5 RAG projection과 measured deficiency 이후 optional local vector만 허용 |
| v0.9 knowledge portability·scan | SQLite 복사 대신 checksummed `.hivekb` canonical bundle export·import, 고정 normalized table의 `collection_id`, explicit `hive-knowledge-scan`, 기존 query Skill의 bounded automatic retrieval. Secret·confidential·runtime·absolute path·retrieved instruction authority 제외 |
| Global onboarding | Minimal bootstrap 뒤 mandatory `setup-hive`; 첫 질문은 language, 이후 모든 질문과 host 지침은 선택 언어. Wiki language·profile·persona·host·Skill·Wiki·usage·update-check preference 정본은 `~/.hive/config/user-setup.yml` |
| orchestration | 검증된 host-native capability가 기본 소유. OMX·OMC는 사용자 명시 선택 또는 고정된 `0.8.x` 실행의 호환 owner만 허용하며 mid-run switch 없음 |
| orchestration 독립 | OMX/OMC는 현재 replaceable compatibility dependency; 장기적으로 host-native·provider-neutral capability로 대체 후 제거, canonical data·path·schema·Skill identity 의존 없음 |
| runtime 관찰 | active host capability metadata, side-effect-free public `--version`, pinned-qualified usage sensor의 fixed-argv·JSON-RPC read만 허용; foreign state·provider credential read 금지 |
| 재구현 금지 | Hive plan·Ralph·team·provider session engine 없음 |
| Skill | `0.8.0` target은 setup/reconfigure core + recommended suite 또는 개별 selected built-in; optional third-party는 이름·source·revision·content digest·권한의 개별 수동 승인 |
| Skill 양방향 reuse | Hive-owned source↔consumer Skill reuse 허용; shared canonical은 `harness/skills/`, source는 exact `.agents/skills/` projection, scope·safety·consent·conformance review 필수 |
| Source docs Wiki | `docs/` human graph와 tracked `docs/facts/en/`·`ko/` atomic pair 정본, `omx_wiki/`·`.omx/wiki/`·consumer `.hive/knowledge/` 금지, SQLite는 ignored source projection, OMX/OMC retirement 시 knowledge migration 0건 |
| Wiki autocapture | Wiki enabled 상태의 material task 종료 전 agent-reviewed task fact 기록. Outcome·tool/project·criteria·originating request summary만 bounded capture, exact request는 explicit retention intent 필요, raw transcript·hook·tool output·runtime ingestion 금지 |
| prompt refine | `hive-prompt-refine`; 명시적인 prompt 작성·정제 intent에서만 자동 선택, `refine-only` 기본. 일반 요청이 모호하거나 핵심 세부가 부족하면 작업을 막지 않는 한 줄 optional refine 제안, hidden·automatic rewrite 금지 |
| optional hooks | host가 exact integrity event를 지원하고 사용자가 capability/event/path/digest를 승인한 경우에만 project-local hook 허용 |
| 사용량 | Global setup explicit opt-in과 enabled 기본 remaining `20%`; Codex app-server JSON-RPC, Claude Code status-line JSON capture, 향후 qualified Antigravity structured surface를 native primary로 사용; CodexBar는 세 provider 모두 explicit-consent fallback-only |
| Claude sensor ownership | Plugin executable만 제공; user가 Claude host의 `/statusline`으로 opt-in하며 Hive의 `~/.claude/settings.json` mutation 없음, existing status line non-clobber |
| Antigravity sensor truth | Official structured surface 확인 전 `native=unsupported`; interactive TUI·private LSP/HTTP·credential·browser state parsing 금지 |
| sensor fallback 설치 | Active-host native sensor 불가와 CodexBar 미설치 때만 필요성·대상·command preview 제공 후 current action explicit consent 요청; 수락 시 supported package manager 사용, 거절 시 core 유지와 automatic dispatch fail-closed |
| dispatch replay | Hive는 같은 authorization 재발급을 거부하지만 capture된 JSON의 외부 replay는 차단하지 못함; host/orchestration owner가 authorization ID를 한 번만 소비 |
| judge | verdict 전 digest-bound assignment, exact roster/slot/instance/evidence/timestamp, requester/task-agent 배제, verdict 후 별도 human approval; elevated 2/3, critical 3/3+human |
| judge 신뢰 | consumer target 밖의 agent-write-denied TOML public-key trust root, purpose-bound detached Ed25519 signature와 aggregate-only output; Hive는 strict verification만 수행하고 private-key custody/signing은 외부 authority가 소유 |
| release 신뢰 | TUF 1.0.31-compatible offline root 2-of-3와 분리된 targets/snapshot/timestamp, 전역 unique role key, strict Ed25519 verification, old+new root rotation, semantic in-toto/SLSA·platform evidence; signing/private key는 Hive 밖의 external authority |
| backup | update 전 canonical config/team/run/knowledge와 changed path snapshot, SQLite/runtime/backup/foreign orchestration 제외, exact 7일 경계 이후 validated unreferenced backup만 정리 |
| 저장소 | 비기밀 canonical source와 data는 Git 추적, runtime/cache/SQLite 제외 |
| 배포 정본 | `0.8.0` npm 시험 배포 이력 보존. `0.9.0` 정식 릴리스는 protected `main` exact final candidate·annotated tag·GitHub Release·npm·direct installer를 동일 native binary와 digest로 결합 |
| npm 설치 | Public `aigent-hive` umbrella + exact `@aigent-hive/*` platform package. `0.8.0|latest`를 기본 설치로 제공하고 기존 `0.8.0-test.1|test`는 immutable 검증 이력으로 유지. 최초 등록만 임시 `NPM_TOKEN`, 이후 6개 Trusted Publisher·OIDC 전용 |
| update 확인 | Global setup explicit opt-in, 성공 확인 24시간 throttle, offline 실패는 성공 시각 미기록·다음 host session 재시도, 확인만으로 install 금지 |
| binary update | Bare `hive update`가 즉시 확인하고 새 version이 있으면 선택 언어로 질문. 명시적 수락 뒤 authenticated install owner의 exact adapter만 실행 |
| host projection | User `~/.agents/directives`·`~/.agents/skills` provider-neutral projection + selected host의 thin native adapter; project Codex·Antigravity `.agents/skills`, Claude `.claude/skills`; foreign byte 보존 |
| role/run | shared role HANDOFF, PLAN-derived criterion, exact evidence locator, immutable owner pin, sensor-independent manual과 one-role usage-guarded automatic no-spawn resume |
| 현재 버전 | 정식 릴리스 준비 target `0.9.0`; root Cargo workspace version과 `workspace.metadata.hive.release-date`가 정본 |
| 버전 증가 | feature는 원칙적으로 `Y`, compatible quick bugfix는 `Z`; `X`는 exact target을 사용자가 명시하고 human confirmation한 경우에만 |
| 호환성 | major `0`을 포함해 같은 major만 non-breaking upgrade 보장 |
| cross-major | 사전 경고, 자동 migration, project/docs/preferences 보존, SQLite rebuild |
| release workflow | `develop` 사전 후보 뒤 `main` final candidate 재빌드. 5개 target·6개 npm·digest·attestation·OS signing·TUF 검증 뒤 annotated `v0.9.0`·GitHub Release·npm `latest` publication |
| install ownership | Direct receipt binary만 Hive-owned. npm binary는 npm 소유이며 Hive의 직접 덮어쓰기 금지; bare update의 사용자 승인 뒤 exact npm command 위임만 허용. Homebrew·WinGet은 기존 owner 경계 유지 |
| Antigravity plugin ownership | Hive는 `~/.hive/marketplaces/antigravity/` source package만 소유. `agy` staging·import manifest는 host 소유이며 Hive ledger에서 제외. Mutation 전 staging 전체를 authenticated prior와 exact 비교하고 foreign entry는 보존. 신규 rollback은 uninstall, refresh rollback은 prior source 재설치 |
| Git | `develop` 일반 fast-forward direct push, `main` production PR·required checks. `staging`은 명시적 release 필요·승인 때만 생성하고 strict ruleset 적용 |
| 라이선스 | CLI/source, `harness/**`와 생성된 Hive 소유 material 모두 `Apache-2.0` |

Source Wiki의 독립성, OMX Wiki Skill 제외 이유와 장기 OMX/OMC retirement 방향:
[`ADR-0011`](ADR-0011-source-wiki-independence.md).

Global onboarding, Wiki opt-out, selected Skill projection과 user-root 단일 shared index:
[`ADR-0012`](ADR-0012-global-onboarding-shared-index.md).

v0.9 cross-project retrieval, mandatory durable memory, portable bundle·directory scan과 derived RAG index:
[`ADR-0016`](ADR-0016-global-knowledge-rag.md).

`0.9.0` 정식 GitHub·npm release identity, final candidate와 production signing:
[`ADR-0017`](ADR-0017-0.9-full-release.md).

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
