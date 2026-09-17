# Aigent Hive 활성 계획

> Revision: 333
> 기준일: 2026-09-17
> Product version: `0.11.0`
> 공개 Stable: `0.10.3`
> 현재 단계: 기존 `0.10.4` 변경을 계승하는 `0.11.0` 리팩터링 계획
> 첫 공개 시험: `0.11.0-test.1`; 현재 제품 파일의 버전: `0.10.3`
> 결정: [ADR-0023](../decisions/ADR-0023-foundation-refactor.md), [기존 ADR-0022](../decisions/ADR-0022-global-user-update.md)

## 현재 요청과 경계

- 요청: 새 브랜치와 다섯 영역의 구현 계획. 이번 작업의 제품 구현·설치·게시 제외
- 브랜치: `codex/refactor-hive-foundations`, 기준 `develop@87b84f42`
- 확정: 기존 기능·명령 유지, 핵심 지식 흐름 우선, Codex 검증 후 다른 호스트 확대
- 목표: 지식 이어 가기·호스트 검증·코드 중심 안전 검사·공통 파일 처리·상태 단일 정본
- [총괄 계획](active/refactor-foundations.md)의 21개 구현 항목. 계획 작성 완료와 제품 구현 완료의 별도 판정
- 버전 결정: 미출시 `0.10.4` 변경을 `0.11.0`으로 계승, 별도 `0.10.4` 출시 제외. 제품 파일의 번호 변경은 `RF-B01` 구현 단계
- `RB104-004`: 기존 소유 항목 유지, 검증된 호스트 계약 부재. 이번 리팩터링의 복제 항목·완료 처리 제외
- 안정판 권한은 후속 버전별 명시 승인 필요

## 기존 출시 목표와 보존 근거

- 모든 Hive 프로젝트·소스의 옛 `halt.json`을 같은 위치에서 교체·제거하는 공통 복구, 추가 영구 상태 파일 0건: `UGR103-*`

- 소스·harness 지침의 승인 경계·지속성·현재 참조 개선: `INS102-*`

- `hive update` 한 번으로 실행 파일과 전역 사용자 설정·호스트 투영을 수렴
- 새 전역 질문은 답변 전 일반 Hive 작업을 차단하고, 답변 뒤 같은 transaction을 재개
- npm 설치는 실행 파일만 제공하고 최초 전역 초기화는 `hive update`가 담당
- `0.10.3-test.1` 세 운영체제 공개 수용 뒤 stable `0.10.3` 공개

## 기존 출시 완료 조건

- `0.9.5–0.10.1` 인증된 전역 사용자 설치의 자동 이관·검증
- 질문 없는 갱신은 자동 완료, 질문 있는 갱신은 `setup-required` 차단과 답변 뒤 자동 재개
- npm 신규 설치는 `hive update`의 호스트 선택만으로 최소 사용자 투영 준비
- 프로젝트 접근·변경 `0건`, foreign bytes 보존, 변조·충돌 무변경 거부
- Rust·Python·문서·보안·release gate와 `0.10.2-test.1` 세 운영체제 공개 수용

## 중지 경계

- 실제 사용자 루트 변경과 등록 프로젝트 harness 변경
- Provider API·provider credential·OMX/OMC 사용

## Completion index

| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 전역 사용자 갱신 | 10 | 0 | 100% |
| 지침 품질 개선 | 6 | 0 | 100% |
| `0.10.2` 공개 시험·안정판 승격 | 9 | 0 | 100% |
| `0.10.3` 사용량 보호 기존 설치 복구 | 10 | 0 | 100% |
| 기존 후속 사용량 보호 `RB104-*` | 3 | 1 | 75% |
| 리팩터링 공통 기준·통합 `RF-B*`, `RF-R*` | 0 | 3 | 0% |
| 리팩터링 지식·호스트 `RF-K*`, `RF-H*` | 0 | 6 | 0% |
| 리팩터링 안전·설치 `RF-P*`, `RF-T*` | 0 | 8 | 0% |
| 리팩터링 계획·상태 `RF-S*` | 0 | 4 | 0% |
| **등록 항목 합계** | **38** | **22** | **63.3%** |

이번 리팩터링 구현: **0/21**. `0.11.0` 범위: 기존 사용량 보호 포함 **3/25**. 기존 38개 완료 표시는 이전 근거의 보존이며 이번 환경의 재시험 성공과 구분.

## Required load order

1. 설치 product usage guard
2. `docs/plans/PLAN.md`
3. `docs/state/CURRENT.md`
4. [리팩터링 총괄](active/refactor-foundations.md)
5. 현재 항목의 상세 계획과 [ADR-0023](../decisions/ADR-0023-foundation-refactor.md)
6. 직접 관련 코드·시험·기존 결정

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [refactor-foundations.md](active/refactor-foundations.md) | `RF-B*`, `RF-R*` | 공통 기준·실행 순서·통합 수용 |
| [refactor-context-hosts.md](active/refactor-context-hosts.md) | `RF-K*`, `RF-H*` | 지식 흐름·실제 호스트 검증 |
| [refactor-policy-transactions.md](active/refactor-policy-transactions.md) | `RF-P*`, `RF-T*` | 안전 검사·설치·복구 |
| [refactor-plan-state.md](active/refactor-plan-state.md) | `RF-S*` | Markdown 정본·집계 생성 |
| [usage-recovery-0.10.2.md](active/usage-recovery-0.10.2.md) | `UGR103-*` | 기존 표식 교체·모든 Hive 대상의 공통 복구 |
| [quota-reset-guard-0.11.0.md](active/quota-reset-guard-0.11.0.md) | `RB104-*` | 미출시 `0.10.4`에서 계승한 사용량 보호 |
| [instruction-quality-0.10.2.md](active/instruction-quality-0.10.2.md) | `INS102-*` | 소스·harness 지침 품질 |
| [`global-user-update-0.10.2.md`](active/global-user-update-0.10.2.md) | `GUU102-*` | 전역 사용자 설치·질문 대기·자동 재개 |
| [`release-0.10.2.md`](active/release-0.10.2.md) | `REL102-*` | 공개 시험·세 운영체제 수용·stable 공개 |

## 실행 순서

리팩터링: `RF-B01–B02` → `RF-S01–S02` → `RF-K01`·`RF-H01` → 안전·지식 개선 → 설치 분리 → 호스트 확대·상태 이관 → `RF-R01`. 상세 선행 조건은 [총괄 계획](active/refactor-foundations.md) 참조.

현재 요청의 종료 조건: 브랜치·계획·선택 사항 반영·문서 검사·로컬 커밋. 제품 구현은 후속 작업.

### 이전 출시의 실행 순서

기존 안정판 공개 완료 기록 보존. 공개된 안정판에 새 시험판 추가 금지; 후속 제품 버전은 현재 사용자 선택으로 확정.

1. `GUU102-001–004`: version·호환성·질문 catalog·전역 transaction
2. `GUU102-005–007`: setup-required 투영·답변·자동 재개·같은 버전 복구
3. `GUU102-008–010`: 신규 설치·안전 거부·문서·회귀
4. `REL102-001–003`: 전체 회귀·후보·공개 시험판 게시
5. `REL102-004–005`: 세 운영체제 공개 수용·현재 상태 정합화
6. `REL102-006–009`: 수용 시험판 결합 promotion·stable 공개·독립 확인

## 비활성 자료

- `0.10.0` 완료 계획: 기존 `docs/plans/active/*-0.10.0.md` 기록
- 버전 비종속 후보: [`backlog/README.md`](backlog/README.md)
- 완료·대체 기록: [`../archive/README.md`](../archive/README.md)
