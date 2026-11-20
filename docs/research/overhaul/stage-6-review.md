# 단계 6 검토 — 0.16.0 메시지·작업 중재

- 기준일: 2026-11-20, 대상: `feature/overhaul-6-0.16.0-mediation`의 `origin/develop` 대비 전체(`81b7e0ee`까지, OV160-001–008)
- 검토자: Fable 5.2 일반 하위 에이전트, 기본 수준·수준 지정 없음. 이 세션에 advisor 하위 에이전트 없음, [위임 지침](../../../.agents/directives/references/agent-delegation.md#stage-review-and-release)의 대체 경로. 읽기 전용, 실험은 `/tmp` 일회용 소비 프로젝트
- 요청에 포함한 유지관리자 원문(2026-11-12): "This code base is yours. What it's for and how it's built are your call, and you'll keep building on it. If you think that editing something big would contribute to the goal, then you are free to point that out and delegate tasks."
- 판정: 1차 REQUEST CHANGES(차단 2건 B1·B2, 훅 반환 경로 한정). 수정 뒤 같은 검토자의 B1·B2 재검토
- 검토자 확인 범위: Claude 공식 훅 문서 조회, `target/debug/hive`로 B1·B2·S1·S2·S4·S7·S8 재현, 대상 시험 102개 통과. Codex·Gemini 공식 문서는 이 세션 망 정책으로 미조회

## 지적과 처리

| ID | 지적 | 처리 |
| --- | --- | --- |
| B1 | Claude 기본 Stop 반환이 전달 결과 없이도 `additionalContext`, 공식 문서상 대화 계속이라 매 Stop 추가 모델 호출 | PENDING |
| B2 | 결과 파일과 확인증 지문·크기 미대조, 변조 결과의 훅 주입 | PENDING |
| S1 | 운영 명령의 `--now` 시계 덮어쓰기로 기한 만료 뒤 확인증 기록 가능 | PENDING |
| S2 | 사건 정체성이 본문 지문, 같은 입력 재전송 때 옛 응답 재생·새 결과 억제 | PENDING |
| S3 | 매 훅 호출의 대기·보류 ID 무제한 반복 | PENDING |
| S4 | `max_workers` 세기 뒤 게시, 동시 소비자 N명이 N-1 초과 | PENDING |
| S5 | `msg ack`에 소비권 토큰 불필요 | PENDING |
| S6 | 읽기 때 현재 명단 재해석, 명단 변경 뒤 기존 메시지 격리 | PENDING |
| S7 | 미검증 Gemini 훅 적용 허용, 시간 단위 미확인 | PENDING |
| S8 | 정상 파일이 아닌 기록 하나가 수신 상자 전체 실패 | PENDING |
| S9 | `.hive/mediation/`이 작업자 쓰기 범위 안·Git 미제외 | PENDING |
| S10 | 배포 Skill·지침에 중재 명령 안내 없음 | PENDING |
| O1 | Discord 알림 한국어 전용 | PENDING |
| O2 | 끝난 메시지와 같은 키 재전송의 안내 없음 | PENDING |
| O3 | 다른 세션 전달 결과의 복구 경로 미안내 | PENDING |
| O4 | `max_workers`가 템플릿·스키마에 없음 | PENDING |
| O5 | 근거 문서 문구(003 기본 알림, 005 초과 1, 같은 수신 상자, 권한 재사용) | PENDING |

## 제안과 처리

| ID | 제안 | 처리 |
| --- | --- | --- |
| P1 | 남은 Skill 통합(31개, 안팎 15는 기본 묶음·Wiki 끔에서만) | 0.16.0은 승인된 통합 범위 완료로 수용. 후속은 [OV170-014](../../plans/overhaul/0.17.0-overnight-loop.md) |
| P2 | 서명 소환과 파일 수신 상자의 두 봉투·확인증 모델 단일화 또는 ADR | 작업자 실행 추가 전 단계 7 첫 항목군 [OV170-012](../../plans/overhaul/0.17.0-overnight-loop.md) |
| P3 | 호스트 쪽 안내 Skill과 참조 처리기 적합성 시험 | 정식 전 S10과 함께 이 단계에서 처리 |
| P4 | 수신 상자 보존 정리와 1,000개 지연 예산 | S3의 일회 안내·상한은 이 단계, 정리 명령은 [OV170-013](../../plans/overhaul/0.17.0-overnight-loop.md) |
| P5 | 실제 수용 전 실험 기능 표시 | 0.16.0-test.1 출시 노트·local 인계에 "실제 호스트 수용 전" 한계 명시 |
