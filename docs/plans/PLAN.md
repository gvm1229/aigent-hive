# Aigent Hive 활성 계획

> Revision: 361
> 기준일: 2026-09-23
> Product version: `0.11.0`
> 공개 Stable: `0.10.3`
> 현재 단계: 전체 공개 안정판 검증·시험판 수용·브랜치 통합 완료
> 공개 수용 완료: `0.11.0-test.7`; test.3 수용 보류, 미공개 test.4·5 취소
> 결정: [ADR-0023](../decisions/ADR-0023-foundation-refactor.md), [기존 ADR-0022](../decisions/ADR-0022-global-user-update.md)

## 현재 요청과 경계

- 후속 승인: `PRF-001–006` 스킬 접근성·구버전 갱신 검증 후 실제 예시 6단계 적용

- 이전 요청 완료: `PDC-001–003`·test.2 수용·예시 수정 계획
- 브랜치: `develop`; 초기 기준 `develop@87b84f42`, 작업 이력 통합 완료
- 확정: 기존 기능·명령 유지, 핵심 지식 흐름 우선, Codex 검증 후 다른 호스트 확대
- 목표: 지식 이어 가기·호스트 검증·코드 중심 안전 검사·공통 파일 처리·상태 단일 정본
- [총괄 계획](active/refactor-foundations.md)의 21개 구현 항목. 계획 작성 완료와 제품 구현 완료의 별도 판정
- 버전 결정: 미출시 `0.10.4` 변경을 `0.11.0`으로 계승, 별도 `0.10.4` 출시 제외. 제품 파일의 번호 변경은 `RFB-001` 구현 단계
- `RB104-004`: 작업별 제어·명시적 재개 유지. 사용자 승인으로 15초 상시 감시·진행 중 자동 중단만 [버전 미정 후속 목표](backlog/periodic-usage-interruption.md)로 이전
- 사용자 승인 범위 변경: `RFH-003`·`HK-003`·`HK-004`의 Claude 실제 검증과 해당 `RFR-001` 의존 범위를 `0.11.0`에서 제외. [후속 목표](backlog/claude-host-acceptance.md)의 버전은 사용자 결정 전 미정; 기존 구현·증거 보존
- 안정판 권한은 후속 버전별 명시 승인 필요
- 전체 적용 분석: [정책 검사와 훅의 책임](../research/project-policy-enforcement-2026-09-18.md). 승인 제안은 기존 기준 보강과 `HK-005` 추가, 완료 항목 증가 없음

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

- 승인된 PRF 설치·예시 적용 범위를 벗어난 사용자 루트·프로젝트 변경
- Provider API·provider credential·OMX/OMC 사용

## Completion index

<!-- HIVE:PLAN-STATE:START -->
| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 갱신 스킬 노출·검증·실제 적용 | 6 | 0 | 100.0% |
| 모든 공개 버전 갱신 검증·브랜치 통합 | 4 | 0 | 100.0% |
| 브랜치 규칙 강제·호스트 조사 | 5 | 0 | 100.0% |
| 호스트 정책 훅 제품 계획 | 3 | 0 | 100.0% |
| 호스트 훅 독립 평가 | 1 | 0 | 100.0% |
| 작업 후 개선 후보·사람 검토 | 1 | 0 | 100.0% |
| 공통 기준·실행 순서·통합 수용 | 3 | 0 | 100.0% |
| 지식 흐름·실제 호스트 검증 | 6 | 0 | 100.0% |
| 안전 검사·설치·복구 | 8 | 0 | 100.0% |
| Markdown 정본·집계 생성 | 4 | 0 | 100.0% |
| 협업자 설치 선택권·프로젝트 지침 | 3 | 0 | 100.0% |
| 미출시 `0.10.4`에서 계승한 사용량 보호 | 4 | 0 | 100.0% |
| **현재 범위 합계** | **48** | **0** | **100.0%** |
<!-- HIVE:PLAN-STATE:END -->

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
| [project-refresh-exposure-0.11.0.md](active/project-refresh-exposure-0.11.0.md) | `PRF-001–006` | 갱신 스킬 노출·검증·실제 적용 |
| [all-public-project-refresh-0.11.0.md](active/all-public-project-refresh-0.11.0.md) | `APC-001–004` | 모든 공개 버전 갱신 검증·브랜치 통합 |
| [branch-enforcement-0.11.0.md](active/branch-enforcement-0.11.0.md) | `BR-*` | 브랜치 규칙 강제·호스트 조사 |
| [host-policy-hooks-0.11.0.md](active/host-policy-hooks-0.11.0.md) | `HK-001–003` | 호스트 정책 훅 제품 계획 |
| [hook-policy-evaluation-0.11.0.md](active/hook-policy-evaluation-0.11.0.md) | `HK-004` | 호스트 훅 독립 평가 |
| [hook-review-candidates-0.11.0.md](active/hook-review-candidates-0.11.0.md) | `HK-005` | 작업 후 개선 후보·사람 검토 |
| [refactor-foundations.md](active/refactor-foundations.md) | `RFB-*`, `RFR-*` | 공통 기준·실행 순서·통합 수용 |
| [refactor-context-hosts.md](active/refactor-context-hosts.md) | `RFK-*`, `RFH-*` | 지식 흐름·실제 호스트 검증 |
| [refactor-policy-transactions.md](active/refactor-policy-transactions.md) | `RFP-*`, `RFT-*` | 안전 검사·설치·복구 |
| [refactor-plan-state.md](active/refactor-plan-state.md) | `RFS-*` | Markdown 정본·집계 생성 |
| [project-directives-collaboration-0.11.0.md](active/project-directives-collaboration-0.11.0.md) | `PDC-001–003` | 협업자 설치 선택권·프로젝트 지침 |
| [quota-reset-guard-0.11.0.md](active/quota-reset-guard-0.11.0.md) | `RB104-*` | 미출시 `0.10.4`에서 계승한 사용량 보호 |

## 실행 순서

리팩터링: 기준 보존 → `RFS-001–004` → `RFB-001–002` → `RFK-001`·`RFH-001` → 안전·지식 개선 → 설치 분리 → 호스트 확대·상태 이관 → `RFR-001`. 상세 선행 조건은 [총괄 계획](active/refactor-foundations.md) 참조.

훅 상세 순서: `HK-001` 실패 분류·결과 합산 → `HK-002` 변환·진단·전달 → `HK-003` 실제 수용 → `HK-004` 보류 평가. `HK-005`는 `HK-001`과 기존 실행 결과 계약 뒤 구현, `RFK-002`는 문맥 유효성 소유.

현재 순서: `APC-001` → `APC-002` → `APC-003` → `APC-004`. 완료된 PRF·PDC 근거는 해당 계획 보존.

### 이전 출시의 실행 순서

기존 안정판 공개 완료 기록 보존. 공개된 안정판에 새 시험판 추가 금지; 후속 제품 버전은 현재 사용자 선택으로 확정.

완료한 `GUU102-001–010`·`REL102-001–009`의 순서와 근거는 아래 전역 갱신·출시 계획에 보존.

## 비활성 자료

- `0.10.0` 완료 계획: 기존 `docs/plans/active/*-0.10.0.md` 기록
- 버전 비종속 후보: [`backlog/README.md`](backlog/README.md)
- 완료·대체 기록: [`../archive/README.md`](../archive/README.md)

- [리팩터링 ID 대응표](refactor-id-mapping.md): 현재 ID 정합화, 과거 조사 기록 보존

## 이전 출시 완료 기록

현재 0.11.0 집계에서 제외, 기존 체크 표시와 증거 보존.

- [usage-recovery-0.10.2.md](active/usage-recovery-0.10.2.md)
- [instruction-quality-0.10.2.md](active/instruction-quality-0.10.2.md)
- [global-user-update-0.10.2.md](active/global-user-update-0.10.2.md)
- [release-0.10.2.md](active/release-0.10.2.md)
