# Aigent Hive active plan index

> Revision: 1.35
> 기준일: 2026-07-25
> Product version: `0.7.0`
> 현재 milestone: Phase 7 public qualification `0.8.0`
> Entrypoint: `docs/plans/PLAN.md`

## Goal parameters

- Objective: Phase 7의 남은 protected public qualification 7개 완료
- Success: signed multi-platform CLI·실제 host matrix·`0.8.x` release candidate·GitHub Release provenance 검증
- Stop boundary: protected credential, irreversible production publication, exact `1.0.0` authority, source usage guard remaining `60%`
- Invariants: provider-neutral, ownership·consent 보존, provider API·credential 경로 없음, force-push 없음, explicit-only major
- Native Goal compatibility: 변경 불가 objective의 “unchecked item in `docs/plans/PLAN.md`”는 `PLAN.md` 내부 checkbox가 아니라 아래 `Active fragments`의 unchecked item을 뜻함

## Completion index

측정 정본: Phase 0–7 milestone checklist와 non-phase active checklist. Stage checklist는 같은 구현의 workflow acceptance를 반복하므로 합계에서 제외.

| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| Phase 0–6 | 70 | 0 | 100% |
| Phase 7 | 33 | 7 | 82.5% |
| Documentation style | 5 | 0 | 100% |
| Security review | 4 | 0 | 100% |
| **Canonical total** | **112** | **7** | **94.1%** |

External production boundary 항목도 미완료 합계에 포함. Protected authority 없이 완료 처리 금지.

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
| [`active/documentation-style.md`](active/documentation-style.md) | `DOC-*` | 사람용 문서 style completion gate |
| [`active/security-review.md`](active/security-review.md) | `SEC-*` | 독립 code·security review finding completion gate |

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

1. Active checklist reconciliation
2. Protected authority·credential availability 확인
3. P7-011 macOS Intel·P7-012 Windows release build·install·runtime qualification
4. P7-013·021 실제 host session E2E·capability matrix
5. P7-018·020·037 signed release candidate·CLI·GitHub Release qualification
6. Exact `1.0.0` 사용자 authority 확인

## External production boundary

- macOS Developer ID signing·notarization
- Windows Authenticode signing
- 실제 Codex·Claude·Gemini Antigravity protected session E2E
- Externally signed TUF authorization과 GitHub Release publication
- Exact `1.0.0` 사용자 authority
