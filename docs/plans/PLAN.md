# Aigent Hive 활성 계획

> Revision: 383
> 기준일: 2026-10-05
> Product version: `0.11.1`
> 공개 Stable: `0.11.1`
> 현재 단계: 0.11.1 정식 게시·승인 공지 전송 완료
> 공개 시험 수용: `0.11.1-test.8`
> 다음 단계: 현재 범위 완료, 새 요청 대기

## 현재 요청과 경계

- 2026-10-02 사용자 승인: 0.11.1 안정판 main 통합·태그·npm·GitHub 게시와 자율 진행
- 실행 정본: [0.11.1 정식 출시](0.11.1-stable-release.md)
- 현재 사용자 test.8·프로젝트 결합·실제 스킬 선택 수용 완료, 0.11.1 정식 게시와 승인된 Discord 공지 전송 완료

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
| 원격 출시 검증 수정 | 4 | 0 | 100.0% |
| 지침 이식·기본 스킬·자연어 활용 | 5 | 0 | 100.0% |
| 새 Hive 개선·사용자 스킬 결합 | 6 | 0 | 100.0% |
| 시험 용량 제한·매일 정리 | 4 | 0 | 100.0% |
| **현재 범위 합계** | **28** | **0** | **100.0%** |
<!-- HIVE:PLAN-STATE:END -->

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [skill-delivery-0.11.1.md](active/skill-delivery-0.11.1.md) | `SDP-001–005` | Codex 스킬 제공·자동 정리 |
| [claude-user-install-0.11.1.md](active/claude-user-install-0.11.1.md) | `CUI-001–004` | Claude Windows 전역 설치 |
| [release-qualification-repairs-0.11.1.md](active/release-qualification-repairs-0.11.1.md) | `RQP-001–004` | 원격 출시 검증 수정 |
| [directive-localization-and-project-skills-0.11.1.md](active/directive-localization-and-project-skills-0.11.1.md) | `DPS-001–005` | 지침 이식·기본 스킬·자연어 활용 |
| [skill-merge-0.11.1.md](active/skill-merge-0.11.1.md) | `SGM-001–006` | 새 Hive 개선·사용자 스킬 결합 |
| [test-cleanup-0.11.1.md](active/test-cleanup-0.11.1.md) | `TCL-001–004` | 시험 용량 제한·매일 정리 |

## 실행 순서

test.8·실제 프로젝트 호출 수용 → 보호 main 통합 → 0.11.1 안정판 게시·승인 공지 전송 완료

## 보존 자료

- [0.11.0 출시 계획](0.11.0-stable-release.md)
- [후속 목표](backlog/README.md)
