# 현재 상태

- 작업 branch: `develop`; stable source branch: `main`
- 제품 버전: `0.10.2`; 현재 공개 stable: `0.10.1`
- 완료 작업: 소스·harness 지침 품질 개선 `INS102-*` 6/6
- 출시 상태: 유지보수자 요청으로 보류. 이번 개선 완료가 출시 재개 승인은 아님
- 활성 계획: [PLAN.md](../plans/PLAN.md)
- 개선 계획: [instruction-quality-0.10.2.md](../plans/active/instruction-quality-0.10.2.md)
- 전역 갱신 계획: [global-user-update-0.10.2.md](../plans/active/global-user-update-0.10.2.md)
- 출시 계획: [release-0.10.2.md](../plans/active/release-0.10.2.md)
- 결정: [ADR-0022](../decisions/ADR-0022-global-user-update.md)

## 권한과 범위

- 현재 승인: 지침 개선 구현·검증·로컬 commit
- 현재 제외: 실제 사용자 설치·등록 프로젝트 변경·stable 게시
- 과거 project/user base·외부 파일·사용자 선택 보존
- 전역 사용량 보호: 유지보수자가 지정한 남은 사용량 `2%`

## 개선 작업

- 모호한 일반 요청의 작업 권한 유지와 명시적 프롬프트 작성 경로 분리
- 현재 Skill 참조·기본 stable 채널·공유 지식 색인·종료 hook 규칙 정합화
- 사용자 설정 진입 문서와 작업별 참조 분리, Rust·정적 호스트 투영에 동반 자료 포함
- 인증 실패의 읽기 전용 진단 우선, 전체 제거·재설치의 별도 권한 유지
- 독립 문서 기반 행동 평가와 관련 시험 완료. 실제 장시간 호스트 성공률 증명은 제외
- 완료 수치는 개선 계획의 검증된 체크리스트 기준

## 보류된 전역 갱신과 출시

- 기존 `GUU102-*`, `REL102-*`의 미완료 기준 유지
- 지침 개선 완료와 전역 갱신 기능·공개 시험·stable 출시 완료는 별도
- 이전 출시 증거와 조사 기록: [이전 상태](../archive/state/0.10.2-before-instruction-closeout.md)
