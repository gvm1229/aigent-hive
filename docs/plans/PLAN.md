# Aigent Hive active plan index

> Revision: 1.83
> 기준일: 2026-08-01
> Product version: `0.8.0`
> 현재 milestone: Phase 7 qualification + global onboarding·shared index `0.8.0`
> Entrypoint: `docs/plans/PLAN.md`

## Goal parameters

- Objective: `Aigent Hive 0.8.0` 후보의 Linux·macOS·Windows native artifact,
  npm `0.8.0-test.N`·직접 설치 시험, bilingual onboarding, update
  discovery·activation과 provenance gate 완료
- Queued `0.9.0`: host-native loop·Wiki·Skill suite·전역 RAG·knowledge 이식·scan
- Success: Mandatory user setup, global preference 기반 expedited/custom project setup,
  user-root 단일 SQLite, 세 host selected Skill projection, Wiki default-on opt-out,
  usage guard opt-in `20%`, native-first·CodexBar fallback-only, consumer
  PowerShell 5.1·`cmd.exe`, source-only PowerShell 7, Linux musl x86_64·arm64,
  first `aigent-hive@0.8.0-test.1`, SHA-256·GitHub attestation과 실제 Windows
  acceptance
- Stop boundary: GitHub Release·npm `latest`, protected credential, exact `1.0.0`
  authority, 현재 source usage guard remaining `5%`
- Invariants: provider-neutral, canonical Markdown 우선, OMX/OMC replaceable adapter,
  ownership·consent·foreign byte 보존, provider API·credential 경로 없음, force-push 없음,
  explicit-only major, scheduler·tmux·Stop continuation 없음
- Native Goal compatibility: 변경 불가 objective의 “unchecked item in `docs/plans/PLAN.md`”는 `PLAN.md` 내부 checkbox가 아니라 아래 `Active fragments`의 unchecked item을 뜻함

## Completion index

측정 정본: Phase 0–7 milestone checklist와 non-phase active checklist. Stage checklist는 같은 구현의 workflow acceptance를 반복하므로 합계에서 제외.

| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| Phase 0–6 | 70 | 0 | 100% |
| Phase 7 | 44 | 5 | 89.8% |
| User plugin/project lifecycle | 38 | 0 | 100% |
| Host-native usage sensors | 24 | 0 | 100% |
| Global onboarding·shared index | 19 | 0 | 100% |
| Source docs Wiki | 11 | 0 | 100% |
| Windows shell install boundary | 3 | 0 | 100% |
| 문서 말투 | 6 | 0 | 100% |
| Security review | 4 | 0 | 100% |
| Docs Wiki migration | 4 | 0 | 100% |
| v0.9 loop·Wiki·Skill suite | 0 | 25 | 0% |
| v0.9 global knowledge RAG | 0 | 20 | 0% |
| v0.9 knowledge portability·scan | 0 | 18 | 0% |
| **Canonical total** | **223** | **68** | **76.6%** |

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
- Bare `hive update`의 npm `test` 확인, npm·direct owner 인증, 선택 언어 prompt,
  명시적 수락 뒤 exact adapter 실행과 owner·version 재검증
- 독립 final blocker review: critical·high·medium·low finding `0`건
- Remote candidate `30647361507`의 5개 target·6개 npm PASS. Publication
  `30647959771`: 선행 검사 PASS, 2FA 우회 token 부재로 등록 0건

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

완료된 선행 조건: PR #8–12 병합, exact `ef55325` candidate `30647361507` PASS,
publication `30647959771`의 npm 쓰기 직전까지 모든 gate PASS.

1. `NPM_TOKEN`을 6개 package 게시·2FA 우회 권한의 granular token으로 교체
2. 기존 candidate `30647361507`로 `release-publish.yml` 재실행·승인
3. npm·Unix·PowerShell·CMD clean install·repeat·recovery와 P7 미완료 5개 검증
4. 시험 배포 성공 commit의 `develop` → `main` PR 병합
5. 별도 지시 뒤 V9-025 → KPX-001–007 → `RAG-*` → KPX-008–018 → 잔여 `V9-*`

## `0.8.0` 비차단 deferred boundary

- macOS Developer ID signing·notarization
- Windows Authenticode signing
- 실제 Claude protected session E2E와 Pro/Max usage parity
- Externally signed TUF production authorization
- Exact `1.0.0` 사용자 authority
- GitHub normal release와 npm `latest` 안정 channel
