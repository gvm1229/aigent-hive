# Aigent Hive 활성 계획

> Revision: 348
> 기준일: 2026-09-20
> Product version: `0.11.0`
> 공개 Stable: `0.10.3`
> 현재 단계: `0.11.0` 초기화 감지 제외 구현·실제 지식 연결 검증, 호스트 수용 진행
> 첫 공개 시험: `0.11.0-test.1`; 현재 제품 파일의 버전: `0.11.0`
> 결정: [ADR-0023](../decisions/ADR-0023-foundation-refactor.md), [기존 ADR-0022](../decisions/ADR-0022-global-user-update.md)

## 현재 요청과 경계

- 현재 요청: 승인된 전체 구현 계획 실행. 안정판 권한·실제 사용자 설치 경계 유지
- 브랜치: `refactor/hive-foundations`, 기준 `develop@87b84f42`
- 확정: 기존 기능·명령 유지, 핵심 지식 흐름 우선, Codex 검증 후 다른 호스트 확대
- 목표: 지식 이어 가기·호스트 검증·코드 중심 안전 검사·공통 파일 처리·상태 단일 정본
- [총괄 계획](active/refactor-foundations.md)의 21개 구현 항목. 계획 작성 완료와 제품 구현 완료의 별도 판정
- 버전 결정: 미출시 `0.10.4` 변경을 `0.11.0`으로 계승, 별도 `0.10.4` 출시 제외. 제품 파일의 번호 변경은 `RFB-001` 구현 단계
- `RB104-004`: 기존 소유 항목 유지, 검증된 호스트 계약 부재. 이번 리팩터링의 복제 항목·완료 처리 제외
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

- 실제 사용자 루트 변경과 등록 프로젝트 harness 변경
- Provider API·provider credential·OMX/OMC 사용

## Completion index

<!-- HIVE:PLAN-STATE:START -->
| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 브랜치 규칙 강제·호스트 조사 | 5 | 0 | 100.0% |
| 호스트 정책 훅 제품 계획 | 2 | 1 | 66.7% |
| 호스트 훅 독립 평가 | 0 | 1 | 0.0% |
| 작업 후 개선 후보·사람 검토 | 1 | 0 | 100.0% |
| 공통 기준·실행 순서·통합 수용 | 2 | 1 | 66.7% |
| 지식 흐름·실제 호스트 검증 | 6 | 0 | 100.0% |
| 안전 검사·설치·복구 | 8 | 0 | 100.0% |
| Markdown 정본·집계 생성 | 4 | 0 | 100.0% |
| 미출시 `0.10.4`에서 계승한 사용량 보호 | 3 | 1 | 75.0% |
| **현재 범위 합계** | **31** | **4** | **88.6%** |
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
| [branch-enforcement-0.11.0.md](active/branch-enforcement-0.11.0.md) | `BR-*` | 브랜치 규칙 강제·호스트 조사 |
| [host-policy-hooks-0.11.0.md](active/host-policy-hooks-0.11.0.md) | `HK-001–003` | 호스트 정책 훅 제품 계획 |
| [hook-policy-evaluation-0.11.0.md](active/hook-policy-evaluation-0.11.0.md) | `HK-004` | 호스트 훅 독립 평가 |
| [hook-review-candidates-0.11.0.md](active/hook-review-candidates-0.11.0.md) | `HK-005` | 작업 후 개선 후보·사람 검토 |
| [refactor-foundations.md](active/refactor-foundations.md) | `RFB-*`, `RFR-*` | 공통 기준·실행 순서·통합 수용 |
| [refactor-context-hosts.md](active/refactor-context-hosts.md) | `RFK-*`, `RFH-*` | 지식 흐름·실제 호스트 검증 |
| [refactor-policy-transactions.md](active/refactor-policy-transactions.md) | `RFP-*`, `RFT-*` | 안전 검사·설치·복구 |
| [refactor-plan-state.md](active/refactor-plan-state.md) | `RFS-*` | Markdown 정본·집계 생성 |
| [quota-reset-guard-0.11.0.md](active/quota-reset-guard-0.11.0.md) | `RB104-*` | 미출시 `0.10.4`에서 계승한 사용량 보호 |

## 실행 순서

리팩터링: 기준 보존 → `RFS-001–004` → `RFB-001–002` → `RFK-001`·`RFH-001` → 안전·지식 개선 → 설치 분리 → 호스트 확대·상태 이관 → `RFR-001`. 상세 선행 조건은 [총괄 계획](active/refactor-foundations.md) 참조.

훅 상세 순서: `HK-001` 실패 분류·결과 합산 → `HK-002` 변환·진단·전달 → `HK-003` 실제 수용 → `HK-004` 보류 평가. `HK-005`는 `HK-001`과 기존 실행 결과 계약 뒤 구현, `RFK-002`는 문맥 유효성 소유.

현재 요청의 종료 조건: 범위 내 안전한 구현·검증 완료. 실제 호스트 승인·외부 증거가 필요한 항목은 소유자와 한계를 명시하고 독립 구현 지속.

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

- [리팩터링 ID 대응표](refactor-id-mapping.md): 현재 ID 정합화, 과거 조사 기록 보존

## 이전 출시 완료 기록

현재 0.11.0 집계에서 제외, 기존 체크 표시와 증거 보존.

- [usage-recovery-0.10.2.md](active/usage-recovery-0.10.2.md)
- [instruction-quality-0.10.2.md](active/instruction-quality-0.10.2.md)
- [global-user-update-0.10.2.md](active/global-user-update-0.10.2.md)
- [release-0.10.2.md](active/release-0.10.2.md)
