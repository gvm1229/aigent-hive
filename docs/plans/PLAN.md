# Aigent Hive active plan index

> Revision: 1.65
> 기준일: 2026-07-31
> Product version: `0.7.0`
> 현재 milestone: Phase 7 qualification + global onboarding·shared index `0.8.0`
> Entrypoint: `docs/plans/PLAN.md`

## Goal parameters

- Objective: `Aigent Hive 0.8.0`의 Linux·macOS·Windows native artifact, npm·curl
  시험 설치, bilingual onboarding, update discovery·activation과 provenance gate 완료
- Success: Mandatory user setup, global preference 기반 expedited/custom project setup,
  user-root 단일 SQLite, 세 host selected Skill projection, Wiki default-on opt-out,
  usage guard opt-in `20%`, native-first·CodexBar fallback-only, consumer
  PowerShell 5.1·`cmd.exe`, source-only PowerShell 7, Linux musl x86_64·arm64,
  exact `aigent-hive@0.8.0`, SHA-256·GitHub attestation과 실제 Windows acceptance
- Stop boundary: GitHub Release·npm `latest`, protected credential, exact `1.0.0`
  authority, 현재 source usage guard remaining `30%`
- Invariants: provider-neutral, canonical Markdown 우선, OMX/OMC replaceable adapter,
  ownership·consent·foreign byte 보존, provider API·credential 경로 없음, force-push 없음,
  explicit-only major
- Native Goal compatibility: 변경 불가 objective의 “unchecked item in `docs/plans/PLAN.md`”는 `PLAN.md` 내부 checkbox가 아니라 아래 `Active fragments`의 unchecked item을 뜻함

## Completion index

측정 정본: Phase 0–7 milestone checklist와 non-phase active checklist. Stage checklist는 같은 구현의 workflow acceptance를 반복하므로 합계에서 제외.

| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| Phase 0–6 | 70 | 0 | 100% |
| Phase 7 | 41 | 8 | 83.7% |
| User plugin/project lifecycle | 38 | 0 | 100% |
| Host-native usage sensors | 24 | 0 | 100% |
| Global onboarding·shared index | 19 | 0 | 100% |
| Source bilingual LLM Wiki | 11 | 0 | 100% |
| Windows shell install boundary | 3 | 0 | 100% |
| Documentation style | 5 | 0 | 100% |
| Security review | 4 | 0 | 100% |
| Docs Wiki migration | 1 | 3 | 25% |
| **Canonical total** | **216** | **11** | **95.2%** |

External production boundary 항목도 미완료 합계에 포함. Protected authority 없이 완료 처리 금지.

## 최신 완료 증거

- Local Windows: Rust workspace 전체 PASS, PowerShell 5.1·7.6.4 installer와
  `cmd.exe` bootstrap 계약 PASS
- Codex Skill metadata: project projection implicit owner 1개, 나머지 explicit-only,
  fresh-session 중복 warning 0건
- 실제 Windows 11 x86_64: Codex user install·global setup·project auto onboarding,
  shared index 1개 project, repeat update·rollback·재검증 PASS
- Current remote: `9fb2552` CI run `30479010450` 7/7 PASS; native runtime
  `d39ce7f` run `30477685720` 3/3 historical PASS
- Windows shell: WSI-001–003 완료, consumer PowerShell 7 dependency 0건,
  source dependency helper의 exact WinGet preview·동의·재검증 PASS
- Strict Clippy all targets·all features, format check PASS
- Copier/Rust current projection parity `3/3`, Source Wiki lint finding·warning `0`
- Shared index 동일 입력 재실행 byte-exact no-op, `changed_paths=[]`
- Codex·Antigravity expedited/custom connected onboarding matrix `4/4`
- Initial `Expedited — set everything to default`와 project zero-question inference contract
- English·한국어 language-first setup과 선택 언어 global guidance, `hive-cli` 211개 PASS
- 독립 final blocker review: critical·high·medium·low finding `0`건

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
| [`active/source-llm-wiki.md`](active/source-llm-wiki.md) | `SLW-*` | Provider-neutral bilingual source Wiki와 Skill reuse |
| [`active/windows-shell-install.md`](active/windows-shell-install.md) | `WSI-*` | Consumer PowerShell 5.1·`cmd.exe`와 source-only PowerShell 7 |
| [`active/documentation-style.md`](active/documentation-style.md) | `DOC-*` | 사람용 문서 style completion gate |
| [`active/security-review.md`](active/security-review.md) | `SEC-*` | 독립 code·security review finding completion gate |
| [`active/docs-wiki-migration.md`](active/docs-wiki-migration.md) | `DWK-*` | 지식 보존과 `docs/` Wiki·atomic fact 전환 |
| [`active/release-0.8.0.md`](active/release-0.8.0.md) | `P7-*` reference | `0.8.0` Linux·npm·직접 설치 실행 순서와 범위 |

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

1. DWK-002 docs home·index·topic MOC와 README knowledge 복원
2. DWK-003 bilingual atomic fact 분할·이동
3. DWK-004 Source Wiki CLI·Skill·index 경로 전환과 `llm-wiki/` 제거
4. P7-048 opt-in daily update check·offline next-session retry
5. P7-049 interactive owner-aware `hive update`
6. P7-043 Linux x86_64·arm64 musl release qualification
7. P7-044 public `aigent-hive` npm package family와 native smoke
8. P7-045 npm-backed Unix·PowerShell·CMD installer와 digest 검증
9. P7-020 5개 platform archive·npm tarball provenance
10. P7-018 protected `main` release candidate qualification
11. P7-037 GitHub Release 없이 npm `test` publication·clean install 검증

## `0.8.0` 비차단 deferred boundary

- macOS Developer ID signing·notarization
- Windows Authenticode signing
- 실제 Claude protected session E2E와 Pro/Max usage parity
- Externally signed TUF production authorization
- Exact `1.0.0` 사용자 authority
- GitHub normal release와 npm `latest` 안정 channel
