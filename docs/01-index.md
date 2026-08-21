# Aigent Hive 전체 문서 색인

[문서 홈](00-home.md)에서 목적별 탐색 가능. 이 문서는 tracked `docs/` Markdown의
current catalog.

## Overview·README

| 문서 | 설명 |
| --- | --- |
| [Overview 안내](overview/README.md) | 제품 설명 영역 MOC |
| [제품 개요](overview/product.md) | 목적·지원 범위·원칙·기능 |
| [스킬 모음](skills.md) | Source·product Skill 이름·기능·사용 예시 |
| [English README](../README.md) | 간결한 English 입구 |
| [한국어 README](readme/README.ko.md) | 간결한 한국어 입구 |
| [License](licensing.md) | Apache-2.0 적용 범위 |
| [Guidance schema](guidance-schema.md) | Consumer shared marker contract |

## Architecture

| 문서 | 설명 |
| --- | --- |
| [Architecture 안내](architecture/README.md) | Architecture MOC |
| [Source layout](architecture/source-layout.md) | Source·release·consumer tree와 crate |
| [Role lifecycle](architecture/role-lifecycle.md) | Persistent role·handoff |
| [Run lifecycle](architecture/run-lifecycle.md) | Checkpoint·event·scheduler·receipt·cancel·resume |
| [Skill consent](architecture/skill-consent.md) | Optional Skill approval |
| [Hook consent](architecture/hook-consent.md) | Fallback hook approval |
| [Judge trust](architecture/judge-trust-boundary.md) | Clean-context Ed25519 quorum |
| [Release·update trust](architecture/release-update-trust-boundary.md) | Attestation·local integrity·migration·recovery |

## Decisions

| 문서 | 설명 |
| --- | --- |
| [Decision 안내](decisions/README.md) | ADR MOC |
| [Product release decisions](decisions/product-release-decisions.md) | Current product decision summary |
| [ADR-0001](decisions/ADR-0001-source-release-installed-boundary.md) | Source·release·installed 분리 |
| [ADR-0002](decisions/ADR-0002-subscription-host-only.md) | Subscription host-only |
| [ADR-0003](decisions/ADR-0003-markdown-sqlite-boundary.md) | Markdown 정본·SQLite projection |
| [ADR-0004](decisions/ADR-0004-orchestration-ownership.md) | Orchestration ownership |
| [ADR-0005](decisions/ADR-0005-license-boundary.md) | License boundary |
| [ADR-0006](decisions/ADR-0006-version-lifecycle.md) | Version lifecycle |
| [ADR-0007](decisions/ADR-0007-ed25519-judge-trust.md) | Judge trust |
| [ADR-0008](decisions/ADR-0008-release-integrity.md) | Release 출처·local 무결성 |
| [ADR-0009](decisions/ADR-0009-user-plugin-project-knowledge-boundary.md) | User·project knowledge |
| [ADR-0010](decisions/ADR-0010-native-first-usage-sensors.md) | Native-first usage sensor |
| [ADR-0011](decisions/ADR-0011-source-wiki-independence.md) | Source Wiki independence |
| [ADR-0012](decisions/ADR-0012-global-onboarding-shared-index.md) | Global onboarding·shared index |
| [ADR-0013](decisions/ADR-0013-0.8-release-scope.md) | `0.8.0` test distribution |
| [ADR-0014](decisions/ADR-0014-docs-wiki-architecture.md) | `docs/` Wiki architecture |
| [ADR-0015](decisions/ADR-0015-host-native-skill-composition.md) | v0.9 host-native Skill 조합 |
| [ADR-0016](decisions/ADR-0016-global-knowledge-rag.md) | v0.9 전역 knowledge RAG |
| [ADR-0017](decisions/ADR-0017-0.9-full-release.md) | `0.9.0` 정식 릴리스 |
| [ADR-0018](decisions/ADR-0018-notion-wiki-backend.md) | Notion Wiki backend |
| [ADR-0019](decisions/ADR-0019-hive-native-iterative-execution.md) | Hive-native 반복 실행 소유권 |

## Guides

| 문서 | 설명 |
| --- | --- |
| [Guide 안내](guides/README.md) | Guide MOC |
| [Development](guides/development.md) | Dependency·build·test |
| [Test lanes](guides/test-lanes.md) | Python 대장·실행 lane·fixture 경계 |
| [Branching](guides/branching-rules.md) | `develop`·`main` integration |
| [Commit](guides/commit-rules.md) | Task별 independent commit |
| [Installed usage guard](guides/installed-usage-guard.md) | 설치본 단일 정책의 source 적용 |
| [Judge attestation](guides/ed25519-judge-attestations.md) | External signature ceremony |
| [Release·update](guides/release-update.md) | Update·candidate·publication procedure |
| [Code signing policy](guides/code-signing-policy.md) | 무료 platform signing 상태·privacy·검증 경계 |
| [npm Trusted Publisher](guides/npm-trusted-publisher.md) | six npm package OIDC 연결·test·stable publication |
| [출시 검증용 빌드](guides/release-verification-builds.md) | Developer test version의 목적·설치·수용 기준 |
| [공개 HTML 디자인 원칙](guides/public-html-design-principles.md) | Hive 안내 HTML의 브랜드·정보 구조·반응형·명령 정확성 기준 |

## Releases

| 문서 | 설명 |
| --- | --- |
| [Release 안내](releases/README.md) | 제품 버전별 출시 안내 MOC |
| [`0.8.0`](releases/0.8.0.md) | npm 시험 배포용 제품 후보 |
| [`0.9.0`](releases/0.9.0.md) | `0.8.0` 대비 변경점·정식 출시 gate |
| [`0.9.3`](releases/0.9.3.md) | 프로젝트 간 지식 접근·자동 공유 정식 출시 |
| [`0.9.4`](releases/0.9.4.md) | Skill 표시·전역 검증·지식 안내·프롬프트 기본값 정식 출시 |

## Research

| 문서 | 설명 |
| --- | --- |
| [Research 안내](research/README.md) | Dated external research MOC |
| [Codex app-server usage](research/codex-app-server-usage-sensor.md) | Codex native sensor |
| [Claude usage](research/claude-code-native-usage-sensor.md) | Claude status-line sensor |
| [Antigravity usage](research/antigravity-native-usage-sensor.md) | Antigravity native surface |
| [CodexBar usage](research/codexbar-usage-sensor.md) | Optional fallback sensor |
| [Codex Skill budget](research/codex-skill-context-budget.md) | Skill metadata·context cost |
| [Plugin host surface](research/user-plugin-host-surfaces.md) | 세 host install surface |
| [SQLite index](research/rusqlite-sqlite-index.md) | `rusqlite`·FTS5 evidence |
| [Knowledge 이식·scan](research/knowledge-portability-ingestion-retrieval.md) | Portable bundle·collection·retrieval evidence |
| [`AI_Learning` 적용 후보](research/ai-learning-hive-application-candidates-2026-08-21.md) | Graphify·Markdown 관계 graph·`0.9.5` 대비 권장 `0.10.0` 범위 |
| [`0.10.0` 후보 검토](research/0.10-backlog-archive-candidate-review-2026-08-22.md) | Backlog 전체·Archive 미완료 checklist의 승격 가치 |
| [v0.9 capability inventory](research/v0.9-omx-omc-capability-inventory.md) | OMX·OMC·Hive `adopt|merge|exclude` 근거표 |
| [Discord·Notion host integration](research/discord-notion-host-integrations.md) | Host plugin·MCP·outbound 알림 경계 |

## Facts

| 문서 | 설명 |
| --- | --- |
| [Atomic fact 안내](facts/README.md) | Fact schema·pair·index contract |
| `facts/en/*.md` | English atomic fact |
| `facts/ko/*.md` | Korean exact pair |

Fact별 catalog는 migration 완료 뒤 이 section과 [Fact 안내](facts/README.md)에 추가.

## State

| 문서 | 설명 |
| --- | --- |
| [State 안내](state/README.md) | State MOC |
| [CURRENT](state/CURRENT.md) | Evidence-backed handoff·next action |
| [Marketing deck record](state/artifacts/aigent-hive-marketing-deck.md) | External artifact locator·resume |

## Archive

| 문서 | 설명 |
| --- | --- |
| [Archive 안내](archive/README.md) | 완료·대체 계획과 과거 상태 |
| [이전 명세](archive/MANIFEST.md) | 이전 경로·digest·대체 정본 |

## Plans

| 문서 | 설명 |
| --- | --- |
| [Plan 안내](plans/README.md) | Plan structure |
| [Active plan](plans/PLAN.md) | Sole active plan entrypoint |
| [Backlog](plans/backlog/README.md) | 버전 비종속 후보 |
| [References](plans/references.md) | Non-normative references |
| [`0.9.5` 출시 마감](archive/plans/releases/0.9.5/release-0.9.5-stable-publication.md) | Windows 공개 안정판 수용 완료 기록 |
| [문서 구조 정리](archive/plans/releases/0.10.0/documentation-structure-0.10.0.md) | Archive·Backlog·현재 정본 축소 완료 기록 |
| [시험 구조 재편](archive/plans/releases/0.10.0/test-organization-0.10.0.md) | 목적별 시험·fixture 완료 기록 |
| [Graphify 지식 graph](archive/plans/releases/0.10.0/graphify-knowledge-graph-0.10.0.md) | 조사·도입 중단 기록 |
| [작업 자동 분담 조사](archive/plans/releases/0.10.0/host-work-delegation-research-0.10.0.md) | 공식·실제 가능성 조사 완료 기록 |
| [`0.10.0` 관계 graph](plans/active/knowledge-relationship-graph-0.10.0.md) | Markdown 관계·Graphify code-only adapter |
| [`0.10.0` Skill 예약](plans/active/host-owned-skill-reservations-0.10.0.md) | Host-owned Skill 세션 예약 |
| [`0.10.0` nested scan](plans/active/nested-project-knowledge-scan-0.10.0.md) | 상위 Git 저장소 아래 등록 project scan |
| [`0.10.0` 출시](plans/active/release-0.10.0.md) | 번호 시험판·안정판 |
