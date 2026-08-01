# Aigent Hive active plan index

> Revision: 1.88
> 기준일: 2026-08-01
> Product version: `0.9.0`
> 현재 milestone: `0.9.0` 정식 릴리스 준비
> Entrypoint: `docs/plans/PLAN.md`

## Goal parameters

- Objective: `Aigent Hive 0.9.0-test` 독립 시험 배포·수용 뒤 별도 `0.9.0` 정식
  GitHub·npm 릴리스와 public update 검증
- Success: Mandatory user setup, global preference 기반 expedited/custom project setup,
  user-root 단일 SQLite, 세 host selected Skill projection, Wiki default-on opt-out,
  usage guard opt-in `20%`, native-first·CodexBar fallback-only, consumer
  PowerShell 5.1·`cmd.exe`, source-only PowerShell 7, Linux musl x86_64·arm64,
  bare `aigent-hive@0.9.0-test|test`, 선택형 numbered test, stable `latest` 보존,
  시험·정식 feature parity, `aigent-hive@0.9.0|latest`, OS signing·TUF·SHA-256·
  GitHub attestation과 public install·update acceptance
- Stop boundary: protected review·environment approval, signing·TUF·npm credential,
  exact `1.0.0` authority, 현재 source usage guard remaining `60%`
- Invariants: provider-neutral, canonical Markdown 우선, OMX/OMC replaceable adapter,
  ownership·consent·foreign byte 보존, provider API·credential 경로 없음, force-push 없음,
  explicit-only major, scheduler·tmux·Stop continuation 없음
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
| Source docs Wiki | 11 | 0 | 100% |
| Windows shell install boundary | 3 | 0 | 100% |
| 문서 말투 | 6 | 0 | 100% |
| Security review | 4 | 0 | 100% |
| Docs Wiki migration | 4 | 0 | 100% |
| v0.9 loop·Wiki·Skill suite | 25 | 0 | 100% |
| v0.9 global knowledge RAG | 20 | 0 | 100% |
| v0.9 knowledge portability·scan | 18 | 0 | 100% |
| v0.9 full release | 1 | 25 | 3.8% |
| **Canonical total** | **292** | **25** | **92.1%** |

External production boundary 항목도 미완료 합계에 포함. Protected authority 없이 완료 처리 금지.

## 최신 완료 증거

- Local Windows: Rust workspace 전체 PASS, PowerShell 5.1·7.6.4 installer와
  `cmd.exe` bootstrap 계약 PASS
- Codex Skill metadata: project projection implicit owner 1개, 나머지 explicit-only,
  fresh-session 중복 warning 0건
- 실제 Windows 11 x86_64: Codex user install·global setup·project auto onboarding,
  shared index 1개 project, repeat update·rollback·재검증 PASS
- Windows shell: WSI-001–003 완료, consumer PowerShell 7 dependency 0건,
  source dependency helper의 exact WinGet preview·동의·재검증 PASS
- Strict Clippy all targets·all features, format check PASS
- Copier/Rust current projection parity `3/3`, Source Wiki lint finding·warning `0`
- Shared index 동일 입력 재실행 byte-exact no-op, `changed_paths=[]`
- Codex·Antigravity expedited/custom connected onboarding matrix `4/4`
- 개발·소비자 검증 결과 보고 규칙: 범위·이유·현재 환경·실행 여부·입증 범위·
  미검증 범위 명시, 투영 시험 PASS
- Opt-in daily update check의 24시간 success throttle, offline next-session retry,
  fixed npm metadata endpoint와 no-install contract
- Bare `hive update`의 npm `latest` 확인, legacy test·stable npm·direct owner 인증,
  선택 언어 prompt,
  명시적 수락 뒤 exact adapter 실행과 owner·version 재검증
- 독립 final blocker review: critical·high·medium·low finding `0`건
- npm publication `30658188721`: exact `0.8.0` 여섯 package·provenance·
  `latest=0.8.0` PASS. 기존 `test=0.8.0-test.1` 보존
- 실제 Windows npm·CMD clean install, repeat, pending receipt recovery PASS. npm·direct
  native SHA-256 `330f4e0c8da5b6347400b9b16a9f76b2fb4f94406a2eacfe8c641367ca344ef9`
- v0.9 host-native loop adapter와 5개 canonical Skill의 세 host projection·hostile
  conformance PASS, tmux·scheduler·OMX/OMC 자동 의존성 0건
- v0.9 전역 RAG 50,000 chunk: cold p95 `163.3569ms`, prepared-resident warm p95
  `0.1178ms`, bilingual recall@5 `>= 90%`
- v0.9 `.hivekb` 100 collection·50,000 chunk: export p95 `1066.9209ms`,
  import+rebuild p95 `3255.1537ms`

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
| [`active/release-0.9.0.md`](active/release-0.9.0.md) | `REL9-*` | 정식 GitHub·npm 릴리스와 public acceptance |

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

완료: PR #13 병합, exact `420e244` candidate run `30657669889`, publication run
`30658188721`, npm·Unix·PowerShell·CMD 계약과 실제 Windows clean install·repeat·
recovery 검증.

완료: V9-001–025, RAG-001–020, KPX-001–018.

다음: REL9-002–005 release activation.
