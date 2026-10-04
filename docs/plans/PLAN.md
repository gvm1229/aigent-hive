# Aigent Hive 활성 계획

> Revision: 379
> 기준일: 2026-10-05
> Product version: `0.11.1`
> 공개 Stable: `0.11.0`
> 현재 단계: 부속 자료 정리 회귀 수정·test.8 검증과 추가 정리
> 공개 시험 수용: `0.11.1-test.7`
> 다음 시험 대상: `0.11.1-test.8`

## 현재 요청과 경계

- 2026-10-02 사용자 승인: 0.11.1 안정판 main 통합·태그·npm·GitHub 게시와 자율 진행
- 실행 정본: [0.11.1 정식 출시](0.11.1-stable-release.md)
- 현재 사용자 test.5 설치 승인·적용 완료; 공지 문구·기존 Discord 채널 전송 승인 확보, 실제 발송은 안정판 게시 뒤; Codex 재시작·실제 대화 호출의 사용자 증거 대기

- 브랜치: `develop`
- Codex 우선, 사용자·프로젝트 기본 스킬의 플러그인 제공과 자동 갱신 정리
- 설치 기록·원본 일치 파일만 제거, 사용자 수정·외부 파일 보존
- 전역 갱신의 프로젝트 일괄 변경 제외; 프로젝트 갱신에 자동 정리 포함
- 구현·시험판·검증과 0.11.1 안정판·보호 main 통합 승인, 현재 사용자 설치는 별도 승인
- 기존 0.11.0 완료 기준과 근거: [이전 계획](../archive/plans/0.11.0-before-skill-delivery.md), [이전 상태](../archive/state/0.11.0-before-skill-delivery.md)

## Completion index

<!-- HIVE:PLAN-STATE:START -->
| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| Codex 스킬 제공·자동 정리 | 5 | 0 | 100.0% |
| Claude Windows 전역 설치 | 4 | 0 | 100.0% |
| 원격 출시 검증 수정 | 2 | 0 | 100.0% |
| 지침 이식·기본 스킬·자연어 활용 | 4 | 1 | 80.0% |
| 새 Hive 개선·사용자 스킬 결합 | 4 | 2 | 66.7% |
| 시험 용량 제한·매일 정리 | 3 | 1 | 75.0% |
| **현재 범위 합계** | **22** | **4** | **84.6%** |
<!-- HIVE:PLAN-STATE:END -->

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [skill-delivery-0.11.1.md](active/skill-delivery-0.11.1.md) | `SDP-001–005` | Codex 스킬 제공·자동 정리 |
| [claude-user-install-0.11.1.md](active/claude-user-install-0.11.1.md) | `CUI-001–004` | Claude Windows 전역 설치 |
| [release-qualification-repairs-0.11.1.md](active/release-qualification-repairs-0.11.1.md) | `RQP-001–002` | 원격 출시 검증 수정 |
| [directive-localization-and-project-skills-0.11.1.md](active/directive-localization-and-project-skills-0.11.1.md) | `DPS-001–005` | 지침 이식·기본 스킬·자연어 활용 |
| [skill-merge-0.11.1.md](active/skill-merge-0.11.1.md) | `SGM-001–006` | 새 Hive 개선·사용자 스킬 결합 |
| [test-cleanup-0.11.1.md](active/test-cleanup-0.11.1.md) | `TCL-001–003` | 시험 용량 제한·매일 정리 |

## 실행 순서

test.7 공개 수용·결합 미리보기 완료 → 사용자 설치·프로젝트 반영 승인 → `DPS-003` 실제 호출 → 승인된 안정판 게시

## 보존 자료

- [0.11.0 출시 계획](0.11.0-stable-release.md)
- [후속 목표](backlog/README.md)
