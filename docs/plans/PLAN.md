# Aigent Hive active plan index

> Revision: 2.70
> 기준일: 2026-08-12
> Product version: `0.9.1`
> 현재 milestone: `0.9.1` 최종 호환 patch 검증·공개
> Entrypoint: `docs/plans/PLAN.md`

## Goal parameters

- Objective: Hive-native 실행·custom subagent·Discord·Markdown Wiki와 `0.9.0` 시험·정식
  릴리스 수용, 남은 호환 결함과 npm README 동기화를 `0.9.1`에서 마감. Notion은
  `0.10.0-test` 후보로 분리
- Success: Active fragment의 증거 기반 완료와 `MRA-*`, `REL9-*` 핵심 gate 충족
- Stop boundary: protected `main` review, stable publication environment approval,
  exact `1.0.0` authority, 설치 product usage guard remaining threshold `5%`
- Invariants: provider-neutral, backend별 canonical source 우선, SQLite 파생 상태,
  Source Wiki·run·role·plan·orchestration event Markdown/TOML 정본,
  ownership·consent·foreign byte 보존, provider API·credential·direct model process 경로 없음,
  신규 OMX/OMC dependency 없음, pointer authority 없음, force-push 없음, explicit-only major
- Native Goal compatibility: 변경 불가 objective의 “unchecked item in `docs/plans/PLAN.md`”는 `PLAN.md` 내부 checkbox가 아니라 아래 `Active fragments`의 unchecked item을 뜻함

## Completion index

측정 정본: Phase 0–7 milestone checklist와 non-phase active checklist. Stage checklist는 같은 구현의 workflow acceptance를 반복하므로 합계에서 제외.

| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| Phase 0–6 | 70 | 0 | 100% |
| Phase 7 | 49 | 0 | 100% |
| User plugin/project lifecycle | 38 | 0 | 100% |
| Host-native usage sensors | 27 | 0 | 100% |
| Global onboarding·shared index | 22 | 0 | 100% |
| Source docs Wiki | 13 | 0 | 100% |
| Windows shell install boundary | 3 | 0 | 100% |
| 문서 말투 | 6 | 0 | 100% |
| 공개 한국어 HTML 안내 | 6 | 0 | 100% |
| 복수 호스트 사용자 설치 | 5 | 0 | 100% |
| Security review | 4 | 0 | 100% |
| Docs Wiki migration | 4 | 0 | 100% |
| v0.9 loop·Wiki·Skill suite | 25 | 0 | 100% |
| v0.9 global knowledge RAG | 20 | 0 | 100% |
| v0.9 knowledge autocapture 회귀 | 12 | 0 | 100% |
| v0.9 knowledge portability·scan | 18 | 0 | 100% |
| Hive-native 반복 실행 | 1 | 23 | 4.2% |
| Model-routed custom subagent | 0 | 32 | 0% |
| Prompt refine 자동 routing | 12 | 0 | 100% |
| v0.9 test 기능 마감 | 18 | 0 | 100% |
| v0.9 full release | 28 | 0 | 100% |
| Test release setup routing | 4 | 0 | 100% |
| Bootstrap·user projection recovery | 13 | 0 | 100% |
| 한국어 setup 용어 복구 | 6 | 0 | 100% |
| Global Skill 선택 단순화 | 8 | 0 | 100% |
| Product-only Skill identity·localization | 15 | 0 | 100% |
| 전역·프로젝트 사용량 보호 정책 | 11 | 0 | 100% |
| Discord `v0.9` 연결 UX | 11 | 0 | 100% |
| Windows global setup hardening | 13 | 0 | 100% |
| Agent 자율 실행 지속 | 8 | 0 | 100% |
| Notion `v0.10` 후보 | 1 | 10 | 9.1% |
| **Canonical total** | **471** | **65** | **87.9%** |

External production boundary 항목도 미완료 합계에 포함. Protected authority 없이 완료 처리 금지.

## Required load order

1. 설치 product usage guard
2. `docs/plans/PLAN.md`
3. `docs/state/CURRENT.md`
4. 아래 active fragment의 checklist reconciliation
5. 선택한 task의 직접 관련 contract fragment만 추가 load

완료 history, unrelated stage, reference fragment의 선행 load 금지.

## Active fragments

| Fragment | Checklist ID | 범위 |
| --- | --- | --- |
| [`phases/07-public-qualification.md`](phases/07-public-qualification.md) | `P7-*` | Phase 7 local·external qualification과 completion gate |
| [`active/plugin-project-lifecycle.md`](active/plugin-project-lifecycle.md) | `RPH-*` | User plugin·project bootstrap·root knowledge·upgrade merge |
| [`active/native-usage-sensor.md`](active/native-usage-sensor.md) | `NUS-*` | 세 host native-first sensor·CodexBar fallback-only |
| [`active/user-onboarding-shared-index.md`](active/user-onboarding-shared-index.md) | `UOS-*` | Mandatory global setup·selected projection·shared index |
| [`active/source-docs-wiki.md`](active/source-docs-wiki.md) | `SLW-*` | `docs/` graph와 bilingual atomic fact·Skill reuse |
| [`active/windows-shell-install.md`](active/windows-shell-install.md) | `WSI-*` | Consumer PowerShell 5.1·`cmd.exe`와 source-only PowerShell 7 |
| [`active/documentation-style.md`](active/documentation-style.md) | `DOC-*` | 사람용 문서 style completion gate |
| [`active/public-html-guides.md`](active/public-html-guides.md) | `PHG-*` | Hive 핵심 기능·간단 설치 한국어 HTML과 기존 README branding 확인 |
| [`active/multi-host-user-install.md`](active/multi-host-user-install.md) | `MHI-*` | `--hosts`·반복 `--host` 사용자 설치·update와 문서 계약 |
| [`active/security-review.md`](active/security-review.md) | `SEC-*` | 독립 code·security review finding completion gate |
| [`active/docs-wiki-migration.md`](active/docs-wiki-migration.md) | `DWK-*` | 지식 보존과 `docs/` Wiki·atomic fact 전환 |
| [`active/release-0.8.0.md`](active/release-0.8.0.md) | `P7-*` reference | `0.8.0` Linux·npm·직접 설치 실행 순서와 범위 |
| [`active/v0.9.0-loop-wiki-skills.md`](active/v0.9.0-loop-wiki-skills.md) | `V9-*` | Host-native graph engineering·통합 Wiki·초기 Skill suite |
| [`active/v0.9.0-global-knowledge-rag.md`](active/v0.9.0-global-knowledge-rag.md) | `RAG-*` | 전역 RAG |
| [`active/v0.9.0-knowledge-autocapture-regression.md`](active/v0.9.0-knowledge-autocapture-regression.md) | `KAC-*` | 모든 Wiki 활성 turn의 mandatory canonical write 회귀 보정 |
| [`active/v0.9.0-knowledge-portability-scan.md`](active/v0.9.0-knowledge-portability-scan.md) | `KPX-*` | Knowledge 이식·directory scan·automatic query |
| [`active/native-iterative-execution.md`](active/native-iterative-execution.md) | `NAT-*` | Hive-native iterative·team·multi-goal execution |
| [`active/model-routed-custom-subagents.md`](active/model-routed-custom-subagents.md) | `MRA-*` | Codex·Claude custom subagent·Judge 정책 |
| [`active/prompt-refine-auto-routing.md`](active/prompt-refine-auto-routing.md) | `PRF-*` | Material ambiguity 자동 refine·승인 전 정지 |
| [`active/v0.9.0-test-finalization.md`](active/v0.9.0-test-finalization.md) | `TST9-*` | Markdown Wiki, Discord outbound, 문제 보고와 시험판 기능 마감 |
| [`active/release-0.9.0.md`](active/release-0.9.0.md) | `REL9-001–017` | 시험 수용·main 통합 |
| [`active/release-0.9.0-stable-publication.md`](active/release-0.9.0-stable-publication.md) | `REL9-019–030` | 최소 release trust 정리·stable publication·public acceptance |
| [`active/test-release-setup-routing.md`](active/test-release-setup-routing.md) | `TUR-*` | Global·project setup routing과 numbered test user projection 인증 |
| [`active/bootstrap-global-setup-recovery.md`](active/bootstrap-global-setup-recovery.md) | `BGR-*` | 선택형 bootstrap, 쉬운 global setup 복구, user projection merge |
| [`active/korean-setup-terminology.md`](active/korean-setup-terminology.md) | `KST-*` | 한국어 global setup product term·질문 표기 |
| [`active/global-skill-selection.md`](active/global-skill-selection.md) | `GSS-*` | all-built-in 기본값·개별 토글·목록 표기 |
| [`active/skill-identity-localization.md`](active/skill-identity-localization.md) | `SIL-*` | product-only Skill·source Skill 폐기·표시 언어 |
| [`active/usage-guard-policy.md`](active/usage-guard-policy.md) | `UGP-*` | 전역·project 한도·product guard |
| [`active/discord-onboarding-v09.md`](active/discord-onboarding-v09.md) | `DIS9-*` | Discord global setup·프로젝트별 중단 알림·HTML 안내 |
| [`active/windows-global-setup-hardening.md`](active/windows-global-setup-hardening.md) | `WGS-*` | Mac 원본 복구 유지·Windows CLI 탐색·설정·fresh-session 수용 |
| [`active/agent-autonomous-continuation.md`](active/agent-autonomous-continuation.md) | `AAC-*` | Agent 소유 작업 지속·terminal state·중간 종료 회귀 |
| [`active/v0.10.0-notion-candidate.md`](active/v0.10.0-notion-candidate.md) | `N10-*` | Notion 연결·freshness·write-through와 `0.10.0-test` 후보 |

## Reconciliation gate

- 시작·재개 시 active checklist를 current evidence와 대조. 충족 항목만 owning fragment에서
  갱신, missing·stale·indirect·contradictory evidence는 유지. 완료 전 새 구현 금지. ID 중복과
  `PLAN.md` 내부 checklist 금지

## Fragment map

| Fragment | 역할 | 기본 load |
| --- | --- | --- |
| [`contracts/README.md`](contracts/README.md) | 번호별 product·artifact·implementation contract index | 아니요 |
| [`stages/README.md`](stages/README.md) | Stage 0–11 workflow fragment index | 아니요 |
| [`phases/README.md`](phases/README.md) | Phase 0–7 milestone fragment index | 아니요 |
| [`references.md`](references.md) | Review 후보와 외부 reference | 아니요 |

## Current execution order

완료 증거: [`CURRENT.md`](../state/CURRENT.md)와 owning active fragment.
현재: `0.9.1` protected publication·Windows public install 완료. 설치 product 사용량 보호를
source 개발의 단일 정본으로 수렴하고 source Python watcher·별도 threshold state 제거 우선.
기존 stable candidate run `31482918509`: historical qualification only, publication 금지.
이후 호환 patch candidate·protected stable publication 진행. Antigravity·Claude 공개 제외 유지.
완료: `KAC-001·007–008` 보정·Windows Codex fresh-session 기록→회수·replacement candidate 수용.
Notion: `N10-002–011`·`0.10.0-test` 보류.
병행 문서 작업: `PHG-001–005` 공개 한국어 HTML·디자인 원칙·복수 호스트 안내 완료.
복수 호스트 CLI: `MHI-001–004` 구현·검증·문서 동기화·`develop` push 완료.
