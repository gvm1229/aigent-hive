# 제품·배포 결정

기준일: 2026-08-22

| 영역 | 결정 |
| --- | --- |
| 구현 언어 | Rust stable, Cargo workspace |
| 제품 형태 | macOS·Linux·Windows CLI-first, 별도 GUI 없음 |
| 실행 경계 | 사용자가 로그인한 정액제 subscription host 위에서만 동작 |
| API | model-provider API 호출·SDK·API key 전부 금지 |
| 소스와 출하 | Hive source, release bundle, consumer harness 분리 |
| setup template | Copier 9.17.0을 authoring·CI에 사용, 소비자 runtime dependency 금지 |
| canonical data | Source Wiki·role·run·plan은 Markdown. Consumer Wiki는 `markdown|notion` user-scope backend 중 하나. Setup/config/approval은 tracked YAML/TOML, Raw는 허용된 source object |
| SQLite | `~/.hive/index/hive.sqlite3` 단일 disposable projection. Markdown mode는 무네트워크 rebuild, Notion mode는 selected remote scope 기반 rebuild. Project DB 없음 |
| v0.9 knowledge RAG | 모든 질문의 simple-question 이전 bounded retrieval, named project scope, durable user fact·preference·workflow mandatory write, citation-ready chunk·score·locator 반환. Selected backend 정본 우선, SQLite는 incremental FTS5 RAG projection과 measured deficiency 이후 optional local vector만 허용 |
| Notion Wiki backend | Notion 유일 정본·active local Wiki Markdown 0건·SQLite changed-only projection. Official plugin/app → hosted MCP → consented REST, 매 turn freshness gate, Notion-first write, Webhook·Notion AI 이중 검색·양방향 Markdown sync 0건 |
| Discord integration | 초기 범위는 usage guard 중단의 optional outbound webhook. Claude inbound는 official Discord Channel 위임, Codex inbound continuation은 official capability 전 `unsupported` |
| v0.9 knowledge portability·scan | SQLite 복사 대신 checksummed `.hivekb` canonical bundle export·import, 고정 normalized table의 `collection_id`, explicit `knowledge-import`, 기존 `knowledge-recall` Skill의 bounded automatic retrieval. Secret·confidential·runtime·absolute path·retrieved instruction authority 제외 |
| Global onboarding | Minimal bootstrap 뒤 mandatory `user-setup`; 첫 질문은 language, 이후 모든 질문과 host 지침은 선택 언어. Wiki language·profile·persona·host·Skill·Wiki·usage·update-check preference 정본은 `~/.hive/config/user-setup.yml` |
| orchestration | Hive가 iterative judgment·logical scheduler·lease·receipt·cancel·team·multi-goal state 소유, host가 declarative envelope의 model·subagent 실행 소유 |
| orchestration 독립 | 신규 workflow의 OMX/OMC functional dependency 없음. Legacy external-owner run은 read-only provenance, migration은 새 Hive-native identity 생성 |
| runtime 관찰 | active host capability metadata, side-effect-free public `--version`, pinned-qualified usage sensor의 fixed-argv·JSON-RPC read만 허용; foreign state·provider credential read 금지 |
| runtime 경계 | Hive-native plan·Ralph급 loop·team·multi-goal 구현 허용. Provider session engine·model runtime·direct process launcher 금지 |
| Skill | `0.8.0` target은 setup/reconfigure core + recommended suite 또는 개별 selected built-in; optional third-party는 이름·source·revision·content digest·권한의 개별 수동 승인 |
| Skill 정본 | Product-only built-in Skill은 `harness/skills/` 정본. Source 개발은 설치 product Skill·tracked repository directive와 명시 유지보수자 요청의 비출하 `update-summary` source-project Skill 1건 사용 |
| Source docs Wiki | `docs/` human graph와 tracked `docs/facts/en/`·`ko/` atomic pair 정본, `omx_wiki/`·`.omx/wiki/`·consumer `.hive/knowledge/` 금지, SQLite는 ignored source projection, OMX/OMC retirement 시 knowledge migration 0건 |
| Wiki autocapture | Wiki enabled 상태의 material task 종료 전 agent-reviewed task fact 기록. Outcome·tool/project·criteria·originating request summary만 bounded capture, exact request는 explicit retention intent 필요, raw transcript·hook·tool output·runtime ingestion 금지 |
| prompt refine | `prompt-refine`; 명시적 작성·정제 intent와 materially ambiguous ordinary work에서 자동 선택, `refine-only` 기본. Refined prompt 제시 뒤 exact 사용자 승인까지 정지. Same-request 실행은 explicit `--run`만 허용, simple/editless question·clear work·hidden rewrite·prompt-classifier hook 제외 |
| 프롬프트 언어 | 응답 언어와 분리. Hive 작성·개선·복사용 프롬프트는 현재 프롬프트 언어의 명시 요청이 없으면 영어. 명시 언어는 기본값보다 우선. 설명·질문은 선택 응답 언어 유지 |
| optional hooks | host가 exact integrity event를 지원하고 사용자가 capability/event/path/digest를 승인한 경우에만 project-local hook 허용 |
| 사용량 | Global setup 핵심 기능, 활성화 권장. 신속 기본 profile은 남은 사용량 `20%`, custom setup은 사용자 선택 한도. Registered project별 더 이른 중지 override와 단일 product `usage-guard`; Codex app-server JSON-RPC, Claude Code status-line JSON capture, 향후 qualified Antigravity structured surface를 native primary로 사용 |
| Claude sensor ownership | Plugin executable만 제공; user가 Claude host의 `/statusline`으로 opt-in하며 Hive의 `~/.claude/settings.json` mutation 없음, existing status line non-clobber |
| Antigravity sensor truth | Official structured surface 확인 전 `native=unsupported`; interactive TUI·private LSP/HTTP·credential·browser state parsing 금지 |
| sensor fallback 설치 | Normal setup·reconfigure의 CodexBar 노출 `0건`. Setup 후 또는 첫 실제 guard check의 native-only probe가 unavailable·unsupported·malformed를 확정한 때만 필요성·대상·command preview 제공 후 current-action explicit consent 요청. Native success·limited: 질문·호출 `0건`; integrity failure: fallback 없이 fail-closed |
| dispatch replay | Hive는 같은 authorization 재발급을 거부하지만 capture된 JSON의 외부 replay는 차단하지 못함; host/orchestration owner가 authorization ID를 한 번만 소비 |
| orchestration authority | Selected session pointer는 selector only. Mutation은 exact target·event head·control epoch·request digest·external trust root의 one-time authority 필수 |
| orchestration uncertainty | Host idempotency·receipt proof 부재 시 automatic reclaim 금지와 `dispatch-uncertain` 중지 |
| judge | verdict 전 digest-bound assignment, exact roster/slot/instance/evidence/timestamp, requester/task-agent 배제, verdict 후 별도 human approval; elevated 2/3, critical 3/3+human |
| judge 신뢰 | consumer target 밖의 agent-write-denied TOML public-key trust root, purpose-bound detached Ed25519 signature와 aggregate-only output; Hive는 strict verification만 수행하고 private-key custody/signing은 외부 authority가 소유 |
| release 신뢰 | Protected `main` exact tag, same-candidate GitHub Release, SHA-256 sidecar, GitHub artifact attestation, npm Trusted Publishing OIDC·registry provenance. GitHub stable environment의 human approval 한 번, npm 별도 승인 없음 |
| 안정판 Discord 구독자 알림 | GitHub Release 생성 성공 뒤 `release-publication` 환경 비밀 값으로 Discord webhook 두 번 전송. 한국어 배너 PNG 성공 뒤 `update-summary` 결과 전송, 시험판 전송 없음 |
| GitHub Release 설명 | `docs/releases/<product-version>.md` 정본의 English-first·Korean-second 이중 언어 설명. 영어는 ASD-STE100 Simplified Technical English, 한국어는 한국어 언어 계약 적용. 두 section의 기능·호환성·검증 경계 동등성 필수 |
| platform signing | macOS explicit ad-hoc seal은 publisher identity·notarization 아님을 공개. Windows unsigned 공개. Developer ID·notarization, Authenticode·SignPath는 optional enhancement이며 stable gate 아님 |
| release trust 폐기 | Release TUF·offline root·threshold signer·external authorization ceremony·platform certificate evidence gate 삭제. Judge external trust root와 frozen historical release base는 별도 경계 |
| user projection purge | Setup·update·uninstall은 authenticated inventory와 retired-name ledger로 Hive-owned projection을 current closure에 수렴. 중첩 빈 directory·owned transient state 제거, knowledge·saved preference·foreign byte·developer rollback state 보존 |
| backup | update 전 canonical config/team/run/knowledge와 changed path snapshot, SQLite/runtime/backup/foreign orchestration 제외, exact 7일 경계 이후 validated unreferenced backup만 정리 |
| 저장소 | 비기밀 canonical source와 data는 Git 추적, runtime/cache/SQLite 제외 |
| 배포 정본 | `0.8.0` npm 시험 배포 이력 보존. `0.9.0` 정식 릴리스는 protected `main` exact final candidate·annotated tag·GitHub Release·npm·direct installer를 동일 native binary와 digest로 결합 |
| npm 설치 | Public `aigent-hive` umbrella + exact `@aigent-hive/*` platform package. `0.8.0|latest`를 기본 설치로 제공하고 기존 `0.8.0-test.1|test`는 immutable 검증 이력으로 유지. 최초 등록만 임시 `NPM_TOKEN`, 이후 6개 Trusted Publisher·OIDC 전용 |
| update 확인 | Global setup explicit opt-in, 성공 확인 24시간 throttle, offline 실패는 성공 시각 미기록·다음 host session 재시도, 확인만으로 install 금지 |
| binary update | Bare `hive update`가 즉시 확인하고 새 version이 있으면 선택 언어로 package owner·exact target·authenticated saved host scope의 post-update projection refresh를 함께 표시. 명시적 수락 뒤 authenticated install owner의 exact adapter 실행, target binary 재검증 뒤 새 executable로 saved scope만 `hive install --scope user --hosts <resolved-hosts> --apply --output json` 실행·validate. valid setup·authenticated host manifest 교집합 부재·invalid: default host 없이 binary-only 결과와 recovery command. `--check` 설치 금지 |
| host projection | User `~/.agents/directives`·`~/.agents/skills` provider-neutral projection + selected host의 thin native adapter; project Codex·Antigravity `.agents/skills`, Claude `.claude/skills`; foreign byte 보존 |
| `0.10.0` host-owned Skill 세션 예약 | 구현 목표: 예약은 대상 경로 접근 아닌 `.hive/runtime` 충돌 조정 기록. Codex·Antigravity는 `.agents/skills/<safe-skill>/...`, Claude는 `.claude/skills/<safe-skill>/...`만 예약 허용. 다른 host 경로는 `hive.session-host-owned-namespace`, live·unverifiable reservation만 세션 해결 안내 |
| `0.10.0` nested project scan | 등록 project root가 상위 Git repository의 하위 폴더여도 해당 root 안에서 knowledge scan 허용. Sibling read·write, 전역 Git 설정 mutation, symlink·junction·reparse point 탈출 금지 |
| `0.10.0` 관계·검색 | Markdown 명시 관계는 Hive-native derived graph, code 관계는 승인형 Graphify `0.9.47` full-rebuild `--code-only`, 직접 사실은 기존 SQLite FTS. Metadata-first retrieval·scope별 물리 격리·drift gate·JSON/HTML export 포함. Semantic vector는 Qdrant Edge·SQLite engine·local embedding hard gate 통과 뒤 optional hybrid adapter, 실패 시 dependency `0건` |
| role/run | shared role HANDOFF, PLAN-derived criterion, exact evidence locator, immutable owner pin, sensor-independent manual과 one-role usage-guarded automatic no-spawn resume |
| 현재 버전 | 정식 릴리스 준비 target `0.9.0`; root Cargo workspace version과 `workspace.metadata.hive.release-date`가 정본 |
| `0.9.x` release line | `0.9.5`가 마지막 `0.9.x` release. `0.9.6` 미출시. 이후 수정·기능은 `0.10.0` 범위에서 수용 |
| 버전 증가 | feature는 원칙적으로 `Y`, compatible quick bugfix는 `Z`; `X`는 exact target을 사용자가 명시하고 human confirmation한 경우에만 |
| 호환성 | major `0`을 포함해 같은 major만 non-breaking upgrade 보장 |
| cross-major | 사전 경고, 자동 migration, project/docs/preferences 보존, SQLite rebuild |
| release workflow | `develop` 사전 후보 뒤 protected `main` final candidate를 한 번 build. 5개 target·6개 npm·3개 installer의 digest·attestation·byte identity 검증 뒤 rebuild 없이 annotated `v0.9.0`·GitHub Release·npm `latest` publication |
| release compatibility qualification | Release metadata의 declared compatibility range: executable contract. Candidate build 전 compiled CLI·release bundle·npm/direct package matrix, exact historical base·preservation·negative recovery evidence와 digest-bound coverage report 필수. Public test: prior stable·oldest distinct full-ledger project·선택 host state의 representative acceptance. Stable promotion: accepted public test·coverage report·artifact digest 동일성 전제 |
| `0.9.5` 현 작업 경계 | 유지보수자 명시 지시에 따라 local implementation·compiled/package qualification까지만 수행. `HBC95-006`·`AUP95-007`·`REL95-001–006` public test·protected `main` 통합·stable publication·stable 설치는 재승인 전 보류. macOS source qualification은 `MAC95-001`로 유지보수자 external evidence 대기 |
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

`0.9.0` 정식 GitHub·npm release identity와 minimal trust:
[`ADR-0017`](ADR-0017-0.9-full-release.md).

Notion canonical backend·SQLite projection·Discord outbound 경계:
[`ADR-0018`](ADR-0018-notion-wiki-backend.md).

Hive-native iterative·team·multi-goal execution과 OMX·OMC 신규 dependency 제거:
[`ADR-0019`](ADR-0019-hive-native-iterative-execution.md).

`0.10.0` 관계·검색·nested scan·Skill 예약의 최종 포함·제외 범위:
[`ADR-0020`](ADR-0020-0.10.0-product-scope.md).

## 미확정 항목

- Antigravity의 official machine-readable structured quota surface
- Optional Developer ID·notarization 또는 Authenticode 도입 시점

Judge identity authentication은
[`ADR-0007`](ADR-0007-ed25519-judge-trust.md)의 external protected trust root와
detached Ed25519 attestation으로 확정. Hive는 judge private key를 소유하지
않으며 release artifact trust와도 분리. Release publication은
[`ADR-0017`](ADR-0017-0.9-full-release.md)의 protected main·same-byte GitHub Release·
npm OIDC 최소 계약 적용.
