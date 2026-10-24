# Aigent Hive 활성 계획

> Revision: 437
> 기준일: 2026-10-25
> Product version: `0.14.0`
> 공개 Stable: `0.13.1`
> 현재 단계: 기존 감시 해제의 부분 수용, 새 활성화 동의와 나머지 설정 통합 진행
> 다음 공개 시험: `0.14.0-test.1`

## 목표와 결정

- [상세 구현 계획](0.14.0-implementation.md): AI 없는 전역·프로젝트 설정과 첫 갱신의 질문·적용, 설정 출처·기록, 사용량 보호 기본 비활성, 보존형 제거
- 사용자 선택: 프로젝트까지 CLI 확장에 따라0.14.0으로 변경. 제거는 미리보기·확인·자료 보존만, 비대화형 갱신은 기존 선택 보존·질문 보류
- 필수 수용: 수정하지 않은 공개0.13.1에서 시작한 갱신 중 새 질문·답변·적용. 다음 버전·AI 수동 재설정으로 대체 금지
- 기존0.13.2 통신 호환·0~99% 구현과test.2 증거 보존. 정식0.13.2 공개는 보류 유지, 해당 수정은0.14.0 통합
- 최종 계획의 구현 승인 수신. 제품 소스 메타데이터는 버전 단계에서0.14.0으로 동기화, 실제 사용자 설치0.13.1 유지
- 작업은develop, 검증 단위별 로컬 커밋. 타 작업의 미추적 학습·평가·별도 계획 문서 보존
- 정식0.14.0의 main 통합·태그·게시·실제 설치와 새 공지 전송은 해당 범위 승인 전 제외.0.13.2 승인 자동 이전 없음

## Completion index

현재 집계는0.14.0 제품 구현·공개 시험 수용. 계획 문서 완료와 제품 완료의 구분.

<!-- HIVE:PLAN-STATE:START -->
| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 설정 출처·의도·이전 | 1 | 1 | 50.0% |
| 공통 엔진·전역/프로젝트 설정·AI 연결 | 0 | 4 | 0.0% |
| 첫 갱신 대화·완료·결과 | 0 | 3 | 0.0% |
| 사용량 보호·감시 선택 | 0 | 2 | 0.0% |
| 미리보기·확인·보존형 제거 | 0 | 1 | 0.0% |
| 비용 측정·통합 수용·공개 시험 | 0 | 3 | 0.0% |
| **현재 범위 합계** | **1** | **14** | **6.7%** |
<!-- HIVE:PLAN-STATE:END -->

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [preference-provenance-0.14.0.md](active/preference-provenance-0.14.0.md) | `P140-001–002` | 설정 출처·의도·이전 |
| [cli-setup-0.14.0.md](active/cli-setup-0.14.0.md) | `S140-001–004` | 공통 엔진·전역/프로젝트 설정·AI 연결 |
| [cli-update-0.14.0.md](active/cli-update-0.14.0.md) | `U140-001–003` | 첫 갱신 대화·완료·결과 |
| [usage-defaults-0.14.0.md](active/usage-defaults-0.14.0.md) | `G140-001–002` | 사용량 보호·감시 선택 |
| [cli-uninstall-0.14.0.md](active/cli-uninstall-0.14.0.md) | `D140-001` | 미리보기·확인·보존형 제거 |
| [cli-qualification-0.14.0.md](active/cli-qualification-0.14.0.md) | `Q140-001–003` | 비용 측정·통합 수용·공개 시험 |

## 결정과 이전 근거

- [0.14.0 결정](../decisions/interactive-cli-0.14.0.md)·[조사 근거](../research/interactive-cli-0.14.0.md)
- [이전0.13.2 활성 계획](../archive/plans/0.13.2-before-interactive-cli.md)·[이전 상태](../archive/state/0.13.2-before-interactive-cli.md)·[최초 일괄 해제안](../archive/plans/usage-default-off-0.13.2-original.md)
- [기존0.13.2 구현/수용](active/usage-compatibility-0.13.2.md)·[출시 보류 기록](0.13.2-stable-release.md). 기존 완료5개는 새15개에 중복 집계하지 않는 구분
