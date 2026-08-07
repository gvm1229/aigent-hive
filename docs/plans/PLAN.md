# Aigent Hive active plan index

> Revision: 2.18
> 기준일: 2026-08-07
> Product version: `0.9.0`
> 현재 milestone: `0.9.0` 정식 릴리스 준비
> Entrypoint: `docs/plans/PLAN.md`

## Goal parameters

- Objective: Hive-native execution·custom subagent routing·Judge policy 안전 계약, Discord·Notion
  end-to-end global setup·프로젝트별 usage-guard 알림, `0.9.0-test` 독립 시험 배포·수용,
  별도 `0.9.0` 정식 GitHub·npm 릴리스와 public update 검증
- Success: Active fragment evidence-backed completion. 시험판 핵심 gate: `MRA-*`, `PRF-*`,
  `TST9-*`, `REL9-*` 독립 test·stable publication과 public acceptance
- Stop boundary: protected `main` review, signing·TUF·npm credential,
  exact `1.0.0` authority, source usage guard remaining threshold `30%`
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
| Host-native usage sensors | 24 | 0 | 100% |
| Global onboarding·shared index | 19 | 0 | 100% |
| Source docs Wiki | 12 | 0 | 100% |
| Windows shell install boundary | 3 | 0 | 100% |
| 문서 말투 | 6 | 0 | 100% |
| Security review | 4 | 0 | 100% |
| Docs Wiki migration | 4 | 0 | 100% |
| v0.9 loop·Wiki·Skill suite | 25 | 0 | 100% |
| v0.9 global knowledge RAG | 20 | 0 | 100% |
| v0.9 knowledge portability·scan | 18 | 0 | 100% |
| Hive-native 반복 실행 | 1 | 23 | 4.2% |
| Model-routed custom subagent | 0 | 32 | 0% |
| Prompt refine 자동 routing | 12 | 0 | 100% |
| v0.9 test 기능 마감 | 18 | 0 | 100% |
| v0.9 full release | 13 | 13 | 50% |
| Test release setup routing | 4 | 0 | 100% |
| Bootstrap·user projection recovery | 11 | 0 | 100% |
| 한국어 setup 용어 복구 | 5 | 1 | 83.3% |
| Global Skill 선택 단순화 | 8 | 0 | 100% |
| Discord·Notion 연결 UX | 1 | 17 | 5.6% |
| **Canonical total** | **365** | **86** | **80.9%** |

External production boundary 항목도 미완료 합계에 포함. Protected authority 없이 완료 처리 금지.

## 최신 완료 증거

- 상세 완료 증거·실행 수치·publication 식별자: [`CURRENT.md`](../state/CURRENT.md)
- `0.8.0` npm·Windows clean install·repeat·recovery와 public update acceptance PASS
- v0.9 loop·Wiki·Skill suite, 50,000-chunk RAG, `.hivekb` portability qualification PASS
- Hive-native 반복 실행 정책·계획: RALPLAN-DR, Architect `APPROVE_WITH_CHANGES`,
  Critic 최종 `APPROVE`, stale-pointer authority·typed receipt·legacy migration 계약 정본화

## Required load order

1. Source usage gate
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
| [`active/security-review.md`](active/security-review.md) | `SEC-*` | 독립 code·security review finding completion gate |
| [`active/docs-wiki-migration.md`](active/docs-wiki-migration.md) | `DWK-*` | 지식 보존과 `docs/` Wiki·atomic fact 전환 |
| [`active/release-0.8.0.md`](active/release-0.8.0.md) | `P7-*` reference | `0.8.0` Linux·npm·직접 설치 실행 순서와 범위 |
| [`active/v0.9.0-loop-wiki-skills.md`](active/v0.9.0-loop-wiki-skills.md) | `V9-*` | Host-native graph engineering·통합 Wiki·초기 Skill suite |
| [`active/v0.9.0-global-knowledge-rag.md`](active/v0.9.0-global-knowledge-rag.md) | `RAG-*` | 전역 RAG |
| [`active/v0.9.0-knowledge-portability-scan.md`](active/v0.9.0-knowledge-portability-scan.md) | `KPX-*` | Knowledge 이식·directory scan·automatic query |
| [`active/native-iterative-execution.md`](active/native-iterative-execution.md) | `NAT-*` | Hive-native iterative·team·multi-goal execution |
| [`active/model-routed-custom-subagents.md`](active/model-routed-custom-subagents.md) | `MRA-*` | Codex·Claude custom subagent·Judge 정책 |
| [`active/prompt-refine-auto-routing.md`](active/prompt-refine-auto-routing.md) | `PRF-*` | Material ambiguity 자동 refine·승인 전 정지 |
| [`active/v0.9.0-test-finalization.md`](active/v0.9.0-test-finalization.md) | `TST9-*` | Notion·SQLite, Discord outbound, 문제 보고와 시험판 기능 마감 |
| [`active/release-0.9.0.md`](active/release-0.9.0.md) | `REL9-*` | 정식 GitHub·npm 릴리스와 public acceptance |
| [`active/test-release-setup-routing.md`](active/test-release-setup-routing.md) | `TUR-*` | Global·project setup routing과 numbered test user projection 인증 |
| [`active/bootstrap-global-setup-recovery.md`](active/bootstrap-global-setup-recovery.md) | `BGR-*` | 선택형 bootstrap, 쉬운 global setup 복구, user projection merge |
| [`active/korean-setup-terminology.md`](active/korean-setup-terminology.md) | `KST-*` | 한국어 global setup product term·질문 표기 |
| [`active/global-skill-selection.md`](active/global-skill-selection.md) | `GSS-*` | all-built-in 기본값·개별 토글·목록 표기 |
| [`active/discord-notion-onboarding.md`](active/discord-notion-onboarding.md) | `DNI-*` | Discord·Notion global setup·프로젝트별 중단 알림·HTML 안내 |

## Reconciliation gate

- Goal start·resume마다 active fragment의 모든 checklist를 current evidence와 대조
- 이미 충족된 unchecked item의 owning fragment 우선 갱신
- Evidence가 missing·stale·indirect·contradictory이면 unchecked 유지
- Reconciliation 완료 전 새 구현 선택 금지
- Checklist ID의 fragment 간 중복과 `PLAN.md` 내부 checklist 금지

## Fragment map

| Fragment | 역할 | 기본 load |
| --- | --- | --- |
| [`contracts/README.md`](contracts/README.md) | 번호별 product·artifact·implementation contract index | 아니요 |
| [`stages/README.md`](stages/README.md) | Stage 0–11 workflow fragment index | 아니요 |
| [`phases/README.md`](phases/README.md) | Phase 0–7 milestone fragment index | 아니요 |
| [`references.md`](references.md) | Review 후보와 외부 reference | 아니요 |

## Current execution order

완료 증거: [`CURRENT.md`](../state/CURRENT.md)와 owning active fragment.
다음 구현: `DNI-001–017` → 동일 numbered test release의 `KST-006`·`DNI-018`;
stable 경로는 별도 `main` authority까지 보류.
