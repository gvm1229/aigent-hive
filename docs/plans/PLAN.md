# Aigent Hive 활성 계획

> Revision: 316
> 기준일: 2026-09-06
> Product version: `0.10.1`
> Stable baseline: `0.10.0`
> 수용 공개 시험: `0.10.1-test.1`
> 현재 단계: accepted `0.10.1-test.1`의 stable promotion
> 결정: [`ADR-0021`](../decisions/ADR-0021-0.10.1-upgrade-usage-fix.md)

## 목표

- 지원 이전 버전의 설치 상태를 먼저 인증한 뒤 현재 상태로 바꾸는 공통 호환성 등록표와 project migration
- 선언된 Skill rename·merge의 정상 수렴과 실제 중복·변조의 무변경 거부
- 사용량 기준 변경 직후 같은 session에서 보호 활성 상태를 유지한 새 측정
- `0.10.1-test.1` 세 운영체제 공개 설치·갱신 수용

## 완료 조건

- Migration table의 모든 source release와 full project base·user projection·실행 시험 대응
- `0.9.5` 25개 Skill 선택의 24개 canonical 선택 수렴, retired discovery `0건`
- `scan → dry-run → rollback → apply → validate`와 사용자·외부 bytes 보존
- 기준 변경 뒤 `session disable`·새 session 없이 fresh usage 판정, stale policy marker 재사용 `0건`
- Rust·Python·문서·보안·release gate 통과
- `0.10.1-test.1` Windows x64·macOS arm64·Linux musl x64 공개 수용

## 중지 경계

- 실제 DuckSoul `.hive` 수정·`hive project upgrade --apply`
- Provider API·provider credential·OMX/OMC 사용

## Completion index

| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 일반 harness 갱신 | 10 | 0 | 100% |
| 사용량 보호 재평가 | 8 | 0 | 100% |
| `0.10.1` 공개 시험·안정판 승격 | 7 | 2 | 77.8% |
| **합계** | **25** | **2** | **92.6%** |

## Required load order

1. 설치 product usage guard
2. `docs/plans/PLAN.md`
3. `docs/state/CURRENT.md`
4. [`harness-upgrade-0.10.1.md`](active/harness-upgrade-0.10.1.md)
5. [`release-0.10.1.md`](active/release-0.10.1.md)
6. 직접 관련 architecture·decision·guide

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [`harness-upgrade-0.10.1.md`](active/harness-upgrade-0.10.1.md) | `HUP101-*`, `UGR101-*` | Project migration·사용량 정책 재평가 |
| [`release-0.10.1.md`](active/release-0.10.1.md) | `REL101-*` | 공개 시험·세 운영체제 수용 |

## 실행 순서

1. `HUP101-001–004`: 실제 실패 fixture·공통 등록표·인증 우선 migration
2. `HUP101-005–010`: typed state 변환·원자 적용·호환성 자동 gate
3. `UGR101-001–008`: policy-bound halt·same-session fresh recheck·자동 지침
4. `REL101-001–003`: 전체 회귀·후보·공개 시험판 게시
5. `REL101-004–005`: 세 운영체제 공개 수용·현재 상태 정합화
6. `REL101-006–009`: 수용 시험판 결합 promotion·stable 공개·독립 확인

## 비활성 자료

- `0.10.0` 완료 계획: 기존 `docs/plans/active/*-0.10.0.md` 기록
- 버전 비종속 후보: [`backlog/README.md`](backlog/README.md)
- 완료·대체 기록: [`../archive/README.md`](../archive/README.md)
