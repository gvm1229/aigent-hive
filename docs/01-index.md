# Aigent Hive 전체 문서 색인

[문서 홈](00-home.md)에서 목적별 탐색 가능. 이 문서는 tracked `docs/` Markdown의
current catalog.

## Overview·README

| 문서 | 설명 |
| --- | --- |
| [Overview 안내](overview/README.md) | 제품 설명 영역 MOC |
| [제품 개요](overview/product.md) | 목적·지원 범위·원칙·기능 |
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
| [Run lifecycle](architecture/run-lifecycle.md) | Checkpoint·resume·owner pin |
| [Skill consent](architecture/skill-consent.md) | Optional Skill approval |
| [Hook consent](architecture/hook-consent.md) | Fallback hook approval |
| [Judge trust](architecture/judge-trust-boundary.md) | Clean-context Ed25519 quorum |
| [Release·update trust](architecture/release-update-trust-boundary.md) | TUF·migration·backup·recovery |

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
| [ADR-0008](decisions/ADR-0008-verifier-only-tuf-updates.md) | Verifier-only TUF update |
| [ADR-0009](decisions/ADR-0009-user-plugin-project-knowledge-boundary.md) | User·project knowledge |
| [ADR-0010](decisions/ADR-0010-native-first-usage-sensors.md) | Native-first usage sensor |
| [ADR-0011](decisions/ADR-0011-source-wiki-independence.md) | Source Wiki independence |
| [ADR-0012](decisions/ADR-0012-global-onboarding-shared-index.md) | Global onboarding·shared index |
| [ADR-0013](decisions/ADR-0013-0.8-release-scope.md) | `0.8.0` test distribution |
| [ADR-0014](decisions/ADR-0014-docs-wiki-architecture.md) | `docs/` Wiki architecture |
| [ADR-0015](decisions/ADR-0015-host-native-skill-composition.md) | v0.9 host-native Skill 조합 |
| [ADR-0016](decisions/ADR-0016-global-knowledge-rag.md) | v0.9 전역 knowledge RAG |

## Guides

| 문서 | 설명 |
| --- | --- |
| [Guide 안내](guides/README.md) | Guide MOC |
| [Development](guides/development.md) | Dependency·build·test |
| [Branching](guides/branching-rules.md) | `develop`·`main` integration |
| [Commit](guides/commit-rules.md) | Task별 independent commit |
| [Source usage guard](guides/source-usage-guard.md) | Source quota safeguard |
| [Judge attestation](guides/ed25519-judge-attestations.md) | External signature ceremony |
| [Signed update·release](guides/signed-update-and-release.md) | Update·candidate·publication procedure |

## Releases

| 문서 | 설명 |
| --- | --- |
| [Release 안내](releases/README.md) | 제품 버전별 출시 안내 MOC |
| [`0.8.0`](releases/0.8.0.md) | npm 시험 배포용 제품 후보 |

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

## Plans

| 문서 | 설명 |
| --- | --- |
| [Plan 안내](plans/README.md) | Plan structure |
| [Active plan](plans/PLAN.md) | Sole active plan entrypoint |
| [References](plans/references.md) | Non-normative references |
| [Phase 안내](plans/phases/README.md) | Phase MOC |
| [Phase 0](plans/phases/00-source-bootstrap.md) | Source bootstrap |
| [Phase 1](plans/phases/01-setup-renderer.md) | Setup renderer |
| [Phase 2](plans/phases/02-knowledge-index.md) | Knowledge·index |
| [Phase 3](plans/phases/03-skills-projection.md) | Skill·projection |
| [Phase 4](plans/phases/04-role-run-interoperability.md) | Role·run |
| [Phase 5](plans/phases/05-usage-judge.md) | Usage·judge |
| [Phase 6](plans/phases/06-update-migration-release.md) | Update·migration·release |
| [Phase 7](plans/phases/07-public-qualification.md) | Public qualification |
| [Contract 안내](plans/contracts/README.md) | Product contract MOC |
| [Product goal](plans/contracts/01-product-goals.md) | Goal·non-goal |
| [Artifact source](plans/contracts/02-artifacts-source.md) | Artifact contract |
| [Consumer harness](plans/contracts/04-consumer-harness.md) | Installed harness contract |
| [Rust boundary](plans/contracts/05-rust-boundaries.md) | Crate responsibility |
| [Stage 안내](plans/stages/README.md) | Workflow stage MOC |
| [Stage 0](plans/stages/00-entry-routing.md) | Entry routing |
| [Stage 1a](plans/stages/01a-setup-discovery-consent.md) | Discovery·consent |
| [Stage 1b](plans/stages/01b-setup-rendering-contract.md) | Rendering contract |
| [Stage 2](plans/stages/02-harness-ownership.md) | Harness ownership |
| [Stage 3](plans/stages/03-simple-question-isolation.md) | Simple-question isolation |
| [Stage 4](plans/stages/04-prompt-refine.md) | Prompt refinement |
| [Stage 5](plans/stages/05-roles-orchestration.md) | Role·orchestration |
| [Stage 6](plans/stages/06-durable-run-completion.md) | Durable run |
| [Stage 7](plans/stages/07-usage-guard.md) | Usage guard |
| [Stage 8](plans/stages/08-verification-judge.md) | Verification·judge |
| [Stage 9](plans/stages/09-knowledge-memory.md) | Knowledge·memory |
| [Stage 10](plans/stages/10-completion-resume.md) | Completion·resume |
| [Stage 11](plans/stages/11-update-migration.md) | Update·migration |
| [Plugin lifecycle](plans/active/plugin-project-lifecycle.md) | User plugin·project lifecycle |
| [Native usage sensor](plans/active/native-usage-sensor.md) | Host-native sensor |
| [User onboarding](plans/active/user-onboarding-shared-index.md) | Global setup·shared index |
| [Source docs Wiki](plans/active/source-docs-wiki.md) | `docs/` graph·atomic fact contract |
| [Windows install](plans/active/windows-shell-install.md) | PowerShell 5.1·CMD boundary |
| [Documentation style](plans/active/documentation-style.md) | Human document style |
| [Security review](plans/active/security-review.md) | Security finding gate |
| [Docs Wiki migration](plans/active/docs-wiki-migration.md) | Knowledge preservation·path migration |
| [`0.8.0` release](plans/active/release-0.8.0.md) | Test distribution execution |
| [v0.9 loop·Wiki·Skill suite](plans/active/v0.9.0-loop-wiki-skills.md) | Host-native graph engineering 계획 |
| [v0.9 전역 knowledge RAG](plans/active/v0.9.0-global-knowledge-rag.md) | Cross-project retrieval·mandatory memory 계획 |
