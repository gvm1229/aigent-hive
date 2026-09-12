# Aigent Hive 활성 계획

> Revision: 332
> 기준일: 2026-09-12
> Product version: `0.10.3`
> 공개 Stable: `0.10.3`
> 현재 단계: `0.10.3` 안정판 공개·독립 확인 완료
> 첫 공개 시험: `0.10.3-test.1`
> 결정: [`ADR-0022`](../decisions/ADR-0022-global-user-update.md)

## 목표

- 모든 Hive 프로젝트·소스의 옛 `halt.json`을 같은 위치에서 교체·제거하는 공통 복구, 추가 영구 상태 파일 0건: `UGR103-*`

- 소스·harness 지침의 승인 경계·지속성·현재 참조 개선: `INS102-*`

- `hive update` 한 번으로 실행 파일과 전역 사용자 설정·호스트 투영을 수렴
- 새 전역 질문은 답변 전 일반 Hive 작업을 차단하고, 답변 뒤 같은 transaction을 재개
- npm 설치는 실행 파일만 제공하고 최초 전역 초기화는 `hive update`가 담당
- `0.10.3-test.1` 세 운영체제 공개 수용 뒤 stable `0.10.3` 공개

## 완료 조건

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
| **합계** | **35** | **0** | **100%** |

## Required load order

1. 설치 product usage guard
2. `docs/plans/PLAN.md`
3. `docs/state/CURRENT.md`
4. [`global-user-update-0.10.2.md`](active/global-user-update-0.10.2.md)
5. [`release-0.10.2.md`](active/release-0.10.2.md)
6. 직접 관련 architecture·decision·guide

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [usage-recovery-0.10.2.md](active/usage-recovery-0.10.2.md) | `UGR103-*` | 기존 표식 교체·모든 Hive 대상의 공통 복구 |
| [reset-booster-0.10.4.md](active/reset-booster-0.10.4.md) | `RB104-*` | 초기화된 사용량 감지·자동 실행 차단 |
| [instruction-quality-0.10.2.md](active/instruction-quality-0.10.2.md) | `INS102-*` | 소스·harness 지침 품질 |
| [`global-user-update-0.10.2.md`](active/global-user-update-0.10.2.md) | `GUU102-*` | 전역 사용자 설치·질문 대기·자동 재개 |
| [`release-0.10.2.md`](active/release-0.10.2.md) | `REL102-*` | 공개 시험·세 운영체제 수용·stable 공개 |

## 실행 순서

현재 활성 완료 항목: 0건. Stable 공개 뒤 제품 변경: 다음 patch version과 해당 version의 `-test.1`부터 새 수용

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
