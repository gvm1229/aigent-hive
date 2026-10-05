# Aigent Hive 활성 계획

> Revision: 391
> 기준일: 2026-10-05
> Product version: `0.12.0`
> 공개 Stable: `0.11.1`
> 현재 단계: 0.12.0 전환·소진 보호 구현·호스트 수용
> 다음 공개 시험: `0.12.0-test.2`
> 다음 단계: Windows의 사용량·제어 연결·Graphify·진단 구현, macOS 전용 수용 마지막

## 현재 요청과 경계

- [사용자 결정](../decisions/ADR-0025-0.11.2-scope.md): 주기 감시·중단, 작업 분담, Graphify 잔여를 0.11.2 편입
- [조사 결과](../research/followup-feasibility-0.11.2.md): Antigravity 사용량, 호스트 검사, 토큰 계측, Windows 샌드박스, macOS 진단의 구현 후보와 선행 조건
- 이번 요청: 확정 계획의 구현·검증·번호 공개 시험. 실제 검증된 기능만 활성, 미지원은 비활성·대기로 분리
- 활성 관계 기능: 지식 저장 뒤 현재 호스트의 자동 분석. 저장 성공 보존, 최대 10개·16KiB·교정 1회
- 기본 환경 Windows, 작업 브랜치 develop. macOS 전용 작업은 마지막 QLF112-005
- 현재 소스 0.12.0, 설치·공개 안정판 0.11.1. 다음 번호 시험 0.12.0-test.2
- 0.12.0 안정판·보호 main·게시·설치의 현재 버전별 승인 없음
- Claude 실제 대화·구독 사용량과 게시자 서명: 버전 미정 대기 유지
- Notion 후보·오래된 발표 자료 폐기. 로고·벡터 완료, 엔진 비교 참고 보존. Obsidian 전용 플러그인 후보 종료

## Completion index

<!-- HIVE:PLAN-STATE:START -->
| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 호스트 사용량·중단·분담 | 1 | 5 | 16.7% |
| Graphify 미완료 확장 | 2 | 1 | 66.7% |
| 계측·진단·운영체제 수용 | 1 | 4 | 20.0% |
| **현재 범위 합계** | **4** | **10** | **28.6%** |
<!-- HIVE:PLAN-STATE:END -->

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [host-control-0.11.2.md](active/host-control-0.11.2.md) | `HCT112-001–005`, `HCT120-001` | 호스트 사용량·중단·분담 |
| [knowledge-graph-0.11.2.md](active/knowledge-graph-0.11.2.md) | `GPH112-001–003` | Graphify 미완료 확장 |
| [qualification-0.11.2.md](active/qualification-0.11.2.md) | `QLF112-001–005` | 계측·진단·운영체제 수용 |

## 실행 순서

1. Windows의 지원 계약 검증과 독립 구현: HCT112-001·002·005, GPH112-001, QLF112-001·002·003
2. 검증된 연결 기반 HCT112-003·004, GPH112-002·003
3. Windows 실제 수용·관련 Linux 검사·공개 시험 준비 QLF112-004
4. 마지막 macOS arm64 재현·실제 호스트·공개 수용 QLF112-005

## 보존 자료

- [0.11.1 완료 계획](../archive/plans/0.11.1-complete.md)·[완료 상태](../archive/state/0.11.1-complete.md)·[정식 출시](0.11.1-stable-release.md)
- [후속 목록과 종료된 후보](backlog/README.md)
- [이번 조사 절차](0.11.2-scope-research.md)

## 추가 승인

- 2026-10-05 사용자 지정 0.12.0 전환. 원래 13개 ID·문서 경로·실행 근거 보존, 추가 소진 보호 HCT120-001 별도 집계
- [0% 보호 구현](usage-zero-guard-0.12.0.md): 일반 보호 해제와 독립, 기본 활성·현재 대화 한정 명시적 제외
